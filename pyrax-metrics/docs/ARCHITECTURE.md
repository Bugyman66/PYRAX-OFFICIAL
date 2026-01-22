# PYRAX Observability & Monitoring Architecture

## Overview

This document describes the **observability architecture for the PYRAX blockchain**, designed to monitor:

1. **Individual node health and performance**
2. **Whole-chain (Devnet/Testnet/Mainnet) health and correctness**
3. **Stream-specific health (A/B/C mining streams)**
4. **User experience and transaction flow**

The goal is to provide **production-grade visibility** into PYRAX as a *distributed system*, not just as isolated nodes.

This design intentionally separates **node-level observability** from **chain-level observability**, which is critical for detecting real blockchain failures such as forks, stalls, and network partitions.

---

## Architecture Layers

We are building a **three-layer monitoring system**:

### Layer 1: Node-Level Observability

**Used by:**
- Node operators
- Validator/miner operators
- Core developers

**Purpose:**
- Debug node performance
- Inspect execution, mining, networking
- Diagnose crashes or degraded behavior

**This layer answers:**
> "Is this node healthy?"

---

### Layer 2: Chain-Level Observability (Critical)

**Used by:**
- Core protocol engineers
- Devnet/Testnet maintainers
- CI/CD and release validation
- Leadership and ecosystem operators

**Purpose:**
- Detect consensus failures
- Detect network partitions
- Detect forks or height divergence
- Monitor global liveness and finality
- Track stream-specific health (A/B/C)

**This layer answers:**
> "Is the chain itself healthy?"

---

### Layer 3: User Experience Observability (NEW)

**Used by:**
- Product team
- dApp developers
- Support engineers

**Purpose:**
- Monitor RPC response times and availability
- Track transaction confirmation latency
- Measure end-to-end user experience
- Detect issues invisible to infrastructure monitoring

**This layer answers:**
> "Are users having a good experience?"

---

## Why This Architecture Is Necessary

A blockchain can be **globally unhealthy while individual nodes appear healthy**.

Examples:
- A fork where nodes disagree on head block
- Partial network partition
- Consensus stall visible only across peers
- RPC endpoints returning inconsistent state
- Slow transaction confirmations despite "healthy" metrics

Traditional "per-node monitoring" cannot detect these issues alone.

Therefore, PYRAX monitoring is explicitly designed to be **chain-centric**, not only host-centric.

---

## High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    PYRAX Observability Stack                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐         │
│  │   Node 1     │   │   Node 2     │   │   Node N     │         │
│  │  /metrics    │   │  /metrics    │   │  /metrics    │         │
│  │  Stream A/B/C│   │  Stream A/B/C│   │  Stream A/B/C│         │
│  └──────┬───────┘   └──────┬───────┘   └──────┬───────┘         │
│         │                  │                  │                  │
│         └──────────────────┼──────────────────┘                  │
│                            ▼                                     │
│  ┌───────────────────────────────────────────────────────┐      │
│  │            Prometheus (Node Metrics)                   │      │
│  │  • Scrapes /metrics from each node                     │      │
│  │  • Node-level performance data                         │      │
│  └─────────────────────────┬─────────────────────────────┘      │
│                            │                                     │
│                            ▼                                     │
│  ┌───────────────────────────────────────────────────────┐      │
│  │                    Grafana                             │◄──┐ │
│  │  • Node Performance Dashboards                         │   │ │
│  │  • Chain Health Dashboards                             │   │ │
│  │  • Stream-Specific Dashboards (A/B/C)                  │   │ │
│  │  • User Experience Dashboards                          │   │ │
│  └─────────────────────────┬─────────────────────────────┘   │ │
│                            │                                  │ │
│                            ▼                                  │ │
│  ┌───────────────────────────────────────────────────────┐   │ │
│  │                 Alertmanager                           │   │ │
│  │  • PagerDuty / Slack / Email / Discord                 │   │ │
│  │  • Node alerts vs Chain alerts                         │   │ │
│  └───────────────────────────────────────────────────────┘   │ │
│                                                               │ │
│  ┌───────────────────────────────────────────────────────┐   │ │
│  │           Chain Observer (pyrax-metrics)               │───┘ │
│  │                                                        │     │
│  │  Components:                                           │     │
│  │  ├── RPC Poller        - Polls all RPC endpoints       │     │
│  │  ├── State Aggregator  - Computes chain metrics        │     │
│  │  ├── Fork Detector     - Detects height divergence     │     │
│  │  ├── Finality Tracker  - Tracks finalization lag       │     │
│  │  ├── Stream Monitor    - Per-stream health (A/B/C)     │     │
│  │  ├── Canary TxSender   - Synthetic transactions        │     │
│  │  └── Metrics Server    - Exposes /metrics endpoint     │     │
│  │                                                        │     │
│  │  Endpoints:                                            │     │
│  │  • GET /metrics    - Prometheus metrics                │     │
│  │  • GET /health     - Health check JSON                 │     │
│  │  • GET /status     - Detailed chain status             │     │
│  │                                                        │     │
│  └───────────────────────────────────────────────────────┘     │
│                                                                  │
│  ┌───────────────────────────────────────────────────────┐      │
│  │                 Loki (Log Aggregation)                 │      │
│  │  • Structured logs from all nodes                      │      │
│  │  • Searchable via Grafana                              │      │
│  └───────────────────────────────────────────────────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Layer 1: Node-Level Monitoring

