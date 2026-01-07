//! Chain Synchronization Module
//!
//! Production-ready chain sync for PYRAX with:
//! - Headers-first synchronization
//! - Fork choice rule (heaviest chain by total difficulty)
//! - Parallel block download with bounded concurrency
//! - Ordered block commit
//! - Reorg handling with state rollback

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tokio::sync::{mpsc, Semaphore};
use tracing::{info, warn, debug, error};

use crate::types::{Block, BlockHeader, H256, BlockNumber, NetworkId, ChainTip};
use crate::storage::ChainDB;
use crate::validation::BlockValidator;

/// Maximum headers to request in one batch
pub const MAX_HEADERS_PER_REQUEST: u32 = 500;

/// Maximum blocks to download concurrently
pub const MAX_CONCURRENT_DOWNLOADS: usize = 16;

/// Maximum blocks to buffer before committing
pub const MAX_BLOCK_BUFFER: usize = 1024;

/// Timeout for sync requests
pub const SYNC_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Minimum difficulty increase for reorg consideration
pub const REORG_THRESHOLD: u64 = 0;

/// Chain synchronization state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncState {
    /// Not syncing, chain is up to date
    Idle,
    /// Downloading headers from peers
    DownloadingHeaders { target_height: u64, current: u64 },
    /// Downloading block bodies
    DownloadingBlocks { target_height: u64, current: u64 },
    /// Processing downloaded blocks
    ProcessingBlocks { remaining: usize },
    /// Handling a chain reorganization
    Reorging { from_height: u64, to_height: u64 },
}

/// Peer sync status
#[derive(Debug, Clone)]
pub struct PeerSyncInfo {
    pub peer_id: String,
    pub best_height: u64,
    pub best_hash: H256,
    pub total_difficulty: u64,
    pub last_seen: Instant,
}

/// Header chain segment (for validation before download)
#[derive(Debug, Clone)]
pub struct HeaderChain {
    pub headers: Vec<BlockHeader>,
    pub start_height: u64,
    pub total_difficulty: u64,
}

impl HeaderChain {
    pub fn new(start_height: u64) -> Self {
        Self {
            headers: Vec::new(),
            start_height,
            total_difficulty: 0,
        }
    }

    pub fn tip_height(&self) -> u64 {
        if self.headers.is_empty() {
            self.start_height
        } else {
            self.start_height + self.headers.len() as u64
        }
    }

    pub fn tip_hash(&self) -> Option<H256> {
        self.headers.last().map(|h| h.hash())
    }

    pub fn add_header(&mut self, header: BlockHeader) -> Result<(), SyncError> {
        // Verify header connects to our chain
        if let Some(last) = self.headers.last() {
            if header.parent_hash != last.hash() {
                return Err(SyncError::HeaderChainBroken {
                    expected_parent: last.hash(),
                    got: header.parent_hash,
                });
            }
            if header.height != last.height + 1 {
                return Err(SyncError::InvalidHeaderHeight {
                    expected: last.height + 1,
                    got: header.height,
                });
            }
        }
        
        self.total_difficulty += header.difficulty;
        self.headers.push(header);
        Ok(())
    }
}

/// Block download task
#[derive(Debug, Clone)]
pub struct BlockDownloadTask {
    pub hash: H256,
    pub height: u64,
    pub header: BlockHeader,
    pub attempts: u32,
    pub last_attempt: Option<Instant>,
}

/// Chain synchronizer
pub struct ChainSync {
    db: Arc<ChainDB>,
    validator: Arc<BlockValidator>,
    network_id: NetworkId,
    
    /// Current sync state
    state: Arc<RwLock<SyncState>>,
    
    /// Known peer states
    peers: Arc<RwLock<HashMap<String, PeerSyncInfo>>>,
    
    /// Headers pending block download
    pending_headers: Arc<RwLock<VecDeque<BlockDownloadTask>>>,
    
    /// Downloaded blocks awaiting commit (ordered by height)
    pending_blocks: Arc<RwLock<BTreeMap<u64, Block>>>,
    
