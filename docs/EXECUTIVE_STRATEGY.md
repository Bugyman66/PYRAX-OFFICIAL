# PYRAX Executive Strategy

> **Version:** 2.0 | **Updated:** 2026-01-05 | **Status:** ACTIVE

---

## Table of Contents
1. [Executive Summary](#1-executive-summary)
2. [Vision & Mission](#2-vision--mission)
3. [Market Analysis](#3-market-analysis)
4. [Technical Architecture](#4-technical-architecture)
5. [Product Portfolio](#5-product-portfolio)
6. [AI Compute Platform](#6-ai-compute-platform)
7. [Token Economics](#7-token-economics)
8. [Go-to-Market Strategy](#8-go-to-market-strategy)
9. [Roadmap](#9-roadmap)
10. [Risk Management](#10-risk-management)

---

## 1. Executive Summary

### The Opportunity
The convergence of blockchain infrastructure and AI compute represents a **transformational market opportunity**. PYRAX uniquely positions itself at this intersection by providing:

1. **TriStream DAG Layer 1** - Three parallel mining streams (ASIC, GPU, ZK) for maximum security and decentralization
2. **EVM Sidechain** - Full Solidity compatibility with 2-second blocks and 1k-5k TPS
3. **ZK-Rollups** - Layer 2 scaling to 500k+ TPS for high-throughput applications
4. **Native AI Marketplace** - Decentralized compute marketplace with on-chain job settlement
5. **Dual-Purpose Mining** - GPU miners secure the network AND provide AI compute power

### Key Differentiators

| Feature | PYRAX | Traditional L1s |
|---------|-------|-----------------|
| Consensus | TriStream DAG (BLAKE3 + KAWPOW + ZK) | Single algorithm |
| Mining Streams | 3 parallel streams (ASIC, GPU, ZK) | Single stream |
| AI Integration | Native on-chain job settlement | External bridges required |
| Smart Contracts | EVM + Rust/WASM dual VM | Usually single VM |
| Block Time | 6-10s L1, 2s EVM sidechain | 10-60s typical |
| L2 Support | Native ZK-Rollups (500k+ TPS) | Third-party solutions |

### Current Status
- ✅ Core node implementation complete (Phase 1-2)
- ✅ P2P networking with libp2p operational
- ✅ Desktop wallet application functional
- ✅ GPU miner (CUDA/OpenCL) complete
- ✅ EVM sidechain with full eth_* RPC
- ✅ Rust/WASM smart contracts runtime
- ✅ AI compute infrastructure production-ready
- ✅ ZK-Rollup framework implemented
- ✅ Block explorer with real-time data
- 🔄 Mainnet launch preparation in progress

---

## 2. Vision & Mission

### Vision
*"To become the foundational infrastructure layer where blockchain security meets decentralized AI compute, enabling a new era of accessible, verifiable artificial intelligence."*

### Mission
*"PYRAX democratizes access to AI compute by creating an open marketplace where anyone can contribute GPU resources, access AI capabilities, and have all transactions settled transparently on a secure, high-performance blockchain."*

### Core Values

| Value | Description |
|-------|-------------|
| **Decentralization** | No single point of failure, globally distributed network |
| **Accessibility** | AI compute for everyone, low barriers to entry |
| **Security** | Cryptographic guarantees, multi-stream consensus |
| **Performance** | Speed without sacrifice, L2 scaling |
| **Transparency** | Open source code, on-chain verification |

---

## 3. Market Analysis

### Total Addressable Market

| Segment | 2024 | 2027 | 2030 |
|---------|------|------|------|
| Blockchain Infrastructure | $65B | $150B | $300B |
| AI/ML Cloud Services | $50B | $120B | $250B |
| Decentralized AI Compute | $2B | $25B | $100B |
| GPU Mining Hardware | $10B | $20B | $35B |

### Target Customer Segments

**Primary Segments:**

| Segment | Need | PYRAX Solution |
|---------|------|----------------|
| **AI Developers** | Affordable GPU compute | Decentralized compute marketplace |
| **GPU Miners** | Maximize hardware ROI | Dual-purpose mining (security + AI jobs) |
| **Enterprise AI Teams** | Scalable, verifiable compute | Multi-node consensus verification |
| **dApp Developers** | AI-enhanced smart contracts | Native AI opcodes in EVM |

**Secondary Segments:**
- Data centers seeking compute monetization
- Academic institutions requiring research compute
- Game studios needing AI asset generation
- Content creators using generative AI

### Competitive Landscape

| Competitor | Focus | PYRAX Advantage |
|------------|-------|-----------------|
| Render Network | Graphics rendering | Broader AI workloads, native blockchain |
| Akash Network | General compute | AI-optimized, dual-purpose mining |
| io.net | AI compute | Native L1 blockchain, ZK verification |
| Bittensor | AI model training | Simpler tokenomics, EVM compatibility |
| Filecoin | Storage | Compute-focused, AI marketplace |

---

## 4. Technical Architecture

### TriStream DAG Consensus

PYRAX implements a unique three-stream Directed Acyclic Graph (DAG) consensus mechanism using GHOSTDAG ordering:

```
┌─────────────────────────────────────────────────────────────────┐
│ LAYER 3: ZK-ROLLUPS (500k+ TPS)                                 │
│   AI Compute | DeFi | Gaming | High-Frequency Applications      │
├─────────────────────────────────────────────────────────────────┤
│ LAYER 2: EVM SIDECHAIN (1k-5k TPS, 2s blocks)                   │
│   Solidity 0.8.x | Rust/WASM | L1 Bridge                        │
├─────────────────────────────────────────────────────────────────┤
│ LAYER 1: TRISTREAM DAG (GHOSTDAG Ordering)                      │
│   Stream A (ASIC)  │ Stream B (GPU)   │ Stream C (ZK)           │
│   BLAKE3 PoW       │ KAWPOW PoW       │ PoS + ZK-STARK          │
│   10s blocks       │ 60s blocks       │ Checkpoint Finality     │
│   50 PYRAX/block   │ 100 PYRAX/block  │ 10 PYRAX/checkpoint     │
└─────────────────────────────────────────────────────────────────┘
```

### Stream Parameters

| Stream | Algorithm | Block Time | Reward | Fee Share | Purpose |
|--------|-----------|------------|--------|-----------|---------|
| **A** | BLAKE3 | 10s | 50 PYRAX | 20% | Fast finality, ASIC mining |
| **B** | KAWPOW | 60s | 100 PYRAX | 40% | GPU mining, ASIC-resistant |
| **C** | ZK-STARK | Variable | 10 PYRAX | 30% | ZK proofs, checkpoint finality |

**Protocol Treasury:** 10% of all gas fees

### Cryptographic Primitives

| Purpose | Algorithm | Notes |
|---------|-----------|-------|
| Address derivation | Keccak-256 | EVM-compatible |
| Block header hash | BLAKE3 (A) / KAWPOW (B) | Stream-dependent |
| Transaction hash | Keccak-256 | Standard |
| Signatures | ECDSA secp256k1 | BIP-32/39/44 compatible |
| ZK Proofs | STARK, SNARK, Plonk, Groth16 | Multi-scheme support |

### EVM Sidechain

- **Block Time:** 2 seconds
- **Throughput:** 1,000-5,000 TPS
- **Compatibility:** Full Solidity 0.8.x support
- **Precompiles:** ECRECOVER, SHA256, BLAKE3, ZK_VERIFY, L1_BRIDGE
- **Bridge:** 2-way peg with 6-confirmation deposits, 7-day challenge withdrawal

### Rust/WASM Contracts

- **Runtime:** Wasmtime 15.0
- **Gas Metering:** Fuel-based instruction counting
- **Memory:** 16MB limit, 512KB stack
- **Host Functions:** storage_read/write, emit_log, blake3, keccak256, cross-VM calls
- **Interop:** EVM↔WASM cross-contract calls via ABI encoding

### ZK-Rollup Infrastructure

- **Batch Size:** Up to 1,000 transactions
- **Batch Interval:** 60 seconds
- **Proof Types:** STARK, SNARK, Plonk, Groth16
- **Data Availability:** L1 Calldata, Blobs, Celestia, EigenDA support
- **Sequencer:** Decentralized with 10,000 PYRAX stake requirement
- **Challenge Period:** 7 days for withdrawals

---

## 5. Product Portfolio

### Infrastructure Layer

| Product | Description | Status |
|---------|-------------|--------|
| **pyrax-node** | Core blockchain node (consensus, P2P, RPC, storage) | ✅ Complete |
| **pyrax-miner** | GPU mining software (CUDA + OpenCL) | ✅ Complete |
| **pyrax-wallet** | Rust wallet library (keys, signing, HD derivation) | ✅ Complete |

### Application Layer

| Product | Description | Status |
|---------|-------------|--------|
| **pyrax-desktop** | Desktop wallet & node dashboard (Tauri) | ✅ Complete |
| **pyrax-explorer** | Block explorer (Next.js, real-time data) | ✅ Complete |
| **pyrax-cli** | Command-line interface (all operations) | ✅ Complete |

### AI Services Layer

| Product | Description | Status |
|---------|-------------|--------|
| **Foundry** | Model registry + training job submission | ✅ Complete |
| **Crucible** | Job execution + provider protocol | ✅ Complete |
| **AI Marketplace** | Model/compute listings, orders, settlements | ✅ Complete |

### Explorer Features

- Real-time chain statistics with Recharts visualization
- Paginated blocks, transactions, tokens, NFTs
- EVM and WASM contract verification wizards
- Smart autocomplete search (blocks, txs, addresses)
- AI dashboard with models, providers, jobs
- Faucet integration for testnet

---

## 6. AI Compute Platform

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     AI MARKETPLACE                               │
│  Model Listings | Compute Listings | Orders | Settlements        │
├─────────────────────────────────────────────────────────────────┤
│  FOUNDRY (Model Registry)    │    CRUCIBLE (Job Execution)      │
│  - Model registration        │    - Job submission               │
│  - Versioning                │    - Provider claiming            │
│  - Benchmarks                │    - Multi-node execution         │
│  - Discovery                 │    - Result verification          │
├─────────────────────────────────────────────────────────────────┤
│                    COMPUTE POOL                                  │
│  Node Registration | Heartbeat | Reputation | Staking            │
└─────────────────────────────────────────────────────────────────┘
```

### Job Lifecycle

```
Pending → Assigned → Running → Completed → Verified → Settled
```

1. **Submit** - Requester submits job spec + PYRAX payment
2. **Assign** - Scheduler matches job to qualified providers
3. **Execute** - Provider executes inference/training task
4. **Complete** - Provider submits results with proof
5. **Verify** - Multi-node consensus or ZK proof verification
6. **Settle** - Automatic payment distribution

### Supported Job Types

| Type | Description | Use Case |
|------|-------------|----------|
| **Inference** | Model prediction/generation | API endpoints, chatbots |
| **Training** | Full model training | New model development |
| **FineTuning** | Model adaptation | Domain-specific models |
| **Embedding** | Vector generation | Search, RAG systems |
| **ImageGen** | Image generation | Creative tools |
| **BatchInference** | Bulk predictions | Data processing |

### Provider Economics

| Tier | GPU Requirement | Stake | Expected Revenue |
|------|-----------------|-------|------------------|
| **Hobbyist** | 1x RTX 3080+ | 1,000 PYRAX | 200-500 PYRAX/mo |
| **Professional** | 4x RTX 4090 | 5,000 PYRAX | 1,500-3,000 PYRAX/mo |
| **Enterprise** | 8x A100/H100 | 25,000 PYRAX | 5,000-15,000 PYRAX/mo |

### Verification Methods

| Method | Description | Use Case |
|--------|-------------|----------|
| **SingleNode** | Trust provider result | Low-value jobs |
| **Consensus** | Multi-node agreement | Standard jobs |
| **ZkProof** | Cryptographic verification | High-value, privacy |
| **TeeAttestation** | Hardware enclave proof | Sensitive data |

---

## 7. Token Economics

### Token Overview

| Property | Value |
|----------|-------|
| **Name** | PYRAX |
| **Symbol** | $PYRAX |
| **Total Supply** | 100,000,000,000 (100 Billion) |
| **Decimals** | 8 |
| **Smallest Unit** | 0.00000001 PYRAX |

### Token Allocation

| Pool | Percentage | Amount | Release Schedule |
|------|------------|--------|------------------|
| **Presale** | 15% | 15B | 30% TGE, 1mo cliff, 12mo vesting |
| **BDAG Community** | 10% | 10B | 12mo cliff, 12mo vesting |
| **Mining Rewards** | 35% | 35B | Per block mined (halving ~4 years) |
| **ZK Prover Rewards** | 5% | 5B | Per attestation (10% burned) |
| **Team & Founders** | 4% | 4B | 12mo cliff, 48mo vesting |
| **Advisors** | 3% | 3B | 12mo cliff, 48mo vesting |
| **Ecosystem** | 10% | 10B | Milestone-based, DAO approval |
| **Marketing** | 5% | 5B | As needed, multi-sig approval |
| **Liquidity** | 10% | 10B | 100% at TGE |
| **Treasury** | 2% | 2B | 75% DAO approval required |
| **Reserve** | 1% | 1B | Protocol buffer |

### Mining Rewards Distribution

| Stream | Share | Amount | Algorithm |
|--------|-------|--------|-----------|
| **Stream A (ASIC)** | 60% | 21B PYRAX | BLAKE3 PoW |
| **Stream B (GPU)** | 40% | 14B PYRAX | KAWPOW PoW |

- **Initial Block Reward:** ~1,666 PYRAX per block
- **Halving Interval:** ~21,000,000 blocks (~4 years)

### Gas Fee Distribution

| Recipient | Share | Purpose |
|-----------|-------|---------|
| Stream A Miners | 20% | ASIC mining rewards |
| Stream B Miners | 40% | GPU mining rewards |
| Stream C Validators | 30% | ZK prover rewards |
| Protocol Treasury | 10% | Development fund |

### Deflationary Mechanisms

- **ZK Prover Burns:** 10% of ZK rewards burned
- **Bridge Fees:** 0.1% (10 bps) per L1↔EVM conversion
- **AI Platform Fee:** 2% of job value (portion burned)

### Token Utility

| Use Case | Description |
|----------|-------------|
| **Transaction Fees** | Pay for on-chain operations |
| **AI Job Payments** | Pay for compute jobs |
| **Provider Staking** | Stake to become compute provider |
| **Validator Staking** | Stake to become Stream C validator |
| **EVM Gas** | Gas unit ("cinders") for sidechain |
| **Sequencer Staking** | Stake to run L2 sequencer node |

### Staking Parameters

| Parameter | Value |
|-----------|-------|
| Minimum Validator Stake | 100,000 PYRAX |
| Minimum Delegation | 100 PYRAX |
| Compute Provider Stake | 1,000 PYRAX |
| Sequencer Stake | 10,000 PYRAX |
| Unbonding Period | 7 days |
| Double-Sign Slash | 5% |
| Downtime Slash | 0.1% |

---

## 8. Go-to-Market Strategy

### Launch Phases

| Phase | Timeline | Focus | Status |
|-------|----------|-------|--------|
| **Foundation** | Q1 2026 | Core protocol, devnet, internal testing | ✅ Complete |
| **Testnet** | Q2 2026 | Public testnet, bug bounty, community | 🔄 In Progress |
| **Mainnet** | Q3 2026 | Genesis, exchange listings, AI platform | ⏳ Planned |
| **Scale** | Q4 2026+ | Enterprise, global expansion, bridges | ⏳ Planned |

### Marketing Channels

| Channel | Strategy |
|---------|----------|
| **Developer Relations** | Hackathons, grants program, documentation |
| **Mining Community** | Pool partnerships, setup guides, benchmarks |
| **AI Community** | Research publications, model benchmarks |
| **Social Media** | Twitter/X, Discord, Telegram |
| **Content Marketing** | Technical blog, YouTube tutorials |
| **Events** | Blockchain/AI conferences, meetups |

### Partnership Strategy

**Tier 1 - Strategic:**
- Cloud providers (compute capacity)
- AI research labs (model validation)
- Centralized exchanges (listings)

**Tier 2 - Integration:**
- Wallet providers (hardware/software)
- DeFi protocols (liquidity)
- Oracle networks (data feeds)

**Tier 3 - Community:**
- Mining pools (hash power)
- Node operators (decentralization)
- Developer communities (ecosystem growth)

---

## 9. Roadmap

### 2026 Development Timeline

#### Q1 - Foundation ✅ COMPLETE
- Core node with BLAKE3 PoW mining
- P2P networking (libp2p, GossipSub, mDNS)
- RocksDB storage layer
- JSON-RPC API server
- Desktop wallet application
- GPU miner (CUDA/OpenCL)

#### Q2 - Platform Expansion 🔄 IN PROGRESS
- Public testnet launch
- EVM sidechain activation
- Rust/WASM contract runtime
- AI marketplace beta
- Mobile wallet development
- Exchange listing preparations

#### Q3 - Mainnet Launch ⏳ PLANNED
- Security audits completion
- Genesis block generation
- Seed node deployment
- CEX/DEX listings
- AI platform production launch
- Bridge activations

#### Q4 - Scale ⏳ PLANNED
- Enterprise partnerships
- Cross-chain bridges (ETH, BSC, Polygon)
- Advanced AI features
- Global node expansion
- Performance optimizations

### Long-term Vision

| Year | Target |
|------|--------|
| **2027** | Top 50 blockchain by market cap, leading decentralized AI platform |
| **2028** | Top 20 blockchain, Fortune 500 enterprise partnerships |
| **2030** | Core infrastructure layer for decentralized AI economy |

---

## 10. Risk Management

### Risk Matrix

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Smart contract vulnerabilities | Medium | High | Audits, bug bounty, formal verification |
| 51% attack | Low | High | TriStream consensus (3 algorithms), checkpointing |
| Regulatory challenges | Medium | Medium | Legal counsel, compliance-first approach |
| Competition | Medium | Medium | AI niche focus, community building |
| Key person dependency | Low | Medium | Documentation, team expansion, decentralization |
| GPU supply constraints | Medium | Low | Multi-vendor support, alternative hardware |

### Security Framework

| Layer | Measures |
|-------|----------|
| **Code** | Rust memory safety, comprehensive testing, fuzzing |
| **Protocol** | Formal verification, economic modeling, game theory |
| **Operations** | External audits, bug bounties, incident response |
| **Ecosystem** | Security standards, developer education, insurance |

### Implemented Security Tools

- **ChaosEngine**: Network partition, packet loss, latency, Byzantine simulation testing
- **SecurityAuditor**: CVSS-like scoring, vulnerability tracking, automated checks
- **HealthChecker**: Liveness/readiness probes, component health monitoring
- **NetworkMonitor**: Prometheus metrics, alerts, node health tracking

---

## 11. Success Metrics

### Key Performance Indicators

| Metric | Q2 2026 | Q4 2026 | Q4 2027 |
|--------|---------|---------|---------|
| Active Nodes | 100 | 500 | 2,000 |
| Daily Transactions | 10K | 100K | 1M |
| AI Jobs/Month | 1K | 10K | 100K |
| Registered Models | 100 | 500 | 5,000 |
| Compute Providers | 50 | 500 | 5,000 |

### Network Targets

| Metric | Target |
|--------|--------|
| Uptime | 99.9% |
| L1 Finality | <60 seconds |
| EVM Block Time | 2 seconds |
| L2 Throughput | 500K+ TPS |

### Community Targets

| Metric | 2026 | 2027 |
|--------|------|------|
| Token Holders | 50K | 250K |
| Discord Members | 25K | 100K |
| Active Developers | 500 | 2,000 |
| GitHub Stars | 1K | 5K |

---

## Appendix: Technical References

### Repository Structure

```
pyrax/
├── pyrax-node/        # Core blockchain node (Rust)
├── pyrax-miner/       # GPU miner (C++/CUDA/OpenCL)
├── pyrax-wallet/      # Wallet library (Rust)
├── pyrax-desktop/     # Desktop application (Tauri)
├── pyrax-explorer/    # Block explorer (Next.js)
├── pyrax-ai/          # AI marketplace (Rust)
├── docs/              # Documentation
└── deployment/        # Deployment configs
```

### Documentation

| Document | Description |
|----------|-------------|
| [BUILD.md](../BUILD.md) | Build instructions |
| [TESTNET.md](../TESTNET.md) | Testnet guide |
| [build_plan.md](../build_plan.md) | Development tracker |
| [windsurf-spec.md](windsurf-spec.md) | Protocol specification |
| [threat-model.md](threat-model.md) | Security analysis |
| [GENESIS.md](GENESIS.md) | Genesis documentation |

### Network Configuration

| Network | Chain ID | RPC Port | P2P Port |
|---------|----------|----------|----------|
| Mainnet | 0x505952_01 | 8545 | 30303 |
| Testnet | 0x505952_FF | 8545 | 30303 |
| Devnet | 0x505952_FE | 28545 | 30303 |

---

*This document outlines the strategic vision for PYRAX. It should be reviewed and updated quarterly.*

**Website:** [pyrax.org](https://pyrax.org)  
**Documentation:** [docs.pyrax.org](https://docs.pyrax.org)  
**GitHub:** [github.com/pyrax-official](https://github.com/pyrax-official)
