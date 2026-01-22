//! State Pruning for Archive Node Optimization
//!
//! Reduces disk usage while maintaining functionality:
//! - Configurable pruning modes (full, archive, pruned)
//! - State trie pruning with configurable retention
//! - Block body pruning (keep headers only)
//! - Receipt pruning
//! - Historical state API for archive nodes

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

/// Pruning mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PruningMode {
    /// Keep all historical state (archive node)
    Archive,
    /// Keep last N blocks of state
    Pruned(u64),
    /// Aggressive pruning - minimal disk usage
    Light,
}

impl PruningMode {
    pub fn blocks_to_keep(&self) -> Option<u64> {
        match self {
            PruningMode::Archive => None,
            PruningMode::Pruned(n) => Some(*n),
            PruningMode::Light => Some(128),
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            PruningMode::Archive => "archive",
            PruningMode::Pruned(_) => "pruned",
            PruningMode::Light => "light",
        }
    }
}

/// State pruning configuration
#[derive(Debug, Clone)]
pub struct PruningConfig {
    pub mode: PruningMode,
    /// Blocks between pruning runs
    pub prune_interval: u64,
    /// Enable receipt pruning
    pub prune_receipts: bool,
    /// Enable block body pruning (keep headers only)
    pub prune_bodies: bool,
    /// State rent: prune untouched state after N blocks
    pub state_rent_blocks: Option<u64>,
    /// Batch size for pruning operations
    pub batch_size: usize,
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self {
            mode: PruningMode::Pruned(10000),
            prune_interval: 1000,
            prune_receipts: true,
            prune_bodies: false,
            state_rent_blocks: None,
            batch_size: 1000,
        }
    }
}

/// Pruning statistics
#[derive(Debug, Clone, Default)]
pub struct PruningStats {
    pub last_pruned_height: u64,
    pub states_pruned: u64,
    pub receipts_pruned: u64,
    pub bodies_pruned: u64,
    pub bytes_freed: u64,
    pub last_prune_time: Option<Instant>,
    pub last_prune_duration_ms: u64,
}

/// State trie node reference for pruning
#[derive(Debug, Clone)]
pub struct TrieNodeRef {
    pub hash: [u8; 32],
    pub height: u64,
    pub last_accessed: u64,
    pub ref_count: u32,
}

/// State pruner
pub struct StatePruner {
    config: PruningConfig,
    stats: PruningStats,
    protected_roots: HashSet<[u8; 32]>,
    pending_deletes: Vec<[u8; 32]>,
}

impl StatePruner {
    pub fn new(config: PruningConfig) -> Self {
        Self {
            config,
            stats: PruningStats::default(),
            protected_roots: HashSet::new(),
            pending_deletes: Vec::new(),
        }
    }
    
    /// Check if pruning should run
    pub fn should_prune(&self, current_height: u64) -> bool {
        if self.config.mode == PruningMode::Archive {
            return false;
        }
        
        current_height > self.stats.last_pruned_height + self.config.prune_interval
    }
    
    /// Protect a state root from pruning (e.g., checkpoint)
    pub fn protect_root(&mut self, root: [u8; 32]) {
        self.protected_roots.insert(root);
    }
    
    /// Remove protection from a state root
    pub fn unprotect_root(&mut self, root: &[u8; 32]) {
        self.protected_roots.remove(root);
    }
    
    /// Calculate blocks to prune based on current height
    pub fn blocks_to_prune(&self, current_height: u64) -> Option<(u64, u64)> {
        let keep = self.config.mode.blocks_to_keep()?;
        
        if current_height <= keep {
            return None;
        }
        
        let start = self.stats.last_pruned_height + 1;
        let end = current_height.saturating_sub(keep);
        
        if end > start {
            Some((start, end))
        } else {
            None
        }
    }
    
    /// Mark state nodes for deletion
    pub fn mark_for_deletion(&mut self, nodes: Vec<[u8; 32]>) {
        for node in nodes {
            if !self.protected_roots.contains(&node) {
                self.pending_deletes.push(node);
            }
        }
    }
    
    /// Get pending deletes in batches
    pub fn get_delete_batch(&mut self) -> Vec<[u8; 32]> {
        let batch_size = self.config.batch_size.min(self.pending_deletes.len());
        self.pending_deletes.drain(..batch_size).collect()
    }
    