Each PYRAX node embeds a **native metrics service** implemented with an Axum HTTP server.

### Metrics Endpoint

```
http://localhost:9091/metrics
```

### Node Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `pyrax_block_height` | Gauge | Current block height |
| `pyrax_tps` | Gauge | Transactions per second |
| `pyrax_avg_block_time_ms` | Gauge | Average block time in ms |
| `pyrax_pending_txs` | Gauge | Pending transactions in mempool |
| `pyrax_total_peers` | Gauge | Connected peer count |
| `pyrax_hash_rate` | Gauge | Current hash rate |
| `pyrax_active_validators` | Gauge | Active validator count |
| `pyrax_total_staked` | Gauge | Total staked tokens |

These metrics represent **the node's local view** of the chain and its own performance.

---

## Layer 2: Chain Observer (Core Component)

### Purpose

The Chain Observer provides a **single source of truth** for the Devnet/Testnet/Mainnet by observing the chain externally.

It does **not**:
- Participate in consensus
- Mine blocks
- Validate transactions

It only **observes and aggregates**.

---

### Chain Observer Components

#### 1. RPC Poller
For each configured RPC endpoint:
- Calls `eth_blockNumber`
- Calls `eth_syncing`
- Calls `net_peerCount`
- Measures RPC latency and availability

#### 2. State Aggregator
Computes derived chain-level metrics:
- Highest block seen across the network
- Lowest block seen across the network
- Height divergence (fork indicator)
- Percentage of reachable nodes
- Chain liveness / stalled state

#### 3. Fork Detector (NEW)
Tracks block history to detect:
- Chain reorganizations (reorgs)
- Height divergence between nodes
- Conflicting block hashes at same height

#### 4. Finality Tracker (NEW)
Monitors:
- Time from block proposal to finalization
- Finality lag in blocks
- Finality failures

#### 5. Stream Monitor (NEW)
For PYRAX's multi-stream mining (A/B/C):
- Per-stream block height
- Per-stream block rate
- Cross-stream height delta
- Stream-specific stall detection

#### 6. Canary Transaction Sender (NEW)
Periodically:
1. Sends test transaction to network
2. Monitors confirmation time
3. Detects mempool/miner issues
4. Exposes end-to-end latency metrics

---

### Chain-Level Metrics

These metrics are intentionally **aggregated** (low cardinality):

| Metric | Type | Description |
|--------|------|-------------|
| `pyrax_chain_head_block` | Gauge | Highest block seen |
| `pyrax_chain_height_delta` | Gauge | Max height difference between nodes |
| `pyrax_chain_block_rate` | Gauge | Blocks per minute |
| `pyrax_chain_stalled` | Gauge | 1 if no new blocks in threshold |
| `pyrax_nodes_total` | Gauge | Total configured nodes |
| `pyrax_nodes_reachable` | Gauge | Currently reachable nodes |
| `pyrax_nodes_synced` | Gauge | Nodes fully synced |

---

### Stream-Specific Metrics (NEW)

