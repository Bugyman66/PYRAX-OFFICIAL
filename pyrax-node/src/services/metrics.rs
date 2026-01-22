//! Metrics Dashboard Service
//!
//! Production metrics collection and dashboard API

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::METRICS_INTERVAL_SECS;

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Collection interval (seconds)
    pub interval_secs: u64,
    /// Retention period (hours)
    pub retention_hours: u64,
    /// Enable Prometheus endpoint
    pub prometheus_enabled: bool,
    /// Prometheus port
    pub prometheus_port: u16,
    /// Enable JSON API
    pub json_api_enabled: bool,
    /// Enable real-time WebSocket
    pub websocket_enabled: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: METRICS_INTERVAL_SECS,
            retention_hours: 24,
            prometheus_enabled: true,
            prometheus_port: 9090,
            json_api_enabled: true,
            websocket_enabled: true,
        }
    }
}

/// Chain metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// Block height
    pub block_height: u64,
    /// Transactions per second
    pub tps: f64,
    /// Average block time (ms)
    pub avg_block_time_ms: u64,
    /// Pending transactions
    pub pending_txs: u64,
    /// Gas price (average)
    pub avg_gas_price: u64,
    /// Total transactions (24h)
    pub txs_24h: u64,
    /// Total blocks (24h)
    pub blocks_24h: u64,
    /// Active addresses (24h)
    pub active_addresses_24h: u64,
    /// Total value transferred (24h)
    pub volume_24h: u64,
    /// Chain utilization (0-100)
    pub utilization_percent: f64,
    /// Finality time (seconds)
    pub finality_secs: u64,
}

impl ChainMetrics {
    /// Create new chain metrics
    pub fn new() -> Self {
        Self {
            timestamp: current_timestamp(),
            block_height: 0,
            tps: 0.0,
            avg_block_time_ms: 6000,
            pending_txs: 0,
            avg_gas_price: 1_000_000_000,
            txs_24h: 0,
            blocks_24h: 0,
            active_addresses_24h: 0,
            volume_24h: 0,
            utilization_percent: 0.0,
            finality_secs: 12,
        }
    }
}

impl Default for ChainMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// Total nodes
    pub total_nodes: u32,
    /// Active nodes
    pub active_nodes: u32,
    /// Total validators
    pub total_validators: u32,
    /// Active validators
    pub active_validators: u32,
    /// Total stake
    pub total_stake: u64,
    /// Network hash rate
    pub hash_rate: u64,
    /// Average peer count
    pub avg_peer_count: f64,
    /// Geographic distribution
    pub geo_distribution: HashMap<String, u32>,
    /// Version distribution
    pub version_distribution: HashMap<String, u32>,
}

impl NetworkMetrics {
    /// Create new network metrics
    pub fn new() -> Self {
        Self {
            timestamp: current_timestamp(),
            total_nodes: 0,
            active_nodes: 0,
            total_validators: 0,
            active_validators: 0,
            total_stake: 0,
            hash_rate: 0,
            avg_peer_count: 0.0,
            geo_distribution: HashMap::new(),
            version_distribution: HashMap::new(),
        }
    }
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Mining metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// Current difficulty
    pub difficulty: u64,
    /// Network hash rate
    pub hash_rate: u64,
    /// Block reward
    pub block_reward: u64,
    /// Stream A (ASIC) stats
    pub stream_a: StreamMetrics,
    /// Stream B (GPU) stats
    pub stream_b: StreamMetrics,
    /// Stream C (ZK-STARK + PoS) stats
    pub stream_c: StreamMetrics,
    /// Time to next halving (blocks)
    pub blocks_to_halving: u64,
    /// Current epoch
    pub epoch: u32,
}

/// Stream-specific metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamMetrics {
    /// Active miners
    pub active_miners: u32,
    /// Hash rate
    pub hash_rate: u64,
    /// Blocks found (24h)
    pub blocks_24h: u64,
    /// Reward per block
    pub reward_per_block: u64,
    /// Total mined
    pub total_mined: u64,
}

