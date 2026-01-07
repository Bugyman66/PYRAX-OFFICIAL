# PYRAX Testnet Guide

Welcome to the PYRAX devnet! This guide will help you set up a node and join the network.

## Quick Start (5 minutes)

### 1. Download or Build

**Option A: Build from source**
```bash
git clone https://github.com/pyrax-org/pyrax.git
cd pyrax/pyrax-node
cargo build --release
```

**Option B: Download binary** (coming soon)
- Windows: `pyrax-node-windows-x64.zip`
- Linux: `pyrax-node-linux-x64.tar.gz`
- macOS: `pyrax-node-macos-x64.tar.gz`

### 2. Create Data Directory
```bash
mkdir -p ~/pyrax-data
```

### 3. Start Your Node

**Join the devnet:**
```bash
./pyrax-node \
  --network devnet \
  --datadir ~/pyrax-data \
  --p2p \
  --p2p-addr /ip4/0.0.0.0/tcp/30303 \
  --rpc \
  --rpc-addr 127.0.0.1:8545
```

**Connect to a seed peer:**
```bash
./pyrax-node \
  --network devnet \
  --datadir ~/pyrax-data \
  --p2p \
  --rpc \
  --peer /ip4/SEED_IP/tcp/30303
```

### 4. Verify Your Node
```bash
# Check chain info
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pyrax_getChainInfo","params":[],"id":1}'
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "best_block_height": 1234,
    "best_block_hash": "0x...",
    "utxo_count": 1234
  },
  "id": 1
}
```

## Mining on Devnet

To run a mining node:

```bash
./pyrax-node \
  --network devnet \
  --datadir ~/pyrax-data \
  --p2p \
  --rpc \
  --mine \
  --miner-address 0xYOUR_ADDRESS_HERE
```

**Note**: Replace `0xYOUR_ADDRESS_HERE` with your PYRAX address (20 bytes hex).

## RPC API Reference

### Available Methods

| Method | Description |
|--------|-------------|
| `pyrax_getChainInfo` | Get current chain height and tip hash |
| `pyrax_getBlock` | Get block by hash or height |
| `pyrax_getBalance` | Get address balance |
| `pyrax_getUtxos` | Get UTXOs for an address |
| `pyrax_sendRawTransaction` | Broadcast a signed transaction |
| `pyrax_getMempoolInfo` | Get mempool statistics |

### Example: Get Balance
```bash
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pyrax_getBalance","params":["0xYOUR_ADDRESS"],"id":1}'
```

## Network Parameters

| Parameter | Devnet Value |
|-----------|--------------|
| Network ID | 3 |
| P2P Port | 30303 |
| RPC Port | 8545 |
| Block Time | ~100ms (low difficulty for testing) |
| Genesis Hash | `0x8d7310236912b9b0caa57c9c248e7f9ccb2d6babd361980e713867fc2c23fa1b` |

## Troubleshooting

### Node won't start
- Check that the data directory exists and is writable
- Ensure ports 30303 (P2P) and 8545 (RPC) are not in use
- Check firewall settings

### Not syncing
- Verify you're connected to at least one peer
- Check that the peer address is correct
- Ensure your clock is synchronized

### RPC not responding
- Confirm the node is running
- Check the RPC address and port
- Verify firewall allows localhost connections

## Getting Help

- GitHub Issues: https://github.com/pyrax-org/pyrax/issues
- Documentation: https://docs.pyrax.org

## Seed Nodes

**Devnet seed nodes:**
```
/ip4/127.0.0.1/tcp/30303  # Local testing
```

*More public seed nodes coming soon!*

---

**Happy testing!** 🚀