| Metric | Type | Description |
|--------|------|-------------|
| `pyrax_stream_block_height{stream="a"}` | Gauge | Stream A block height |
| `pyrax_stream_block_height{stream="b"}` | Gauge | Stream B block height |
| `pyrax_stream_block_height{stream="c"}` | Gauge | Stream C block height |
| `pyrax_stream_block_rate{stream="a"}` | Gauge | Stream A blocks/min |
| `pyrax_stream_height_delta` | Gauge | Max delta between streams |
| `pyrax_stream_stalled{stream="a"}` | Gauge | 1 if stream stalled |

---

### Fork Detection Metrics (NEW)

| Metric | Type | Description |
|--------|------|-------------|
| `pyrax_chain_reorg_count` | Counter | Total reorgs detected |
| `pyrax_chain_reorg_depth` | Gauge | Depth of last reorg |
| `pyrax_chain_fork_detected` | Gauge | 1 if active fork |
| `pyrax_chain_conflicting_blocks` | Gauge | Blocks with hash conflicts |

---

### Finality Metrics (NEW)

| Metric | Type | Description |
|--------|------|-------------|
| `pyrax_chain_finality_lag_blocks` | Gauge | Blocks behind finality |
| `pyrax_chain_finality_lag_seconds` | Gauge | Time to finality |
| `pyrax_chain_finality_failures` | Counter | Failed finalizations |

---

### Canary Transaction Metrics (NEW)

| Metric | Type | Description |
|--------|------|-------------|
| `pyrax_canary_tx_latency_ms` | Histogram | Transaction confirmation time |
| `pyrax_canary_tx_success_total` | Counter | Successful canary txs |
| `pyrax_canary_tx_failure_total` | Counter | Failed canary txs |
| `pyrax_canary_tx_pending` | Gauge | Currently pending canary txs |
| `pyrax_canary_tx_last_success_timestamp` | Gauge | Unix timestamp of last success |

---

## Layer 3: User Experience Metrics (NEW)

| Metric | Type | Description |
|--------|------|-------------|
| `pyrax_rpc_latency_ms{endpoint="...",method="..."}` | Histogram | RPC response times |
| `pyrax_rpc_success_rate{endpoint="..."}` | Gauge | RPC success percentage |
| `pyrax_rpc_errors_total{endpoint="...",error="..."}` | Counter | RPC errors by type |
| `pyrax_tx_confirmation_latency_ms` | Histogram | User tx confirmation time |
| `pyrax_gas_estimation_accuracy` | Gauge | Gas estimate vs actual |

---

## Chain Observer Endpoints

### GET /metrics
Prometheus-compatible metrics endpoint.

```
# HELP pyrax_chain_head_block Current chain head block
# TYPE pyrax_chain_head_block gauge
pyrax_chain_head_block 12345

# HELP pyrax_chain_height_delta Maximum height difference between nodes
# TYPE pyrax_chain_height_delta gauge
pyrax_chain_height_delta 0
...
```

### GET /health
Health check endpoint for load balancers and CI/CD.

```json
{
  "status": "healthy",
  "chain_status": "live",
  "nodes_checked": 5,
  "nodes_reachable": 5,
  "head_block": 12345,
  "height_delta": 0,
  "streams": {
    "a": {"height": 12345, "status": "live"},
    "b": {"height": 12344, "status": "live"},
    "c": {"height": 12343, "status": "live"}
  },
  "last_check": "2026-01-18T17:12:00Z"
}
```

### GET /status
Detailed chain status for debugging.

```json
{
  "chain": {
    "head_block": 12345,
    "block_rate": 12.5,
    "stalled": false,
    "fork_detected": false
  },
  "nodes": [
    {"endpoint": "http://node1:8545", "height": 12345, "synced": true, "latency_ms": 45},
    {"endpoint": "http://node2:8545", "height": 12345, "synced": true, "latency_ms": 52}
  ],
  "streams": {
    "a": {"height": 12345, "block_rate": 4.2},
    "b": {"height": 12344, "block_rate": 4.1},
    "c": {"height": 12343, "block_rate": 4.2}
  },
  "canary": {
    "last_success": "2026-01-18T17:11:45Z",
    "avg_latency_ms": 2500,
    "success_rate": 0.99
  }
}
```

