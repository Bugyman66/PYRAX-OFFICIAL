#!/bin/bash
# PYRAX Devnet Full Deployment Script
# Deploys the complete TriStream blockchain + web services to Digital Ocean
# Usage: ./deploy-devnet.sh [--node-only|--web-only|--all]

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

# Parse arguments
DEPLOY_NODE=true
DEPLOY_WEB=true

case "${1:-all}" in
    --node-only)
        DEPLOY_WEB=false
        ;;
    --web-only)
        DEPLOY_NODE=false
        ;;
    --all|"")
        DEPLOY_NODE=true
        DEPLOY_WEB=true
        ;;
    *)
        echo "Usage: $0 [--node-only|--web-only|--all]"
        exit 1
        ;;
esac

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║     PYRAX Devnet Deployment - TriStream DAG Blockchain        ║"
echo "║   Stream A (BLAKE3) | Stream B (KAWPOW) | Stream C (ZK)       ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "Server: $SERVER_IP"
echo "Deploy Node: $DEPLOY_NODE | Deploy Web Services: $DEPLOY_WEB"
echo ""

# SSH options for non-interactive deployment
SSH_OPTS="-o ConnectTimeout=10 -o StrictHostKeyChecking=accept-new -o BatchMode=yes"

# Check if we can connect to the server
echo "[1/10] Testing SSH connection..."
if ! ssh $SSH_OPTS $SERVER "echo 'SSH OK'" > /dev/null 2>&1; then
    echo "ERROR: Cannot connect to $SERVER"
    echo "Please ensure SSH access is configured."
    exit 1
fi
echo "✓ SSH connection successful"

# Create required directories on server
echo ""
echo "[2/10] Setting up server directories..."
ssh $SSH_OPTS $SERVER << 'ENDSSH'
mkdir -p /opt/pyrax
mkdir -p /var/lib/pyrax/devnet
mkdir -p /var/log
useradd -r -s /bin/false pyrax 2>/dev/null || true
chown -R pyrax:pyrax /var/lib/pyrax
ENDSSH
echo "✓ Directories created"

# Configure firewall (always needed)
echo ""
echo "[3/10] Configuring firewall..."
ssh $SSH_OPTS $SERVER << ENDSSH
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

# ==========================================
# NODE DEPLOYMENT (if enabled)
# ==========================================
if [ "$DEPLOY_NODE" = true ]; then
    echo ""
    echo "============================================"
    echo "DEPLOYING PYRAX NODE (TriStream Blockchain)"
    echo "============================================"

    # Check if Rust is installed on server
    echo ""
    echo "[4/10] Checking Rust prerequisites..."
    ssh $SSH_OPTS $SERVER "command -v cargo > /dev/null 2>&1 || (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source ~/.cargo/env)"
    echo "✓ Rust/Cargo available"

    # Sync source code
    echo ""
    echo "[5/10] Syncing pyrax-node source code..."
    rsync -avz --delete \
        --exclude 'target' \
        --exclude '.git' \
        --exclude 'node_modules' \
        ./pyrax-node/ $SERVER:$REMOTE_PATH/pyrax-node/
    echo "✓ Source synced"

    # Build on server
    echo ""
    echo "[6/10] Building pyrax-node on server (this may take a few minutes)..."
    ssh $SSH_OPTS $SERVER << 'ENDSSH'
source ~/.cargo/env
cd /opt/pyrax/pyrax-node
cargo build --release
cp target/release/pyrax-node /usr/local/bin/pyrax-node
chmod +x /usr/local/bin/pyrax-node
echo "Build complete: $(pyrax-node --version)"
ENDSSH
    echo "✓ Build complete"

    # Deploy systemd service
    echo ""
    echo "[7/10] Installing systemd service..."
    ssh $SSH_OPTS $SERVER << 'ENDSSH'
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

    # Start the node with fresh genesis
    echo ""
    echo "[8/10] Starting PYRAX node with fresh genesis..."
    ssh $SSH_OPTS $SERVER << 'ENDSSH'
# Stop existing node
systemctl stop pyrax-node 2>/dev/null || true
sleep 2