/// Staking metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// Total staked
    pub total_staked: u64,
    /// Staking ratio (% of supply)
    pub staking_ratio: f64,
    /// Average APY
    pub avg_apy: f64,
    /// Total validators
    pub total_validators: u32,
    /// Active validators
    pub active_validators: u32,
    /// Total delegators
    pub total_delegators: u64,
    /// Average stake per validator
    pub avg_stake: u64,
    /// Minimum stake
    pub min_stake: u64,
}

impl Default for StakingMetrics {
    fn default() -> Self {
        Self {
            timestamp: current_timestamp(),
            total_staked: 0,
            staking_ratio: 0.0,
            avg_apy: 0.0,
            total_validators: 0,
            active_validators: 0,
            total_delegators: 0,
            avg_stake: 0,
            min_stake: 100_000 * 100_000_000,
        }
    }
}

/// ZK metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// Proofs verified (24h)
    pub proofs_24h: u64,
    /// Average proof time (ms)
    pub avg_proof_time_ms: u64,
    /// Active provers
    pub active_provers: u32,
    /// Total rewards distributed
    pub total_rewards: u64,
    /// Total burned
    pub total_burned: u64,
    /// Burn rate (%)
    pub burn_rate: f64,
}

impl Default for ZkMetrics {
    fn default() -> Self {
        Self {
            timestamp: current_timestamp(),
            proofs_24h: 0,
            avg_proof_time_ms: 0,
            active_provers: 0,
            total_rewards: 0,
            total_burned: 0,
            burn_rate: 10.0,
        }
    }
}

/// Token metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// Total supply
    pub total_supply: u64,
    /// Circulating supply
    pub circulating_supply: u64,
    /// Total burned
    pub total_burned: u64,
    /// Total holders
    pub total_holders: u64,
    /// Market cap (if available)
    pub market_cap_usd: Option<f64>,
    /// Price (if available)
    pub price_usd: Option<f64>,
    /// 24h volume (if available)
    pub volume_24h_usd: Option<f64>,
}

impl Default for TokenMetrics {
    fn default() -> Self {
        Self {
            timestamp: current_timestamp(),
            total_supply: 100_000_000_000 * 100_000_000,
            circulating_supply: 0,
            total_burned: 0,
            total_holders: 0,
            market_cap_usd: None,
            price_usd: None,
            volume_24h_usd: None,
        }
    }
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Timestamp
    pub timestamp: u64,
    /// Value
    pub value: f64,
}

/// Metrics time series
#[derive(Debug, Clone, Default)]
pub struct TimeSeries {
    /// Data points
    points: Vec<DataPoint>,
    /// Maximum points to retain
    max_points: usize,
}

impl TimeSeries {
    /// Create new time series
    pub fn new(max_points: usize) -> Self {
        Self {
            points: Vec::new(),
            max_points,
        }
    }

    /// Add data point
    pub fn add(&mut self, value: f64) {
        let point = DataPoint {
            timestamp: current_timestamp(),
            value,
        };
        self.points.push(point);

        // Trim if over limit
        if self.points.len() > self.max_points {
            self.points.remove(0);
        }
    }

    /// Get latest value
    pub fn latest(&self) -> Option<f64> {
        self.points.last().map(|p| p.value)
    }

    /// Get average
    pub fn average(&self) -> f64 {
        if self.points.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.points.iter().map(|p| p.value).sum();
        sum / self.points.len() as f64
    }

