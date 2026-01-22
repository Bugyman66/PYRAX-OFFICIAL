//! Fork Detector Module
//!
//! Detects chain forks by comparing block hashes across nodes.

use std::collections::{HashMap, HashSet, VecDeque};

/// Information about a detected fork
#[derive(Debug, Clone)]
pub struct ForkInfo {
    pub height: u64,
    pub hashes: Vec<String>,
    pub sources: Vec<String>,
}

/// Block snapshot for history tracking
#[derive(Debug, Clone)]
struct BlockSnapshot {
    height: u64,
    hash: String,
    source: String,
    timestamp: u64,
}

/// Fork detector
pub struct ForkDetector {
    /// Block history (ring buffer)
    history: VecDeque<BlockSnapshot>,
    /// Maximum history size
    max_history: usize,
    /// Hashes seen at each height: height -> (hash -> sources)
    hashes_by_height: HashMap<u64, HashMap<String, HashSet<String>>>,
    /// Detected reorg count
    reorg_count: u64,
    /// Last reorg depth
    last_reorg_depth: u64,
}

impl ForkDetector {
    /// Create a new fork detector
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            hashes_by_height: HashMap::new(),
            reorg_count: 0,
            last_reorg_depth: 0,
        }
    }
    
    /// Record a block observation
    pub fn record_block(&mut self, height: u64, hash: String, source: &str) {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        
        let snapshot = BlockSnapshot {
            height,
            hash: hash.clone(),
            source: source.to_string(),
            timestamp,
        };
        
        // Add to history
        self.history.push_back(snapshot);
        if self.history.len() > self.max_history {
            if let Some(old) = self.history.pop_front() {
                // Clean up old height data
                if let Some(height_map) = self.hashes_by_height.get_mut(&old.height) {
                    if let Some(sources) = height_map.get_mut(&old.hash) {
                        sources.remove(&old.source);
                        if sources.is_empty() {
                            height_map.remove(&old.hash);
                        }
                    }
                    if height_map.is_empty() {
                        self.hashes_by_height.remove(&old.height);
                    }
                }
            }
        }
        
        // Track hashes by height
        let height_map = self.hashes_by_height.entry(height).or_insert_with(HashMap::new);
        let sources = height_map.entry(hash).or_insert_with(HashSet::new);
        sources.insert(source.to_string());
    }
    
    /// Detect if there's a fork
    pub fn detect_fork(&self) -> Option<ForkInfo> {
        // Check for heights with multiple different hashes
        for (height, hash_map) in &self.hashes_by_height {
            if hash_map.len() > 1 {
                let hashes: Vec<String> = hash_map.keys().cloned().collect();
                let sources: Vec<String> = hash_map.values()
                    .flat_map(|s| s.iter().cloned())
                    .collect();
                
                return Some(ForkInfo {
                    height: *height,
                    hashes,
                    sources,
                });
            }
        }
        None
    }
    
    /// Check if fork is currently detected
    pub fn is_forked(&self) -> bool {
        self.detect_fork().is_some()
    }
    
    /// Get number of conflicting blocks (heights with multiple hashes)
    pub fn conflicting_blocks(&self) -> usize {
        self.hashes_by_height.values()
            .filter(|m| m.len() > 1)
            .count()
    }
    
    /// Get reorg count
    pub fn reorg_count(&self) -> u64 {
        self.reorg_count
    }
    
    /// Get last reorg depth
    pub fn last_reorg_depth(&self) -> u64 {
        self.last_reorg_depth
    }
    
    /// Record a reorg
    pub fn record_reorg(&mut self, depth: u64) {
        self.reorg_count += 1;
        self.last_reorg_depth = depth;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fork_detection() {
        let mut detector = ForkDetector::new(100);
        
        // Same hash at height 100 - no fork
        detector.record_block(100, "0xabc".to_string(), "node1");
        detector.record_block(100, "0xabc".to_string(), "node2");
        assert!(!detector.is_forked());
        
        // Different hash at height 101 - fork!
        detector.record_block(101, "0xdef".to_string(), "node1");
        detector.record_block(101, "0xghi".to_string(), "node2");
        assert!(detector.is_forked());
        
        let fork = detector.detect_fork().unwrap();
        assert_eq!(fork.height, 101);
        assert_eq!(fork.hashes.len(), 2);
    }
}
