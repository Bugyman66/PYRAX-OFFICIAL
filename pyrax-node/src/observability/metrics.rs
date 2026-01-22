//! Prometheus Metrics for PYRAX Node

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;

/// Metric types
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
}

/// Single metric with labels
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub help: String,
    pub metric_type: String,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
}

/// Node-level metrics
#[derive(Debug, Clone, Default)]
pub struct NodeMetrics {
    pub uptime_secs: u64,
    pub block_height: u64,
    pub block_time_ms: u64,
    pub mempool_size: u64,
    pub mempool_bytes: u64,
    pub db_size_bytes: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
}

/// P2P network metrics
#[derive(Debug, Clone, Default)]
pub struct P2PMetrics {
    pub peer_count: u64,
    pub inbound_peers: u64,
    pub outbound_peers: u64,
    pub mesh_peers: u64,
    pub banned_peers: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub avg_latency_ms: f64,
    pub connection_errors: u64,
    pub discovery_queries: u64,
}

/// Consensus metrics
#[derive(Debug, Clone, Default)]
pub struct ConsensusMetrics {
    pub blocks_validated: u64,
    pub blocks_rejected: u64,
    pub orphan_blocks: u64,
    pub reorgs: u64,
    pub fork_choice_time_ms: u64,
    pub sync_progress: f64,
    pub sync_peers: u64,
}

/// Mining metrics
#[derive(Debug, Clone, Default)]
pub struct MiningMetrics {
    pub hashrate: f64,
    pub shares_submitted: u64,
    pub shares_accepted: u64,
    pub shares_rejected: u64,
    pub blocks_found: u64,
    pub current_difficulty: f64,
    pub estimated_earnings: f64,
    pub power_consumption_watts: f64,
    pub gpu_temperature_celsius: f64,
    pub gpu_utilization_percent: f64,
}