    /// Get points in range
    pub fn range(&self, start: u64, end: u64) -> Vec<DataPoint> {
        self.points.iter()
            .filter(|p| p.timestamp >= start && p.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get all points
    pub fn all(&self) -> Vec<DataPoint> {
        self.points.clone()
    }
}

/// Dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// Chain metrics
    pub chain: ChainMetrics,
    /// Network metrics
    pub network: NetworkMetrics,
    /// Staking metrics
    pub staking: StakingMetrics,
    /// Token metrics
    pub token: TokenMetrics,
    /// ZK metrics
    pub zk: ZkMetrics,
    /// Last updated
    pub last_updated: u64,
}

impl Default for DashboardData {
    fn default() -> Self {
        Self {
            chain: ChainMetrics::default(),
            network: NetworkMetrics::default(),
            staking: StakingMetrics::default(),
            token: TokenMetrics::default(),
            zk: ZkMetrics::default(),
            last_updated: current_timestamp(),
        }
    }
}

/// Metrics service
pub struct MetricsService {
    /// Configuration
    config: MetricsConfig,
    /// Current dashboard data
    dashboard: Arc<RwLock<DashboardData>>,
    /// TPS time series
    tps_series: Arc<RwLock<TimeSeries>>,
    /// Block time series
    block_time_series: Arc<RwLock<TimeSeries>>,
    /// Gas price series
    gas_price_series: Arc<RwLock<TimeSeries>>,
    /// Hash rate series
    hash_rate_series: Arc<RwLock<TimeSeries>>,
    /// Start time
    start_time: Instant,
    /// Statistics
    stats: Arc<RwLock<MetricsStats>>,
}

impl MetricsService {
    /// Create new metrics service
    pub fn new(config: MetricsConfig) -> Self {
        let max_points = (config.retention_hours * 3600 / config.interval_secs) as usize;

        Self {
            config,
            dashboard: Arc::new(RwLock::new(DashboardData::default())),
            tps_series: Arc::new(RwLock::new(TimeSeries::new(max_points))),
            block_time_series: Arc::new(RwLock::new(TimeSeries::new(max_points))),
            gas_price_series: Arc::new(RwLock::new(TimeSeries::new(max_points))),
            hash_rate_series: Arc::new(RwLock::new(TimeSeries::new(max_points))),
            start_time: Instant::now(),
            stats: Arc::new(RwLock::new(MetricsStats::default())),
        }
    }

    /// Update chain metrics
    pub fn update_chain(&self, metrics: ChainMetrics) {
        self.tps_series.write().add(metrics.tps);
        self.block_time_series.write().add(metrics.avg_block_time_ms as f64);
        self.gas_price_series.write().add(metrics.avg_gas_price as f64);

        let mut dashboard = self.dashboard.write();
        dashboard.chain = metrics;
        dashboard.last_updated = current_timestamp();

        self.stats.write().updates += 1;
    }

    /// Update network metrics
    pub fn update_network(&self, metrics: NetworkMetrics) {
        self.hash_rate_series.write().add(metrics.hash_rate as f64);

        let mut dashboard = self.dashboard.write();
        dashboard.network = metrics;
        dashboard.last_updated = current_timestamp();
    }

    /// Update staking metrics
    pub fn update_staking(&self, metrics: StakingMetrics) {
        let mut dashboard = self.dashboard.write();
        dashboard.staking = metrics;
        dashboard.last_updated = current_timestamp();
    }

    /// Update token metrics
    pub fn update_token(&self, metrics: TokenMetrics) {
        let mut dashboard = self.dashboard.write();
        dashboard.token = metrics;
        dashboard.last_updated = current_timestamp();
    }

    /// Update ZK metrics
    pub fn update_zk(&self, metrics: ZkMetrics) {
        let mut dashboard = self.dashboard.write();
        dashboard.zk = metrics;
        dashboard.last_updated = current_timestamp();
    }

    /// Get dashboard data
    pub fn get_dashboard(&self) -> DashboardData {
        self.dashboard.read().clone()
    }

    /// Get TPS history
    pub fn get_tps_history(&self) -> Vec<DataPoint> {
        self.tps_series.read().all()
    }

    /// Get block time history
    pub fn get_block_time_history(&self) -> Vec<DataPoint> {
        self.block_time_series.read().all()
    }

