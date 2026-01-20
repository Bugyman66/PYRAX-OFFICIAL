# PYRAX Metrics

Chain Observer and Monitoring Service for the PYRAX Blockchain.

## Overview

`pyrax-metrics` is a lightweight observability service that monitors the health of the PYRAX blockchain network by polling multiple nodes and aggregating chain-level metrics.

## Features

- **Multi-node monitoring** - Polls multiple RPC endpoints concurrently
- **Chain-level metrics** - Height, block rate, stall detection
- **Stream monitoring** - Tracks Stream A/B/C health independently
- **Fork detection** - Detects chain divergence across nodes
- **Prometheus metrics** - Standard `/metrics` endpoint
- **Health API** - JSON health and status endpoints

## Quick Start

```bash
# Build
cargo build --release

# Run with config
./target/release/pyrax-metrics --config config/devnet.toml

# Or with environment variables
PYRAX_METRICS_CONFIG=config/devnet.toml ./target/release/pyrax-metrics
```

## Configuration

See `config/devnet.toml` for example configuration.

## Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /metrics` | Prometheus metrics |
| `GET /health` | Health check (JSON) |
| `GET /status` | Detailed chain status (JSON) |

## Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - Technical architecture
- [CODEBASE_ALIGNMENT.md](docs/CODEBASE_ALIGNMENT.md) - pyrax-node integration
- [IMPLEMENTATION_ROADMAP.md](docs/IMPLEMENTATION_ROADMAP.md) - Development phases

## License

MIT License - Part of the PYRAX blockchain project.