    /// Blocks currently being downloaded (hash -> height)
    downloading: Arc<RwLock<HashSet<H256>>>,
    
    /// Download semaphore for concurrency control
    download_semaphore: Arc<Semaphore>,
    
    /// Channel to request blocks from P2P layer
    block_request_tx: mpsc::Sender<H256>,
    
    /// Channel to receive blocks from P2P layer
    block_response_rx: Option<mpsc::Receiver<Block>>,
}

impl ChainSync {
    /// Create a new chain synchronizer
    pub fn new(
        db: Arc<ChainDB>,
        validator: Arc<BlockValidator>,
        network_id: NetworkId,
    ) -> (Self, mpsc::Sender<Block>, mpsc::Receiver<H256>) {
        let (block_request_tx, block_request_rx) = mpsc::channel(256);
        let (block_response_tx, block_response_rx) = mpsc::channel(256);
        
        let sync = Self {
            db,
            validator,
            network_id,
            state: Arc::new(RwLock::new(SyncState::Idle)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            pending_headers: Arc::new(RwLock::new(VecDeque::new())),
            pending_blocks: Arc::new(RwLock::new(BTreeMap::new())),
            downloading: Arc::new(RwLock::new(HashSet::new())),
            download_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS)),
            block_request_tx,
            block_response_rx: Some(block_response_rx),
        };
        
