# 🔥 Inferno CLI - Installation & Usage Guide

Inferno CLI is a terminal-based node management tool for the PYRAX blockchain network. It provides complete command-line control over node instances, Docker containers, remote nodes, and includes a web-based monitoring dashboard.

## Quick Install

### Linux / macOS / WSL2

```bash
curl -fsSL https://get.pyrax-devnet.org/cli | bash
```

Or directly from GitHub:
```bash
curl -fsSL https://raw.githubusercontent.com/PYRAX-Chain/PYRAX-OFFICIAL/devnet/inferno-cli/install.sh | bash
```

### macOS (Homebrew)

```bash
brew tap pyrax-chain/tap
brew install inferno
```

### Manual Installation

Download the appropriate binary for your platform from the [releases page](https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases).

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | `inferno-cli-VERSION-linux-x86_64.tar.gz` |
| Linux | ARM64 | `inferno-cli-VERSION-linux-aarch64.tar.gz` |
| macOS | Intel | `inferno-cli-VERSION-darwin-x86_64.tar.gz` |
| macOS | Apple Silicon | `inferno-cli-VERSION-darwin-aarch64.tar.gz` |

Extract and install:

```bash
tar -xzf inferno-cli-*.tar.gz
sudo mv inferno /usr/local/bin/
sudo chmod +x /usr/local/bin/inferno
```

## Getting Started

### 1. Initialize Your Node

```bash
inferno init
```

This launches an interactive setup wizard that will:
- Create your node configuration
- Select network (devnet/testnet/mainnet)
- Configure ports (with auto-conflict resolution)
- Set up mining options

For quick setup with defaults:
```bash
inferno init --quick
```

### 2. Start Your Node

```bash
inferno start
```

Run in foreground mode (for debugging):
```bash
inferno start --foreground
```

### 3. View Logs

```bash
inferno logs
```

Follow logs in real-time:
```bash
inferno logs --follow
```

### 4. Open Dashboard

```bash
inferno dashboard --open
```

This starts a web-based monitoring dashboard at `http://localhost:8080`.

## Running Multiple Nodes

Inferno CLI supports running multiple node instances on the same machine.

### Initialize Additional Instances

```bash
inferno init --instance 2
inferno init --instance 3
```

Ports are automatically assigned to avoid conflicts:
- Instance 1: P2P 30303, RPC 28545
- Instance 2: P2P 30313, RPC 28555
- Instance 3: P2P 30323, RPC 28565

### Start All Instances

```bash
inferno start --all
```

### View All Instances

```bash
inferno list --detailed
```

## Docker Support

### Run Node in Docker

```bash
inferno docker run
```

Run in background:
```bash
inferno docker run --detach
```

With custom ports:
```bash
inferno docker run --p2p-port 30303 --rpc-port 28545
```

### Manage Containers

```bash
inferno docker ps              # List containers
inferno docker logs            # View logs
inferno docker stop            # Stop container
inferno docker shell           # Open shell in container
```

### Direct Docker Usage

```bash
docker pull ghcr.io/pyrax-chain/inferno-node:latest

docker run -d \
  -p 30303:30303 \
  -p 28545:28545 \
  -v ~/.inferno:/data \
  ghcr.io/pyrax-chain/inferno-node:latest
```

## Remote Node Management

Manage nodes on remote servers via SSH.

### Add a Remote

```bash
inferno remote add prod-1 user@192.168.1.100
inferno remote add aws-node ec2-user@ec2-xxx.compute.amazonaws.com -i ~/.ssh/aws.pem
```

### Manage Remote Nodes

```bash
inferno remote list                  # List all remotes
inferno remote status prod-1         # Check status
inferno remote start prod-1          # Start node
inferno remote stop prod-1           # Stop node
inferno remote logs prod-1 --follow  # View logs
inferno remote shell prod-1          # SSH into server
```

### Deploy to Remotes

```bash
inferno remote deploy prod-1         # Deploy/update inferno
inferno remote deploy --all          # Deploy to all remotes
```

## System Service

Install as a system service for automatic startup.

### Linux (systemd)

```bash
sudo inferno service install --instance 1
sudo systemctl enable inferno@1
sudo systemctl start inferno@1
```

### macOS (launchd)

```bash
inferno service install --instance 1
inferno service start --instance 1
```

### Service Commands

```bash
inferno service status --instance 1
inferno service logs --instance 1 --follow
inferno service stop --instance 1
inferno service uninstall --instance 1
```

## Configuration

### View Configuration

```bash
inferno config show
```

### Modify Configuration

```bash
inferno config set network.p2p_port 30303
inferno config set mining.enabled true
inferno config set mining.threads 4
```

### Edit Configuration File

```bash
inferno config edit
```

Configuration file location: `~/.inferno/devnet/instance-1/config.toml`

### Configuration Options

```toml
[node]
instance_id = 1
data_dir = "~/.inferno/devnet/instance-1/data"
log_level = "info"

[network]
network = "devnet"
p2p_port = 30303
rpc_port = 28545
rpc_enabled = true
max_peers = 50
bootnodes = [
  "/ip4/209.38.137.105/tcp/30303/p2p/...",
  "/ip4/137.184.118.228/tcp/30303/p2p/..."
]

[mining]
enabled = false
threads = 4
wallet = ""
```

## Command Reference

| Command | Description |
|---------|-------------|
| `inferno init` | Interactive setup wizard |
| `inferno start` | Start node |
| `inferno stop` | Stop node |
| `inferno restart` | Restart node |
| `inferno status` | Show node status |
| `inferno list` | List all instances |
| `inferno logs` | View logs |
| `inferno peers` | Show connected peers |
| `inferno mining` | Mining status/control |
| `inferno config` | Configuration management |
| `inferno docker` | Docker commands |
| `inferno remote` | Remote node management |
| `inferno service` | System service management |
| `inferno dashboard` | Web monitoring dashboard |
| `inferno --help` | Full help |

## Shell Completions

Generate shell completions:

```bash
# Bash
inferno completions bash > ~/.bash_completion.d/inferno

# Zsh
inferno completions zsh > ~/.zsh/completions/_inferno

# Fish
inferno completions fish > ~/.config/fish/completions/inferno.fish
```

## Troubleshooting

### Port Already in Use

Inferno automatically detects port conflicts. If you need to manually specify ports:

```bash
inferno config set network.p2p_port 30304
inferno config set network.rpc_port 28546
inferno restart
```

### Node Won't Start

1. Check logs: `inferno logs --tail 100`
2. Verify configuration: `inferno config show`
3. Check if another instance is running: `inferno list`
4. Try running in foreground: `inferno start --foreground`

### Connection Issues

1. Check peer count: `inferno peers`
2. Verify bootnodes are reachable
3. Check firewall settings for P2P port

### Docker Issues

1. Ensure Docker is running: `docker info`
2. Check container status: `inferno docker ps --all`
3. View container logs: `inferno docker logs --follow`

## Support

- **GitHub Issues**: https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/issues
- **Discord**: https://discord.gg/pyrax
- **Documentation**: https://docs.pyrax.org

## License

MIT License - See [LICENSE](../LICENSE) for details.
