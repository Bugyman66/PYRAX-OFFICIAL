# Network Overview

This document describes the PYRAX network architecture and how nodes communicate.

## Network Basics

### Network Parameters

| Parameter | Value |
|-----------|-------|
| Block Time | ~60 seconds |
| Network Protocol | DevP2P |
| Default Port | 30303 |
| RPC Port | 8545 |
| WebSocket Port | 8546 |

### Chain IDs

| Network | Chain ID | Purpose |
|---------|----------|---------|
| Mainnet | TBD | Production network |
| Testnet | TBD | Development and testing |
| Devnet | TBD | Local development |

## Node Types

### Full Nodes
Full nodes maintain a complete copy of the blockchain:
- Validate all transactions and blocks
- Store complete state
- Serve data to light nodes
- Can submit transactions

**Requirements:**
- 100+ GB storage
- 8+ GB RAM
- Stable internet

### Archive Nodes
Archive nodes store historical state:
- All full node capabilities
- Historical state at any block
- Required for historical queries

**Requirements:**
- 500+ GB storage
- 16+ GB RAM

### Light Nodes
Light nodes provide basic functionality:
- Header verification only
- Rely on full nodes for data
- Lower resource requirements

**Requirements:**
- 1 GB storage
- 2 GB RAM

### Mining Nodes
Mining nodes produce blocks:
- Full node capabilities
- Connected to GPU mining software
- Submit new blocks to network

## P2P Network

### Discovery Protocol

PYRAX uses Kademlia-based node discovery:

```
1. Bootstrap from seed nodes
2. Query nearby nodes for peers
3. Build routing table
4. Maintain connections
```

**Seed Nodes:**
```
enode://...@seed1.pyrax.org:30303
enode://...@seed2.pyrax.org:30303
enode://...@seed3.pyrax.org:30303
```

### Message Types

| Message | Purpose |
|---------|---------|
| `Status` | Exchange chain state |
| `NewBlock` | Announce new block |
| `Transactions` | Share pending txs |
| `GetBlockHeaders` | Request headers |
| `GetBlockBodies` | Request block data |

### Synchronization

**Full Sync:**
- Download all blocks from genesis
- Verify every transaction
- Build complete state

**Fast Sync:**
- Download recent state
- Verify headers only
- Faster initial sync

**Snap Sync:**
- Download state snapshots
- Fastest sync method
- Recommended for new nodes

## RPC Interface

### JSON-RPC Methods

PYRAX supports standard Ethereum JSON-RPC:

**eth_ namespace:**
- `eth_blockNumber` — Current block
- `eth_getBalance` — Account balance
- `eth_sendTransaction` — Send transaction
- `eth_call` — Execute call
- `eth_getLogs` — Query events

**net_ namespace:**
- `net_version` — Network ID
- `net_peerCount` — Connected peers
- `net_listening` — Accepting connections

**pyrax_ namespace (custom):**
- `pyrax_getStakingInfo` — Staking details
- `pyrax_getMiningStats` — Mining statistics
- `pyrax_getCrucibleJobs` — AI job info

### WebSocket Subscriptions

Subscribe to real-time events:

```javascript
// Subscribe to new blocks
ws.send(JSON.stringify({
  jsonrpc: "2.0",
  method: "eth_subscribe",
  params: ["newHeads"],
  id: 1
}));

// Subscribe to logs
ws.send(JSON.stringify({
  jsonrpc: "2.0",
  method: "eth_subscribe",
  params: ["logs", { address: "0x..." }],
  id: 2
}));
```

## Network Security

### Eclipse Attack Prevention
- Minimum peer diversity requirements
- Peer scoring and rotation
- Connection limits per IP range

### DoS Protection
- Rate limiting on RPC
- Gas limits on transactions
- Peer banning for misbehavior

### Sybil Resistance
- PoW provides economic cost
- Node reputation tracking
- Proof-of-stake for certain operations

## Connecting to the Network

### Direct Connection

```javascript
const { ethers } = require('ethers');

// HTTP Provider
const provider = new ethers.JsonRpcProvider('https://rpc.pyrax.org');

// WebSocket Provider
const wsProvider = new ethers.WebSocketProvider('wss://ws.pyrax.org');
```

### Running Your Own Node

See [Running a Node](./running-a-node) for detailed instructions.

Basic command:
```bash
pyrax-node --mainnet --http --http.api eth,net,web3
```

## Network Monitoring

### Metrics

Nodes expose Prometheus metrics:
- Block height
- Peer count
- Transaction pool size
- Sync status

### Network Statistics

Monitor network health:
- [stats.pyrax.org](https://stats.pyrax.org) — Network dashboard
- [explorer.pyrax.org](https://explorer.pyrax.org) — Block explorer

---

:::info More Details
- [Consensus Mechanism](./consensus)
- [Block Structure](./block-structure)
- [Running a Node](./running-a-node)
:::
