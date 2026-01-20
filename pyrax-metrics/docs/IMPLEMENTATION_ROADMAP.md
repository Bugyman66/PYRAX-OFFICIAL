# PYRAX Metrics - Implementation Roadmap

## Overview

This document outlines the phased implementation plan for `pyrax-metrics`, the PYRAX blockchain observability and monitoring system.

**Total Estimated Duration:** 4-6 weeks  
**Tech Stack:** Rust + Axum + Prometheus + Grafana

---

## Phase 0: Prerequisites (Before Starting)

### Duration: 1-2 days

### Tasks

| # | Task | Priority | Owner |
|---|------|----------|-------|
| 0.1 | Add `stream_c: StreamMetrics` to `MiningMetrics` in pyrax-node | P0 | Core |
| 0.2 | Add stream-specific metrics to `prometheus_export()` | P0 | Core |
| 0.3 | Verify all RPC endpoints are accessible | P0 | Core |
| 0.4 | Set up development environment | P0 | Dev |

### Deliverables
- [ ] Updated `pyrax-node/src/services/metrics.rs` with Stream C
- [ ] All 3 streams exposed via Prometheus
- [ ] Verified RPC endpoints: `get_chain_info`, `get_mining_info`, `get_stream_c_info`

---

## Phase 1: Project Foundation

### Duration: 2-3 days

### 1.1 Project Structure Setup

```
pyrax-metrics/
├── Cargo.toml
├── README.md
├── .env.example
├── docs/
│   ├── ARCHITECTURE.md         ✅ Done
│   ├── CODEBASE_ALIGNMENT.md   ✅ Done
│   └── IMPLEMENTATION_ROADMAP.md  ✅ This file
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   └── lib.rs
├── config/
│   ├── devnet.toml
│   ├── testnet.toml
│   └── mainnet.toml
└── docker/
    └── Dockerfile
```

### 1.2 Core Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
reqwest = { version = "0.11", features = ["json"] }
prometheus = "0.13"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1"
```

### 1.3 Configuration System

```rust
// config.rs
pub struct Config {
    pub observer: ObserverConfig,
    pub nodes: Vec<NodeConfig>,
    pub streams: StreamConfig,
    pub metrics: MetricsServerConfig,
    pub api: ApiServerConfig,
}
```

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 1.1 | Create Cargo.toml with dependencies | 30 min |
| 1.2 | Implement config.rs with TOML parsing | 2 hrs |
| 1.3 | Create config templates (devnet/testnet/mainnet) | 1 hr |
| 1.4 | Set up error handling module | 1 hr |
| 1.5 | Create main.rs with basic CLI | 1 hr |
| 1.6 | Add logging with tracing | 30 min |

### Deliverables
- [ ] Project compiles and runs
- [ ] Loads configuration from TOML file
- [ ] Logging works

### Verification
```bash
cd pyrax-metrics
cargo build
cargo run -- --config config/devnet.toml
# Should print: "Starting PYRAX Chain Observer..."
```

---

## Phase 2: RPC Client & Node Polling

### Duration: 3-4 days

### 2.1 RPC Client Implementation

```rust
// src/observer/rpc_client.rs
pub struct RpcClient {
    endpoint: String,
    client: reqwest::Client,
    timeout: Duration,
}

impl RpcClient {
    pub async fn get_block_number(&self) -> Result<u64>;
    pub async fn get_syncing(&self) -> Result<SyncStatus>;
    pub async fn get_chain_info(&self) -> Result<ChainInfo>;
    pub async fn get_mining_info(&self) -> Result<MiningInfo>;
    pub async fn get_stream_c_info(&self) -> Result<StreamCInfo>;
    pub async fn health(&self) -> Result<bool>;
}
```

### 2.2 Node Poller

```rust
// src/observer/poller.rs
pub struct NodePoller {
    nodes: Vec<RpcClient>,
    poll_interval: Duration,
}

impl NodePoller {
    pub async fn poll_all(&self) -> Vec<NodeStatus>;
    pub async fn start(&self, tx: mpsc::Sender<Vec<NodeStatus>>);
}
```

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 2.1 | Implement RpcClient with all methods | 4 hrs |
| 2.2 | Add connection pooling and timeouts | 2 hrs |
| 2.3 | Create NodePoller with async polling | 3 hrs |
| 2.4 | Add retry logic with exponential backoff | 2 hrs |
| 2.5 | Create NodeStatus data structures | 1 hr |
| 2.6 | Write unit tests for RPC client | 2 hrs |

### Deliverables
- [ ] RPC client connects to pyrax-node
- [ ] Polls multiple nodes concurrently
- [ ] Handles connection failures gracefully

### Verification
```bash
# Start pyrax-node first
cargo run -- --config config/devnet.toml

