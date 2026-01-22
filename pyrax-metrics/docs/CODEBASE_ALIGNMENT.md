# PYRAX Metrics - Codebase Alignment Analysis

## Overview

This document analyzes the alignment between the proposed `pyrax-metrics` architecture and the existing `pyrax-node` codebase.

---

## ✅ Fully Aligned Components

### 1. Node Metrics Already Exist

**Location:** `pyrax-node/src/services/metrics.rs`

The existing metrics system includes:

| Metric Type | Fields | Prometheus Export |
|-------------|--------|-------------------|
| `ChainMetrics` | block_height, tps, avg_block_time_ms, pending_txs, finality_secs, etc. | ✅ |
| `NetworkMetrics` | total_nodes, active_nodes, hash_rate, avg_peer_count, geo_distribution | ✅ |
| `MiningMetrics` | difficulty, hash_rate, stream_a, stream_b metrics | ✅ |
| `StakingMetrics` | total_staked, staking_ratio, avg_apy, validators | ✅ |
| `ZkMetrics` | proofs_24h, avg_proof_time_ms, burn_rate | ✅ |
| `TokenMetrics` | total_supply, circulating_supply, total_burned | ✅ |

**Prometheus endpoint:** `prometheus_export()` method at port 9090 (default).

### 2. Stream Definitions Exist

**Location:** `pyrax-node/src/consensus/streams.rs`

All three streams are defined:
```rust
pub enum Stream {
    A = 0,  // BLAKE3, 10s blocks
    B = 1,  // KAWPOW, 60s blocks
    C = 2,  // ZK-STARK + PoS
}
```

### 3. RPC Endpoints Ready for Chain Observer

The following RPC methods exist and can be polled by the Chain Observer:

| Method | Location | Status |
|--------|----------|--------|
| `get_chain_info` | `rpc/server.rs` | ✅ Available |
| `get_network_info` | `rpc/server.rs` | ✅ Available |
| `get_mining_info` | `rpc/mining_rpc.rs` | ✅ Available |
| `eth_blockNumber` | `evm/rpc.rs`, `rpc/client.rs` | ✅ Available |
| `eth_syncing` | `evm/rpc.rs`, `rpc/client.rs` | ✅ Available |
| `health` | `rpc/server.rs` | ✅ Available |
| `get_stream_c_info` | `rpc/staking_rpc.rs` | ✅ Available |

### 4. Finality Tracking Exists

**Location:** `pyrax-node/src/consensus/staking/checkpoint.rs`

- `finality_depth()` - blocks since last finalized
- `finality_time()` in zkrollup/da.rs
- `finality_secs` field in ChainMetrics

### 5. P2P Metrics Available

**Location:** `pyrax-node/src/p2p/mod.rs`

```rust
pub struct NetworkMetrics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub peers_connected: u32,
    pub peers_discovered: u32,
    ...
}
```

---

## ⚠️ Gaps to Address

### 1. Stream C Missing from MiningMetrics

**Current:** `MiningMetrics` only has `stream_a` and `stream_b` fields.

```rust
// Current in metrics.rs:153-170
pub struct MiningMetrics {
    pub stream_a: StreamMetrics,
    pub stream_b: StreamMetrics,
    // stream_c is MISSING
}
```

**Required:** Add `stream_c` field for complete stream observability.

**Impact on pyrax-metrics:** Architecture document correctly specifies per-stream metrics but implementation needs this field added to pyrax-node.

---

### 2. Prometheus Export Missing Stream C

**Current:** `prometheus_export()` doesn't output stream-specific metrics.

**Required:** Add:
```
pyrax_stream_a_block_height
pyrax_stream_a_hash_rate
pyrax_stream_b_block_height
...
pyrax_stream_c_checkpoint_count
```

---

### 3. No Fork Detection in Node

**Status:** ❌ Not implemented

The node doesn't expose fork/reorg detection metrics. This is correctly delegated to the Chain Observer in the architecture (external observation is the right pattern).

**Action:** Keep in pyrax-metrics as designed.

---

### 4. No Canary Transaction Endpoint

**Status:** ❌ Not implemented in node

This is correctly an external tool (Chain Observer responsibility).

**Action:** Keep in pyrax-metrics as designed.

---

### 5. RPC Latency Not Self-Reported

Nodes don't report their own RPC response times.

**Action:** Chain Observer measures this externally (correct design).

---

## 📋 Recommended Changes to pyrax-node

### High Priority

1. **Add `stream_c: StreamMetrics`** to `MiningMetrics` struct
2. **Add stream-specific metrics** to `prometheus_export()`
3. **Add `get_stream_health`** RPC method that returns all 3 streams' status

### Medium Priority

4. **Add `peer_count`** to `/health` endpoint response
5. **Add `syncing_status`** to `/health` endpoint response

### Low Priority (Can be in pyrax-metrics)

6. Fork detection - correctly external
7. Canary transactions - correctly external
8. RPC latency measurement - correctly external

---

## 📋 Recommended Changes to pyrax-metrics Architecture

### 1. Rename Metrics for Consistency

Current pyrax-node uses:
- `pyrax_block_height` (not `pyrax_chain_head_block`)
- `pyrax_tps` (not chain-level)

**Recommendation:** Use consistent naming:
- Node metrics: `pyrax_node_*` prefix
- Chain metrics: `pyrax_chain_*` prefix
- Stream metrics: `pyrax_stream_*` prefix

### 2. Add Missing RPC Method Calls

The Chain Observer should poll these existing methods:
- `get_chain_info` - primary chain state
- `get_mining_info` - stream A/B mining stats
- `get_stream_c_info` - staking/checkpoint info
- `health` - node health check

### 3. Add Configuration for Devnet/Testnet Ports

| Network | RPC Port | Metrics Port |
|---------|----------|--------------|
| Devnet | 8545 | 9090 |
| Testnet | 8545 | 9090 |
| Mainnet | 8545 | 9090 |

---

## ✅ Final Alignment Status

| Architecture Component | Codebase Support | Status |
|------------------------|------------------|--------|
| Node-level metrics | metrics.rs | ✅ Fully aligned |
| Prometheus export | metrics.rs:490-524 | ✅ Aligned |
| Stream A/B metrics | MiningMetrics | ✅ Aligned |
| Stream C metrics | StakingMetrics | ⚠️ Partial (add to MiningMetrics) |
| Chain-level observer | N/A (external) | ✅ Correct design |
| Fork detection | N/A (external) | ✅ Correct design |
| Canary transactions | N/A (external) | ✅ Correct design |
| RPC endpoints | rpc/*.rs | ✅ All methods available |
| P2P metrics | p2p/mod.rs | ✅ Aligned |
| Finality tracking | checkpoint.rs | ✅ Aligned |

---

## Conclusion

The `pyrax-metrics` architecture is **well-aligned** with the existing `pyrax-node` codebase. The main gaps are:

1. **Stream C needs to be added to MiningMetrics** (minor change to pyrax-node)
2. **Stream-specific Prometheus metrics need export** (minor change to pyrax-node)

The Chain Observer design is correct - it should:
- Poll existing RPC endpoints (`get_chain_info`, `get_mining_info`, `get_stream_c_info`)
- Use `eth_blockNumber` and `eth_syncing` for multi-node comparison
- Aggregate metrics externally (fork detection, canary tx, etc.)

**Ready to proceed with implementation.**