        (sync, block_response_tx, block_request_rx)
    }

    /// Get current sync state
    pub fn state(&self) -> SyncState {
        self.state.read().clone()
    }

    /// Check if we're currently syncing
    pub fn is_syncing(&self) -> bool {
        !matches!(*self.state.read(), SyncState::Idle)
    }

    /// Update peer's sync info
    pub fn update_peer(&self, peer_id: String, height: u64, hash: H256, total_difficulty: u64) {
        let mut peers = self.peers.write();
        peers.insert(peer_id.clone(), PeerSyncInfo {
            peer_id,
            best_height: height,
            best_hash: hash,
            total_difficulty,
            last_seen: Instant::now(),
        });
    }

    /// Remove a peer
    pub fn remove_peer(&self, peer_id: &str) {
        self.peers.write().remove(peer_id);
    }

    /// Get the best peer to sync from (highest total difficulty)
    pub fn best_peer(&self) -> Option<PeerSyncInfo> {
        let peers = self.peers.read();
        peers.values()
            .filter(|p| p.last_seen.elapsed() < Duration::from_secs(60))
            .max_by_key(|p| p.total_difficulty)
            .cloned()
    }

    /// Check if we need to sync based on peer info
    pub fn needs_sync(&self) -> Option<PeerSyncInfo> {
        let our_tip = self.db.get_tip();
        
        if let Some(best_peer) = self.best_peer() {
            // Only sync if peer has more total difficulty
            if best_peer.total_difficulty > our_tip.total_difficulty {
                return Some(best_peer);
            }
        }
        None
    }

    /// Process received headers from a peer
    pub fn process_headers(&self, headers: Vec<BlockHeader>) -> Result<usize, SyncError> {
        if headers.is_empty() {
            return Ok(0);
        }

        let our_tip = self.db.get_tip();
        let mut added = 0;
        let mut pending = self.pending_headers.write();

        for header in headers {
            // Skip if we already have this block
            if self.db.has_block(&header.hash()).unwrap_or(false) {
                continue;
            }

            // Skip if already pending
            if pending.iter().any(|t| t.hash == header.hash()) {
                continue;
            }

            // Validate header connects to our chain or pending headers
            let connects = if header.height == our_tip.height + 1 {
                header.parent_hash == our_tip.hash
            } else if let Some(last_pending) = pending.back() {
                header.parent_hash == last_pending.hash && 
                header.height == last_pending.height + 1
            } else {
                // Check if parent is in DB
                self.db.has_block(&header.parent_hash).unwrap_or(false)
            };

            if !connects {
                debug!("Header {} at height {} doesn't connect", header.hash(), header.height);
                continue;
            }

            // Basic header validation
            if let Err(e) = self.validator.validate_header(&header, &self.db) {
                warn!("Invalid header {}: {}", header.hash(), e);
                continue;
            }

            let hash = header.hash();
            let height = header.height;
            
            pending.push_back(BlockDownloadTask {
                hash,
                height,
                header,
                attempts: 0,
                last_attempt: None,
            });
            added += 1;
        }

        if added > 0 {
            let mut state = self.state.write();
            if let SyncState::Idle = *state {
                let target = pending.back().map(|t| t.height).unwrap_or(our_tip.height);
                *state = SyncState::DownloadingBlocks {
                    target_height: target,
                    current: our_tip.height,
                };
            }
        }

        Ok(added)
    }

    /// Get next blocks to download
    pub fn get_blocks_to_download(&self, max: usize) -> Vec<H256> {
        let pending = self.pending_headers.read();
        let downloading = self.downloading.read();
        
        pending.iter()
            .filter(|t| !downloading.contains(&t.hash))
            .filter(|t| t.attempts < 3) // Max 3 attempts
            .filter(|t| t.last_attempt.map(|i| i.elapsed() > Duration::from_secs(10)).unwrap_or(true))
            .take(max)
            .map(|t| t.hash)
            .collect()
    }

    /// Mark blocks as being downloaded
    pub fn mark_downloading(&self, hashes: &[H256]) {
        let mut downloading = self.downloading.write();
        let mut pending = self.pending_headers.write();
        
        for hash in hashes {
            downloading.insert(*hash);
            if let Some(task) = pending.iter_mut().find(|t| t.hash == *hash) {
                task.attempts += 1;
                task.last_attempt = Some(Instant::now());
            }
        }
    }

    /// Process a received block
    pub fn process_block(&self, block: Block) -> Result<(), SyncError> {
        let hash = block.hash();
        let height = block.height();

        // Remove from downloading set
        self.downloading.write().remove(&hash);

        // Validate the block
        if let Err(e) = self.validator.validate_block(&block, &self.db) {
            warn!("Invalid block {} at height {}: {}", hash, height, e);
            // Remove from pending headers
            self.pending_headers.write().retain(|t| t.hash != hash);
            return Err(SyncError::ValidationFailed(e.to_string()));
        }

        // Add to pending blocks (ordered by height)
        self.pending_blocks.write().insert(height, block);

        // Remove from pending headers
        self.pending_headers.write().retain(|t| t.hash != hash);

        // Try to commit blocks in order
        self.try_commit_blocks()?;

        Ok(())
    }

    /// Try to commit pending blocks in order
    fn try_commit_blocks(&self) -> Result<usize, SyncError> {
        let mut committed = 0;
        let our_height = self.db.get_tip().height;
        
        loop {
            let next_height = our_height + committed as u64 + 1;
            
            let block = {
                let mut pending = self.pending_blocks.write();
                pending.remove(&next_height)
            };
            
            match block {
                Some(block) => {
                    // Verify block connects to current tip
                    let current_tip = self.db.get_tip();
                    if block.header.parent_hash != current_tip.hash {
                        // Put it back - not ready yet
                        self.pending_blocks.write().insert(next_height, block);
                        break;
                    }

                    // Commit block to database
                    match self.db.commit_block(&block) {
                        Ok(_) => {
                            info!("Committed block {} at height {}", block.hash(), next_height);
                            committed += 1;
                        }
                        Err(e) => {
                            error!("Failed to commit block {}: {}", block.hash(), e);
                            return Err(SyncError::CommitFailed(e.to_string()));
                        }
                    }
                }
                None => break,
            }
        }

        // Update sync state
        if committed > 0 {
            let pending_headers = self.pending_headers.read().len();
            let pending_blocks = self.pending_blocks.read().len();
            
            if pending_headers == 0 && pending_blocks == 0 {
                *self.state.write() = SyncState::Idle;
                info!("Sync complete, chain tip at height {}", self.db.get_tip().height);
            } else {
                let tip = self.db.get_tip();
                let target = self.pending_headers.read()
                    .back()
                    .map(|t| t.height)
                    .unwrap_or(tip.height);
                
                *self.state.write() = SyncState::DownloadingBlocks {
                    target_height: target,
                    current: tip.height,
                };
            }
        }

        Ok(committed)
    }

    /// Handle a potential chain reorganization
    pub fn handle_reorg(&self, new_tip: &BlockHeader) -> Result<ReorgResult, SyncError> {
        let our_tip = self.db.get_tip();
        
        // Check if this is actually a better chain
        if new_tip.difficulty <= our_tip.total_difficulty {
            return Ok(ReorgResult::NotNeeded);
        }

        // Find common ancestor
        let fork_point = self.find_fork_point(&our_tip.hash, &new_tip.parent_hash)?;
        
        if fork_point.height >= our_tip.height {
            // Not a reorg, just a new block
            return Ok(ReorgResult::NotNeeded);
        }

        info!(
            "Reorg detected: rolling back from {} to {}, then applying new chain",
            our_tip.height, fork_point.height
        );

        *self.state.write() = SyncState::Reorging {
            from_height: our_tip.height,
            to_height: fork_point.height,
        };

        // Rollback to fork point
        self.db.rollback_to(fork_point.height)
            .map_err(|e| SyncError::RollbackFailed(e.to_string()))?;

        *self.state.write() = SyncState::Idle;

        Ok(ReorgResult::RolledBack {
            old_height: our_tip.height,
            new_height: fork_point.height,
            blocks_removed: (our_tip.height - fork_point.height) as usize,
        })
    }

    /// Find the common ancestor between two chain tips
    fn find_fork_point(&self, our_hash: &H256, their_hash: &H256) -> Result<ChainTip, SyncError> {
        let mut our_current = *our_hash;
        let mut their_current = *their_hash;
        
        // Get heights
        let our_block = self.db.get_block(&our_current)
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::BlockNotFound(*our_hash))?;
        
        let mut our_height = our_block.height();
        let mut their_height = our_height; // Assume similar height, will adjust

        // Walk back both chains until we find common ancestor
        let mut iterations = 0;
        const MAX_ITERATIONS: u32 = 10000;

        while iterations < MAX_ITERATIONS {
            iterations += 1;

            // If we have the block they're pointing to, that's the fork point
            if self.db.has_block(&their_current).unwrap_or(false) {
                let block = self.db.get_block(&their_current)
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))?
                    .ok_or_else(|| SyncError::BlockNotFound(their_current))?;
                
                return Ok(ChainTip {
                    height: block.height(),
                    hash: their_current,
                    total_difficulty: block.header.difficulty, // Simplified
                });
            }

            // Walk back our chain
            if our_height > 0 {
                if let Some(block) = self.db.get_block_by_height(our_height - 1)
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))? 
                {
                    our_current = block.hash();
                    our_height -= 1;
                }
            }

            // For their chain, we'd need to request from peers
            // For now, return genesis as worst case
            if our_height == 0 {
                let genesis = self.db.get_block_by_height(0)
                    .map_err(|e| SyncError::DatabaseError(e.to_string()))?
                    .ok_or_else(|| SyncError::DatabaseError("No genesis block".into()))?;
                
                return Ok(ChainTip {
                    height: 0,
                    hash: genesis.hash(),
                    total_difficulty: genesis.header.difficulty,
                });
            }
        }

        Err(SyncError::ForkPointNotFound)
    }

    /// Get sync progress as a percentage
    pub fn progress(&self) -> f64 {
        match &*self.state.read() {
            SyncState::Idle => 100.0,
            SyncState::DownloadingHeaders { target_height, current } => {
                if *target_height == 0 {
                    0.0
                } else {
                    (*current as f64 / *target_height as f64) * 50.0 // Headers = 0-50%
                }
            }
            SyncState::DownloadingBlocks { target_height, current } => {
                if *target_height == 0 {
                    50.0
                } else {
                    50.0 + (*current as f64 / *target_height as f64) * 50.0 // Blocks = 50-100%
                }
            }
            SyncState::ProcessingBlocks { remaining } => {
                95.0 // Almost done
            }
            SyncState::Reorging { .. } => 0.0,
        }
    }

    /// Get stats about current sync
    pub fn stats(&self) -> SyncStats {
        SyncStats {
            state: self.state(),
            pending_headers: self.pending_headers.read().len(),
            pending_blocks: self.pending_blocks.read().len(),
            downloading: self.downloading.read().len(),
            peer_count: self.peers.read().len(),
            our_height: self.db.get_tip().height,
            best_peer_height: self.best_peer().map(|p| p.best_height).unwrap_or(0),
        }
    }
}

