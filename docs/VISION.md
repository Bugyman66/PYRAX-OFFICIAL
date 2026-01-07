# PYRAX Vision Document

**Version:** 2.0  
**Last Updated:** 2024-12-31  
**Status:** Architectural Blueprint

---

## Executive Summary

PYRAX is a **Layer 1 TriStream DAG blockchain** with:
- **Full EVM compatibility** via dedicated sidechain
- **Dual state model** (UTXO + Account-based)
- **Multi-language smart contracts** (Solidity + Rust/WASM)
- **Decentralized AI/ML compute** leveraging GPU mining infrastructure
- **500,000+ TPS** via Layer 2 ZK-Rollups

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            PYRAX ECOSYSTEM                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        LAYER 2: ZK-ROLLUPS                              │ │
│  │                        (500,000+ TPS)                                   │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │ │
│  │  │ AI Compute   │  │   DeFi       │  │   Gaming     │                  │ │
│  │  │ Rollup       │  │   Rollup     │  │   Rollup     │                  │ │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │ │
│  │         └─────────────────┼─────────────────┘                           │ │
│  │                           │                                              │ │
│  │                    ┌──────▼──────┐                                       │ │
│  │                    │  ZK Proofs  │                                       │ │
│  │                    │  to L1      │                                       │ │
│  │                    └──────┬──────┘                                       │ │
│  └───────────────────────────┼──────────────────────────────────────────────┘ │
│                              │                                               │
│  ┌───────────────────────────┼──────────────────────────────────────────────┐ │
│  │                    PYRAX EVM SIDECHAIN                                    │ │
│  │                    (Full Ethereum Compatibility)                          │ │
│  │                                                                           │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                       │ │
│  │  │  Solidity   │  │ Rust/WASM   │  │   Bridge    │                       │ │
│  │  │  Contracts  │  │  Contracts  │  │  to L1      │                       │ │
│  │  └─────────────┘  └─────────────┘  └──────┬──────┘                       │ │
│  │                                           │                               │ │
│  │  Account-Based State | EVM Execution | Gas Fees → L1 Validators          │ │
│  └───────────────────────────────────────────┼───────────────────────────────┘ │
│                                              │                                │
│  ┌───────────────────────────────────────────┼───────────────────────────────┐ │
│  │                     PYRAX LAYER 1 (TriStream DAG)                         │ │
│  │                                                                           │ │
│  │  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                     │ │
│  │  │  STREAM A   │   │  STREAM B   │   │  STREAM C   │                     │ │
│  │  │   (ASIC)    │   │   (GPU)     │   │   (PoS)     │                     │ │
│  │  │             │   │             │   │             │                     │ │
│  │  │  BLAKE3     │   │  KAWPOW     │   │  ZK-STARK   │                     │ │
│  │  │  10s blocks │   │  60s blocks │   │  Finality   │                     │ │
│  │  │             │   │             │   │             │                     │ │
│  │  │  UTXO Txs   │   │  GPU Mining │   │ Checkpoints │                     │ │
│  │  │             │   │  + AI Jobs  │   │             │                     │ │
│  │  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘                     │ │
│  │         │                 │                 │                             │ │
│  │         └────────┬────────┴────────┬────────┘                             │ │
│  │                  │    GHOSTDAG     │                                      │ │
│  │                  │    Ordering     │                                      │ │
│  │                  └────────┬────────┘                                      │ │
│  │                           │                                               │ │
│  │                    ┌──────▼──────┐                                        │ │
│  │                    │   Unified   │                                        │ │
│  │                    │ State View  │                                        │ │
│  │                    │ (UTXO)      │                                        │ │
│  │                    └─────────────┘                                        │ │
│  └───────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Layer 1: TriStream DAG

| Stream | Consensus | Block Time | Purpose |
|--------|-----------|------------|---------|
| **A** | BLAKE3 PoW (ASIC) | 10 seconds | Fast UTXO transactions |
| **B** | KAWPOW PoW (GPU) | 60 seconds | GPU security + AI compute |
| **C** | PoS + ZK-STARK | Variable | Finality + checkpoints |

**State Model:** UTXO (parallel validation, privacy-friendly)

### 2. EVM Sidechain (PYRAX-EVM)

