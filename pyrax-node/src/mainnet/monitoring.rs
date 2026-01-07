//! Network Monitoring
//!
//! Production-grade monitoring and metrics collection

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};

/// Monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Enable monitoring
    pub enabled: bool,
    /// Metrics collection interval (seconds)
    pub collection_interval_secs: u64,
    /// Retention period for metrics (hours)
    pub retention_hours: u64,
    /// Alert thresholds
    pub thresholds: AlertThresholds,
    /// Enable Prometheus export
    pub prometheus_enabled: bool,
    /// Prometheus port
    pub prometheus_port: u16,
    /// Enable alerting
    pub alerting_enabled: bool,
    /// Webhook URL for alerts
    pub alert_webhook: Option<String>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_secs: 15,
            retention_hours: 24,
            thresholds: AlertThresholds::default(),
            prometheus_enabled: true,
            prometheus_port: 9090,
            alerting_enabled: true,
            alert_webhook: None,
        }
    }
}

/// Alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Maximum block time (seconds)
    pub max_block_time_secs: u64,
    /// Maximum pending transactions
    pub max_pending_txs: u64,
    /// Minimum peer count
    pub min_peer_count: u32,
    /// Maximum memory usage (%)
    pub max_memory_percent: u8,
    /// Maximum CPU usage (%)
    pub max_cpu_percent: u8,
    /// Maximum disk usage (%)
    pub max_disk_percent: u8,
    /// Minimum sync progress (%)
    pub min_sync_percent: u8,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_block_time_secs: 30,
            max_pending_txs: 10000,
            min_peer_count: 3,
            max_memory_percent: 90,
            max_cpu_percent: 90,
            max_disk_percent: 85,
            min_sync_percent: 95,
        }
    }
}

/// Node metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Node address
    pub node_id: Address,
    /// Timestamp
    pub timestamp: u64,
    /// Block height
    pub block_height: u64,
    /// Latest block hash
    pub latest_block: H256,
    /// Pending transactions count
    pub pending_txs: u64,
    /// Connected peers
    pub peer_count: u32,
    /// Inbound peers
    pub inbound_peers: u32,
    /// Outbound peers
    pub outbound_peers: u32,
    /// Syncing status
    pub syncing: bool,
    /// Sync progress (0-100)
    pub sync_progress: u8,
    /// Blocks per second (average)
    pub blocks_per_sec: f64,
    /// Transactions per second
    pub txs_per_sec: f64,
    /// Network bytes received
    pub network_rx_bytes: u64,
    /// Network bytes sent
    pub network_tx_bytes: u64,
    /// CPU usage (%)
    pub cpu_percent: u8,
    /// Memory usage (%)
    pub memory_percent: u8,
    /// Disk usage (%)
    pub disk_percent: u8,
    /// Uptime (seconds)
    pub uptime_secs: u64,
    /// Is validator
    pub is_validator: bool,
    /// Blocks produced (if validator)
    pub blocks_produced: u64,
    /// Blocks missed (if validator)
    pub blocks_missed: u64,
}

impl NodeMetrics {
    /// Create new node metrics
    pub fn new(node_id: Address) -> Self {
        Self {
            node_id,
            timestamp: current_timestamp(),
            block_height: 0,
            latest_block: H256::zero(),
            pending_txs: 0,
            peer_count: 0,
            inbound_peers: 0,
            outbound_peers: 0,
            syncing: false,
            sync_progress: 100,
            blocks_per_sec: 0.0,
            txs_per_sec: 0.0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            cpu_percent: 0,
            memory_percent: 0,
            disk_percent: 0,
            uptime_secs: 0,
            is_validator: false,
            blocks_produced: 0,
            blocks_missed: 0,
        }
    }

    /// Check if metrics indicate healthy node
    pub fn is_healthy(&self, thresholds: &AlertThresholds) -> bool {
        self.peer_count >= thresholds.min_peer_count
            && self.cpu_percent <= thresholds.max_cpu_percent
            && self.memory_percent <= thresholds.max_memory_percent
            && self.disk_percent <= thresholds.max_disk_percent
            && (self.syncing || self.sync_progress >= thresholds.min_sync_percent)
    }

