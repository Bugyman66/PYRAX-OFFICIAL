use crate::observer::NodeStatus;
use crate::config::ChainConfig;
use std::time::{Instant, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChainState {
    pub head_block: u64,
    pub height_delta: u64,
    pub nodes_reachable: usize,
    pub nodes_total: usize,
    pub nodes_synced: usize,
    pub is_stalled: bool,
    pub time_since_last_block: u64, // ms
    pub network_hashrate: f64,
    pub block_rate: f64,        // blocks per minute
    pub avg_latency_ms: u64,
}

pub struct StateAggregator {
    config: ChainConfig,
    state: ChainState,
    last_block_update: Instant,
    prev_head_block: u64,
    prev_check_time: Instant,
}

impl StateAggregator {
    pub fn new(config: ChainConfig) -> Self {
        Self {
            config,
            state: ChainState::default(),
            last_block_update: Instant::now(),
            prev_head_block: 0,
            prev_check_time: Instant::now(),
        }
    }
    
    pub fn update(&mut self, statuses: &[NodeStatus]) {
        if statuses.is_empty() { return; }
        
        self.state.nodes_total = statuses.len();
        self.state.nodes_reachable = statuses.iter().filter(|s| s.reachable).count();
        self.state.nodes_synced = statuses.iter().filter(|s| s.reachable && !s.syncing).count();
        
        // Compute average latency
        let total_latency: u64 = statuses.iter()
            .filter(|s| s.reachable)
            .map(|s| s.latency_ms)
            .sum();
        self.state.avg_latency_ms = if self.state.nodes_reachable > 0 {
            total_latency / self.state.nodes_reachable as u64
        } else {
            0
        };
        
        // Find best height (max height)
        let max_height = statuses.iter()
            .filter_map(|s| if s.reachable { Some(s.block_height) } else { None })
            .max()
            .unwrap_or(0);
            
        // Calculate delta (min vs max reachable)
        let min_height = statuses.iter()
            .filter_map(|s| if s.reachable && s.block_height > 0 { Some(s.block_height) } else { None })
            .min()
            .unwrap_or(0);
            
        self.state.height_delta = if min_height > 0 { max_height.saturating_sub(min_height) } else { 0 };
        
        // Calculate block rate (blocks per minute)
        let elapsed_secs = self.prev_check_time.elapsed().as_secs_f64();
        if elapsed_secs > 0.0 && max_height > self.prev_head_block {
            let blocks_produced = max_height - self.prev_head_block;
            self.state.block_rate = (blocks_produced as f64 / elapsed_secs) * 60.0;
        }
        self.prev_head_block = max_height;
        self.prev_check_time = Instant::now();
        
        // Update head block & stall detection
        if max_height > self.state.head_block {
             self.state.head_block = max_height;
             self.last_block_update = Instant::now();
             self.state.is_stalled = false;
        } else {
             // Check stall condition
             if self.last_block_update.elapsed() > Duration::from_millis(self.config.stall_threshold_ms) {
                 self.state.is_stalled = true;
             }
        }
        self.state.time_since_last_block = self.last_block_update.elapsed().as_millis() as u64;
        
        // Hashrate
        self.state.network_hashrate = statuses.iter()
            .filter_map(|s| s.hashrate.map(|h| h as f64))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
    }
    
    pub fn chain_state(&self) -> ChainState {
        self.state.clone()
    }
}
