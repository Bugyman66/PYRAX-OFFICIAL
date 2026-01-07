#!/bin/bash
# PYRAX Initial Droplet Setup Script
# Run this once on a fresh Digital Ocean droplet
# Usage: curl -sSL https://raw.githubusercontent.com/PYRAX-Chain/PYRAX-OFFICIAL/main/deployment/scripts/initial-setup.sh | sudo bash

set -e

echo "============================================"
echo "PYRAX Initial Server Setup"
echo "============================================"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root (sudo)"
    exit 1
fi

# ===========================================
# System Updates
# ===========================================
echo "[1/7] Updating system packages..."
apt-get update && apt-get upgrade -y

# ===========================================
# Install Docker
# ===========================================
echo "[2/7] Installing Docker..."
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
    rm get-docker.sh
    
    # Enable Docker service
    systemctl enable docker
    systemctl start docker
fi

# Install Docker Compose plugin
echo "[3/7] Installing Docker Compose..."
apt-get install -y docker-compose-plugin

# ===========================================
# Install essential tools
# ===========================================
echo "[4/7] Installing essential tools..."
apt-get install -y \
    git \
    curl \
    wget \
    htop \
    ufw \
    fail2ban \
    unzip

# ===========================================
# Setup firewall
# ===========================================
echo "[5/7] Configuring firewall..."
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

# ===========================================
# Setup directories
# ===========================================
echo "[6/7] Setting up directories..."
mkdir -p /opt/pyrax
mkdir -p /opt/pyrax/deployment/ssl
mkdir -p /var/log/pyrax

# ===========================================
# Clone repository
# ===========================================
echo "[7/7] Cloning PYRAX repository..."
if [ -d /opt/pyrax/.git ]; then
    cd /opt/pyrax
    git pull origin main
else
    git clone https://github.com/PYRAX-Chain/PYRAX-OFFICIAL.git /opt/pyrax
fi

cd /opt/pyrax

echo ""
echo "============================================"
echo "Initial Setup Complete!"
echo "============================================"
echo ""
echo "Next steps:"
echo ""
echo "1. Add Cloudflare Origin SSL Certificates:"
echo "   - Create certificate at Cloudflare Dashboard → SSL/TLS → Origin Server"
echo "   - Save to: /opt/pyrax/deployment/ssl/pyrax.org.pem"
echo "   - Save to: /opt/pyrax/deployment/ssl/pyrax.org.key"
echo ""
echo "2. Configure environment:"
echo "   cp /opt/pyrax/.env.example /opt/pyrax/.env"
echo "   nano /opt/pyrax/.env"
echo ""
echo "3. Deploy services:"
echo "   cd /opt/pyrax"
echo "   docker compose up -d"
echo ""
echo "4. Check status:"
echo "   docker compose ps"
echo "   docker compose logs -f"
echo ""
echo "5. Setup GitHub Actions for auto-deploy:"
echo "   Add these secrets to your GitHub repository:"
echo "   - DROPLET_IP: $(curl -s ifconfig.me)"
echo "   - DROPLET_SSH_KEY: (your SSH private key)"
echo "   - FAUCET_PRIVATE_KEY: (faucet wallet private key)"
echo ""
