# Consensus Mechanism

PYRAX uses a hybrid consensus mechanism combining Proof of Work with additional validation layers.

## Proof of Work (KAWPOW)

### Algorithm Overview

KAWPOW is a GPU-optimized mining algorithm:

**Key Properties:**
- Memory-hard computation
- ASIC-resistant design
- Random program execution
- Optimized for consumer GPUs

### How KAWPOW Works

```
1. Generate DAG (Directed Acyclic Graph)
2. For each nonce:
   a. Mix data from DAG
   b. Execute random math
   c. Calculate final hash
3. Compare to target difficulty
4. If valid, submit block
```

### DAG Generation

The DAG is a large dataset required for mining:

| Property | Value |
|----------|-------|
| Initial Size | ~4 GB |
| Growth Rate | ~8 MB per epoch |
| Epoch Length | 7,500 blocks |
| Regeneration | Required each epoch |

### Difficulty Adjustment

Difficulty adjusts to maintain ~60 second blocks:

```
new_difficulty = old_difficulty * (expected_time / actual_time)
```

**Parameters:**
- Target block time: 60 seconds
- Adjustment: Every block
- Bounds: ±25% per adjustment

## Block Production

### Mining Process

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Transaction │     │    Block    │     │    Valid    │
│    Pool     │────►│  Template   │────►│    Block    │
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │   Mining    │
                    │  (KAWPOW)   │
                    └─────────────┘
```

### Block Validation

Valid blocks must satisfy:

1. **Proof of Work** — Hash below target
2. **Valid Header** — Correct parent, timestamp
3. **Valid Transactions** — All txs valid
4. **Correct State Root** — Matches post-execution
5. **Gas Limits** — Within block gas limit

### Uncle Blocks

Uncle (ommer) blocks provide secondary rewards:
- Reduce orphan rate
- Include valid but non-canonical blocks
- Maximum 2 uncles per block
- Reward: Portion of block reward

## Finality

### Probabilistic Finality

Like Bitcoin/Ethereum PoW:
- No instant finality
- Increases with confirmations
- 12+ confirmations = high confidence

### Confirmation Guidelines

| Use Case | Confirmations | Time |
|----------|---------------|------|
| Low value | 1-3 | ~1-3 min |
| Medium value | 6-12 | ~6-12 min |
| High value | 20+ | ~20+ min |
| Exchanges | 30+ | ~30+ min |

## Chain Selection

### Longest Chain Rule

The canonical chain is determined by:
1. Most total work (not just blocks)
2. First seen on tie
3. No subjective decisions

### Reorganizations

Reorgs can occur when:
- Competing chains exist briefly
- Longer chain discovered
- Usually shallow (1-2 blocks)

## Fork Management

### Soft Forks

Backward-compatible changes:
- Old nodes still validate new blocks
- Gradual upgrade possible
- Lower coordination requirements

### Hard Forks

Breaking changes:
- Requires all nodes to upgrade
- Coordinated activation
- May create chain splits if disagreement

### Upgrade Process

1. Proposal through DAO
2. Community discussion
3. Development and testing
4. Announced activation block
5. Coordinated upgrade

## Security Analysis

### 51% Attack Cost

Economic security from mining investment:
- Hashrate concentration monitored
- High cost to acquire majority
- Ongoing cost to maintain attack

### Double Spend Prevention

- Wait for confirmations
- Monitor for reorgs
- Large values need more confirmations

### Selfish Mining

Mitigations:
- Uncle inclusion reduces advantage
- Monitoring for suspicious patterns
- Economic incentives favor honest mining

## Crucible Integration

### Proof of Compute

Stream B adds additional consensus:
- AI job results verified
- Multi-node consensus
- Incorrect results slashed

### Verification Methods

| Method | Use Case |
|--------|----------|
| Multi-node | Standard jobs |
| ZK Proofs | Privacy-sensitive |
| Sampling | Large-scale verification |

## Technical Specifications

### Block Header Fields

| Field | Description |
|-------|-------------|
| parentHash | Previous block hash |
| uncleHash | Uncle blocks hash |
| coinbase | Miner address |
| stateRoot | State trie root |
| transactionsRoot | Tx trie root |
| receiptsRoot | Receipts trie root |
| logsBloom | Event filter |
| difficulty | Block difficulty |
| number | Block number |
| gasLimit | Max gas allowed |
| gasUsed | Gas consumed |
| timestamp | Block timestamp |
| extraData | Miner data (32 bytes) |
| mixHash | KAWPOW mix |
| nonce | Mining nonce |

---

:::info Related Topics
- [Block Structure](./block-structure)
- [Transaction Format](./transaction-format)
- [Network Overview](./network-overview)
:::
