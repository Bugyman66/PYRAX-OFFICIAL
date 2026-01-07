# Rust Development Setup

This guide covers setting up your environment for WASM/Rust smart contract development on PYRAX.

## Prerequisites

### Install Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart shell or run
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Add WASM Target

```bash
# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Verify
rustup target list --installed
```

### Install Build Tools

```bash
# Install wasm-pack (optional, but recommended)
cargo install wasm-pack

# Install cargo-generate for templates
cargo install cargo-generate

# Install PYRAX CLI
cargo install pyrax-cli
```

## Project Setup

### Using Template (Recommended)

```bash
# Generate from PYRAX template
cargo generate --git https://github.com/PYRAX-Chain/wasm-template

# Enter project name when prompted
cd my-pyrax-contract
```

### Manual Setup

```bash
# Create new library
cargo new --lib my-contract
cd my-contract
```

Add to `Cargo.toml`:

```toml
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
pyrax-sdk = "0.1"
serde = { version = "1.0", features = ["derive"] }

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
panic = "abort"     # Abort on panic
```

## Project Structure

```
my-contract/
├── Cargo.toml          # Dependencies and config
├── src/
│   ├── lib.rs          # Main contract code
│   ├── state.rs        # State definitions
│   ├── msg.rs          # Message types
│   └── error.rs        # Custom errors
├── tests/
│   └── integration.rs  # Integration tests
└── scripts/
    └── deploy.sh       # Deployment script
```

## IDE Setup

### VS Code

Install extensions:
- **rust-analyzer** — Rust language support
- **Even Better TOML** — TOML file support
- **crates** — Dependency management

Add to `.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

### IntelliJ IDEA

Install plugins:
- **Rust** — Official Rust plugin
- **TOML** — TOML support

## Building Contracts

### Development Build

```bash
cargo build --target wasm32-unknown-unknown
```

### Optimized Release Build

```bash
cargo build --release --target wasm32-unknown-unknown
```

### Optimize WASM Size

```bash
# Install wasm-opt
cargo install wasm-opt

# Optimize
wasm-opt -Oz \
  target/wasm32-unknown-unknown/release/my_contract.wasm \
  -o my_contract_optimized.wasm
```

## Testing

### Unit Tests

```bash
# Run unit tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Run integration tests against local node
cargo test --test integration
```

### Test in WASM Environment

```bash
# Install wasm-pack if not already
cargo install wasm-pack

# Run tests in WASM
wasm-pack test --node
```

## PYRAX CLI

### Commands

```bash
# Check contract
pyrax-cli contract check ./my_contract.wasm

# Deploy contract
pyrax-cli contract deploy ./my_contract.wasm \
  --network testnet \
  --private-key $PRIVATE_KEY

# Call contract
pyrax-cli contract call CONTRACT_ADDRESS \
  --method "increment" \
  --network testnet

# Query contract
pyrax-cli contract query CONTRACT_ADDRESS \
  --method "get" \
  --network testnet
```

## Environment Variables

Create `.env`:
```bash
# Network
PYRAX_NETWORK=testnet
PYRAX_RPC_URL=https://testnet.pyrax.org

# Keys (don't commit!)
PRIVATE_KEY=your_private_key_here

# Build
RUSTFLAGS="-C link-arg=-s"
```

## Troubleshooting

### "target not installed"

```bash
rustup target add wasm32-unknown-unknown
```

### "linker not found"

```bash
# Ubuntu/Debian
sudo apt install lld

# macOS
brew install llvm
```

### Large WASM files

- Enable LTO in Cargo.toml
- Use `opt-level = "z"`
- Run wasm-opt after build
- Remove debug symbols

### Compilation errors

- Check Rust version: `rustc --version`
- Update dependencies: `cargo update`
- Check target: `cargo build --target wasm32-unknown-unknown`

---

:::tip Next Steps
Environment ready! Continue to:
- [Contract Development](./rust-contract-development) — Write your first contract
:::