| Feature | Specification |
|---------|---------------|
| **Compatibility** | 100% EVM compatible (Solidity 0.8.x) |
| **State Model** | Account-based |
| **Consensus** | Validated by Stream C stakers |
| **Bridge** | 2-way peg with L1 PYRAX tokens |
| **Gas Token** | PYRAX (bridged from L1) |
| **Block Time** | 2 seconds |
| **TPS** | ~1,000-5,000 |

### 3. Rust/WASM Runtime

| Feature | Specification |
|---------|---------------|
| **Language** | Rust compiled to WASM |
| **Execution** | Alongside EVM on sidechain |
| **Use Case** | High-performance contracts, AI/ML |
| **Interop** | Can call EVM contracts |

### 4. Layer 2: ZK-Rollups

| Feature | Specification |
|---------|---------------|
| **Proof System** | ZK-STARKs (no trusted setup) |
| **Data Availability** | Posted to L1 Stream C |
| **Settlement** | L1 Stream C checkpoints |
| **TPS per Rollup** | 10,000-50,000 |
| **Total Capacity** | 500,000+ TPS (multiple rollups) |

### 5. AI/ML Compute Network

| Feature | Specification |
|---------|---------------|
| **Compute Providers** | GPU miners from Stream B |
| **Job Types** | Inference, Training, Fine-tuning |
| **Payment** | PYRAX tokens |
| **Verification** | ZK proofs of computation |
| **Scheduling** | On-chain job marketplace |

---

## Dual State Model

### UTXO (Layer 1 Native)

```
┌─────────────────────────────────────────┐
│              UTXO Model                 │
├─────────────────────────────────────────┤
│  Transaction: [Inputs] → [Outputs]      │
│                                         │
│  Input:  TxID + OutputIndex + Signature │
│  Output: Amount + LockScript            │
│                                         │
│  Benefits:                              │
│  ✓ Parallel validation                  │
│  ✓ Simple SPV proofs                    │
│  ✓ Privacy (CoinJoin compatible)        │
│  ✓ No nonce tracking                    │
└─────────────────────────────────────────┘
```

### Account (EVM Sidechain)

```
┌─────────────────────────────────────────┐
│            Account Model                │
├─────────────────────────────────────────┤
│  State: Address → {Balance, Nonce,      │
│                    Code, Storage}       │
│                                         │
│  Transaction: From + To + Value + Data  │
│               + Nonce + GasLimit        │
│                                         │
│  Benefits:                              │
│  ✓ Full EVM/Solidity support            │
│  ✓ Complex smart contracts              │
│  ✓ Ethereum tooling compatible          │
│  ✓ DeFi/NFT ecosystem                   │
└─────────────────────────────────────────┘
```

### Bridge Between Models

```
L1 UTXO ←──────────────────────→ EVM Sidechain
         
Lock PYRAX on L1  ────────────→  Mint wrapped PYRAX on EVM
                  ←────────────  Burn wrapped, unlock on L1
```

---

## Gas Fee Distribution

All gas fees (from EVM sidechain and L2 rollups) are distributed:

| Recipient | Share | Rationale |
|-----------|-------|-----------|
| **Stream A Miners** | 20% | ASIC PoW security |
| **Stream B Miners** | 40% | GPU PoW + AI compute |
| **Stream C Stakers** | 30% | Finality + validation |
| **Protocol Treasury** | 10% | Development fund |

---

## AI/ML Compute Architecture

### GPU Mining ↔ AI Compute Dual-Use

