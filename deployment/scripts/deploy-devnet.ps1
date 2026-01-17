# PYRAX Devnet Deployment Script for Windows
# Deploys the complete TriStream blockchain + web services to Digital Ocean
# Usage: .\deploy-devnet.ps1 [-NodeOnly] [-WebOnly] [-All]

param(
    [switch]$NodeOnly,
    [switch]$WebOnly,
    [switch]$All
)

$ErrorActionPreference = "Stop"

# Configuration
$SERVER = "root@209.38.137.105"
$SERVER_IP = "209.38.137.105"
$REMOTE_PATH = "/opt/pyrax"

# Determine what to deploy
$DeployNode = $true
$DeployWeb = $true

if ($NodeOnly) {
    $DeployWeb = $false
} elseif ($WebOnly) {
    $DeployNode = $false
}

Write-Host "╔═══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║     PYRAX Devnet Deployment - TriStream DAG Blockchain        ║" -ForegroundColor Cyan
Write-Host "║   Stream A (BLAKE3) | Stream B (KAWPOW) | Stream C (ZK)       ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Server: $SERVER_IP"
Write-Host "Deploy Node: $DeployNode | Deploy Web: $DeployWeb"
Write-Host ""

# Test SSH connection
Write-Host "[1/10] Testing SSH connection..." -ForegroundColor Yellow
$sshResult = ssh -o ConnectTimeout=5 $SERVER "echo 'SSH OK'" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Cannot connect to $SERVER" -ForegroundColor Red
    Write-Host "Please ensure SSH access is configured."
    exit 1
}
Write-Host "OK: SSH connection successful" -ForegroundColor Green

# Create directories and setup
Write-Host ""
Write-Host "[2/10] Setting up server directories..." -ForegroundColor Yellow
ssh $SERVER @"
mkdir -p /opt/pyrax
mkdir -p /var/lib/pyrax/devnet
mkdir -p /var/log
"@
Write-Host "OK: Directories created" -ForegroundColor Green

# Configure firewall (always needed)
Write-Host ""
Write-Host "[3/10] Configuring firewall..." -ForegroundColor Yellow
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
Write-Host "OK: Firewall configured" -ForegroundColor Green

# ==========================================
# NODE DEPLOYMENT (if enabled)
# ==========================================
if ($DeployNode) {
    Write-Host ""
    Write-Host "============================================" -ForegroundColor Cyan
    Write-Host "DEPLOYING PYRAX NODE (TriStream Blockchain)" -ForegroundColor Cyan
    Write-Host "============================================" -ForegroundColor Cyan

    Write-Host ""
    Write-Host "[4/10] Checking for pyrax-node binary..." -ForegroundColor Yellow
    $repoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
    
    # Sync source code using rsync
    Write-Host "[5/10] Syncing pyrax-node source..." -ForegroundColor Yellow
    rsync -avz --delete --exclude=target --exclude=.git "$repoRoot\pyrax-node\" "${SERVER}:/opt/pyrax/pyrax-node/"
    Write-Host "OK: Source synced" -ForegroundColor Green
    
    # Build on server
    Write-Host ""
    Write-Host "[6/10] Building pyrax-node (this may take several minutes)..." -ForegroundColor Yellow
    ssh $SERVER @"
source ~/.cargo/env 2>/dev/null || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
cd /opt/pyrax/pyrax-node
cargo build --release
cp target/release/pyrax-node /usr/local/bin/pyrax-node
chmod +x /usr/local/bin/pyrax-node
echo "Build complete"
"@
    Write-Host "OK: Build complete" -ForegroundColor Green

    # Deploy systemd service
    Write-Host ""
    Write-Host "[7/10] Installing systemd service..." -ForegroundColor Yellow
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
'@
    Write-Host "OK: Systemd service installed" -ForegroundColor Green

    Write-Host ""
    Write-Host "[8/10] Starting PYRAX node..." -ForegroundColor Yellow
    ssh $SERVER @'
systemctl restart pyrax-node
sleep 3
systemctl status pyrax-node --no-pager
'@
    Write-Host "OK: Node started" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[4-8/10] Skipping node deployment (-WebOnly mode)" -ForegroundColor Gray
}

