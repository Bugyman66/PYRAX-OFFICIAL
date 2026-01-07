# PYRAX Blockchain - Technical Overview

> **Base URL:** https://pyrax.org | **Ticker:** $PYRAX | **Consensus:** TriStream DAG (ASIC + GPU + ZK)

---

## Executive Summary

PYRAX is a **TriStream DAG Layer 1 blockchain** with three parallel consensus streams unified via GHOSTDAG ordering:
- **Stream A (ASIC)**: BLAKE3 PoW, 10-second blocks, high-speed UTXO transactions
- **Stream B (GPU)**: KAWPOW PoW, 60-second blocks, GPU mining + AI compute jobs
- **Stream C (ZK)**: ZK-STARK finality, checkpoint proofs

Built with an EVM-compatible sidechain and ZK-rollup support for 500,000+ TPS.

### Key Features
- **TriStream Mining**: ASIC-friendly + GPU-resistant + ZK finality
- **GHOSTDAG Ordering**: Parallel block production without orphans
- **EVM Sidechain**: Full Ethereum compatibility (Solidity + Rust/WASM)
- **ZK-Rollups**: AI Compute, DeFi, Gaming rollups
- **Native AI Integration**: On-chain job marketplace on Stream B

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            PYRAX ECOSYSTEM                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        LAYER 3: ZK-ROLLUPS                              │ │
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
│  │                    LAYER 2: PYRAX EVM SIDECHAIN                           │ │
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
│  │                     LAYER 1: PYRAX TriStream DAG                          │ │
│  │                                                                           │ │
│  │  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                     │ │
│  │  │  STREAM A   │   │  STREAM B   │   │  STREAM C   │                     │ │
│  │  │   (ASIC)    │   │   (GPU)     │   │   (ZK)      │                     │ │
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
│  │                    │   (UTXO)    │                                        │ │
│  │                    └─────────────┘                                        │ │
│  └───────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## TriStream Consensus Details

| Stream | Algorithm | Block Time | Purpose | Hardware |
|--------|-----------|------------|---------|----------|
| **A** | BLAKE3 PoW | 10 sec | High-speed UTXO transactions | ASIC-friendly |
| **B** | KAWPOW PoW | 60 sec | GPU mining + AI compute jobs | GPU (NVIDIA/AMD) |
| **C** | ZK-STARK + PoS | Event-driven | Finality checkpoints | Validators |

### GHOSTDAG Ordering
All three streams produce blocks in parallel. GHOSTDAG algorithm orders them into a single canonical history without orphaning valid blocks. This enables:
- **Higher throughput**: Parallel block production
- **Lower latency**: Stream A provides 10-second confirmations
- **Hardware diversity**: ASIC, GPU, and validator participation
- **Strong finality**: Stream C ZK proofs provide irreversible finality

---

## Block Structure

```
BLOCK HEADER (112 bytes)
┌──────────────────┬────────────────────────────────────┐
│ Version          │ 4 bytes - Protocol version          │
│ Previous Hash    │ 32 bytes - SHA256 of prev header    │
│ Merkle Root      │ 32 bytes - Transaction tree root    │
│ Timestamp        │ 8 bytes - Unix timestamp            │
│ Difficulty       │ 4 bytes - Compact target            │
│ Nonce            │ 8 bytes - PoW solution              │
│ Height           │ 8 bytes - Block number              │
│ Extra Nonce      │ 8 bytes - Extended nonce            │
└──────────────────┴────────────────────────────────────┘

Header Hash = KAWPOW(header_bytes, nonce, height)
Valid if: hash < target
```

---

## KAWPOW Mining Algorithm

```
Block Header + Nonce
        │
        ▼
┌───────────────────┐
│ Calculate Epoch   │  epoch = height / 7500
└─────────┬─────────┘
          │
    ┌─────┴─────┐
    ▼           ▼
┌────────┐  ┌────────┐
│ Cache  │──│  DAG   │  (4-6 GB GPU memory)
│ 16 MB  │  │        │
└────────┘  └───┬────┘
                │
                ▼
┌───────────────────────────────────┐
│  PROGPOW Lanes (16 parallel)      │
│  • DAG lookups (64 per lane)      │
│  • Math operations (18 per lane)  │
│  • Lane mixing                    │
└─────────────────┬─────────────────┘
                  │
                  ▼
         ┌───────────────┐
         │  Final Hash   │
         │  Keccak256    │
         └───────┬───────┘
                 │
        hash < target?
        /           \
    YES              NO
   Submit         Next Nonce
```

**Difficulty Adjustment:**
- Target: 60 second blocks
- Window: 720 blocks (~12 hours)
- Max change: ±25% per window

