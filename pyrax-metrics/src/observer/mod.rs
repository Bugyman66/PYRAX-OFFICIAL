//! Observer Module
//!
//! Chain observation and state aggregation.

mod rpc_client;
mod aggregator;
pub mod stream_monitor;
pub mod fork_detector;

pub use rpc_client::{RpcClient, NodeStatus, PeerInfo};
pub use aggregator::{StateAggregator, ChainState};
pub use stream_monitor::StreamMonitor;
pub use fork_detector::ForkDetector;

use crate::config::Config;
use crate::state::{AppState, StreamStates, StreamStateSnapshot};
use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn, debug};

/// Chain Observer - monitors multiple nodes and aggregates state
pub struct ChainObserver {
    config: Config,
    clients: Vec<RpcClient>,
    aggregator: Arc<RwLock<StateAggregator>>,
    stream_monitor: Arc<RwLock<StreamMonitor>>,
    fork_detector: Arc<RwLock<ForkDetector>>,
    app_state: AppState,
}

impl ChainObserver {
    /// Create a new chain observer
    pub fn new(config: Config, app_state: AppState) -> Self {
        let clients: Vec<RpcClient> = config.nodes.endpoints
            .iter()
            .map(|endpoint| {
                RpcClient::new(
                    endpoint.clone(),
                    Duration::from_millis(config.observer.request_timeout_ms),
                )
            })
            .collect();
        
        let aggregator = StateAggregator::new(config.chain.clone());
        let stream_monitor = StreamMonitor::new(config.streams.clone());
        let fork_detector = ForkDetector::new(100); // Keep last 100 blocks
        
        Self {
            config,
            clients,
            aggregator: Arc::new(RwLock::new(aggregator)),
            stream_monitor: Arc::new(RwLock::new(stream_monitor)),
            fork_detector: Arc::new(RwLock::new(fork_detector)),
            app_state,
        }
    }
    
    /// Run the observer loop
    pub async fn run(&self) -> Result<()> {
        let poll_interval = Duration::from_millis(self.config.observer.poll_interval_ms);
        let mut interval = interval(poll_interval);
        
        use std::collections::HashMap;
        let mut was_stalled = false;
        let mut node_health: HashMap<String, bool> = HashMap::new(); // Endpoint -> is_online
        let mut last_hashrate = 0.0;
        
        info!("Starting observer loop with {}ms interval", self.config.observer.poll_interval_ms);
        
        loop {
            interval.tick().await;
            
            // Poll all nodes concurrently
            let statuses = self.poll_all_nodes().await;
            
            // Update shared state with node statuses
            self.app_state.update_node_statuses(statuses.clone());
            
            // Collect alerts to send (avoid async in lock)
            let mut pending_alerts: Vec<(String, String)> = Vec::new();

            // Update detector first
            {
                let mut detector = self.fork_detector.write();
                for status in &statuses {
                    if let Some(block_hash) = &status.block_hash {
                        detector.record_block(status.block_height, block_hash.clone(), &status.endpoint);
                    }
                }
                self.app_state.set_fork_detected(detector.is_forked());
                
                if let Some(fork) = detector.detect_fork() {
                    warn!("ALERT: Fork detected at height {}!", fork.height);
                    let msg = format!("⚠️ <b>Fork Detected</b>\nHeight: {}\nConflicting hashes detected!", fork.height);
                    pending_alerts.push(("Fork Alert".to_string(), msg));
                }
            }

            // Update aggregator and check health
            {
                let mut agg = self.aggregator.write();
                agg.update(&statuses);
                let state = agg.chain_state();
                self.app_state.update_chain_state(state.clone());
                
                debug!("Chain state: head={}, delta={}", state.head_block, state.height_delta);
                
                // 1. Chain Health Alerts
                if state.is_stalled && !was_stalled {
                     warn!("ALERT: Chain Stalled!");
                     let msg = format!("🚨 <b>Chain Stalled</b>\nNo new blocks for {}ms\nHeight: {}", 
                        state.time_since_last_block, state.head_block);
                    pending_alerts.push(("Chain Health".to_string(), msg));
                    was_stalled = true;
                } else if !state.is_stalled && was_stalled {
                     info!("Chain Resumed!");
                     let msg = format!("✅ <b>Chain Resumed</b>\nProducing blocks again.\nHeight: {}", state.head_block);
                     pending_alerts.push(("Chain Health".to_string(), msg));
                     was_stalled = false;
                }

                // 2. Node Operations Alerts
                for status in &statuses {
                    // Assuming !syncing means sycned. latency_ms check might need field verification but assuming standard
                    let is_healthy = !status.syncing && status.latency_ms < 2000;
                    let prev_healthy = *node_health.get(&status.endpoint).unwrap_or(&true); 
                    
                    if !is_healthy && prev_healthy {
                        let reason = if status.syncing { "Syncing/Falling Behind" } else { "High Latency" };
                        let msg = format!("🔴 <b>Node degraded</b>\nNode: {}\nIssue: {}", status.endpoint, reason);
                        // Use unique key so multiple nodes can alert independently
                        pending_alerts.push((format!("Node Ops {}", status.endpoint), msg));
                    } else if is_healthy && !prev_healthy {
                         let msg = format!("🟢 <b>Node Recovered</b>\nNode: {}", status.endpoint);
                         pending_alerts.push((format!("Node Ops {}", status.endpoint), msg));
                    }
                    node_health.insert(status.endpoint.clone(), is_healthy);
                }
                
                // RPC Down Global Alert
                if state.nodes_total > 0 && (state.nodes_reachable as f64 / state.nodes_total as f64) < 0.5 {
                     let msg = format!("🔻 <b>Network Critical</b>\nRPC Availability low! {}/{} nodes reachable.", 
                        state.nodes_reachable, state.nodes_total);
                     pending_alerts.push(("RPC Critical".to_string(), msg));
                }

                // 3. Mining Alerts (Hashrate Drop)
                if state.network_hashrate > 0.0 {
                    if last_hashrate > 0.0 && state.network_hashrate < last_hashrate * 0.7 {
                         let drop_pct = ((last_hashrate - state.network_hashrate) / last_hashrate) * 100.0;
                         let msg = format!("⛏ <b>Hashrate Drop</b>\nDropped by {:.1}%\nCurrent: {:.2} MH/s", drop_pct, state.network_hashrate / 1_000_000.0);
                         pending_alerts.push(("Mining Check".to_string(), msg));
                    }
                    last_hashrate = state.network_hashrate;
                }
            }
            
            // Dispatch alerts async outside of locks
            for (title, msg) in pending_alerts {
                self.app_state.alert_manager().send_alert(&title, &msg).await;
            }
        }
    }
    
    /// Poll all nodes concurrently
    async fn poll_all_nodes(&self) -> Vec<NodeStatus> {
        let futures: Vec<_> = self.clients
            .iter()
            .map(|client| client.get_status())
            .collect();
        
        let results = futures::future::join_all(futures).await;
        
        results.into_iter()
            .filter_map(|r| r.ok())
            .collect()
    }
    
    /// Get current chain state (for metrics export)
    pub fn chain_state(&self) -> ChainState {
        self.aggregator.read().chain_state()
    }
}
