# PYRAX Build Instructions

## Prerequisites

### Required Software
- **Rust**: 1.85.0-nightly or later
- **Git**: 2.40+
- **Operating System**: Windows 10/11, Linux, or macOS

### Install Rust (if not installed)
```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install nightly toolchain
rustup install nightly
rustup default nightly
```

## Build Steps

### 1. Clone Repository
```bash
git clone https://github.com/pyrax-org/pyrax.git
cd pyrax
```

### 2. Build pyrax-node (Release)
```bash
cd pyrax-node
cargo build --release
```

### 3. Verify Build
The binary will be located at:
- **Windows**: `target/release/pyrax-node.exe`
- **Linux/macOS**: `target/release/pyrax-node`

### 4. Verify Hash (Windows PowerShell)
```powershell
(Get-FileHash target/release/pyrax-node.exe -Algorithm SHA256).Hash
```

### 4. Verify Hash (Linux/macOS)
```bash
sha256sum target/release/pyrax-node
```

## Expected Build Artifacts

| Component | File | Description |
|-----------|------|-------------|
| pyrax-node | `target/release/pyrax-node(.exe)` | Full node binary |

## Build Verification

### Verification Steps
1. Build completes without errors
2. Binary runs and shows version: `./pyrax-node --version`
3. Node starts successfully: `./pyrax-node --network devnet --datadir ./test`
4. RPC responds: `curl -X POST http://127.0.0.1:8545 -d '{"jsonrpc":"2.0","method":"pyrax_getChainInfo","id":1}'`

### Build Requirements
| Requirement | Version |
|-------------|---------|
| Rust | 1.85.0-nightly or later |
| Cargo | 1.85.0-nightly or later |

**Note**: Binary hashes may vary between builds due to Rust compiler metadata. Verification is based on functionality, not hash matching.

## Running the Node

### Start a node on devnet
```bash
./pyrax-node --network devnet --datadir ./data --p2p --rpc
```

### Start a mining node
```bash
./pyrax-node --network devnet --datadir ./data --p2p --rpc --mine --miner-address 0xYOUR_ADDRESS
```

### Connect to existing peer
```bash
./pyrax-node --network devnet --datadir ./data --p2p --rpc --peer /ip4/PEER_IP/tcp/30303
```

## Troubleshooting

### Build fails with missing dependencies
Ensure you have the required build tools:
- **Windows**: Install Visual Studio Build Tools
- **Linux**: `sudo apt install build-essential pkg-config libssl-dev`
- **macOS**: `xcode-select --install`

### Hash mismatch
1. Ensure you're using the exact Rust version specified
2. Clean build: `cargo clean && cargo build --release`
3. Check for local modifications: `git status`

## Development Build
For development with debug symbols:
```bash
cargo build
```

## Running Tests
```bash
cargo test
```