# Logs should show:
# [INFO] Polling node http://localhost:8545... height=1234
# [INFO] Polling node http://localhost:8546... height=1234
```

---

## Phase 3: State Aggregator & Chain Metrics

### Duration: 3-4 days

### 3.1 State Aggregator

```rust
// src/observer/aggregator.rs
pub struct StateAggregator {
    node_states: HashMap<String, NodeState>,
}

impl StateAggregator {
    pub fn update(&mut self, statuses: Vec<NodeStatus>);
    pub fn chain_head(&self) -> u64;
    pub fn height_delta(&self) -> u64;
    pub fn nodes_reachable(&self) -> usize;
    pub fn is_stalled(&self, threshold: Duration) -> bool;
}
```

### 3.2 Chain Metrics

| Metric | Type | Source |
|--------|------|--------|
| `pyrax_chain_head_block` | Gauge | max(block_heights) |
| `pyrax_chain_height_delta` | Gauge | max - min heights |
| `pyrax_chain_block_rate` | Gauge | blocks/minute |
| `pyrax_chain_stalled` | Gauge | 0 or 1 |
| `pyrax_nodes_total` | Gauge | config count |
| `pyrax_nodes_reachable` | Gauge | successful polls |
| `pyrax_nodes_synced` | Gauge | syncing == false |

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 3.1 | Implement StateAggregator | 3 hrs |
| 3.2 | Create chain metrics calculations | 2 hrs |
| 3.3 | Track block production rate | 2 hrs |
| 3.4 | Implement stall detection | 1 hr |
| 3.5 | Add time-series ring buffer for history | 2 hrs |
| 3.6 | Write unit tests | 2 hrs |

### Deliverables
- [ ] Aggregates state from multiple nodes
- [ ] Calculates chain-level metrics
- [ ] Detects stalled chain

### Verification
```bash
# Run with 2+ nodes
cargo run -- --config config/devnet.toml

# Check aggregator logs:
# [INFO] Chain state: head=1234, delta=0, reachable=3/3
```

---

## Phase 4: Stream Monitor (A/B/C)

### Duration: 2-3 days

### 4.1 Stream Monitor

```rust
// src/observer/stream_monitor.rs
pub struct StreamMonitor {
    stream_states: HashMap<Stream, StreamState>,
}

impl StreamMonitor {
    pub fn update_stream_a(&mut self, info: MiningInfo);
    pub fn update_stream_b(&mut self, info: MiningInfo);
    pub fn update_stream_c(&mut self, info: StreamCInfo);
    pub fn stream_height(&self, stream: Stream) -> u64;
    pub fn stream_delta(&self) -> u64;
    pub fn is_stream_stalled(&self, stream: Stream) -> bool;
}
```

### 4.2 Stream Metrics

| Metric | Type |
|--------|------|
| `pyrax_stream_block_height{stream="a"}` | Gauge |
| `pyrax_stream_block_height{stream="b"}` | Gauge |
| `pyrax_stream_block_height{stream="c"}` | Gauge |
| `pyrax_stream_block_rate{stream="a"}` | Gauge |
| `pyrax_stream_hash_rate{stream="a"}` | Gauge |
| `pyrax_stream_height_delta` | Gauge |
| `pyrax_stream_stalled{stream="a"}` | Gauge |

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 4.1 | Implement StreamMonitor | 3 hrs |
| 4.2 | Add per-stream metrics | 2 hrs |
| 4.3 | Implement stream stall detection | 1 hr |
| 4.4 | Calculate cross-stream delta | 1 hr |
| 4.5 | Write unit tests | 2 hrs |

### Deliverables
- [ ] Tracks all 3 streams independently
- [ ] Detects per-stream stalls
- [ ] Calculates cross-stream divergence

---

## Phase 5: Fork Detector

### Duration: 2-3 days

### 5.1 Fork Detector

```rust
// src/observer/fork_detector.rs
pub struct ForkDetector {
    block_history: RingBuffer<BlockSnapshot>,
    seen_hashes: HashMap<u64, HashSet<H256>>,
}