---

## Prometheus Configuration

Prometheus is used in two roles:

1. **Scraping node-local metrics** (per node)
2. **Scraping chain-level metrics** (single observer)

### Example prometheus.yml

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  # Node-level metrics (scrape each node)
  - job_name: 'pyrax-nodes'
    static_configs:
      - targets:
        - 'node1:9091'
        - 'node2:9091'
        - 'node3:9091'
    relabel_configs:
      - source_labels: [__address__]
        target_label: node
        regex: '(.+):.+'
        replacement: '${1}'

  # Chain-level metrics (scrape observer)
  - job_name: 'pyrax-chain-observer'
    static_configs:
      - targets: ['pyrax-metrics:9092']
    scrape_interval: 10s
```

---

## Grafana Dashboards

### 1. Node Performance Dashboard
- Per-node TPS, block time, peers
- Memory, CPU usage
- Log search integration

**Used by:** Node operators, developers

### 2. Chain Health Dashboard (Most Important)
- Chain head height (all nodes overlay)
- Height divergence indicator
- Block production rate
- % nodes reachable
- Chain stalled indicator
- Fork detection alert

**Used by:** Core protocol team, Devnet/Testnet maintainers

### 3. Stream Health Dashboard (NEW)
- Per-stream block height
- Stream block rates
- Cross-stream height delta
- Stream stall indicators

**Used by:** Mining team, core developers

### 4. User Experience Dashboard (NEW)
- RPC latency (p50, p95, p99)
- Transaction confirmation times
- Canary transaction status
- Error rates by type

**Used by:** Product team, support

---

## Alerting Rules

### Node-Level Alerts

| Alert | Condition | Severity |
|-------|-----------|----------|
| NodeDown | Node metrics unavailable for 2m | warning |
| NodeHighLatency | RPC latency > 500ms for 5m | warning |
| NodeDiskFull | Disk usage > 90% | critical |
| NodeNotSyncing | Node syncing for > 30m | warning |

### Chain-Level Alerts (Critical)

| Alert | Condition | Severity |
|-------|-----------|----------|
| ChainStalled | No new blocks for 5m | critical |
| ChainForkDetected | Height delta > 2 blocks | critical |
| ChainPartition | < 50% nodes reachable | critical |
| ChainHighLatency | Block time > 30s average | warning |

### Stream-Level Alerts (NEW)

| Alert | Condition | Severity |
|-------|-----------|----------|
| StreamStalled | Stream has no blocks for 10m | critical |
| StreamDivergence | Streams differ by > 5 blocks | warning |

### Canary Alerts (NEW)

| Alert | Condition | Severity |
|-------|-----------|----------|
| CanaryTxFailing | Canary tx failed 3 times | warning |
| CanaryTxSlow | Canary latency > 30s | warning |
| CanaryTxDown | No successful canary in 10m | critical |

---

## Chain Observer Implementation

### Technology: Rust

The Chain Observer is implemented in Rust for:
- Consistency with `pyrax-node` codebase
- Shared types and RPC client code
- High performance for frequent polling
- Single deployment pipeline

### Project Structure

```
pyrax-metrics/
├── Cargo.toml
├── README.md
├── docs/
│   └── ARCHITECTURE.md          # This document
├── src/
│   ├── main.rs                   # Entry point
│   ├── config.rs                 # Configuration loading
│   ├── observer/
│   │   ├── mod.rs
│   │   ├── rpc_client.rs         # RPC polling
│   │   ├── aggregator.rs         # State aggregation
│   │   ├── fork_detector.rs      # Fork detection
│   │   ├── finality_tracker.rs   # Finality monitoring
│   │   ├── stream_monitor.rs     # Stream A/B/C health
│   │   └── canary.rs             # Canary transactions
│   ├── metrics/
│   │   ├── mod.rs
│   │   ├── prometheus.rs         # Prometheus exporter
│   │   └── definitions.rs        # Metric definitions
│   └── api/
│       ├── mod.rs
│       ├── health.rs             # /health endpoint
│       └── status.rs             # /status endpoint
├── config/
│   ├── devnet.toml
│   ├── testnet.toml
│   └── mainnet.toml
├── docker/
│   ├── Dockerfile
│   └── docker-compose.yml
└── grafana/
    ├── dashboards/
    │   ├── chain-health.json
    │   ├── node-performance.json
    │   ├── stream-health.json
    │   └── user-experience.json
    ├── alerts/
    │   ├── node-alerts.yaml
    │   ├── chain-alerts.yaml
    │   └── canary-alerts.yaml
    └── provisioning/
        └── datasources.yaml