/// Central metrics registry
pub struct MetricsRegistry {
    pub node: NodeMetrics,
    pub p2p: P2PMetrics,
    pub consensus: ConsensusMetrics,
    pub mining: MiningMetrics,
    pub custom: HashMap<String, Metric>,
    pub start_time: Instant,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            node: NodeMetrics::default(),
            p2p: P2PMetrics::default(),
            consensus: ConsensusMetrics::default(),
            mining: MiningMetrics::default(),
            custom: HashMap::new(),
            start_time: Instant::now(),
        }
    }
    
    pub fn update_uptime(&mut self) {
        self.node.uptime_secs = self.start_time.elapsed().as_secs();
    }
    
    pub fn register_custom(&mut self, name: &str, help: &str, metric_type: &str) {
        self.custom.insert(name.to_string(), Metric {
            name: name.to_string(),
            help: help.to_string(),
            metric_type: metric_type.to_string(),
            value: MetricValue::Counter(0),
            labels: HashMap::new(),
        });
    }
    
    pub fn inc_counter(&mut self, name: &str) {
        if let Some(m) = self.custom.get_mut(name) {
            if let MetricValue::Counter(ref mut v) = m.value {
                *v += 1;
            }
        }
    }
    
    pub fn set_gauge(&mut self, name: &str, value: f64) {
        if let Some(m) = self.custom.get_mut(name) {
            m.value = MetricValue::Gauge(value);
        }
    }
    
    /// Export all metrics in Prometheus text format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        // Node metrics
        output.push_str(&format!("# HELP pyrax_node_uptime_seconds Node uptime in seconds\n"));
        output.push_str(&format!("# TYPE pyrax_node_uptime_seconds counter\n"));
        output.push_str(&format!("pyrax_node_uptime_seconds {}\n\n", self.node.uptime_secs));
        
        output.push_str(&format!("# HELP pyrax_block_height Current block height\n"));
        output.push_str(&format!("# TYPE pyrax_block_height gauge\n"));
        output.push_str(&format!("pyrax_block_height {}\n\n", self.node.block_height));
        
        output.push_str(&format!("# HELP pyrax_mempool_size Transactions in mempool\n"));
        output.push_str(&format!("# TYPE pyrax_mempool_size gauge\n"));
        output.push_str(&format!("pyrax_mempool_size {}\n\n", self.node.mempool_size));
        
        // P2P metrics
        output.push_str(&format!("# HELP pyrax_peers_total Total connected peers\n"));
        output.push_str(&format!("# TYPE pyrax_peers_total gauge\n"));
        output.push_str(&format!("pyrax_peers_total {}\n\n", self.p2p.peer_count));
        
        output.push_str(&format!("# HELP pyrax_peers_inbound Inbound peer connections\n"));
        output.push_str(&format!("# TYPE pyrax_peers_inbound gauge\n"));
        output.push_str(&format!("pyrax_peers_inbound {}\n\n", self.p2p.inbound_peers));
        
        output.push_str(&format!("# HELP pyrax_peers_outbound Outbound peer connections\n"));
        output.push_str(&format!("# TYPE pyrax_peers_outbound gauge\n"));
        output.push_str(&format!("pyrax_peers_outbound {}\n\n", self.p2p.outbound_peers));
        
        output.push_str(&format!("# HELP pyrax_network_bytes_sent_total Total bytes sent\n"));
        output.push_str(&format!("# TYPE pyrax_network_bytes_sent_total counter\n"));
        output.push_str(&format!("pyrax_network_bytes_sent_total {}\n\n", self.p2p.bytes_sent));
        
        output.push_str(&format!("# HELP pyrax_network_bytes_received_total Total bytes received\n"));
        output.push_str(&format!("# TYPE pyrax_network_bytes_received_total counter\n"));
        output.push_str(&format!("pyrax_network_bytes_received_total {}\n\n", self.p2p.bytes_received));
        
        output.push_str(&format!("# HELP pyrax_network_latency_ms Average peer latency\n"));
        output.push_str(&format!("# TYPE pyrax_network_latency_ms gauge\n"));
        output.push_str(&format!("pyrax_network_latency_ms {}\n\n", self.p2p.avg_latency_ms));
        
        // Consensus metrics
        output.push_str(&format!("# HELP pyrax_blocks_validated_total Blocks validated\n"));
        output.push_str(&format!("# TYPE pyrax_blocks_validated_total counter\n"));
        output.push_str(&format!("pyrax_blocks_validated_total {}\n\n", self.consensus.blocks_validated));
        
        output.push_str(&format!("# HELP pyrax_sync_progress Sync progress 0-1\n"));
        output.push_str(&format!("# TYPE pyrax_sync_progress gauge\n"));
        output.push_str(&format!("pyrax_sync_progress {}\n\n", self.consensus.sync_progress));
        
        // Mining metrics
        output.push_str(&format!("# HELP pyrax_mining_hashrate_hps Mining hashrate\n"));
        output.push_str(&format!("# TYPE pyrax_mining_hashrate_hps gauge\n"));
        output.push_str(&format!("pyrax_mining_hashrate_hps {}\n\n", self.mining.hashrate));
        
        output.push_str(&format!("# HELP pyrax_mining_blocks_found_total Blocks found\n"));
        output.push_str(&format!("# TYPE pyrax_mining_blocks_found_total counter\n"));
        output.push_str(&format!("pyrax_mining_blocks_found_total {}\n\n", self.mining.blocks_found));
        
        output.push_str(&format!("# HELP pyrax_gpu_temperature_celsius GPU temperature\n"));
        output.push_str(&format!("# TYPE pyrax_gpu_temperature_celsius gauge\n"));
        output.push_str(&format!("pyrax_gpu_temperature_celsius {}\n\n", self.mining.gpu_temperature_celsius));
        
        // Custom metrics
        for (_, metric) in &self.custom {
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
            output.push_str(&format!("# TYPE {} {}\n", metric.name, metric.metric_type));
            match &metric.value {
                MetricValue::Counter(v) => output.push_str(&format!("{} {}\n\n", metric.name, v)),
                MetricValue::Gauge(v) => output.push_str(&format!("{} {}\n\n", metric.name, v)),
                MetricValue::Histogram(values) => {
                    for (i, v) in values.iter().enumerate() {
                        output.push_str(&format!("{}{{le=\"{}\"}} {}\n", metric.name, i, v));
                    }
                    output.push_str("\n");
                }
            }
        }
        
        output
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self { Self::new() }
}

/// Serve Prometheus metrics over HTTP
pub async fn serve_prometheus(addr: SocketAddr, metrics: Arc<RwLock<MetricsRegistry>>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind Prometheus server: {}", e);
            return;
        }
    };
    
    tracing::info!("Prometheus metrics server listening on {}", addr);
    
    loop {
        if let Ok((mut socket, _)) = listener.accept().await {
            let metrics = Arc::clone(&metrics);
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                if socket.read(&mut buf).await.is_ok() {
                    let registry = metrics.read().await;
                    let body = registry.export_prometheus();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(), body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_registry() {
        let mut registry = MetricsRegistry::new();
        registry.node.block_height = 1000;
        registry.p2p.peer_count = 25;
        
        let output = registry.export_prometheus();
        assert!(output.contains("pyrax_block_height 1000"));
        assert!(output.contains("pyrax_peers_total 25"));
    }
    
    #[test]
    fn test_custom_metrics() {
        let mut registry = MetricsRegistry::new();
        registry.register_custom("my_counter", "A test counter", "counter");
        registry.inc_counter("my_counter");
        registry.inc_counter("my_counter");
        
        if let Some(m) = registry.custom.get("my_counter") {
            if let MetricValue::Counter(v) = m.value {
                assert_eq!(v, 2);
            }
        }
    }
}