impl ForkDetector {
    pub fn record_block(&mut self, height: u64, hash: H256, source: &str);
    pub fn detect_fork(&self) -> Option<ForkInfo>;
    pub fn reorg_depth(&self) -> u64;
}
```

### 5.2 Fork Metrics

| Metric | Type |
|--------|------|
| `pyrax_chain_fork_detected` | Gauge |
| `pyrax_chain_reorg_count` | Counter |
| `pyrax_chain_reorg_depth` | Gauge |
| `pyrax_chain_conflicting_blocks` | Gauge |

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 5.1 | Implement ForkDetector with ring buffer | 3 hrs |
| 5.2 | Track block hashes per height | 2 hrs |
| 5.3 | Implement fork detection logic | 2 hrs |
| 5.4 | Add reorg tracking | 2 hrs |
| 5.5 | Write unit tests | 2 hrs |

### Deliverables
- [ ] Detects when nodes report different block hashes
- [ ] Tracks reorg depth
- [ ] Maintains block history

---

## Phase 6: Prometheus Metrics Server

### Duration: 1-2 days

### 6.1 Metrics Server

```rust
// src/metrics/server.rs
pub struct MetricsServer {
    registry: prometheus::Registry,
    gauges: HashMap<String, Gauge>,
    counters: HashMap<String, Counter>,
}

impl MetricsServer {
    pub fn new() -> Self;
    pub fn register_all_metrics(&mut self);
    pub fn update(&self, chain_state: &ChainState);
    pub async fn serve(&self, addr: SocketAddr);
}
```

### 6.2 Endpoints

| Endpoint | Response |
|----------|----------|
| `GET /metrics` | Prometheus format |
| `GET /health` | JSON health check |
| `GET /status` | JSON detailed status |

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 6.1 | Set up Prometheus registry | 1 hr |
| 6.2 | Register all metrics | 2 hrs |
| 6.3 | Implement /metrics endpoint | 1 hr |
| 6.4 | Implement /health endpoint | 1 hr |
| 6.5 | Implement /status endpoint | 2 hrs |
| 6.6 | Add Axum HTTP server | 1 hr |

### Deliverables
- [ ] `/metrics` returns Prometheus format
- [ ] `/health` returns JSON health status
- [ ] `/status` returns detailed chain status

### Verification
```bash
cargo run -- --config config/devnet.toml

# Test endpoints
curl http://localhost:9092/metrics
curl http://localhost:8080/health
curl http://localhost:8080/status
```

---

## Phase 7: Canary Transaction System (Optional)

### Duration: 2-3 days

### 7.1 Canary Transaction Sender

```rust
// src/observer/canary.rs
pub struct CanarySystem {
    wallet_key: SecretKey,
    interval: Duration,
    pending_txs: HashMap<H256, Instant>,
}

impl CanarySystem {
    pub async fn send_canary_tx(&mut self) -> Result<H256>;
    pub async fn check_confirmations(&mut self) -> Vec<CanaryResult>;
    pub fn metrics(&self) -> CanaryMetrics;
}
```

### 7.2 Canary Metrics

| Metric | Type |
|--------|------|
| `pyrax_canary_tx_latency_ms` | Histogram |
| `pyrax_canary_tx_success_total` | Counter |
| `pyrax_canary_tx_failure_total` | Counter |
| `pyrax_canary_tx_pending` | Gauge |

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 7.1 | Implement transaction signing | 2 hrs |
| 7.2 | Create canary sender loop | 2 hrs |
| 7.3 | Track pending transactions | 2 hrs |
| 7.4 | Measure confirmation latency | 2 hrs |
| 7.5 | Add canary metrics | 1 hr |
| 7.6 | Write integration tests | 2 hrs |

### Deliverables
- [ ] Sends periodic test transactions
- [ ] Measures confirmation time
- [ ] Detects transaction failures

### Note
This phase requires a funded wallet on devnet/testnet.

---

## Phase 8: Grafana Dashboards

### Duration: 2-3 days

### 8.1 Dashboard Files

```
grafana/
├── dashboards/
│   ├── chain-health.json
│   ├── node-performance.json
│   ├── stream-health.json
│   └── user-experience.json
├── alerts/
│   ├── chain-alerts.yaml
│   ├── node-alerts.yaml
│   └── stream-alerts.yaml
└── provisioning/
    ├── dashboards.yaml
    └── datasources.yaml
```

### 8.2 Dashboards

| Dashboard | Panels |
|-----------|--------|
| Chain Health | Head block, height delta, block rate, stalled indicator, fork alert |
| Node Performance | Per-node TPS, block time, peer count, sync status |
| Stream Health | Stream A/B/C heights, stream rates, stream stall indicators |
| User Experience | RPC latency, canary tx status, success rates |

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 8.1 | Create chain-health dashboard | 3 hrs |
| 8.2 | Create node-performance dashboard | 2 hrs |
| 8.3 | Create stream-health dashboard | 2 hrs |
| 8.4 | Create user-experience dashboard | 2 hrs |
| 8.5 | Define alert rules | 2 hrs |
| 8.6 | Create provisioning configs | 1 hr |

### Deliverables
- [ ] 4 Grafana dashboards
- [ ] Alert rules for critical conditions
- [ ] Auto-provisioning configuration

### Verification
```bash
docker-compose up -d