```
┌─────────────────────────────────────────────────────────────────┐
│                    STREAM B GPU NETWORK                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  GPU Miner Decision (per block interval):                       │
│                                                                  │
│  ┌─────────────────┐         ┌─────────────────┐                │
│  │   Mine Block    │   OR    │  Process AI Job │                │
│  │   (KAWPOW)      │         │  (Inference/ML) │                │
│  └────────┬────────┘         └────────┬────────┘                │
│           │                           │                          │
│           ▼                           ▼                          │
│     Block Reward              Job Payment + Tip                  │
│     (PYRAX)                   (PYRAX)                            │
│                                                                  │
│  Economic Equilibrium:                                           │
│  Mining Reward ≈ AI Job Payment (market-driven)                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### AI Job Types

| Job Type | Description | Verification |
|----------|-------------|--------------|
| **Inference** | Run trained model on input | ZK proof of execution |
| **Fine-tuning** | Adapt model to new data | Checkpointed gradients |
| **Training** | Full model training | Distributed + verified |
| **Embedding** | Generate vector embeddings | Deterministic check |

### AI Compute Flow

```
1. User submits AI job + payment to L1
2. Job enters AI Job Mempool
3. GPU miner claims job (stake required)
4. Miner executes computation
5. Miner submits result + ZK proof
6. Stream C validates proof
7. Payment released to miner
8. Result available on-chain
```

---

## TPS Breakdown

| Layer | Component | TPS | Notes |
|-------|-----------|-----|-------|
| **L1** | Stream A (UTXO) | 200-500 | Native transfers |
| **L1** | Stream B | 50-100 | Blocks + AI jobs |
| **L1** | Stream C | 100-200 | Checkpoints |
| **Sidechain** | EVM | 1,000-5,000 | Smart contracts |
| **L2** | ZK-Rollup #1 | 10,000-50,000 | AI Compute |
| **L2** | ZK-Rollup #2 | 10,000-50,000 | DeFi |
| **L2** | ZK-Rollup #3 | 10,000-50,000 | Gaming |
| **Total** | Combined | **500,000+** | All layers |

---

## Token Economics

### PYRAX Token

| Parameter | Value |
|-----------|-------|
| **Ticker** | PYRAX |
| **Max Supply** | 100,000,000,000 (100 billion) |
| **Decimals** | 8 |
| **Initial Distribution** | See below |

### Block Rewards

| Stream | Reward | Halving |
|--------|--------|---------|
| **A** | 50 PYRAX/block | Every 2 years |
| **B** | 100 PYRAX/block | Every 2 years |
| **C** | 10 PYRAX/checkpoint | Fixed |

### Distribution

| Allocation | Percentage | Tokens |
|------------|------------|--------|
| Mining (A+B) | 60% | 12.6B |
| Staking (C) | 15% | 3.15B |
| Team/Dev | 10% | 2.1B (4-year vest) |
| Ecosystem | 10% | 2.1B |
| Treasury | 5% | 1.05B |

---

## Development Roadmap

### Phase 0-12: Core L1 (COMPLETE - Scaffolding)
- TriStream DAG consensus
- UTXO transactions
- Mining (ASIC + GPU)
- Staking + ZK finality
- Desktop apps, Explorer, Website

### Phase 13: EVM Sidechain
- Full EVM execution engine
- Account state management
- 2-way bridge to L1
- Solidity compiler integration

### Phase 14: Dual State Integration
- UTXO ↔ Account bridge
- Cross-model transactions
- Unified wallet support

### Phase 15: Rust/WASM Contracts
- WASM runtime on sidechain
- Rust smart contract SDK
- EVM ↔ WASM interop

### Phase 16: AI/ML Compute
- AI job marketplace
- GPU miner integration
- ZK computation proofs
- Model registry

### Phase 17: Layer 2 ZK-Rollups
- ZK-STARK rollup framework
- Data availability layer
- Rollup SDK
- 500K TPS capacity

### Phase 18: Mainnet Launch
- Security audits
- Testnet graduation
- Genesis block
- Ecosystem launch

---

## Competitive Positioning

| Feature | PYRAX | Ethereum | Solana | Avalanche |
|---------|-------|----------|--------|-----------|
| **L1 TPS** | 500+ | 15 | 65,000 | 4,500 |
| **L2 TPS** | 500k+ | 500k+ | N/A | 500k+ |
| **EVM** | Sidechain | Native | No | Native |
| **UTXO** | Native | No | No | No |
| **GPU Mining** | Yes | No | No | No |
| **AI Compute** | Native | No | No | No |
| **Finality** | ZK-STARK | ~12 min | 400ms | 1s |

---

## Success Metrics

| Metric | Target (Year 1) | Target (Year 3) |
|--------|-----------------|-----------------|
| **TPS** | 10,000 | 500,000+ |
| **Active Wallets** | 500,000 | 10,000,000 |
| **TVL** | $100M | $10B |
| **AI Jobs/Day** | 10,000 | 10,000,000 |
| **GPU Miners** | 1,000 | 500,000 |
| **dApps** | 100 | 10,000 |

---

## Next Steps

See `build_plan.md` for detailed Phase 13-18 implementation plans.
