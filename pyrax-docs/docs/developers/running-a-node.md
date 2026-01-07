# Running a Node

This guide covers setting up and running a PYRAX node.

## Node Types

| Type | Purpose | Storage | RAM |
|------|---------|---------|-----|
| Full Node | Validation & RPC | 100+ GB | 8+ GB |
| Archive Node | Historical data | 500+ GB | 16+ GB |
| Mining Node | Block production | 100+ GB | 8+ GB |

## System Requirements

### Minimum

| Component | Requirement |
|-----------|-------------|
| CPU | 4 cores |
| RAM | 8 GB |
| Storage | 100 GB SSD |
| Network | 25 Mbps |
| OS | Ubuntu 20.04+ / Windows 10+ |

### Recommended

| Component | Requirement |
|-----------|-------------|
| CPU | 8+ cores |
| RAM | 16+ GB |
| Storage | 500 GB NVMe SSD |
| Network | 100+ Mbps |

## Installation

### From Binary (Recommended)

```bash
# Download latest release
wget https://github.com/PYRAX-Chain/pyrax-core/releases/latest/download/pyrax-node-linux-amd64.tar.gz

# Extract
tar -xzf pyrax-node-linux-amd64.tar.gz

# Move to PATH
sudo mv pyrax-node /usr/local/bin/

# Verify
pyrax-node --version
```

### From Source

```bash
# Install Go 1.21+
# Clone repository
git clone https://github.com/PYRAX-Chain/pyrax-core.git
cd pyrax-core

# Build
make pyrax-node

# Install
sudo make install
```

### Docker

```bash
docker pull pyrax/node:latest

docker run -d \
  --name pyrax-node \
  -p 30303:30303 \
  -p 8545:8545 \
  -v pyrax-data:/data \
  pyrax/node:latest
```

## Quick Start

### Mainnet

```bash
pyrax-node --mainnet --http --http.api eth,net,web3
```

### Testnet

```bash
pyrax-node --testnet --http --http.api eth,net,web3
```

## Configuration

### Command Line Options

| Option | Description |
|--------|-------------|
| `--mainnet` | Connect to mainnet |
| `--testnet` | Connect to testnet |
| `--datadir PATH` | Data directory |
| `--http` | Enable HTTP RPC |
| `--http.addr ADDR` | HTTP listen address |
| `--http.port PORT` | HTTP port (default: 8545) |
| `--http.api APIS` | Enabled APIs |
| `--ws` | Enable WebSocket |
| `--ws.port PORT` | WebSocket port |
| `--syncmode MODE` | Sync mode (snap/full) |

### Configuration File

```toml
# config.toml
[Node]
DataDir = "/data/pyrax"
SyncMode = "snap"

[Eth]
NetworkId = 1000

[Node.P2P]
MaxPeers = 50
ListenAddr = ":30303"

[Node.HTTPServer]
Enabled = true
ListenAddr = "127.0.0.1"
Port = 8545
API = ["eth", "net", "web3"]

[Node.WSServer]
Enabled = true
Port = 8546
```

```bash
pyrax-node --config config.toml
```

## Synchronization

### Snap Sync (Recommended)

Fastest initial sync:

```bash
pyrax-node --mainnet --syncmode snap
```

### Full Sync

Complete validation:

```bash
pyrax-node --mainnet --syncmode full
```

### Checking Sync Status

```bash
# Attach console
pyrax-node attach

# Check sync
> eth.syncing
{
  currentBlock: 1234567,
  highestBlock: 2000000,
  startingBlock: 0
}

# When synced
> eth.syncing
false
```

## Systemd Service

Create `/etc/systemd/system/pyrax-node.service`:

```ini
[Unit]
Description=PYRAX Node
After=network.target

[Service]
Type=simple
User=pyrax
ExecStart=/usr/local/bin/pyrax-node --mainnet --http --http.api eth,net,web3
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start
sudo systemctl enable pyrax-node
sudo systemctl start pyrax-node

# Check status
sudo systemctl status pyrax-node

# View logs
journalctl -u pyrax-node -f
```

## Firewall Configuration

```bash
# Allow P2P
sudo ufw allow 30303/tcp
sudo ufw allow 30303/udp

# Allow RPC (only if needed externally)
# WARNING: Secure your RPC if exposing
sudo ufw allow 8545/tcp
```

## Monitoring

### Metrics

Enable Prometheus metrics:

```bash
pyrax-node --metrics --metrics.addr 0.0.0.0 --metrics.port 6060
```

### Health Check

```bash
curl http://localhost:8545 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

## Troubleshooting

### Slow Sync

- Check disk I/O (SSD recommended)
- Verify network bandwidth
- Increase peers: `--maxpeers 100`

### Connection Issues

- Check firewall allows port 30303
- Verify network connectivity
- Check bootnodes are reachable

### Out of Memory

- Increase system RAM
- Enable swap space
- Reduce cache: `--cache 1024`

---

:::info More Information
- [Node Configuration](./node-configuration)
- [Node Maintenance](./node-maintenance)
:::
