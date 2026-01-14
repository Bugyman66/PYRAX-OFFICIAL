#!/bin/bash
# PYRAX Node - Quick Build & Run Script for Linux
# Usage: curl -sSL https://raw.githubusercontent.com/PYRAX-Chain/PYRAX-OFFICIAL/main/scripts/build-and-run.sh | bash
# Or:    chmod +x build-and-run.sh && ./build-and-run.sh

set -e

REPO_URL="https://github.com/PYRAX-Chain/PYRAX-OFFICIAL.git"
INSTALL_DIR="/opt/pyrax"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║           PYRAX Node - Linux Build Script                  ║"
echo "╚════════════════════════════════════════════════════════════╝"

# ============================================
# Step 0: Clone Repository (if needed)
# ============================================
if [ ! -d "$INSTALL_DIR/pyrax-node" ]; then
    echo ""
    echo "[0/4] Cloning PYRAX repository..."
    sudo mkdir -p $INSTALL_DIR
    sudo chown $USER:$USER $INSTALL_DIR
    git clone $REPO_URL $INSTALL_DIR
else
    echo ""
    echo "[0/4] Repository already exists at $INSTALL_DIR"
    echo "      Pulling latest changes..."
    cd $INSTALL_DIR && git pull || true
fi

# ============================================
# Step 1: Install Dependencies
# ============================================
echo ""
echo "[1/4] Installing dependencies..."

if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev libclang-dev cmake git curl
elif command -v yum &> /dev/null; then
    sudo yum groupinstall -y "Development Tools"
    sudo yum install -y openssl-devel clang cmake git curl
elif command -v dnf &> /dev/null; then
    sudo dnf groupinstall -y "Development Tools"
    sudo dnf install -y openssl-devel clang cmake git curl
else
    echo "Warning: Package manager not detected. Please install build-essential, libssl-dev, libclang-dev manually."
fi

# ============================================
# Step 2: Install Rust
# ============================================
echo ""
echo "[2/4] Setting up Rust..."

if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust already installed: $(rustc --version)"
fi

# Ensure cargo is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Install nightly toolchain (required by project)
echo "Installing nightly-2025-01-01 toolchain..."
rustup toolchain install nightly-2025-01-01
rustup default nightly-2025-01-01

# ============================================
# Step 3: Build
# ============================================
echo ""
echo "[3/4] Building pyrax-node..."

# Navigate to pyrax-node directory (handle both /opt/pyrax and local paths)
if [ -d "/opt/pyrax/pyrax-node" ]; then
    cd /opt/pyrax/pyrax-node
elif [ -d "./pyrax-node" ]; then
    cd ./pyrax-node
elif [ -f "./Cargo.toml" ]; then
    # Already in pyrax-node directory
    :
else
    echo "ERROR: pyrax-node directory not found"
    exit 1
fi

echo "Building in: $(pwd)"
cargo build --release --bin pyrax-node

# ============================================
# Step 4: Run
# ============================================
echo ""
echo "[4/4] Build complete!"

BINARY_PATH="../target/release/pyrax-node"
if [ -f "$BINARY_PATH" ]; then
    echo ""
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                    BUILD SUCCESS!                          ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Binary: $BINARY_PATH"
    echo "Size: $(du -h $BINARY_PATH | cut -f1)"
    echo ""
    PEER_IP="0.0.0.0"  # Replace with actual peer IP
    
    echo "To connect to devnet:"
    echo "  $BINARY_PATH --network devnet --rpc --rpc-addr 0.0.0.0:8545 --p2p --peer /ip4/\$PEER_IP/tcp/30303"
    echo ""
    echo "To run in background:"
    echo "  nohup $BINARY_PATH --network devnet --rpc --rpc-addr 0.0.0.0:8545 --p2p --peer /ip4/\$PEER_IP/tcp/30303 > pyrax.log 2>&1 &"
    echo ""
    
    read -p "Connect to devnet now? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Connecting to PYRAX devnet..."
        $BINARY_PATH --network devnet --rpc --rpc-addr 0.0.0.0:8545 --p2p --peer /ip4/$PEER_IP/tcp/30303
    fi
else
    echo "ERROR: Binary not found at $BINARY_PATH"
    exit 1
fi
