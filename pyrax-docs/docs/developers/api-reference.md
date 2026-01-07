# API Reference

Complete API reference for interacting with the PYRAX network.

## JSON-RPC API

PYRAX exposes a standard Ethereum JSON-RPC API.

### Endpoints

| Network | HTTP | WebSocket |
|---------|------|-----------|
| Mainnet | `https://rpc.pyrax.org` | `wss://ws.pyrax.org` |
| Testnet | `https://testnet.pyrax.org` | `wss://ws-testnet.pyrax.org` |

### Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "eth_blockNumber",
  "params": [],
  "id": 1
}
```

### Response Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x1b4"
}
```

## eth_ Methods

### eth_blockNumber
Get current block number.

```javascript
// Request
{ "method": "eth_blockNumber", "params": [] }

// Response
{ "result": "0x1b4" }  // 436 in decimal
```

### eth_getBalance
Get account balance.

```javascript
// Request
{
  "method": "eth_getBalance",
  "params": ["0x...", "latest"]
}

// Response
{ "result": "0x1bc16d674ec80000" }  // 2 PYRAX in wei
```

### eth_getTransactionCount
Get account nonce.

```javascript
// Request
{
  "method": "eth_getTransactionCount",
  "params": ["0x...", "latest"]
}

// Response
{ "result": "0x5" }  // 5 transactions
```

### eth_sendRawTransaction
Submit signed transaction.

```javascript
// Request
{
  "method": "eth_sendRawTransaction",
  "params": ["0xf86c..."]
}

// Response
{ "result": "0x..." }  // Transaction hash
```

### eth_call
Execute read-only call.

```javascript
// Request
{
  "method": "eth_call",
  "params": [{
    "to": "0x...",
    "data": "0x..."
  }, "latest"]
}

// Response
{ "result": "0x..." }  // Return data
```

### eth_estimateGas
Estimate gas for transaction.

```javascript
// Request
{
  "method": "eth_estimateGas",
  "params": [{
    "from": "0x...",
    "to": "0x...",
    "data": "0x..."
  }]
}

// Response
{ "result": "0x5208" }  // 21000 gas
```

### eth_getBlockByNumber
Get block by number.

```javascript
// Request
{
  "method": "eth_getBlockByNumber",
  "params": ["0x1b4", true]  // true = include txs
}

// Response
{
  "result": {
    "number": "0x1b4",
    "hash": "0x...",
    "parentHash": "0x...",
    "transactions": [...]
  }
}
```

### eth_getTransactionReceipt
Get transaction receipt.

```javascript
// Request
{
  "method": "eth_getTransactionReceipt",
  "params": ["0x..."]
}

// Response
{
  "result": {
    "status": "0x1",
    "blockNumber": "0x1b4",
    "gasUsed": "0x5208",
    "logs": [...]
  }
}
```

### eth_getLogs
Query event logs.

```javascript
// Request
{
  "method": "eth_getLogs",
  "params": [{
    "fromBlock": "0x1",
    "toBlock": "latest",
    "address": "0x...",
    "topics": ["0x..."]
  }]
}

// Response
{
  "result": [{
    "address": "0x...",
    "topics": [...],
    "data": "0x..."
  }]
}
```

## net_ Methods

### net_version
Get network ID.

```javascript
{ "method": "net_version", "params": [] }
// Response: { "result": "1000" }
```

### net_peerCount
Get connected peer count.

```javascript
{ "method": "net_peerCount", "params": [] }
// Response: { "result": "0x19" }
```

## pyrax_ Methods (Custom)

### pyrax_getStakingInfo
Get staking information.

```javascript
{
  "method": "pyrax_getStakingInfo",
  "params": ["0x..."]
}
// Response includes staked amount, rewards, etc.
```

### pyrax_getMiningStats
Get network mining statistics.

```javascript
{ "method": "pyrax_getMiningStats", "params": [] }
// Response includes hashrate, difficulty, etc.
```

## WebSocket Subscriptions

### newHeads
Subscribe to new blocks.

```javascript
{
  "method": "eth_subscribe",
  "params": ["newHeads"]
}
```

### logs
Subscribe to contract events.

```javascript
{
  "method": "eth_subscribe",
  "params": ["logs", {
    "address": "0x...",
    "topics": ["0x..."]
  }]
}
```

### pendingTransactions
Subscribe to pending transactions.

```javascript
{
  "method": "eth_subscribe",
  "params": ["pendingTransactions"]
}
```

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid request | Missing fields |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Wrong parameters |
| -32603 | Internal error | Server error |

## Rate Limits

| Tier | Requests/sec | Burst |
|------|--------------|-------|
| Free | 10 | 50 |
| Basic | 100 | 500 |
| Pro | 1000 | 5000 |

---

:::info SDKs
For easier integration, use our official SDKs:
- [JavaScript SDK](./javascript-sdk)
- [Python SDK](./python-sdk)
:::
