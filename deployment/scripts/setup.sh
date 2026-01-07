#!/bin/bash
# PYRAX Droplet Setup Script
# Usage: sudo ./setup.sh <environment>
# Example: sudo ./setup.sh devnet

set -e

ENV=${1:-devnet}
VALID_ENVS="devnet testnet mainnet"

if [[ ! " $VALID_ENVS " =~ " $ENV " ]]; then
    echo "Error: Invalid environment '$ENV'"
    echo "Usage: sudo ./setup.sh <devnet|testnet|mainnet>"
    exit 1
fi

echo "============================================"
echo "PYRAX Setup Script - $ENV Environment"
echo "============================================"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root (sudo)"
    exit 1
fi

# ===========================================
# System Updates
# ===========================================
echo "[1/8] Updating system packages..."
apt-get update && apt-get upgrade -y

# ===========================================
# Install Dependencies
# ===========================================
echo "[2/8] Installing dependencies..."
apt-get install -y \
    nginx \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    postgresql \
    postgresql-contrib \
    ufw \
    fail2ban

# Install Node.js 20 LTS
if ! command -v node &> /dev/null; then
    echo "Installing Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
    apt-get install -y nodejs
fi

# Install Rust
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# ===========================================
# Create pyrax user
# ===========================================
echo "[3/8] Creating pyrax user..."
if ! id "pyrax" &>/dev/null; then
    useradd -r -m -s /bin/bash pyrax
fi

# ===========================================
# Setup directories
# ===========================================
echo "[4/8] Setting up directories..."
mkdir -p /opt/pyrax
mkdir -p /var/lib/pyrax/$ENV
mkdir -p /var/log/pyrax
mkdir -p /etc/ssl/cloudflare
mkdir -p /var/www/pyrax.org

chown -R pyrax:pyrax /opt/pyrax
chown -R pyrax:pyrax /var/lib/pyrax
chown -R pyrax:pyrax /var/log/pyrax

# ===========================================
# Setup Cloudflare SSL
# ===========================================
echo "[5/8] Setting up Cloudflare SSL..."
if [ ! -f /etc/ssl/cloudflare/pyrax.org.pem ]; then
    echo "WARNING: Cloudflare Origin Certificate not found!"
    echo "Please copy your certificates to:"
    echo "  /etc/ssl/cloudflare/pyrax.org.pem (certificate)"
    echo "  /etc/ssl/cloudflare/pyrax.org.key (private key)"
    echo ""
    echo "Generate these in Cloudflare Dashboard:"
    echo "  SSL/TLS → Origin Server → Create Certificate"
fi

# ===========================================
# Setup Nginx
# ===========================================
echo "[6/8] Configuring Nginx..."
rm -f /etc/nginx/sites-enabled/default
cp /opt/pyrax/deployment/nginx/$ENV.conf /etc/nginx/sites-available/pyrax.conf
ln -sf /etc/nginx/sites-available/pyrax.conf /etc/nginx/sites-enabled/pyrax.conf

# Test nginx config
nginx -t

# ===========================================
# Setup Systemd Services
# ===========================================
echo "[7/8] Setting up systemd services..."
cp /opt/pyrax/deployment/systemd/pyrax-node.service /etc/systemd/system/pyrax-node@.service
cp /opt/pyrax/deployment/systemd/pyrax-explorer.service /etc/systemd/system/pyrax-explorer@.service
cp /opt/pyrax/deployment/systemd/pyrax-api.service /etc/systemd/system/pyrax-api@.service

systemctl daemon-reload
systemctl enable pyrax-node@$ENV
systemctl enable pyrax-explorer@$ENV
systemctl enable pyrax-api@$ENV
systemctl enable nginx

# ===========================================
# Setup Firewall
# ===========================================
echo "[8/8] Configuring firewall..."
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow http
ufw allow https
ufw allow 30303/tcp  # P2P
ufw allow 30303/udp  # P2P discovery
ufw --force enable

# Enable fail2ban
systemctl enable fail2ban
systemctl start fail2ban

echo ""
echo "============================================"
echo "Setup Complete!"
echo "============================================"
echo ""
echo "Next steps:"
echo "1. Copy Cloudflare Origin Certificates to /etc/ssl/cloudflare/"
echo "2. Build the pyrax-node: cd /opt/pyrax/pyrax-node && cargo build --release"
echo "3. Build the explorer: cd /opt/pyrax/pyrax-explorer && npm install && npm run build"
echo "4. Start services:"
echo "   sudo systemctl start pyrax-node@$ENV"
echo "   sudo systemctl start pyrax-explorer@$ENV"
echo "   sudo systemctl start pyrax-api@$ENV"
echo "   sudo systemctl restart nginx"
echo ""
echo "Check status:"
echo "   sudo systemctl status pyrax-node@$ENV"
echo "   sudo journalctl -u pyrax-node@$ENV -f"
echo ""
