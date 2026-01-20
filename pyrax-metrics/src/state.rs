//! Shared Application State
//!
//! Thread-safe shared state between observer, metrics, and API servers.

use std::sync::Arc;
use parking_lot::RwLock;
use crate::observer::{ChainState, NodeStatus};
use crate::observer::fork_detector::ForkDetector;
use crate::observer::stream_monitor::{StreamMonitor, Stream, StreamState};
use crate::config::Config;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    /// Current chain state
    chain_state: RwLock<ChainState>,
    /// Node statuses
    node_statuses: RwLock<Vec<NodeStatus>>,
    /// Fork detector state
    fork_detected: RwLock<bool>,
    /// Stream states
    stream_states: RwLock<StreamStates>,
    /// Configuration
    config: Config,
}

/// Stream states for all streams
#[derive(Debug, Clone, Default)]
pub struct StreamStates {
    pub stream_a: StreamStateSnapshot,
    pub stream_b: StreamStateSnapshot,
    pub stream_c: StreamStateSnapshot,
}

/// Snapshot of a stream state (thread-safe copy)
#[derive(Debug, Clone, Default)]
pub struct StreamStateSnapshot {
    pub block_height: u64,
    pub block_rate: f64,
    pub hash_rate: u64,
    pub active_miners: u32,
    pub is_stalled: bool,
}

impl AppState {
    /// Create new shared state
    pub fn new(config: Config) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                chain_state: RwLock::new(ChainState::default()),
                node_statuses: RwLock::new(Vec::new()),
                fork_detected: RwLock::new(false),
                stream_states: RwLock::new(StreamStates::default()),
                config,
            }),
        }
    }
    
    /// Update chain state
    pub fn update_chain_state(&self, state: ChainState) {
        *self.inner.chain_state.write() = state;
    }
    
    /// Get chain state
    pub fn chain_state(&self) -> ChainState {
        self.inner.chain_state.read().clone()
    }
    
    /// Update node statuses
    pub fn update_node_statuses(&self, statuses: Vec<NodeStatus>) {
        *self.inner.node_statuses.write() = statuses;
    }
    
    /// Get node statuses
    pub fn node_statuses(&self) -> Vec<NodeStatus> {
        self.inner.node_statuses.read().clone()
    }
    
    /// Set fork detected flag
    pub fn set_fork_detected(&self, detected: bool) {
        *self.inner.fork_detected.write() = detected;
    }
    
    /// Check if fork is detected
    pub fn is_fork_detected(&self) -> bool {
        *self.inner.fork_detected.read()
    }
    
    /// Update stream states
    pub fn update_stream_states(&self, states: StreamStates) {
        *self.inner.stream_states.write() = states;
    }
    
    /// Get stream states
    pub fn stream_states(&self) -> StreamStates {
        self.inner.stream_states.read().clone()
    }
    
    /// Get config
    pub fn config(&self) -> &Config {
        &self.inner.config
    }
}
