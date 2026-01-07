# RPC Endpoints

This page lists all available RPC endpoints for connecting to PYRAX networks.

## Public Endpoints

### Mainnet

| Type | URL |
|------|-----|
| HTTP | `https://rpc.pyrax.org` |
| WebSocket | `wss://ws.pyrax.org` |

### Testnet

| Type | URL |
|------|-----|
| HTTP | `https://testnet.pyrax.org` |
| WebSocket | `wss://ws-testnet.pyrax.org` |

## Network Configuration

### Mainnet

| Parameter | Value |
|-----------|-------|
| Network Name | PYRAX Mainnet |
| Chain ID | TBD |
| Currency Symbol | PYRAX |
| Block Explorer | `https://explorer.pyrax.org` |

### Testnet

| Parameter | Value |
|-----------|-------|
| Network Name | PYRAX Testnet |
| Chain ID | TBD |
| Currency Symbol | PYRAX |
| Block Explorer | `https://testnet-explorer.pyrax.org` |

## Adding to MetaMask

### Automatic (Recommended)

Visit [pyrax.org](https://pyrax.org) and click "Add Network".

### Manual

1. Open MetaMask
2. Click network dropdown → Add Network
3. Enter network details from above
4. Save

## Connection Examples

### JavaScript (ethers.js)

```javascript
import { ethers } from 'ethers';

// HTTP Provider
const provider = new ethers.JsonRpcProvider('https://rpc.pyrax.org');

// WebSocket Provider
const wsProvider = new ethers.WebSocketProvider('wss://ws.pyrax.org');

// Check connection
const blockNumber = await provider.getBlockNumber();
console.log('Current block:', blockNumber);
```

### Python (web3.py)

```python
from web3 import Web3

# HTTP
w3 = Web3(Web3.HTTPProvider('https://rpc.pyrax.org'))

# WebSocket
w3_ws = Web3(Web3.WebsocketProvider('wss://ws.pyrax.org'))

# Check connection
print('Connected:', w3.is_connected())
print('Block:', w3.eth.block_number)
```

### curl

```bash
curl -X POST https://rpc.pyrax.org \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

## Rate Limits

### Public Endpoints

| Limit | Value |
|-------|-------|
| Requests/second | 10 |
| Burst | 50 |
| Daily limit | 100,000 |

### Getting Higher Limits

For higher rate limits:
1. Run your own node
2. Contact us for enterprise access
3. Use a node provider

## Supported Methods

All standard Ethereum JSON-RPC methods plus PYRAX extensions.

See [API Reference](./api-reference) for complete method list.

## WebSocket Subscriptions

```javascript
const ws = new WebSocket('wss://ws.pyrax.org');

ws.onopen = () => {
  // Subscribe to new blocks
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    method: 'eth_subscribe',
    params: ['newHeads'],
    id: 1
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('New block:', data.params?.result?.number);
};
```

## Running Your Own Node

For production applications, run your own node:

```bash
pyrax-node --mainnet --http --http.api eth,net,web3 --ws
```

See [Running a Node](./running-a-node) for details.

## Troubleshooting

### Connection Refused
- Check URL is correct
- Verify network connectivity
- Try alternative endpoint

### Rate Limited
- Reduce request frequency
- Implement caching
- Run your own node

### Timeout
- Check network conditions
- Try different region endpoint
- Increase timeout settings

---

:::tip Best Practices
1. Use WebSocket for real-time data
2. Cache responses when possible
3. Handle rate limits gracefully
4. Have fallback endpoints
:::