    /// Get validator uptime percentage
    pub fn validator_uptime(&self) -> f64 {
        if !self.is_validator {
            return 0.0;
        }
        let total = self.blocks_produced + self.blocks_missed;
        if total == 0 {
            return 100.0;
        }
        (self.blocks_produced as f64 / total as f64) * 100.0
    }
}

/// Network-wide statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStats {
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
    /// Network height (consensus)
    pub network_height: u64,
    /// Total transactions
    pub total_transactions: u64,
    /// Transactions in last hour
    pub txs_last_hour: u64,
    /// Average block time (ms)
    pub avg_block_time_ms: u64,
    /// Network hash rate
    pub hash_rate: u64,
    /// Total stake
    pub total_stake: u64,
    /// Network TPS
    pub tps: f64,
    /// Finality time (seconds)
    pub finality_secs: u64,
}

impl NetworkStats {
    /// Create new network stats
    pub fn new() -> Self {
        Self {
            timestamp: current_timestamp(),
            ..Default::default()
        }
    }

    /// Update from node metrics
    pub fn update_from_nodes(&mut self, nodes: &[NodeMetrics]) {
        self.timestamp = current_timestamp();
        self.total_nodes = nodes.len() as u32;
        self.active_nodes = nodes.iter().filter(|n| n.peer_count > 0).count() as u32;
        self.total_validators = nodes.iter().filter(|n| n.is_validator).count() as u32;
        self.active_validators = nodes.iter()
            .filter(|n| n.is_validator && n.peer_count > 0)
            .count() as u32;

        if let Some(max_height) = nodes.iter().map(|n| n.block_height).max() {
            self.network_height = max_height;
        }

        let total_tps: f64 = nodes.iter().map(|n| n.txs_per_sec).sum();
        self.tps = total_tps / nodes.len().max(1) as f64;
    }
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: H256,
    /// Severity
    pub severity: AlertSeverity,
    /// Alert type
    pub alert_type: AlertType,
    /// Message
    pub message: String,
    /// Affected node
    pub node_id: Option<Address>,
    /// Timestamp
    pub timestamp: u64,
    /// Is acknowledged
    pub acknowledged: bool,
    /// Is resolved
    pub resolved: bool,
    /// Resolved timestamp
    pub resolved_at: Option<u64>,
}

impl Alert {
    /// Create new alert
    pub fn new(severity: AlertSeverity, alert_type: AlertType, message: String) -> Self {
        let id = {
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(&current_timestamp().to_le_bytes());
            hasher.update(message.as_bytes());
            H256::from_slice(hasher.finalize().as_bytes())
        };

        Self {
            id,
            severity,
            alert_type,
            message,
            node_id: None,
            timestamp: current_timestamp(),
            acknowledged: false,
            resolved: false,
            resolved_at: None,
        }
    }

    /// Set node ID
    pub fn with_node(mut self, node_id: Address) -> Self {
        self.node_id = Some(node_id);
        self
    }

    /// Acknowledge alert
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }

    /// Resolve alert
    pub fn resolve(&mut self) {
        self.resolved = true;
        self.resolved_at = Some(current_timestamp());
    }
}

/// Alert types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    /// Node offline
    NodeOffline,
    /// Low peer count
    LowPeerCount,
    /// Sync stalled
    SyncStalled,
    /// High resource usage
    HighResourceUsage,
    /// Block production stalled
    BlockProductionStalled,
    /// Consensus failure
    ConsensusFailed,
    /// Fork detected
    ForkDetected,
    /// Validator missed blocks
    ValidatorMissedBlocks,
    /// High pending transactions
    HighPendingTxs,
    /// Network partition
    NetworkPartition,
    /// Security incident
    SecurityIncident,
}

/// Network monitor
pub struct NetworkMonitor {
    /// Configuration
    config: MonitorConfig,
    /// Node metrics history
    node_metrics: Arc<RwLock<HashMap<Address, Vec<NodeMetrics>>>>,
    /// Network stats history
    network_stats: Arc<RwLock<Vec<NetworkStats>>>,
    /// Active alerts
    alerts: Arc<RwLock<Vec<Alert>>>,
    /// Alert history
    alert_history: Arc<RwLock<Vec<Alert>>>,
    /// Start time
    start_time: Instant,
    /// Statistics
    stats: Arc<RwLock<MonitorStats>>,
}

