# Block Structure

This document details the structure of PYRAX blocks.

## Block Overview

A PYRAX block consists of:
- Block header
- Transaction list
- Uncle headers

```
┌─────────────────────────────────────┐
│           Block Header              │
├─────────────────────────────────────┤
│         Transaction List            │
│  ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐    │
│  │Tx1│ │Tx2│ │Tx3│ │...│ │TxN│    │
│  └───┘ └───┘ └───┘ └───┘ └───┘    │
├─────────────────────────────────────┤
│          Uncle Headers              │
│  ┌─────────┐ ┌─────────┐           │
│  │ Uncle 1 │ │ Uncle 2 │           │
│  └─────────┘ └─────────┘           │
└─────────────────────────────────────┘
```

## Block Header

### Header Fields

| Field | Type | Size | Description |
|-------|------|------|-------------|
| parentHash | bytes32 | 32 | Hash of parent block |
| uncleHash | bytes32 | 32 | Hash of uncle list |
| coinbase | address | 20 | Miner's address |
| stateRoot | bytes32 | 32 | State trie root |
| transactionsRoot | bytes32 | 32 | Tx trie root |
| receiptsRoot | bytes32 | 32 | Receipts trie root |
| logsBloom | bytes256 | 256 | Bloom filter for logs |
| difficulty | uint256 | var | Block difficulty |
| number | uint256 | var | Block height |
| gasLimit | uint256 | var | Max gas for block |
| gasUsed | uint256 | var | Total gas used |
| timestamp | uint256 | var | Unix timestamp |
| extraData | bytes | ≤32 | Arbitrary data |
| mixHash | bytes32 | 32 | KAWPOW mix hash |
| nonce | uint64 | 8 | Mining nonce |
| baseFeePerGas | uint256 | var | EIP-1559 base fee |

### Block Hash Calculation

```
blockHash = keccak256(RLP(header))
```

## Transactions

### Transaction Types

PYRAX supports multiple transaction types:

| Type | Description |
|------|-------------|
| 0 (Legacy) | Original format |
| 1 (EIP-2930) | Access list transactions |
| 2 (EIP-1559) | Dynamic fee transactions |

### EIP-1559 Transaction Fields

```javascript
{
  chainId: 1000,
  nonce: 5,
  maxPriorityFeePerGas: 1000000000,
  maxFeePerGas: 50000000000,
  gasLimit: 21000,
  to: "0x...",
  value: 1000000000000000000,
  data: "0x...",
  accessList: [],
  v: 0,
  r: "0x...",
  s: "0x..."
}
```

### Transaction Ordering

Transactions are ordered by:
1. Gas price (highest first)
2. Nonce (per account)
3. Arrival time (tie-breaker)

## Uncle Blocks

### Uncle Structure

Uncle blocks include only headers (no transactions):
- Valid PoW solution
- Parent is ancestor of main chain
- Within 7 blocks of current

### Uncle Rewards

| Relationship | Miner Reward | Uncle Miner |
|--------------|--------------|-------------|
| 1 block back | +3.125% | 87.5% |
| 2 blocks back | +3.125% | 75% |
| 3 blocks back | +3.125% | 62.5% |
| ... | ... | ... |

## Gas and Fees

### Block Gas Limit

- Dynamic limit based on parent
- Can adjust ±0.1% per block
- Target: ~15 million gas

### EIP-1559 Mechanics

```
Base Fee = parent_base_fee * (1 + adjustment)
adjustment = (parent_gas_used - target) / target / 8
```

### Fee Distribution

| Component | Destination |
|-----------|-------------|
| Base fee | Burned |
| Priority fee | Miner |

## State Transitions

### Pre-Block State → Post-Block State

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Pre-State   │────►│  Execute Txs │────►│ Post-State   │
│   (Root A)   │     │              │     │   (Root B)   │
└──────────────┘     └──────────────┘     └──────────────┘
```

### State Root Calculation

The state root is the Merkle Patricia Trie root of:
- All account states
- Contract storage
- Code hashes

## Receipts

### Receipt Fields

| Field | Description |
|-------|-------------|
| status | Success (1) or failure (0) |
| cumulativeGasUsed | Total gas at this point |
| logsBloom | Bloom filter for tx logs |
| logs | Event logs from execution |

### Log Structure

```javascript
{
  address: "0x...",      // Contract that emitted
  topics: ["0x...", ...], // Indexed parameters
  data: "0x..."          // Non-indexed data
}
```

## Block Size

### Current Limits

| Metric | Limit |
|--------|-------|
| Gas Limit | ~15M (dynamic) |
| Block Size | Variable |
| Max Transactions | Gas-limited |

### Typical Block

- ~100-300 transactions
- ~1-2 MB size
- Produced every ~60 seconds

## Encoding

### RLP Encoding

Blocks use Recursive Length Prefix (RLP) encoding:

```javascript
block = RLP([
  header,
  transactions,
  uncles
])

header = RLP([
  parentHash,
  uncleHash,
  coinbase,
  stateRoot,
  transactionsRoot,
  receiptsRoot,
  logsBloom,
  difficulty,
  number,
  gasLimit,
  gasUsed,
  timestamp,
  extraData,
  mixHash,
  nonce,
  baseFeePerGas
])
```

## Querying Blocks

### JSON-RPC Examples

**Get block by number:**
```javascript
{
  "jsonrpc": "2.0",
  "method": "eth_getBlockByNumber",
  "params": ["0x1b4", true],
  "id": 1
}
```

**Get block by hash:**
```javascript
{
  "jsonrpc": "2.0",
  "method": "eth_getBlockByHash",
  "params": ["0x...", true],
  "id": 1
}
```

### Using ethers.js

```javascript
// Get block
const block = await provider.getBlock(12345);

// Get block with transactions
const blockWithTxs = await provider.getBlock(12345, true);

// Latest block
const latest = await provider.getBlock('latest');
```

---

:::info Related Topics
- [Transaction Format](./transaction-format)
- [Consensus Mechanism](./consensus)
- [RPC Endpoints](./rpc-endpoints)
:::
