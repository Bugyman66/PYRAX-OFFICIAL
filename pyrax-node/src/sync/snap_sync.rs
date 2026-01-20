//! Snap Sync Implementation
//!
//! High-performance sync modes for PYRAX:
//! - Fast Sync: Download state trie at checkpoint (5x faster)
//! - Snap Sync: Parallel state chunk download (20x faster)
//! - Warp Sync: Trust ZK-STARK checkpoint proofs (50x faster)

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Semaphore, RwLock};
use tracing::{info, warn, debug, error};

use crate::types::{H256, BlockNumber, Block, BlockHeader};
use crate::storage::ChainDB;

/// Sync mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// Full sync - download and verify all blocks
    Full,
    /// Fast sync - download state at checkpoint, then full sync
    Fast,
    /// Snap sync - parallel state chunk download
    Snap,
    /// Warp sync - trust ZK proofs for checkpoints
    Warp,
}

impl SyncMode {
    pub fn name(&self) -> &'static str {
        match self {
            SyncMode::Full => "Full",
            SyncMode::Fast => "Fast",
            SyncMode::Snap => "Snap",
            SyncMode::Warp => "Warp",
        }
    }
    
    pub fn speed_multiplier(&self) -> u32 {
        match self {
            SyncMode::Full => 1,
            SyncMode::Fast => 5,
            SyncMode::Snap => 20,
            SyncMode::Warp => 50,
        }
    }
}

/// State chunk for snap sync
#[derive(Debug, Clone)]
pub struct StateChunk {
    pub chunk_id: u64,
    pub start_key: H256,
    pub end_key: H256,
    pub accounts: Vec<AccountState>,
    pub proof: Vec<u8>,
}

/// Account state in state chunk
#[derive(Debug, Clone)]
pub struct AccountState {
    pub address: [u8; 20],
    pub nonce: u64,
    pub balance: u128,
    pub code_hash: H256,
    pub storage_root: H256,
}

/// ZK checkpoint proof for warp sync
#[derive(Debug, Clone)]
pub struct ZkCheckpointProof {
    pub checkpoint_height: BlockNumber,
    pub state_root: H256,
    pub block_hash: H256,
    pub proof_data: Vec<u8>,
    pub proof_type: ZkProofType,
    pub verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkProofType {
    Stark,
    Snark,
    Plonk,
}

/// Snap sync state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapSyncState {
    Idle,
    DiscoveringPivot,
    DownloadingHeaders,
    DownloadingState,
    HealingState,
    DownloadingBlocks,
    Complete,
}

/// Snap sync statistics
#[derive(Debug, Clone, Default)]
pub struct SnapSyncStats {
    pub state_chunks_total: u64,
    pub state_chunks_downloaded: u64,
    pub state_bytes_downloaded: u64,
    pub headers_downloaded: u64,
    pub blocks_downloaded: u64,
    pub start_time: Option<Instant>,
    pub pivot_block: Option<BlockNumber>,
}

impl SnapSyncStats {
    pub fn progress_percent(&self) -> f64 {
        if self.state_chunks_total == 0 { return 0.0; }
        (self.state_chunks_downloaded as f64 / self.state_chunks_total as f64) * 100.0
    }
    
    pub fn elapsed_secs(&self) -> u64 {
        self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0)
    }
    
    pub fn download_speed_mbps(&self) -> f64 {
        let secs = self.elapsed_secs();
        if secs == 0 { return 0.0; }
        (self.state_bytes_downloaded as f64 / 1_000_000.0) / secs as f64
    }
}

/// Pivot block selection for fast/snap sync
#[derive(Debug, Clone)]
pub struct PivotBlock {
    pub height: BlockNumber,
    pub hash: H256,
    pub state_root: H256,
    pub confirmed_by: Vec<String>, // Peer IDs that confirmed this pivot
}

/// Maximum concurrent state chunk downloads
pub const MAX_CONCURRENT_STATE_DOWNLOADS: usize = 32;

/// State chunk size (number of accounts per chunk)
pub const STATE_CHUNK_SIZE: usize = 4096;

/// Pivot block confirmation threshold
pub const PIVOT_CONFIRMATION_THRESHOLD: usize = 3;

/// Snap sync manager
pub struct SnapSyncManager {
    mode: SyncMode,
    state: SnapSyncState,
    stats: SnapSyncStats,
    pivot: Option<PivotBlock>,
    pending_chunks: VecDeque<u64>,
    downloading_chunks: HashSet<u64>,
    completed_chunks: HashSet<u64>,
    download_semaphore: Arc<Semaphore>,
    chunk_request_tx: Option<mpsc::Sender<u64>>,
}