    /// Record pruning completion
    pub fn record_pruned(&mut self, height: u64, states: u64, receipts: u64, bodies: u64, bytes: u64, duration_ms: u64) {
        self.stats.last_pruned_height = height;
        self.stats.states_pruned += states;
        self.stats.receipts_pruned += receipts;
        self.stats.bodies_pruned += bodies;
        self.stats.bytes_freed += bytes;
        self.stats.last_prune_time = Some(Instant::now());
        self.stats.last_prune_duration_ms = duration_ms;
        
        info!(
            "Pruned to height {}: {} states, {} receipts, {} bodies, {:.2} MB freed in {}ms",
            height, states, receipts, bodies,
            bytes as f64 / 1_000_000.0, duration_ms
        );
    }
    
    pub fn stats(&self) -> &PruningStats { &self.stats }
    pub fn config(&self) -> &PruningConfig { &self.config }
}

/// Historical state API for archive nodes
pub struct HistoricalStateAPI {
    /// Height -> State root mapping
    state_roots: Arc<RwLock<Vec<(u64, [u8; 32])>>>,
    /// Minimum available height
    min_height: u64,
}

impl HistoricalStateAPI {
    pub fn new() -> Self {
        Self {
            state_roots: Arc::new(RwLock::new(Vec::new())),
            min_height: 0,
        }
    }
    
    /// Register a state root for a height
    pub async fn register_root(&self, height: u64, root: [u8; 32]) {
        let mut roots = self.state_roots.write().await;
        roots.push((height, root));
        
        // Keep sorted by height
        roots.sort_by_key(|(h, _)| *h);
    }
    
    /// Get state root at specific height
    pub async fn get_root_at(&self, height: u64) -> Option<[u8; 32]> {
        let roots = self.state_roots.read().await;
        roots.iter()
            .find(|(h, _)| *h == height)
            .map(|(_, root)| *root)
    }
    
    /// Get closest available state root
    pub async fn get_closest_root(&self, height: u64) -> Option<(u64, [u8; 32])> {
        let roots = self.state_roots.read().await;
        roots.iter()
            .filter(|(h, _)| *h <= height)
            .max_by_key(|(h, _)| *h)
            .cloned()
    }
    
    /// Check if state is available at height
    pub async fn has_state_at(&self, height: u64) -> bool {
        height >= self.min_height && self.get_root_at(height).await.is_some()
    }
    
    /// Prune state roots below height
    pub async fn prune_below(&self, height: u64) {
        let mut roots = self.state_roots.write().await;
        roots.retain(|(h, _)| *h >= height);
    }
    
    /// Get range of available heights
    pub async fn available_range(&self) -> (u64, u64) {
        let roots = self.state_roots.read().await;
        let min = roots.first().map(|(h, _)| *h).unwrap_or(0);
        let max = roots.last().map(|(h, _)| *h).unwrap_or(0);
        (min, max)
    }
}

impl Default for HistoricalStateAPI {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pruning_mode() {
        assert_eq!(PruningMode::Archive.blocks_to_keep(), None);
        assert_eq!(PruningMode::Pruned(1000).blocks_to_keep(), Some(1000));
        assert_eq!(PruningMode::Light.blocks_to_keep(), Some(128));
    }
    
    #[test]
    fn test_pruner_blocks_to_prune() {
        let config = PruningConfig {
            mode: PruningMode::Pruned(100),
            ..Default::default()
        };
        let pruner = StatePruner::new(config);
        
        // At height 50, nothing to prune
        assert!(pruner.blocks_to_prune(50).is_none());
        
        // At height 200, prune 1-100
        let range = pruner.blocks_to_prune(200);
        assert!(range.is_some());
        let (start, end) = range.unwrap();
        assert_eq!(start, 1);
        assert_eq!(end, 100);
    }
    
    #[test]
    fn test_protect_roots() {
        let mut pruner = StatePruner::new(PruningConfig::default());
        let root = [1u8; 32];
        
        pruner.protect_root(root);
        pruner.mark_for_deletion(vec![root, [2u8; 32]]);
        
        // Protected root should not be in pending deletes
        assert!(!pruner.pending_deletes.contains(&root));
        assert!(pruner.pending_deletes.contains(&[2u8; 32]));
    }
    
    #[tokio::test]
    async fn test_historical_state_api() {
        let api = HistoricalStateAPI::new();
        
        api.register_root(100, [1u8; 32]).await;
        api.register_root(200, [2u8; 32]).await;
        api.register_root(300, [3u8; 32]).await;
        
        assert_eq!(api.get_root_at(200).await, Some([2u8; 32]));
        assert!(api.has_state_at(200).await);
        assert!(!api.has_state_at(150).await);
        
        let (min, max) = api.available_range().await;
        assert_eq!(min, 100);
        assert_eq!(max, 300);
    }
}
