#!/bin/bash
# PYRAX Devnet Full Deployment Script
# Deploys the complete TriStream blockchain to Digital Ocean
# Usage: ./deploy-devnet.sh

set -e

# Configuration
SERVER="root@209.38.137.105"
SERVER_IP="209.38.137.105"
REMOTE_PATH="/opt/pyrax"
DATA_DIR="/var/lib/pyrax/devnet"
LOG_FILE="/var/log/pyrax-node.log"

# Ports
RPC_PORT=28545
STRATUM_PORT=3333
STAKING_PORT=28547
P2P_PORT=30303

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║     PYRAX Devnet Deployment - TriStream DAG Blockchain        ║"
echo "║   Stream A (BLAKE3) | Stream B (KAWPOW) | Stream C (ZK)       ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "Server: $SERVER_IP"
echo ""

# Check if we can connect to the server
echo "[1/8] Testing SSH connection..."
if ! ssh -o ConnectTimeout=5 $SERVER "echo 'SSH OK'" > /dev/null 2>&1; then
    echo "ERROR: Cannot connect to $SERVER"
    echo "Please ensure SSH access is configured."
    exit 1
fi
echo "✓ SSH connection successful"

# Check if Rust is installed on server
echo ""
echo "[2/8] Checking server prerequisites..."
ssh $SERVER "command -v cargo > /dev/null 2>&1 || (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source ~/.cargo/env)"
echo "✓ Rust/Cargo available"

# Create required directories on server
echo ""
echo "[3/8] Setting up server directories..."
ssh $SERVER << 'ENDSSH'
mkdir -p /opt/pyrax
mkdir -p /var/lib/pyrax/devnet
mkdir -p /var/log
useradd -r -s /bin/false pyrax 2>/dev/null || true
chown -R pyrax:pyrax /var/lib/pyrax
ENDSSH
echo "✓ Directories created"

# Sync source code
echo ""
echo "[4/8] Syncing pyrax-node source code..."
rsync -avz --delete \
    --exclude 'target' \
    --exclude '.git' \
    --exclude 'node_modules' \
    ./pyrax-node/ $SERVER:$REMOTE_PATH/pyrax-node/
echo "✓ Source synced"

# Build on server
echo ""
echo "[5/8] Building pyrax-node on server (this may take a few minutes)..."
ssh $SERVER << 'ENDSSH'
source ~/.cargo/env
cd /opt/pyrax/pyrax-node
cargo build --release
cp target/release/pyrax-node /usr/local/bin/pyrax-node
chmod +x /usr/local/bin/pyrax-node
echo "Build complete: $(pyrax-node --version)"
ENDSSH
echo "✓ Build complete"

# Configure firewall
echo ""
echo "[6/8] Configuring firewall..."
ssh $SERVER << ENDSSH
ufw allow 22/tcp    # SSH
ufw allow 80/tcp    # HTTP
ufw allow 443/tcp   # HTTPS
ufw allow $RPC_PORT/tcp      # Stream A RPC
ufw allow $STRATUM_PORT/tcp  # Stream B Stratum
ufw allow $STAKING_PORT/tcp  # Stream C Staking
ufw allow $P2P_PORT/tcp      # P2P
ufw --force enable
ENDSSH
echo "✓ Firewall configured"

# Deploy systemd service
echo ""
echo "[7/8] Installing systemd service..."
ssh $SERVER << 'ENDSSH'
cat > /etc/systemd/system/pyrax-node.service << 'EOF'
[Unit]
Description=PYRAX TriStream Blockchain Node (Devnet)
After=network.target
Wants=network-online.target
Documentation=https://docs.pyrax.org

[Service]
Type=simple
User=root
WorkingDirectory=/opt/pyrax

ExecStart=/usr/local/bin/pyrax-node \
    --network devnet \
    --datadir /var/lib/pyrax/devnet \
    --rpc \
    --rpc-addr 0.0.0.0:28545 \
    --p2p \
    --p2p-addr /ip4/0.0.0.0/tcp/30303 \
    --stratum \
    --stratum-addr 0.0.0.0:3333 \
    --staking \
    --staking-addr 0.0.0.0:28547 \
    --verbosity 2

Restart=always
RestartSec=5
LimitNOFILE=65535

StandardOutput=journal
StandardError=journal
SyslogIdentifier=pyrax-node

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable pyrax-node
ENDSSH
echo "✓ Systemd service installed"

# Start the node
echo ""
echo "[8/8] Starting PYRAX node..."
ssh $SERVER << 'ENDSSH'
systemctl stop pyrax-node 2>/dev/null || true
sleep 2
systemctl start pyrax-node
sleep 3
systemctl status pyrax-node --no-pager || true
ENDSSH

# Verify services are running
echo ""
echo "============================================"
echo "Verifying services..."
echo "============================================"
sleep 5

# Check RPC
echo ""
echo "Testing Stream A (RPC) on port $RPC_PORT..."
if curl -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"pyrax_getChainInfo","params":[],"id":1}' \
    http://$SERVER_IP:$RPC_PORT 2>/dev/null | grep -q "result"; then
    echo "✓ Stream A RPC is responding"
else
    echo "⚠ Stream A RPC not responding yet (may still be starting)"
fi

# Check ports
echo ""
echo "Checking open ports on server..."
ssh $SERVER "netstat -tlnp | grep -E '($RPC_PORT|$STRATUM_PORT|$STAKING_PORT|$P2P_PORT)'" || echo "Ports check complete"

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                   DEPLOYMENT COMPLETE!                         ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "PYRAX Devnet Services:"
echo "  • Stream A (RPC):     http://$SERVER_IP:$RPC_PORT"
echo "  • Stream B (Stratum): $SERVER_IP:$STRATUM_PORT"
echo "  • Stream C (Staking): http://$SERVER_IP:$STAKING_PORT"
echo "  • P2P:                /ip4/$SERVER_IP/tcp/$P2P_PORT"
echo ""
echo "Useful commands:"
echo "  ssh $SERVER 'systemctl status pyrax-node'"
echo "  ssh $SERVER 'journalctl -u pyrax-node -f'"
echo "  ssh $SERVER 'systemctl restart pyrax-node'"
echo ""
echo "Test RPC:"
echo "  curl -X POST -H 'Content-Type: application/json' \\"
echo "    -d '{\"jsonrpc\":\"2.0\",\"method\":\"pyrax_getChainInfo\",\"params\":[],\"id\":1}' \\"
echo "    http://$SERVER_IP:$RPC_PORT"
echo ""