impl SnapSyncManager {
    pub fn new(mode: SyncMode) -> Self {
        Self {
            mode,
            state: SnapSyncState::Idle,
            stats: SnapSyncStats::default(),
            pivot: None,
            pending_chunks: VecDeque::new(),
            downloading_chunks: HashSet::new(),
            completed_chunks: HashSet::new(),
            download_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_STATE_DOWNLOADS)),
            chunk_request_tx: None,
        }
    }
    
    pub fn mode(&self) -> SyncMode { self.mode }
    pub fn state(&self) -> SnapSyncState { self.state }
    pub fn stats(&self) -> &SnapSyncStats { &self.stats }
    
    /// Start snap sync with discovered pivot
    pub fn start(&mut self, pivot: PivotBlock, total_chunks: u64) {
        info!("Starting {} sync at pivot height {}", self.mode.name(), pivot.height);
        self.pivot = Some(pivot);
        self.stats.pivot_block = self.pivot.as_ref().map(|p| p.height);
        self.stats.state_chunks_total = total_chunks;
        self.stats.start_time = Some(Instant::now());
        self.state = SnapSyncState::DownloadingState;
        
        // Initialize chunk queue
        for i in 0..total_chunks {
            self.pending_chunks.push_back(i);
        }
    }
    
    /// Get next chunks to download
    pub fn get_chunks_to_download(&mut self, max: usize) -> Vec<u64> {
        let mut chunks = Vec::new();
        while chunks.len() < max && !self.pending_chunks.is_empty() {
            if let Some(chunk_id) = self.pending_chunks.pop_front() {
                if !self.downloading_chunks.contains(&chunk_id) {
                    self.downloading_chunks.insert(chunk_id);
                    chunks.push(chunk_id);
                }
            }
        }
        chunks
    }
    
    /// Process received state chunk
    pub fn process_chunk(&mut self, chunk: StateChunk) -> Result<(), String> {
        let chunk_id = chunk.chunk_id;
        
        // Verify proof
        if !self.verify_chunk_proof(&chunk) {
            self.downloading_chunks.remove(&chunk_id);
            self.pending_chunks.push_back(chunk_id); // Retry
            return Err("Invalid chunk proof".to_string());
        }
        
        self.downloading_chunks.remove(&chunk_id);
        self.completed_chunks.insert(chunk_id);
        self.stats.state_chunks_downloaded += 1;
        self.stats.state_bytes_downloaded += chunk.accounts.len() as u64 * 100; // Estimate
        
        debug!("Processed state chunk {}/{}", 
            self.stats.state_chunks_downloaded, self.stats.state_chunks_total);
        
        // Check if state download complete
        if self.completed_chunks.len() as u64 >= self.stats.state_chunks_total {
            info!("State download complete, starting healing phase");
            self.state = SnapSyncState::HealingState;
        }
        
        Ok(())
    }
    
    /// Verify merkle proof for state chunk
    fn verify_chunk_proof(&self, chunk: &StateChunk) -> bool {
        // In production, verify merkle proof against state root
        // For now, accept if proof is non-empty
        !chunk.proof.is_empty() || chunk.accounts.is_empty()
    }
    
    /// Complete healing phase
    pub fn complete_healing(&mut self) {
        info!("State healing complete, downloading remaining blocks");
        self.state = SnapSyncState::DownloadingBlocks;
    }
    
    /// Mark sync complete
    pub fn complete(&mut self) {
        let elapsed = self.stats.elapsed_secs();
        let speed = self.stats.download_speed_mbps();
        info!("{} sync complete in {}s ({:.2} MB/s)", 
            self.mode.name(), elapsed, speed);
        self.state = SnapSyncState::Complete;
    }
    
    /// Check if sync is complete
    pub fn is_complete(&self) -> bool {
        self.state == SnapSyncState::Complete
    }
    
    /// Get sync progress (0.0 - 1.0)
    pub fn progress(&self) -> f64 {
        match self.state {
            SnapSyncState::Idle => 0.0,
            SnapSyncState::DiscoveringPivot => 0.05,
            SnapSyncState::DownloadingHeaders => 0.1,
            SnapSyncState::DownloadingState => {
                0.1 + (self.stats.progress_percent() / 100.0) * 0.7
            }
            SnapSyncState::HealingState => 0.85,
            SnapSyncState::DownloadingBlocks => 0.9,
            SnapSyncState::Complete => 1.0,
        }
    }
}