    /// Export Prometheus metrics
    pub fn prometheus_export(&self) -> String {
        let dashboard = self.dashboard.read();
        let mut output = String::new();

        // Chain metrics
        output.push_str(&format!("pyrax_block_height {}\n", dashboard.chain.block_height));
        output.push_str(&format!("pyrax_tps {:.2}\n", dashboard.chain.tps));
        output.push_str(&format!("pyrax_pending_txs {}\n", dashboard.chain.pending_txs));
        output.push_str(&format!("pyrax_avg_block_time_ms {}\n", dashboard.chain.avg_block_time_ms));
        output.push_str(&format!("pyrax_avg_gas_price {}\n", dashboard.chain.avg_gas_price));
        output.push_str(&format!("pyrax_txs_24h {}\n", dashboard.chain.txs_24h));
        output.push_str(&format!("pyrax_volume_24h {}\n", dashboard.chain.volume_24h));

        // Network metrics
        output.push_str(&format!("pyrax_total_nodes {}\n", dashboard.network.total_nodes));
        output.push_str(&format!("pyrax_active_nodes {}\n", dashboard.network.active_nodes));
        output.push_str(&format!("pyrax_total_validators {}\n", dashboard.network.total_validators));
        output.push_str(&format!("pyrax_active_validators {}\n", dashboard.network.active_validators));
        output.push_str(&format!("pyrax_total_stake {}\n", dashboard.network.total_stake));
        output.push_str(&format!("pyrax_hash_rate {}\n", dashboard.network.hash_rate));

        // Staking metrics
        output.push_str(&format!("pyrax_staking_ratio {:.2}\n", dashboard.staking.staking_ratio));
        output.push_str(&format!("pyrax_avg_apy {:.2}\n", dashboard.staking.avg_apy));

        // Token metrics
        output.push_str(&format!("pyrax_circulating_supply {}\n", dashboard.token.circulating_supply));
        output.push_str(&format!("pyrax_total_burned {}\n", dashboard.token.total_burned));
        output.push_str(&format!("pyrax_total_holders {}\n", dashboard.token.total_holders));

        // ZK metrics
        output.push_str(&format!("pyrax_zk_proofs_24h {}\n", dashboard.zk.proofs_24h));
        output.push_str(&format!("pyrax_zk_burned {}\n", dashboard.zk.total_burned));

        // Stream metrics (for pyrax-metrics Chain Observer)
        // Note: Stream metrics require MiningMetrics to be populated separately
        // These are placeholder values - actual stream data comes from MiningMetrics updates
        output.push_str(&format!("pyrax_finality_secs {}\n", dashboard.chain.finality_secs));

        output
    }

    /// Get uptime
    pub fn uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get statistics
    pub fn stats(&self) -> MetricsStats {
        self.stats.read().clone()
    }
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new(MetricsConfig::default())
    }
}

/// Metrics statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsStats {
    pub updates: u64,
    pub prometheus_requests: u64,
    pub api_requests: u64,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_metrics() {
        let metrics = ChainMetrics::new();
        assert_eq!(metrics.block_height, 0);
    }

    #[test]
    fn test_time_series() {
        let mut series = TimeSeries::new(10);
        series.add(1.0);
        series.add(2.0);
        series.add(3.0);

        assert_eq!(series.latest(), Some(3.0));
        assert_eq!(series.average(), 2.0);
    }

    #[test]
    fn test_metrics_service() {
        let service = MetricsService::default();
        
        let mut chain = ChainMetrics::new();
        chain.block_height = 100;
        chain.tps = 50.0;
        
        service.update_chain(chain);
        
        let dashboard = service.get_dashboard();
        assert_eq!(dashboard.chain.block_height, 100);
    }

    #[test]
    fn test_prometheus_export() {
        let service = MetricsService::default();
        let output = service.prometheus_export();
        
        assert!(output.contains("pyrax_block_height"));
        assert!(output.contains("pyrax_tps"));
    }
}