# ==========================================
# WEB SERVICES DEPLOYMENT (if enabled)
# ==========================================
if ($DeployWeb) {
    Write-Host ""
    Write-Host "============================================" -ForegroundColor Cyan
    Write-Host "DEPLOYING WEB SERVICES (Docker Compose)" -ForegroundColor Cyan
    Write-Host "============================================" -ForegroundColor Cyan

    Write-Host ""
    Write-Host "[7/10] Checking Docker prerequisites..." -ForegroundColor Yellow
    ssh $SERVER @'
if ! command -v docker &> /dev/null; then
    echo "Installing Docker..."
    curl -fsSL https://get.docker.com | sh
    systemctl enable docker
    systemctl start docker
fi
echo "Docker version: $(docker --version)"
'@
    Write-Host "OK: Docker available" -ForegroundColor Green

    Write-Host ""
    Write-Host "[8/10] Syncing web services..." -ForegroundColor Yellow
    
    $repoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
    
    # Sync web service directories
    $services = @("pyrax-website", "pyrax-explorer", "pyrax-faucet", "pyrax-docs", "pyrax-core-marketing")
    foreach ($service in $services) {
        $servicePath = Join-Path $repoRoot $service
        if (Test-Path $servicePath) {
            Write-Host "  Syncing $service..." -ForegroundColor Gray
            # Use rsync via ssh or scp
            $excludes = "--exclude=node_modules --exclude=.next --exclude=build --exclude=.git"
            ssh $SERVER "mkdir -p $REMOTE_PATH/$service"
            rsync -avz --delete $excludes "$servicePath/" "${SERVER}:${REMOTE_PATH}/${service}/"
        }
    }
    
    # Sync deployment files
    Write-Host "  Syncing deployment config..." -ForegroundColor Gray
    scp "$repoRoot\docker-compose.devnet.yml" "${SERVER}:${REMOTE_PATH}/"
    rsync -avz "$repoRoot\deployment\" "${SERVER}:${REMOTE_PATH}/deployment/"
    
    Write-Host "OK: Source synced" -ForegroundColor Green

    Write-Host ""
    Write-Host "[9/10] Building and starting Docker containers..." -ForegroundColor Yellow
    ssh $SERVER @'
cd /opt/pyrax
docker compose -f docker-compose.devnet.yml down --remove-orphans 2>/dev/null || true
docker compose -f docker-compose.devnet.yml build --parallel
docker compose -f docker-compose.devnet.yml up -d
sleep 10
docker compose -f docker-compose.devnet.yml ps
'@
    Write-Host "OK: Web services deployed" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[7-9/10] Skipping web deployment (-NodeOnly mode)" -ForegroundColor Gray
}

# ==========================================
# VERIFICATION
# ==========================================
Write-Host ""
Write-Host "[10/10] Verifying deployment..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

if ($DeployNode) {
    Write-Host "Checking node ports..." -ForegroundColor Yellow
    ssh $SERVER "netstat -tlnp | grep -E '(28545|3333|28547|30303)'" 2>$null
}

if ($DeployWeb) {
    Write-Host ""
    Write-Host "Docker containers status:" -ForegroundColor Yellow
    ssh $SERVER "docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'" 2>$null
}

Write-Host ""
Write-Host "╔═══════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║                   DEPLOYMENT COMPLETE!                         ║" -ForegroundColor Green
Write-Host "╚═══════════════════════════════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""
Write-Host "PYRAX Devnet Services:" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Blockchain (TriStream):" -ForegroundColor White
Write-Host "  • Stream A (RPC):     http://${SERVER_IP}:28545"
Write-Host "  • Stream B (Stratum): ${SERVER_IP}:3333"
Write-Host "  • Stream C (Staking): http://${SERVER_IP}:28547"
Write-Host "  • P2P:                /ip4/${SERVER_IP}/tcp/30303"
Write-Host ""
Write-Host "  Web Services:" -ForegroundColor White
Write-Host "  • Website:    https://pyrax-devnet.org"
Write-Host "  • Explorer:   https://explorer.pyrax-devnet.org"
Write-Host "  • Faucet:     https://faucet.pyrax-devnet.org"
Write-Host "  • Docs:       https://docs.pyrax-devnet.org"
Write-Host "  • Marketing:  https://marketing.pyrax-devnet.org"
Write-Host "  • RPC Proxy:  https://rpc.pyrax-devnet.org"
Write-Host ""
Write-Host "Useful commands:" -ForegroundColor Yellow
Write-Host "  Node:"
Write-Host "    ssh $SERVER 'systemctl status pyrax-node'"
Write-Host "    ssh $SERVER 'journalctl -u pyrax-node -f'"
Write-Host ""
Write-Host "  Docker:"
Write-Host "    ssh $SERVER 'cd /opt/pyrax && docker compose -f docker-compose.devnet.yml ps'"
Write-Host "    ssh $SERVER 'cd /opt/pyrax && docker compose -f docker-compose.devnet.yml logs -f'"
Write-Host ""
Write-Host "Test RPC:" -ForegroundColor Yellow
Write-Host "  Invoke-RestMethod -Uri 'http://${SERVER_IP}:28545' -Method POST -ContentType 'application/json' -Body '{`"jsonrpc`":`"2.0`",`"method`":`"pyrax_getChainInfo`",`"params`":[],`"id`":1}'"
Write-Host ""