```

---

## Configuration

### Example config/devnet.toml

```toml
[observer]
poll_interval_ms = 5000
request_timeout_ms = 3000

[nodes]
endpoints = [
  "http://devnet-node1.pyrax.io:8545",
  "http://devnet-node2.pyrax.io:8545",
  "http://devnet-node3.pyrax.io:8545"
]

[chain]
expected_block_time_ms = 5000
stall_threshold_ms = 300000       # 5 minutes
fork_threshold_blocks = 2

[streams]
enabled = true
stream_ids = ["a", "b", "c"]

[canary]
enabled = true
interval_ms = 60000               # 1 minute
wallet_private_key = "${CANARY_WALLET_KEY}"
gas_limit = 21000
value_wei = 0

[metrics]
listen_addr = "0.0.0.0:9092"

[api]
listen_addr = "0.0.0.0:8080"
```

---

## Deployment

### Docker Compose

```yaml
version: '3.8'

services:
  pyrax-metrics:
    build: .
    ports:
      - "9092:9092"   # Prometheus metrics
      - "8080:8080"   # Health/Status API
    environment:
      - CONFIG_PATH=/config/devnet.toml
      - CANARY_WALLET_KEY=${CANARY_WALLET_KEY}
    volumes:
      - ./config:/config:ro
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./grafana/provisioning:/etc/grafana/provisioning/datasources
      - grafana_data:/var/lib/grafana
    restart: unless-stopped

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml
    restart: unless-stopped

volumes:
  prometheus_data:
  grafana_data:
```

---

## Implementation Phases

### Phase 1: Core Observer (MVP)
- [ ] RPC polling and state aggregation
- [ ] Basic chain metrics
- [ ] Prometheus /metrics endpoint
- [ ] /health endpoint
- [ ] Basic Grafana dashboard

### Phase 2: Enhanced Detection
- [ ] Fork detector
- [ ] Stream monitoring (A/B/C)
- [ ] Stream-specific dashboards
- [ ] Enhanced alerts

### Phase 3: User Experience
- [ ] Canary transaction system
- [ ] RPC latency tracking
- [ ] User experience dashboard
- [ ] Finality tracking

### Phase 4: Production Hardening
- [ ] Grafana Cloud integration
- [ ] High availability setup
- [ ] Documentation
- [ ] Runbooks

---

## Design Principles

1. **Chain-centric, not host-centric** - Focus on whole-chain health
2. **Low cardinality metrics** - Avoid label explosion for scalability
3. **Prometheus-compatible everywhere** - Industry standard
4. **Observer-based aggregation** - Single source of truth
5. **Stream-aware** - Understands PYRAX's multi-stream architecture
6. **Scales from Devnet → Mainnet** - Same architecture, different configs
7. **Grafana Cloud–ready but self-host friendly** - Flexible deployment

---

## Current Status

| Component | Status |
|-----------|--------|
| Node-level metrics (`/metrics`) | ✅ Implemented |
| PLG Stack (Prometheus/Loki/Grafana) | ✅ Implemented |
| Chain Observer service | ⏳ Planned |
| Stream monitoring (A/B/C) | ⏳ Planned |
| Fork/reorg detection | ⏳ Planned |
| Canary transaction system | ⏳ Planned |
| User experience metrics | ⏳ Planned |
| Chain dashboards & alerts | ⏳ Planned |

---

## End Goal

The end goal is a **robust, production-grade observability system** that allows the PYRAX team to confidently answer:

- ✅ Is the chain alive?
- ✅ Is consensus healthy?
- ✅ Are nodes diverging?
- ✅ Is the network partitioned?
- ✅ Are individual streams healthy?
- ✅ Are users experiencing issues?
- ✅ Is this release safe to promote?

If those questions can be answered reliably, the monitoring system is considered complete.

---

## License

Part of the PYRAX blockchain project.