# CRITICAL: Wipe old data directory to regenerate genesis
echo "Wiping old chain data for fresh genesis..."
rm -rf /var/lib/pyrax/devnet/*
mkdir -p /var/lib/pyrax/devnet
chown -R root:root /var/lib/pyrax/devnet

# Start node - will create fresh genesis from current code
systemctl start pyrax-node
sleep 5

# Verify node is running
systemctl status pyrax-node --no-pager || true

# Show initial genesis hash
echo ""
echo "Checking genesis hash..."
sleep 3
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"pyrax_getChainInfo","params":[],"id":1}' \
    http://localhost:28545 | head -c 500
echo ""
ENDSSH
    echo "✓ Node started with fresh genesis"
else
    echo ""
    echo "[4-8/10] Skipping node deployment (--web-only mode)"
fi

# ==========================================
# WEB SERVICES DEPLOYMENT (if enabled)
# ==========================================
if [ "$DEPLOY_WEB" = true ]; then
    echo ""
    echo "============================================"
    echo "DEPLOYING WEB SERVICES (Docker Compose)"
    echo "============================================"

    # Check Docker is installed
    echo ""
    echo "[9/10] Checking Docker prerequisites..."
    ssh $SSH_OPTS $SERVER << 'ENDSSH'
if ! command -v docker &> /dev/null; then
    echo "Installing Docker..."
    curl -fsSL https://get.docker.com | sh
    systemctl enable docker
    systemctl start docker
fi
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "Installing Docker Compose..."
    curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose
fi
echo "Docker version: $(docker --version)"
ENDSSH
    echo "✓ Docker available"

    # Sync web service source code
    echo ""
    echo "[10/10] Syncing web services and deploying..."
    
    # Sync all web service directories
    for SERVICE in pyrax-website pyrax-explorer pyrax-faucet pyrax-docs pyrax-core-marketing; do
        if [ -d "./$SERVICE" ]; then
            echo "  Syncing $SERVICE..."
            rsync -avz --delete \
                --exclude 'node_modules' \
                --exclude '.next' \
                --exclude 'build' \
                --exclude '.git' \
                ./$SERVICE/ $SERVER:$REMOTE_PATH/$SERVICE/
        fi
    done
    
    # Sync deployment files
    rsync -avz ./docker-compose.devnet.yml $SERVER:$REMOTE_PATH/
    rsync -avz ./deployment/ $SERVER:$REMOTE_PATH/deployment/
    
    # Deploy with Docker Compose
    echo ""
    echo "Building and starting Docker containers..."
    ssh $SSH_OPTS $SERVER << 'ENDSSH'
cd /opt/pyrax
docker compose -f docker-compose.devnet.yml down --remove-orphans 2>/dev/null || true
docker compose -f docker-compose.devnet.yml build --parallel
docker compose -f docker-compose.devnet.yml up -d
sleep 10
docker compose -f docker-compose.devnet.yml ps
ENDSSH
    echo "✓ Web services deployed"
else
    echo ""
    echo "[9-10/10] Skipping web deployment (--node-only mode)"
fi

# ==========================================
# VERIFICATION
# ==========================================
echo ""
echo "============================================"
echo "Verifying services..."
echo "============================================"
sleep 5

if [ "$DEPLOY_NODE" = true ]; then
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
fi

if [ "$DEPLOY_WEB" = true ]; then
    # Check web services
    echo ""
    echo "Testing web services..."
    for URL in "http://$SERVER_IP" "http://$SERVER_IP:80"; do
        if curl -s -o /dev/null -w "%{http_code}" "$URL" 2>/dev/null | grep -q "200\|301\|302"; then
            echo "✓ Nginx is responding"
            break
        fi
    done
    
    # Show running containers
    echo ""
    echo "Docker containers status:"
    ssh $SSH_OPTS $SERVER "docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'" || true
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                   DEPLOYMENT COMPLETE!                         ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "PYRAX Devnet Services:"
echo ""
echo "  Blockchain (TriStream):"
echo "  • Stream A (RPC):     http://$SERVER_IP:$RPC_PORT"
echo "  • Stream B (Stratum): $SERVER_IP:$STRATUM_PORT"
echo "  • Stream C (Staking): http://$SERVER_IP:$STAKING_PORT"
echo "  • P2P:                /ip4/$SERVER_IP/tcp/$P2P_PORT"
echo ""
echo "  Web Services:"
echo "  • Website:    https://pyrax-devnet.org"
echo "  • Explorer:   https://explorer.pyrax-devnet.org"
echo "  • Faucet:     https://faucet.pyrax-devnet.org"
echo "  • Docs:       https://docs.pyrax-devnet.org"
echo "  • Marketing:  https://marketing.pyrax-devnet.org"
echo "  • RPC Proxy:  https://rpc.pyrax-devnet.org"
echo ""
echo "Useful commands:"
echo "  Node:"
echo "    ssh $SERVER 'systemctl status pyrax-node'"
echo "    ssh $SERVER 'journalctl -u pyrax-node -f'"
echo "    ssh $SERVER 'systemctl restart pyrax-node'"
echo ""
echo "  Docker:"
echo "    ssh $SERVER 'cd /opt/pyrax && docker compose -f docker-compose.devnet.yml ps'"
echo "    ssh $SERVER 'cd /opt/pyrax && docker compose -f docker-compose.devnet.yml logs -f'"
echo "    ssh $SERVER 'cd /opt/pyrax && docker compose -f docker-compose.devnet.yml restart'"
echo ""
echo "Test RPC:"
echo "  curl -X POST -H 'Content-Type: application/json' \\"
echo "    -d '{\"jsonrpc\":\"2.0\",\"method\":\"pyrax_getChainInfo\",\"params\":[],\"id\":1}' \\"
echo "    http://$SERVER_IP:$RPC_PORT"
echo ""