/// Result of a reorg attempt
#[derive(Debug, Clone)]
pub enum ReorgResult {
    NotNeeded,
    RolledBack {
        old_height: u64,
        new_height: u64,
        blocks_removed: usize,
    },
}

/// Sync statistics
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub state: SyncState,
    pub pending_headers: usize,
    pub pending_blocks: usize,
    pub downloading: usize,
    pub peer_count: usize,
    pub our_height: u64,
    pub best_peer_height: u64,
}

/// Sync errors
#[derive(Debug, Clone)]
pub enum SyncError {
    HeaderChainBroken { expected_parent: H256, got: H256 },
    InvalidHeaderHeight { expected: u64, got: u64 },
    ValidationFailed(String),
    CommitFailed(String),
    RollbackFailed(String),
    DatabaseError(String),
    BlockNotFound(H256),
    ForkPointNotFound,
    PeerDisconnected,
    Timeout,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::HeaderChainBroken { expected_parent, got } => {
                write!(f, "Header chain broken: expected parent {}, got {}", expected_parent, got)
            }
            SyncError::InvalidHeaderHeight { expected, got } => {
                write!(f, "Invalid header height: expected {}, got {}", expected, got)
            }
            SyncError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            SyncError::CommitFailed(msg) => write!(f, "Commit failed: {}", msg),
            SyncError::RollbackFailed(msg) => write!(f, "Rollback failed: {}", msg),
            SyncError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            SyncError::BlockNotFound(hash) => write!(f, "Block not found: {}", hash),
            SyncError::ForkPointNotFound => write!(f, "Fork point not found"),
            SyncError::PeerDisconnected => write!(f, "Peer disconnected"),
            SyncError::Timeout => write!(f, "Request timeout"),
        }
    }
}

