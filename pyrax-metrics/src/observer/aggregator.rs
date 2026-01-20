//! State Aggregator Module
//!
//! Aggregates state from multiple nodes into chain-level metrics.

use crate::config::ChainConfig;
use super::rpc_client::NodeStatus;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Aggregated chain state
#[derive(Debug, Clone)]
pub struct ChainState {
    /// Highest block seen across all nodes
    pub head_block: u64,
    /// Lowest block seen across reachable nodes
    pub lowest_block: u64,
    /// Maximum height difference between nodes
    pub height_delta: u64,
    /// Total configured nodes
    pub nodes_total: usize,
    /// Currently reachable nodes
    pub nodes_reachable: usize,
    /// Nodes that are fully synced (not syncing)
    pub nodes_synced: usize,
    /// Average latency across nodes (ms)
    pub avg_latency_ms: u64,
    /// Block production rate (blocks per minute)
    pub block_rate: f64,
    /// Whether the chain appears stalled
    pub is_stalled: bool,
    /// Last update timestamp
    pub last_update: u64,
}

/// State aggregator that computes chain-level metrics
pub struct StateAggregator {
    config: ChainConfig,
    node_states: HashMap<String, NodeStatus>,
    last_block_height: u64,
    last_block_time: Option<Instant>,
    block_times: Vec<u64>, // Ring buffer of block times in ms
}

impl StateAggregator {
    /// Create a new state aggregator
    pub fn new(config: ChainConfig) -> Self {
        Self {
            config,
            node_states: HashMap::new(),
            last_block_height: 0,
            last_block_time: None,
            block_times: Vec::with_capacity(100),
        }
    }
    
    /// Update aggregator with new node statuses
    pub fn update(&mut self, statuses: &[NodeStatus]) {
        // Update node states
        for status in statuses {
            self.node_states.insert(status.endpoint.clone(), status.clone());
        }
        
        // Track block production rate
        let current_head = self.calculate_head_block();
        if current_head > self.last_block_height {
            if let Some(last_time) = self.last_block_time {
                let elapsed = last_time.elapsed().as_millis() as u64;
                let blocks = current_head - self.last_block_height;
                let time_per_block = elapsed / blocks;
                
                // Add to ring buffer
                self.block_times.push(time_per_block);
                if self.block_times.len() > 100 {
                    self.block_times.remove(0);
                }
            }
            
            self.last_block_height = current_head;
            self.last_block_time = Some(Instant::now());
        }
    }
    
    /// Get current chain state
    pub fn chain_state(&self) -> ChainState {
        let reachable_nodes: Vec<&NodeStatus> = self.node_states
            .values()
            .filter(|s| s.reachable)
            .collect();
        
        let nodes_total = self.node_states.len();
        let nodes_reachable = reachable_nodes.len();
        let nodes_synced = reachable_nodes.iter().filter(|s| !s.syncing).count();
        
        let head_block = self.calculate_head_block();
        let lowest_block = reachable_nodes
            .iter()
            .map(|s| s.block_height)
            .min()
            .unwrap_or(0);
        
        let height_delta = if nodes_reachable > 0 {
            head_block.saturating_sub(lowest_block)
        } else {
            0
        };
        
        let avg_latency_ms = if nodes_reachable > 0 {
            reachable_nodes.iter().map(|s| s.latency_ms).sum::<u64>() / nodes_reachable as u64
        } else {
            0
        };
        
        let block_rate = self.calculate_block_rate();
        
        // Check if chain is stalled
        let is_stalled = self.is_stalled();
        
        let last_update = chrono::Utc::now().timestamp() as u64;
        
        ChainState {
            head_block,
            lowest_block,
            height_delta,
            nodes_total,
            nodes_reachable,
            nodes_synced,
            avg_latency_ms,
            block_rate,
            is_stalled,
            last_update,
        }
    }
    
    /// Calculate the head block (max height across nodes)
    fn calculate_head_block(&self) -> u64 {
        self.node_states
            .values()
            .filter(|s| s.reachable)
            .map(|s| s.block_height)
            .max()
            .unwrap_or(0)
    }
    
    /// Calculate block production rate (blocks per minute)
    fn calculate_block_rate(&self) -> f64 {
        if self.block_times.is_empty() {
            return 0.0;
        }
        
        let avg_time_ms: f64 = self.block_times.iter().sum::<u64>() as f64 / self.block_times.len() as f64;
        
        if avg_time_ms > 0.0 {
            60_000.0 / avg_time_ms
        } else {
            0.0
        }
    }
    
    /// Check if the chain appears stalled
    fn is_stalled(&self) -> bool {
        if let Some(last_time) = self.last_block_time {
            let elapsed = last_time.elapsed();
            elapsed > Duration::from_millis(self.config.stall_threshold_ms)
        } else {
            false
        }
    }
}

impl Default for ChainState {
    fn default() -> Self {
        Self {
            head_block: 0,
            lowest_block: 0,
            height_delta: 0,
            nodes_total: 0,
            nodes_reachable: 0,
            nodes_synced: 0,
            avg_latency_ms: 0,
            block_rate: 0.0,
            is_stalled: false,
            last_update: 0,
        }
    }
}
