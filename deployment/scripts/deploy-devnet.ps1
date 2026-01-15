# PYRAX Devnet Deployment Script for Windows
# Deploys the complete TriStream blockchain to Digital Ocean
# Usage: .\deploy-devnet.ps1

$ErrorActionPreference = "Stop"

# Configuration
$SERVER = "root@209.38.137.105"
$SERVER_IP = "209.38.137.105"

Write-Host "╔═══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║     PYRAX Devnet Deployment - TriStream DAG Blockchain        ║" -ForegroundColor Cyan
Write-Host "║   Stream A (BLAKE3) | Stream B (KAWPOW) | Stream C (ZK)       ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Server: $SERVER_IP"
Write-Host ""

# Test SSH connection
Write-Host "[1/6] Testing SSH connection..." -ForegroundColor Yellow
try {
    ssh -o ConnectTimeout=5 $SERVER "echo 'SSH OK'" 2>$null
    Write-Host "✓ SSH connection successful" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Cannot connect to $SERVER" -ForegroundColor Red
    Write-Host "Please ensure SSH access is configured."
    exit 1
}

# Create directories and setup
Write-Host ""
Write-Host "[2/6] Setting up server directories..." -ForegroundColor Yellow
ssh $SERVER @"
mkdir -p /opt/pyrax
mkdir -p /var/lib/pyrax/devnet
mkdir -p /var/log
"@
Write-Host "✓ Directories created" -ForegroundColor Green

# Check if pyrax-node binary exists on server, if not we need to build
Write-Host ""
Write-Host "[3/6] Checking for pyrax-node binary..." -ForegroundColor Yellow
$binaryExists = ssh $SERVER "test -f /usr/local/bin/pyrax-node && echo 'exists' || echo 'missing'"

if ($binaryExists -eq "missing") {
    Write-Host "Binary not found. Building on server..." -ForegroundColor Yellow
    
    # Sync source code
    Write-Host "Syncing source code..." -ForegroundColor Yellow
    $repoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
    scp -r "$repoRoot\pyrax-node" "${SERVER}:/opt/pyrax/"
    
    # Build on server
    Write-Host "Building (this may take several minutes)..." -ForegroundColor Yellow
    ssh $SERVER @"
source ~/.cargo/env 2>/dev/null || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
cd /opt/pyrax/pyrax-node
cargo build --release
cp target/release/pyrax-node /usr/local/bin/pyrax-node
chmod +x /usr/local/bin/pyrax-node
"@
}
Write-Host "✓ Binary ready" -ForegroundColor Green

# Configure firewall
Write-Host ""
Write-Host "[4/6] Configuring firewall..." -ForegroundColor Yellow
ssh $SERVER @"
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 28545/tcp
ufw allow 3333/tcp
ufw allow 28547/tcp
ufw allow 30303/tcp
ufw --force enable
"@
Write-Host "✓ Firewall configured" -ForegroundColor Green

# Deploy systemd service
Write-Host ""
Write-Host "[5/6] Installing and starting service..." -ForegroundColor Yellow
ssh $SERVER @'
cat > /etc/systemd/system/pyrax-node.service << 'EOF'
[Unit]
Description=PYRAX TriStream Blockchain Node (Devnet)
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/pyrax-node \
    --network devnet \
    --datadir /var/lib/pyrax/devnet \
    --rpc --rpc-addr 0.0.0.0:28545 \
    --p2p --p2p-addr /ip4/0.0.0.0/tcp/30303 \
    --stratum --stratum-addr 0.0.0.0:3333 \
    --staking --staking-addr 0.0.0.0:28547 \
    --verbosity 2
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable pyrax-node
systemctl restart pyrax-node
sleep 3
systemctl status pyrax-node --no-pager
'@
Write-Host "✓ Service started" -ForegroundColor Green

# Verify
Write-Host ""
Write-Host "[6/6] Verifying deployment..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

Write-Host "Checking ports..." -ForegroundColor Yellow
ssh $SERVER "netstat -tlnp | grep -E '(28545|3333|28547|30303)'"

Write-Host ""
Write-Host "╔═══════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║                   DEPLOYMENT COMPLETE!                         ║" -ForegroundColor Green
Write-Host "╚═══════════════════════════════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""
Write-Host "PYRAX Devnet Services:" -ForegroundColor Cyan
Write-Host "  • Stream A (RPC):     http://${SERVER_IP}:28545"
Write-Host "  • Stream B (Stratum): ${SERVER_IP}:3333"
Write-Host "  • Stream C (Staking): http://${SERVER_IP}:28547"
Write-Host "  • P2P:                /ip4/${SERVER_IP}/tcp/30303"
Write-Host ""
Write-Host "Test RPC:" -ForegroundColor Yellow
Write-Host "  Invoke-RestMethod -Uri 'http://${SERVER_IP}:28545' -Method POST -ContentType 'application/json' -Body '{`"jsonrpc`":`"2.0`",`"method`":`"pyrax_getChainInfo`",`"params`":[],`"id`":1}'"
Write-Host ""
