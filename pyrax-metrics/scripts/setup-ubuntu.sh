#!/bin/bash
# PYRAX Metrics Node Setup for Ubuntu
# Run as: bash setup-ubuntu.sh

set -e

echo "====================================="
echo "  PYRAX Metrics Node Setup (Ubuntu)"
echo "====================================="
echo ""

# Update system
echo "[1/6] Updating system..."
sudo apt-get update -y
sudo apt-get upgrade -y

# Install dependencies
echo "[2/6] Installing dependencies..."
sudo apt-get install -y build-essential pkg-config libssl-dev git curl

# Install Rust
echo "[3/6] Installing Rust..."
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
else
    echo "    Rust already installed"
fi

# Clone repository
echo "[4/6] Cloning PYRAX repository..."
if [ ! -d "PYRAX-OFFICIAL" ]; then
    git clone -b metric https://github.com/PYRAX-Chain/PYRAX-OFFICIAL.git
else
    echo "    Repository already exists, pulling latest..."
    cd PYRAX-OFFICIAL && git checkout metric && git pull && cd ..
fi

# Build pyrax-metrics
echo "[5/6] Building pyrax-metrics..."
cd PYRAX-OFFICIAL/pyrax-metrics
cargo build --release

# Create systemd service
echo "[6/6] Creating systemd service..."
sudo tee /etc/systemd/system/pyrax-metrics.service > /dev/null << 'EOF'
[Unit]
Description=PYRAX Metrics Observer
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/PYRAX-OFFICIAL/pyrax-metrics
ExecStart=/root/PYRAX-OFFICIAL/target/release/pyrax-metrics --config config/devnet.toml
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable pyrax-metrics

echo ""
echo "====================================="
echo "  Setup Complete!"
echo "====================================="
echo ""
echo "  Start the service:"
echo "    sudo systemctl start pyrax-metrics"
echo ""
echo "  Check status:"
echo "    sudo systemctl status pyrax-metrics"
echo ""
echo "  View logs:"
echo "    journalctl -u pyrax-metrics -f"
echo ""
echo "  Endpoints:"
echo "    Health: http://YOUR_IP:8080/health"
echo "    Status: http://YOUR_IP:8080/status"
echo "    Metrics: http://YOUR_IP:9092/metrics"
echo ""
echo "  Open firewall:"
echo "    sudo ufw allow 9092"
echo "    sudo ufw allow 8080"
echo ""
echo "  For Grafana Cloud, use:"
echo "    http://YOUR_IP:9092/metrics"
echo ""
