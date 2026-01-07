# PYRAX Blockchain

> **A Proof-of-Work blockchain with integrated AI/ML compute marketplace**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Website](https://img.shields.io/badge/website-pyrax.org-green.svg)](https://pyrax.org)

## Overview

PYRAX is a next-generation Layer 1 blockchain that combines:
- **KAWPOW Proof-of-Work**: ASIC-resistant GPU mining algorithm
- **AI Compute Marketplace**: Decentralized AI job execution and settlement
- **Dual-Purpose Mining**: Miners secure the network AND provide AI compute

## Quick Start

### Prerequisites
- Rust 1.85.0-nightly or later
- Git 2.40+

### Build and Run

```bash
# Clone repository
git clone https://github.com/pyrax-official/pyrax.git
cd pyrax

# Build pyrax-node
cd pyrax-node
cargo build --release

# Run node on devnet
../target/release/pyrax-node --network devnet --datadir ./data --p2p --rpc

# Run mining node
../target/release/pyrax-node --network devnet --datadir ./data --p2p --rpc \
  --mine --miner-address 0xYOUR_ADDRESS
```

### Join the Testnet
See [TESTNET.md](TESTNET.md) for detailed instructions on joining the devnet.

### Build Instructions
See [BUILD.md](BUILD.md) for complete build documentation.

### Components

| Component | Description | Language |
|-----------|-------------|----------|
| `pyrax-node` | Full node (consensus, P2P, RPC) | Rust |
| `pyrax-wallet` | Wallet library | Rust |
| `pyrax-miner` | GPU miner (KAWPOW) | C++/CUDA |
| `pyrax-desktop` | Desktop application | Tauri/Rust |
| `pyrax-explorer` | Block explorer | TypeScript |
| `pyrax-ai` | AI marketplace | Rust/TypeScript |

## Documentation

- [Protocol Specification](docs/windsurf-spec.md)
- [Build Plan](build_plan.md)
- [Technical Overview](overview.md)
- [Threat Model](docs/threat-model.md)
- [Genesis Documentation](docs/GENESIS.md)

## Network Information

| Network | Chain ID | RPC Endpoint |
|---------|----------|--------------|
| Mainnet | 1 | https://rpc.pyrax.org |
| Testnet | 2 | https://testnet-rpc.pyrax.org |
| Devnet | 3 | http://localhost:8545 |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Security

Report security vulnerabilities to security@pyrax.org. See [threat-model.md](docs/threat-model.md).

## License

MIT License - see [LICENSE](LICENSE)