---

## Transaction Format

```
TRANSACTION
┌──────────────────┬────────────────────────────────────┐
│ Version          │ 2 bytes                            │
│ Type             │ 1 byte (transfer/ai_job/etc)       │
│ Chain ID         │ 4 bytes                            │
│ Nonce            │ 8 bytes (sender sequence)          │
│ Gas Price        │ 8 bytes                            │
│ Gas Limit        │ 8 bytes                            │
│ To Address       │ 20 bytes                           │
│ Value            │ 32 bytes (amount)                  │
│ Data             │ Variable (payload)                 │
│ Signature (r,s,v)│ 65 bytes (ECDSA secp256k1)        │
└──────────────────┴────────────────────────────────────┘
```

---

## AI Job Lifecycle

```
SUBMIT → PENDING → MATCHED → RUNNING → VERIFY → SETTLE
   │                                              │
   └──── escrow ──────────────────────────────────┘
                                                  │
                              ┌───────────────────┴───────────────────┐
                              ▼                                       ▼
                          SUCCESS                                 DISPUTE
                        (payment to                             (arbitration)
                         worker)                                      │
                                                        ┌─────────────┴─────────────┐
                                                        ▼                           ▼
                                                    SLASH                        REFUND
                                                   (worker)                      (user)
```

**Verification Ladder:**
1. **Hash Check** - Deterministic output comparison
2. **N-of-M** - Redundant execution (2-of-3, 3-of-5)
3. **ZK Proofs** - Cryptographic proof (future)
4. **TEE** - Hardware attestation (future)

---

## Economic Model

### Token Distribution (21 Billion Total)
| Allocation | Percentage | Amount |
|------------|------------|--------|
| Mining Rewards | 70% | 14.7B |
| AI Compute Rewards | 15% | 3.15B |
| Development Fund | 8% | 1.68B |
| Community/Ecosystem | 5% | 1.05B |
| Genesis Allocation | 2% | 420M |

### Emission Schedule
| Era | Blocks | Reward | Total |
|-----|--------|--------|-------|
| 1 | 0 - 2.1M | 5000 PYRAX | 10.5B |
| 2 | 2.1M - 4.2M | 2500 PYRAX | 5.25B |
| 3 | 4.2M - 6.3M | 1250 PYRAX | 2.625B |
| ... | Halving continues | ... | →21B |

### Fees
- Simple transfer: 21,000 gas
- AI job submission: 50,000+ gas
- **Distribution:** 80% miner, 20% burned

---

## P2P Protocol Messages

| Category | Messages |
|----------|----------|
| **Handshake** | Hello, HelloAck, Disconnect |
| **Block Sync** | GetHeaders, Headers, GetBlocks, Block, NewBlock |
| **Transaction** | InvTx, GetTx, Tx, TxPool |
| **Status** | Ping, Pong, GetStatus, Status |

---

## Key Derivation

```
BIP-39 Seed (24 words)
        │
        ▼ PBKDF2-SHA512
512-bit Seed
        │
        ▼ BIP-32 HD (m/44'/PYRAX'/account'/change/index)
secp256k1 Private Key
        │
        ▼ EC Multiply
secp256k1 Public Key
        │
        ▼ Keccak256 (last 20 bytes)
PYRAX Address (0x + 40 hex)
```

---

## Repository Structure

```
pyrax-official/
├── build_plan.md          # Single source of truth
├── overview.md            # This file
├── Cargo.toml             # Rust workspace
├── docs/
│   ├── windsurf-spec.md   # Protocol spec
│   ├── threat-model.md    # Security
│   └── GENESIS.md         # Genesis docs
├── pyrax-node/            # Rust node
├── pyrax-wallet/          # Rust wallet lib
├── pyrax-miner/           # C++/CUDA miner
├── pyrax-desktop/         # Tauri app
├── pyrax-explorer/        # Web explorer
└── pyrax-ai/              # AI marketplace
```

---

## Reality Gates (Mandatory)

| Gate | Requirement |
|------|-------------|
| **A** | Two nodes sync and stay synced 24h |
| **B** | Wallet sends TX that gets mined |
| **C** | Miner finds valid devnet block |
| **D** | 10-node testnet survives partitions |
| **E** | Fresh machine reproduces binaries |
| **F** | External user joins testnet via docs |

---

## CI Reality Gate (Blocks Merges)

Forbidden patterns:
- `mock`, `stub`, `fake`, `TODO`
- `unimplemented`, `panic!("not implemented")`
- `--dev-bypass-consensus`, `--skip-verify`
- UI "demo mode" data providers

---

*For detailed task tracking, see `build_plan.md`*
