# Node Configuration

Advanced configuration options for PYRAX nodes.

## Configuration Methods

1. **Command line flags** — Direct arguments
2. **Configuration file** — TOML format
3. **Environment variables** — For containers

## Complete Configuration Reference

### Network Options

```bash
--mainnet              # Use mainnet
--testnet              # Use testnet
--networkid VALUE      # Custom network ID
--bootnodes NODES      # Comma-separated enode URLs
--maxpeers VALUE       # Maximum peers (default: 50)
--nodiscover           # Disable peer discovery
```

### Data Options

```bash
--datadir PATH         # Data directory
--ancient PATH         # Ancient data directory
--cache VALUE          # Cache memory in MB (default: 4096)
--gcmode MODE          # GC mode: full, archive
```

### Sync Options

```bash
--syncmode MODE        # Sync mode: snap, full
--exitwhensynced       # Exit after sync complete
```

### RPC Options

```bash
# HTTP
--http                 # Enable HTTP RPC
--http.addr ADDR       # Listen address (default: localhost)
--http.port PORT       # Port (default: 8545)
--http.api APIS        # Enabled APIs
--http.corsdomain "*"  # CORS domains
--http.vhosts "*"      # Virtual hosts

# WebSocket  
--ws                   # Enable WebSocket
--ws.addr ADDR         # Listen address
--ws.port PORT         # Port (default: 8546)
--ws.api APIS          # Enabled APIs
--ws.origins "*"       # Allowed origins
```

### Mining Options

```bash
--mine                 # Enable mining
--miner.etherbase ADDR # Reward address
--miner.threads VALUE  # Mining threads
--miner.gasprice VALUE # Minimum gas price
```

### Performance Options

```bash
--cache VALUE          # Cache size in MB
--cache.database %     # Database cache percentage
--cache.trie %         # Trie cache percentage
--cache.gc %           # GC cache percentage
```

## Sample Configurations

### Full Node

```toml
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
VirtualHosts = ["localhost"]
```

### Archive Node

```toml
[Node]
DataDir = "/data/pyrax-archive"
SyncMode = "full"

[Eth]
NetworkId = 1000
NoPruning = true
GcMode = "archive"

[Node.P2P]
MaxPeers = 100
```

### Mining Node

```toml
[Node]
DataDir = "/data/pyrax-miner"
SyncMode = "snap"

[Eth]
NetworkId = 1000

[Eth.Miner]
Etherbase = "0xYOUR_ADDRESS"
GasPrice = 1000000000

[Node.P2P]
MaxPeers = 50
```

### RPC Provider

```toml
[Node]
DataDir = "/data/pyrax"
SyncMode = "snap"

[Node.HTTPServer]
Enabled = true
ListenAddr = "0.0.0.0"
Port = 8545
API = ["eth", "net", "web3", "txpool"]
VirtualHosts = ["*"]
Cors = ["*"]

[Node.WSServer]
Enabled = true
ListenAddr = "0.0.0.0"
Port = 8546
API = ["eth", "net", "web3"]
Origins = ["*"]
```

## API Namespaces

| Namespace | Description |
|-----------|-------------|
| `eth` | Ethereum protocol |
| `net` | Network info |
| `web3` | Web3 utilities |
| `txpool` | Transaction pool |
| `debug` | Debugging (careful!) |
| `admin` | Node administration |
| `personal` | Account management |
| `pyrax` | PYRAX-specific |

## Security Configuration

### Secure RPC

```bash
# Local only (recommended)
--http.addr 127.0.0.1

# With authentication
--authrpc.jwtsecret /path/to/jwt.hex

# Rate limiting via reverse proxy
```

### Firewall

```bash
# Allow only P2P
sudo ufw allow 30303/tcp
sudo ufw allow 30303/udp

# Block external RPC
sudo ufw deny 8545
```

## Docker Configuration

```yaml
# docker-compose.yml
version: '3.8'
services:
  pyrax-node:
    image: pyrax/node:latest
    ports:
      - "30303:30303"
      - "8545:8545"
    volumes:
      - pyrax-data:/data
      - ./config.toml:/config.toml
    command: --config /config.toml
    restart: unless-stopped

volumes:
  pyrax-data:
```

## Environment Variables

```bash
export PYRAX_DATADIR=/data/pyrax
export PYRAX_NETWORK=mainnet
export PYRAX_HTTP_PORT=8545
```

---

:::tip Best Practices
1. Use configuration files for complex setups
2. Never expose RPC publicly without auth
3. Monitor disk space regularly
4. Keep node software updated
:::
