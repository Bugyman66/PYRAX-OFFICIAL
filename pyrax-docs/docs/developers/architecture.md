# Architecture Overview

This document describes the technical architecture of the PYRAX network.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        PYRAX Network                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │   Miners     │  │    Nodes     │  │   Crucible   │              │
│  │  (KAWPOW)    │  │  (Full/Light)│  │  (AI Compute)│              │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
│         │                 │                  │                      │
│         ▼                 ▼                  ▼                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Consensus Layer                          │   │
│  │              (Proof of Work + Proof of Compute)             │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Execution Layer                          │   │
│  │                  (EVM Compatible)                           │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                     Data Layer                              │   │
│  │              (State, Transactions, Blocks)                  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Layer Breakdown

### Consensus Layer

The consensus layer determines how blocks are produced and validated.

**Components:**
- **KAWPOW Mining**: GPU-based proof-of-work
- **Block Validation**: Verifies block correctness
- **Chain Selection**: Chooses canonical chain
- **Difficulty Adjustment**: Maintains ~60 second blocks

**Key Features:**
- ASIC-resistant algorithm
- Dynamic difficulty adjustment per block
- Uncle block support for reduced orphans

### Execution Layer

The execution layer processes transactions and smart contracts.

**Components:**
- **EVM**: Ethereum Virtual Machine for contracts
- **State Machine**: Tracks account balances and contract states
- **Gas Metering**: Measures and limits computation
- **Precompiles**: Optimized native functions

**Compatibility:**
- Solidity 0.8+ support
- ERC-20, ERC-721, ERC-1155 standards
- OpenZeppelin library compatible
- Existing Ethereum tooling works

### Data Layer

The data layer stores and retrieves blockchain data.

**Components:**
- **Block Storage**: Complete block history
- **State Database**: Current account/contract states
- **Transaction Pool**: Pending transactions
- **Receipt Storage**: Transaction execution results

**Database:**
- LevelDB for performance
- Merkle Patricia Trie for state
- Efficient pruning options

## Network Topology

### Node Types

| Type | Purpose | Requirements |
|------|---------|--------------|
| **Full Node** | Complete validation | High storage, moderate CPU |
| **Archive Node** | Historical data | Very high storage |
| **Light Node** | Basic validation | Low requirements |
| **Mining Node** | Block production | Full node + GPU |

### P2P Network

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│ Node A  │◄───►│ Node B  │◄───►│ Node C  │
└────┬────┘     └────┬────┘     └────┬────┘
     │               │               │
     ▼               ▼               ▼
┌─────────┐     ┌─────────┐     ┌─────────┐
│ Node D  │◄───►│ Node E  │◄───►│ Node F  │
└─────────┘     └─────────┘     └─────────┘
```

**Protocol:**
- DevP2P for node discovery
- RLPx for encrypted communication
- Gossip protocol for propagation

## Crucible Architecture

Crucible is the AI computing layer built on PYRAX.

```
┌─────────────────────────────────────────────────────────────┐
│                      Crucible Platform                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │    Job      │    │   Matcher   │    │   Verify    │     │
│  │   Queue     │───►│   Service   │───►│   Layer     │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         ▲                  │                  │            │
│         │                  ▼                  ▼            │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │    Job      │    │    GPU      │    │   Payment   │     │
│  │  Submitters │    │  Providers  │    │   Escrow    │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Crucible Components

**Job Queue**
- Receives AI job submissions
- Validates job specifications
- Manages job lifecycle

**Matcher Service**
- Matches jobs to suitable GPUs
- Considers hardware, reputation, price
- Optimizes for efficiency

**GPU Providers**
- Execute AI workloads
- Return computation results
- Build reputation over time

**Verification Layer**
- Multi-node consensus for results
- Zero-knowledge proofs (optional)
- Dispute resolution

**Payment Escrow**
- Holds job payment during execution
- Automatic release on verification
- Slashing for bad actors

## Transaction Flow

### Standard Transaction

```
User                Network               Miners
  │                    │                    │
  │──1. Submit Tx─────►│                    │
  │                    │──2. Validate───────│
  │                    │◄─3. Propagate──────│
  │                    │                    │
  │                    │    4. Include      │
  │                    │    in Block        │
  │                    │                    │
  │                    │◄─5. New Block──────│
  │◄─6. Confirmation───│                    │
  │                    │                    │
```

### Smart Contract Interaction

```
User                Contract              EVM
  │                    │                    │
  │──1. Call/Send─────►│                    │
  │                    │──2. Execute───────►│
  │                    │                    │
  │                    │◄─3. State Change───│
  │                    │                    │
  │◄─4. Result/Receipt─│                    │
  │                    │                    │
```

## Smart Contract Architecture

### Contract Types

**Core Contracts:**
- Token contract (PYRAX)
- Staking contract
- DAO governance
- Treasury

**Crucible Contracts:**
- Job registry
- Provider registry
- Escrow
- Verification

### Upgrade Pattern

PYRAX uses proxy patterns for upgradability:

```
┌──────────────┐     ┌──────────────┐
│    Proxy     │────►│Implementation│
│  (Storage)   │     │   (Logic)    │
└──────────────┘     └──────────────┘
       │
       ▼
┌──────────────┐
│   Admin      │
│  (Upgrade)   │
└──────────────┘
```

## Security Architecture

### Network Security

- **PoW Security**: Economic cost to attack
- **Node Diversity**: Geographic distribution
- **Eclipse Protection**: Peer management
- **DoS Mitigation**: Rate limiting, gas limits

### Smart Contract Security

- **Access Control**: Role-based permissions
- **Reentrancy Guards**: Standard protections
- **Overflow Protection**: SafeMath/Solidity 0.8+
- **Audit Requirements**: All core contracts audited

### Crucible Security

- **Sandboxed Execution**: Isolated containers
- **Verification**: Multi-node consensus
- **Slashing**: Penalty for misbehavior
- **Encryption**: Optional data encryption

## Scalability

### Current Approach

- ~60 second block time
- Gas limit optimization
- Efficient state management

### Future Roadmap

- **Layer 2**: Rollup solutions
- **State Channels**: Off-chain computation
- **Sharding**: Parallel processing (research)

## Integration Points

### RPC API
Standard JSON-RPC for:
- Transaction submission
- State queries
- Block information
- Event logs

### Crucible API
REST API for:
- Job submission
- Status queries
- Provider management

### SDKs
- JavaScript/TypeScript
- Python
- (More planned)

---

:::info Technical Details
For implementation specifics, see:
- [Network Overview](./network-overview)
- [Consensus Mechanism](./consensus)
- [Block Structure](./block-structure)
:::
