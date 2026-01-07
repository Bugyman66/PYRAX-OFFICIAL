# Developer Overview

Welcome to the PYRAX developer documentation. This section provides technical guides, API references, and resources for building on the PYRAX network.

## What Can You Build?

### Decentralized Applications (dApps)
Build applications that interact with the PYRAX blockchain:
- DeFi protocols
- NFT marketplaces
- Gaming applications
- DAO tools

### AI-Powered Applications
Leverage Crucible for decentralized AI computing:
- AI inference services
- Model training pipelines
- Data processing applications

### Mining & Infrastructure
Develop tools for the mining ecosystem:
- Mining pool software
- Monitoring dashboards
- Wallet integrations

### Developer Tools
Create utilities for other developers:
- SDKs and libraries
- Block explorers
- Analytics platforms

## Technical Stack

### Blockchain Layer
| Component | Technology |
|-----------|------------|
| Consensus | Proof of Work (KAWPOW) |
| Block Time | ~60 seconds |
| Smart Contracts | EVM Compatible |
| Native Token | PYRAX |

### Smart Contracts
- Solidity 0.8+ supported
- OpenZeppelin compatible
- Standard ERC token support
- Custom PYRAX extensions

### APIs & SDKs
- JSON-RPC API (Ethereum compatible)
- REST API for Crucible
- JavaScript/TypeScript SDK
- Python SDK

## Quick Links

| Resource | Description |
|----------|-------------|
| [Quickstart](./quickstart) | Get started in 5 minutes |
| [Architecture](./architecture) | System design overview |
| [API Reference](./api-reference) | Complete API documentation |
| [Smart Contracts](./smart-contracts-intro) | Contract development guide |
| [Crucible Integration](./crucible-integration) | AI compute integration |

## Development Environment

### Recommended Setup

**Operating System:**
- Linux (Ubuntu 22.04 recommended)
- macOS
- Windows with WSL2

**Languages:**
- Solidity for smart contracts
- JavaScript/TypeScript for dApps
- Python for scripts and tools
- Rust for core contributions

**Tools:**
- Node.js 18+
- Hardhat or Foundry
- Git
- Docker (for local nodes)

### Network Endpoints

| Network | RPC URL | Chain ID |
|---------|---------|----------|
| Mainnet | `https://rpc.pyrax.org` | TBD |
| Testnet | `https://testnet.pyrax.org` | TBD |

## Getting Started Path

### For Smart Contract Developers
1. [Quickstart](./quickstart) — Set up your environment
2. [Smart Contracts Intro](./smart-contracts-intro) — Learn the basics
3. [Contract Development](./contract-development) — Build your first contract
4. [Testnet](./testnet) — Deploy and test

### For dApp Developers
1. [Quickstart](./quickstart) — Environment setup
2. [JavaScript SDK](./javascript-sdk) — Frontend integration
3. [RPC Endpoints](./rpc-endpoints) — Connect to the network
4. [API Reference](./api-reference) — Full API documentation

### For AI/Crucible Developers
1. [Crucible Overview](./crucible-overview) — Understand the platform
2. [Crucible Integration](./crucible-integration) — Connect your application
3. [Job Submission](./job-submission) — Submit AI jobs
4. [GPU Provider](./gpu-provider) — Become a compute provider

### For Node Operators
1. [Running a Node](./running-a-node) — Set up a full node
2. [Node Configuration](./node-configuration) — Optimize settings
3. [Node Maintenance](./node-maintenance) — Ongoing operations

## Developer Resources

### Code Repositories
- **Core Protocol**: [github.com/PYRAX-Chain/pyrax-core](https://github.com/PYRAX-Chain/pyrax-core)
- **Smart Contracts**: [github.com/PYRAX-Chain/contracts](https://github.com/PYRAX-Chain/contracts)
- **SDKs**: [github.com/PYRAX-Chain/pyrax-sdk](https://github.com/PYRAX-Chain/pyrax-sdk)
- **Documentation**: [github.com/PYRAX-Chain/docs](https://github.com/PYRAX-Chain/docs)

### Community
- **Discord Developer Channel**: Technical discussions
- **GitHub Issues**: Bug reports and feature requests
- **Developer Forum**: Long-form technical discussions

### Grants & Funding
The PYRAX DAO funds developer projects:
- Builder grants for tools and infrastructure
- Integration grants for bringing projects to PYRAX
- Research grants for innovative solutions

See [DAO Overview](/general/dao-overview) for grant information.

## Support

### Getting Help
- **Discord**: #dev-support channel
- **GitHub**: Open issues for bugs
- **Stack Overflow**: Tag with `pyrax`

### Contributing
We welcome contributions:
1. Fork the repository
2. Create feature branch
3. Submit pull request
4. Follow contribution guidelines

---

:::tip Ready to Build?
Start with the [Quickstart Guide](./quickstart) to set up your development environment in minutes.
:::
