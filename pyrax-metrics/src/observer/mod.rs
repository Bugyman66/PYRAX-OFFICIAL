//! Observer Module
//!
//! Chain observation and state aggregation.

mod rpc_client;
mod aggregator;
pub mod stream_monitor;
pub mod fork_detector;

pub use rpc_client::{RpcClient, NodeStatus};
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
        
        info!("Starting observer loop with {}ms interval", self.config.observer.poll_interval_ms);
        
        loop {
            interval.tick().await;
            
            // Poll all nodes concurrently
            let statuses = self.poll_all_nodes().await;
            
            // Update shared state with node statuses
            self.app_state.update_node_statuses(statuses.clone());
            
            // Update aggregator and shared state
            {
                let mut agg = self.aggregator.write();
                agg.update(&statuses);
                
                let state = agg.chain_state();
                
                // Update shared state
                self.app_state.update_chain_state(state.clone());
                
                debug!(
                    "Chain state: head={}, delta={}, reachable={}/{}",
                    state.head_block,
                    state.height_delta,
                    state.nodes_reachable,
                    state.nodes_total
                );
                
                // Check for alerts
                if state.is_stalled {
                    warn!("ALERT: Chain appears stalled!");
                }
                if state.height_delta > self.config.chain.fork_threshold_blocks {
                    warn!("ALERT: Height divergence detected: {} blocks", state.height_delta);
                }
            }
            
            // Update fork detector
            {
                let mut detector = self.fork_detector.write();
                for status in &statuses {
                    if let Some(block_hash) = &status.block_hash {
                        detector.record_block(status.block_height, block_hash.clone(), &status.endpoint);
                    }
                }
                
                let fork_detected = detector.is_forked();
                self.app_state.set_fork_detected(fork_detected);
                
                if let Some(fork) = detector.detect_fork() {
                    warn!("ALERT: Fork detected at height {}!", fork.height);
                }
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