impl NetworkMonitor {
    /// Create new monitor
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            node_metrics: Arc::new(RwLock::new(HashMap::new())),
            network_stats: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
            stats: Arc::new(RwLock::new(MonitorStats::default())),
        }
    }

    /// Record node metrics
    pub fn record_metrics(&self, metrics: NodeMetrics) {
        let node_id = metrics.node_id;
        
        // Check for alerts
        self.check_node_alerts(&metrics);

        // Store metrics
        let mut node_metrics = self.node_metrics.write();
        let history = node_metrics.entry(node_id).or_insert_with(Vec::new);
        history.push(metrics);

        // Prune old metrics
        let cutoff = current_timestamp() - (self.config.retention_hours * 3600);
        history.retain(|m| m.timestamp >= cutoff);

        self.stats.write().metrics_collected += 1;
    }

    /// Check for node-level alerts
    fn check_node_alerts(&self, metrics: &NodeMetrics) {
        let thresholds = &self.config.thresholds;

        // Low peer count
        if metrics.peer_count < thresholds.min_peer_count {
            self.create_alert(
                AlertSeverity::Warning,
                AlertType::LowPeerCount,
                format!("Node has only {} peers (min: {})", 
                    metrics.peer_count, thresholds.min_peer_count),
                Some(metrics.node_id),
            );
        }

        // High CPU
        if metrics.cpu_percent > thresholds.max_cpu_percent {
            self.create_alert(
                AlertSeverity::Warning,
                AlertType::HighResourceUsage,
                format!("CPU usage at {}% (max: {}%)", 
                    metrics.cpu_percent, thresholds.max_cpu_percent),
                Some(metrics.node_id),
            );
        }

        // High memory
        if metrics.memory_percent > thresholds.max_memory_percent {
            self.create_alert(
                AlertSeverity::Warning,
                AlertType::HighResourceUsage,
                format!("Memory usage at {}% (max: {}%)", 
                    metrics.memory_percent, thresholds.max_memory_percent),
                Some(metrics.node_id),
            );
        }

        // High disk
        if metrics.disk_percent > thresholds.max_disk_percent {
            self.create_alert(
                AlertSeverity::Error,
                AlertType::HighResourceUsage,
                format!("Disk usage at {}% (max: {}%)", 
                    metrics.disk_percent, thresholds.max_disk_percent),
                Some(metrics.node_id),
            );
        }

        // Validator missed blocks
        if metrics.is_validator && metrics.blocks_missed > 10 {
            self.create_alert(
                AlertSeverity::Warning,
                AlertType::ValidatorMissedBlocks,
                format!("Validator missed {} blocks", metrics.blocks_missed),
                Some(metrics.node_id),
            );
        }
    }

    /// Create alert
    fn create_alert(
        &self,
        severity: AlertSeverity,
        alert_type: AlertType,
        message: String,
        node_id: Option<Address>,
    ) {
        if !self.config.alerting_enabled {
            return;
        }

        let mut alert = Alert::new(severity, alert_type, message);
        if let Some(node) = node_id {
            alert = alert.with_node(node);
        }

        self.alerts.write().push(alert);
        self.stats.write().alerts_created += 1;
    }

    /// Update network statistics
    pub fn update_network_stats(&self) {
        let node_metrics = self.node_metrics.read();
        
        // Get latest metrics from each node
        let latest: Vec<NodeMetrics> = node_metrics.values()
            .filter_map(|history| history.last().cloned())
            .collect();

        let mut stats = NetworkStats::new();
        stats.update_from_nodes(&latest);

        self.network_stats.write().push(stats);

        // Prune old stats
        let cutoff = current_timestamp() - (self.config.retention_hours * 3600);
        self.network_stats.write().retain(|s| s.timestamp >= cutoff);
    }

    /// Get latest metrics for node
    pub fn get_node_metrics(&self, node_id: &Address) -> Option<NodeMetrics> {
        self.node_metrics.read()
            .get(node_id)
            .and_then(|h| h.last().cloned())
    }

    /// Get all active nodes
    pub fn get_active_nodes(&self) -> Vec<Address> {
        let cutoff = current_timestamp() - 60; // Active in last minute
        self.node_metrics.read()
            .iter()
            .filter_map(|(id, history)| {
                history.last().and_then(|m| {
                    if m.timestamp >= cutoff {
                        Some(*id)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Get latest network stats
    pub fn get_network_stats(&self) -> Option<NetworkStats> {
        self.network_stats.read().last().cloned()
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        self.alerts.read()
            .iter()
            .filter(|a| !a.resolved)
            .cloned()
            .collect()
    }

    /// Acknowledge alert
    pub fn acknowledge_alert(&self, alert_id: &H256) -> bool {
        let mut alerts = self.alerts.write();
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == *alert_id) {
            alert.acknowledge();
            return true;
        }
        false
    }

    /// Resolve alert
    pub fn resolve_alert(&self, alert_id: &H256) -> bool {
        let mut alerts = self.alerts.write();
        if let Some(pos) = alerts.iter().position(|a| a.id == *alert_id) {
            let mut alert = alerts.remove(pos);
            alert.resolve();
            self.alert_history.write().push(alert);
            return true;
        }
        false
    }

    /// Export metrics for Prometheus
    pub fn prometheus_export(&self) -> String {
        let mut output = String::new();
        
        // Network stats
        if let Some(stats) = self.get_network_stats() {
            output.push_str(&format!("pyrax_network_height {}\n", stats.network_height));
            output.push_str(&format!("pyrax_active_nodes {}\n", stats.active_nodes));
            output.push_str(&format!("pyrax_active_validators {}\n", stats.active_validators));
            output.push_str(&format!("pyrax_tps {:.2}\n", stats.tps));
            output.push_str(&format!("pyrax_total_stake {}\n", stats.total_stake));
        }

        // Node metrics
        for (node_id, history) in self.node_metrics.read().iter() {
            if let Some(metrics) = history.last() {
                let labels = format!("{{node=\"{:?}\"}}", node_id);
                output.push_str(&format!("pyrax_node_height{} {}\n", labels, metrics.block_height));
                output.push_str(&format!("pyrax_node_peers{} {}\n", labels, metrics.peer_count));
                output.push_str(&format!("pyrax_node_cpu{} {}\n", labels, metrics.cpu_percent));
                output.push_str(&format!("pyrax_node_memory{} {}\n", labels, metrics.memory_percent));
            }
        }

        // Alerts
        let alert_count = self.get_active_alerts().len();
        output.push_str(&format!("pyrax_active_alerts {}\n", alert_count));

        output
    }

    /// Get monitor uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get statistics
    pub fn stats(&self) -> MonitorStats {
        self.stats.read().clone()
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new(MonitorConfig::default())
    }
}

/// Monitor statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonitorStats {
    pub metrics_collected: u64,
    pub alerts_created: u64,
    pub alerts_resolved: u64,
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
    fn test_node_metrics() {
        let metrics = NodeMetrics::new(Address([1u8; 20]));
        assert_eq!(metrics.block_height, 0);
        assert!(!metrics.syncing);
    }

    #[test]
    fn test_node_health() {
        let mut metrics = NodeMetrics::new(Address([1u8; 20]));
        metrics.peer_count = 5;
        metrics.cpu_percent = 50;
        metrics.memory_percent = 60;
        metrics.disk_percent = 70;
        metrics.sync_progress = 100;

        let thresholds = AlertThresholds::default();
        assert!(metrics.is_healthy(&thresholds));
    }

    #[test]
    fn test_network_stats() {
        let mut stats = NetworkStats::new();
        
        let metrics = vec![
            NodeMetrics::new(Address([1u8; 20])),
            NodeMetrics::new(Address([2u8; 20])),
        ];

        stats.update_from_nodes(&metrics);
        assert_eq!(stats.total_nodes, 2);
    }

    #[test]
    fn test_alert() {
        let mut alert = Alert::new(
            AlertSeverity::Warning,
            AlertType::LowPeerCount,
            "Test alert".to_string(),
        );

        assert!(!alert.acknowledged);
        alert.acknowledge();
        assert!(alert.acknowledged);
    }

    #[test]
    fn test_network_monitor() {
        let monitor = NetworkMonitor::default();
        
        let mut metrics = NodeMetrics::new(Address([1u8; 20]));
        metrics.peer_count = 10;
        
        monitor.record_metrics(metrics);
        assert!(monitor.get_node_metrics(&Address([1u8; 20])).is_some());
    }

    #[test]
    fn test_prometheus_export() {
        let monitor = NetworkMonitor::default();
        let output = monitor.prometheus_export();
        assert!(output.contains("pyrax_active_alerts"));
    }
}