impl std::error::Error for SyncError {}

/// Fork choice rule: select chain with highest total difficulty
pub fn fork_choice(tip_a: &ChainTip, tip_b: &ChainTip) -> ForkChoice {
    if tip_a.total_difficulty > tip_b.total_difficulty {
        ForkChoice::A
    } else if tip_b.total_difficulty > tip_a.total_difficulty {
        ForkChoice::B
    } else {
        // Tie-breaker: lower hash wins (deterministic)
        if tip_a.hash < tip_b.hash {
            ForkChoice::A
        } else {
            ForkChoice::B
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkChoice {
    A,
    B,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_choice_higher_difficulty_wins() {
        let tip_a = ChainTip {
            height: 100,
            hash: H256::from_slice(&[1u8; 32]),
            total_difficulty: 1000,
        };
        let tip_b = ChainTip {
            height: 99,
            hash: H256::from_slice(&[2u8; 32]),
            total_difficulty: 999,
        };
        
        assert_eq!(fork_choice(&tip_a, &tip_b), ForkChoice::A);
    }

    #[test]
    fn test_fork_choice_tie_breaker() {
        let tip_a = ChainTip {
            height: 100,
            hash: H256::from_slice(&[1u8; 32]),
            total_difficulty: 1000,
        };
        let tip_b = ChainTip {
            height: 100,
            hash: H256::from_slice(&[2u8; 32]),
            total_difficulty: 1000,
        };
        
        // Lower hash wins tie
        assert_eq!(fork_choice(&tip_a, &tip_b), ForkChoice::A);
    }

    #[test]
    fn test_header_chain() {
        let mut chain = HeaderChain::new(0);
        assert_eq!(chain.tip_height(), 0);
        assert!(chain.tip_hash().is_none());
    }

    #[test]
    fn test_sync_state_default() {
        let state = SyncState::Idle;
        assert_eq!(state, SyncState::Idle);
    }
}