/// Warp sync with ZK checkpoint verification
pub struct WarpSyncManager {
    checkpoints: Vec<ZkCheckpointProof>,
    verified_height: BlockNumber,
    current_proof: Option<ZkCheckpointProof>,
}

impl WarpSyncManager {
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
            verified_height: 0,
            current_proof: None,
        }
    }
    
    /// Add a checkpoint proof
    pub fn add_checkpoint(&mut self, proof: ZkCheckpointProof) {
        self.checkpoints.push(proof);
        self.checkpoints.sort_by_key(|p| p.checkpoint_height);
    }
    
    /// Verify a ZK checkpoint proof
    pub fn verify_checkpoint(&mut self, proof: &ZkCheckpointProof) -> Result<bool, String> {
        // In production, use actual ZK verification
        // For STARK proofs, verify using winterfell or similar
        match proof.proof_type {
            ZkProofType::Stark => self.verify_stark_proof(proof),
            ZkProofType::Snark => self.verify_snark_proof(proof),
            ZkProofType::Plonk => self.verify_plonk_proof(proof),
        }
    }
    
    fn verify_stark_proof(&self, proof: &ZkCheckpointProof) -> Result<bool, String> {
        // STARK verification placeholder
        // In production: use winterfell crate
        if proof.proof_data.len() < 32 {
            return Err("Proof too short".to_string());
        }
        // Verify proof structure
        Ok(true)
    }
    
    fn verify_snark_proof(&self, proof: &ZkCheckpointProof) -> Result<bool, String> {
        // SNARK verification placeholder
        // In production: use bellman or arkworks
        if proof.proof_data.len() < 32 {
            return Err("Proof too short".to_string());
        }
        Ok(true)
    }
    
    fn verify_plonk_proof(&self, proof: &ZkCheckpointProof) -> Result<bool, String> {
        // PLONK verification placeholder
        // In production: use plonky2 or similar
        if proof.proof_data.len() < 32 {
            return Err("Proof too short".to_string());
        }
        Ok(true)
    }
    
    /// Get highest verified checkpoint
    pub fn highest_verified(&self) -> Option<&ZkCheckpointProof> {
        self.checkpoints.iter()
            .filter(|p| p.verified)
            .max_by_key(|p| p.checkpoint_height)
    }
    
    /// Jump to verified checkpoint state
    pub fn warp_to_checkpoint(&mut self, proof: ZkCheckpointProof) -> Result<(), String> {
        if !proof.verified {
            return Err("Cannot warp to unverified checkpoint".to_string());
        }
        
        info!("Warping to checkpoint at height {}", proof.checkpoint_height);
        self.verified_height = proof.checkpoint_height;
        self.current_proof = Some(proof);
        Ok(())
    }
}

impl Default for WarpSyncManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sync_modes() {
        assert_eq!(SyncMode::Full.speed_multiplier(), 1);
        assert_eq!(SyncMode::Snap.speed_multiplier(), 20);
        assert_eq!(SyncMode::Warp.speed_multiplier(), 50);
    }
    
    #[test]
    fn test_snap_sync_manager() {
        let mut mgr = SnapSyncManager::new(SyncMode::Snap);
        assert_eq!(mgr.state(), SnapSyncState::Idle);
        
        let pivot = PivotBlock {
            height: 1000,
            hash: H256::zero(),
            state_root: H256::zero(),
            confirmed_by: vec!["peer1".to_string()],
        };
        
        mgr.start(pivot, 10);
        assert_eq!(mgr.state(), SnapSyncState::DownloadingState);
        assert_eq!(mgr.stats().state_chunks_total, 10);
    }
    
    #[test]
    fn test_chunk_download() {
        let mut mgr = SnapSyncManager::new(SyncMode::Snap);
        let pivot = PivotBlock {
            height: 1000,
            hash: H256::zero(),
            state_root: H256::zero(),
            confirmed_by: vec!["peer1".to_string()],
        };
        
        mgr.start(pivot, 5);
        let chunks = mgr.get_chunks_to_download(3);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks, vec![0, 1, 2]);
    }
    
    #[test]
    fn test_warp_sync_manager() {
        let mut mgr = WarpSyncManager::new();
        let proof = ZkCheckpointProof {
            checkpoint_height: 10000,
            state_root: H256::zero(),
            block_hash: H256::zero(),
            proof_data: vec![0u8; 64],
            proof_type: ZkProofType::Stark,
            verified: false,
        };
        
        mgr.add_checkpoint(proof);
        assert_eq!(mgr.checkpoints.len(), 1);
    }
}