# Open Grafana at http://localhost:3000
# Verify dashboards load with data
```

---

## Phase 9: Docker & Deployment

### Duration: 1-2 days

### 9.1 Docker Setup

```dockerfile
# docker/Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/pyrax-metrics /usr/local/bin/
CMD ["pyrax-metrics", "--config", "/config/config.toml"]
```

### 9.2 Docker Compose

```yaml
# docker-compose.yml
services:
  pyrax-metrics:
    build: .
    ports:
      - "9092:9092"
      - "8080:8080"
    volumes:
      - ./config:/config
    environment:
      - RUST_LOG=info

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - ./grafana:/etc/grafana/provisioning
```

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 9.1 | Create Dockerfile | 1 hr |
| 9.2 | Create docker-compose.yml | 1 hr |
| 9.3 | Create prometheus.yml | 30 min |
| 9.4 | Set up volume mounts | 30 min |
| 9.5 | Write deployment documentation | 1 hr |
| 9.6 | Test full stack deployment | 2 hrs |

### Deliverables
- [ ] Working Dockerfile
- [ ] docker-compose.yml with full stack
- [ ] Deployment documentation

### Verification
```bash
docker-compose up -d
docker-compose ps
# All services should show "Up"

curl http://localhost:9092/metrics
curl http://localhost:9090/api/v1/targets
```

---

## Phase 10: Testing & Documentation

### Duration: 2-3 days

### 10.1 Test Coverage

| Test Type | Coverage |
|-----------|----------|
| Unit Tests | RPC client, aggregator, fork detector |
| Integration Tests | Full polling loop, metrics export |
| E2E Tests | Docker stack, Grafana queries |

### 10.2 Documentation

| Document | Purpose |
|----------|---------|
| README.md | Quick start guide |
| ARCHITECTURE.md | Technical design |
| DEPLOYMENT.md | Production deployment |
| OPERATIONS.md | Runbooks and alerts |

### Tasks

| # | Task | Est. Time |
|---|------|-----------|
| 10.1 | Write unit tests (80% coverage) | 4 hrs |
| 10.2 | Write integration tests | 3 hrs |
| 10.3 | Create README.md | 1 hr |
| 10.4 | Create DEPLOYMENT.md | 2 hrs |
| 10.5 | Create OPERATIONS.md (runbooks) | 2 hrs |
| 10.6 | Final code review | 2 hrs |

### Deliverables
- [ ] 80%+ test coverage
- [ ] Complete documentation
- [ ] Ready for production

---

## Summary Timeline

```
Week 1:
├── Phase 0: Prerequisites (1-2 days)
├── Phase 1: Project Foundation (2-3 days)
└── Phase 2: RPC Client (start)

Week 2:
├── Phase 2: RPC Client (complete)
├── Phase 3: State Aggregator (3-4 days)
└── Phase 4: Stream Monitor (start)

Week 3:
├── Phase 4: Stream Monitor (complete)
├── Phase 5: Fork Detector (2-3 days)
└── Phase 6: Prometheus Server (1-2 days)

Week 4:
├── Phase 7: Canary System (optional, 2-3 days)
├── Phase 8: Grafana Dashboards (2-3 days)
└── Phase 9: Docker (start)

Week 5:
├── Phase 9: Docker (complete)
├── Phase 10: Testing & Docs (2-3 days)
└── Final review and release
```

---

## Milestone Checkpoints

### Milestone 1: MVP (End of Week 2)
- [x] Project compiles
- [ ] Polls multiple nodes
- [ ] Basic chain metrics
- [ ] `/metrics` endpoint works

### Milestone 2: Feature Complete (End of Week 3)
- [ ] All 3 streams monitored
- [ ] Fork detection works
- [ ] All endpoints functional

### Milestone 3: Production Ready (End of Week 4-5)
- [ ] Grafana dashboards complete
- [ ] Docker deployment works
- [ ] Documentation complete
- [ ] 80% test coverage

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| pyrax-node RPC changes | Version check, graceful degradation |
| Network connectivity issues | Retry logic, circuit breaker |
| High cardinality metrics | Label limits, aggregation |
| Canary wallet runs out of funds | Auto-funding alerts |

---

## Success Criteria

1. **Chain Observer detects stalled chain within 1 minute**
2. **Fork detection triggers alert within 30 seconds**
3. **Stream divergence visible in dashboard**
4. **< 100ms overhead for metrics collection**
5. **99.9% uptime for observer service**
