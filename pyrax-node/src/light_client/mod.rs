//! Mobile Light Client Infrastructure
//!
//! SPV (Simplified Payment Verification) and lightweight sync:
//! - Merkle proof verification
//! - Header chain sync
//! - Bloom filter-based transaction filtering
//! - Waku-based P2P messaging (optional)

pub mod spv;
pub mod bloom;
pub mod headers;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use spv::{SpvClient, MerkleProof, ProofVerificationResult};
pub use bloom::{BloomFilter, AddressFilter};
pub use headers::{HeaderChain, LightHeader};

/// Light client configuration
#[derive(Debug, Clone)]
pub struct LightClientConfig {
    /// Maximum headers to store in memory
    pub max_headers: usize,
    /// Enable bloom filtering
    pub bloom_enabled: bool,
    /// Bloom filter false positive rate
    pub bloom_fp_rate: f64,
    /// Checkpoint heights for trusted sync
    pub checkpoints: Vec<(u64, [u8; 32])>,
    /// Enable Waku messaging
    pub waku_enabled: bool,
}

impl Default for LightClientConfig {
    fn default() -> Self {
        Self {
            max_headers: 10000,
            bloom_enabled: true,
            bloom_fp_rate: 0.0001,
            checkpoints: Vec::new(),
            waku_enabled: false,
        }
    }
}

/// Light client manager
pub struct LightClient {
    config: LightClientConfig,
    headers: Arc<RwLock<HeaderChain>>,
    spv: SpvClient,
    address_filter: Option<AddressFilter>,
    watched_addresses: Vec<[u8; 20]>,
    pending_proofs: HashMap<[u8; 32], MerkleProof>,
}

impl LightClient {
    pub fn new(config: LightClientConfig) -> Self {
        let bloom = if config.bloom_enabled {
            Some(AddressFilter::new(config.bloom_fp_rate))
        } else {
            None
        };
        
        Self {
            config: config.clone(),
            headers: Arc::new(RwLock::new(HeaderChain::new(config.max_headers))),
            spv: SpvClient::new(),
            address_filter: bloom,
            watched_addresses: Vec::new(),
            pending_proofs: HashMap::new(),
        }
    }
    
    /// Add address to watch list
    pub fn watch_address(&mut self, address: [u8; 20]) {
        self.watched_addresses.push(address);
        if let Some(filter) = &mut self.address_filter {
            filter.add_address(&address);
        }
    }
    
    /// Remove address from watch list
    pub fn unwatch_address(&mut self, address: &[u8; 20]) {
        self.watched_addresses.retain(|a| a != address);
        // Note: Cannot remove from bloom filter, need to rebuild
    }
    
    /// Get current sync height
    pub async fn sync_height(&self) -> u64 {
        self.headers.read().await.height()
    }
    
    /// Process incoming header
    pub async fn process_header(&self, header: LightHeader) -> Result<(), String> {
        let mut chain = self.headers.write().await;
        chain.add_header(header)
    }
    
    /// Verify a transaction exists in a block
    pub async fn verify_transaction(
        &self,
        tx_hash: [u8; 32],
        block_hash: [u8; 32],
        proof: MerkleProof,
    ) -> ProofVerificationResult {
        let headers = self.headers.read().await;
        
        // Get block header
        let header = match headers.get_by_hash(&block_hash) {
            Some(h) => h,
            None => return ProofVerificationResult::BlockNotFound,
        };
        
        // Verify merkle proof
        self.spv.verify_proof(&tx_hash, &header.merkle_root, &proof)
    }
    
    /// Check if transaction matches any watched addresses
    pub fn matches_filter(&self, tx_data: &[u8]) -> bool {
        if let Some(filter) = &self.address_filter {
            // Extract addresses from transaction and check filter
            // Simplified: check if any watched address appears in tx data
            for addr in &self.watched_addresses {
                if tx_data.windows(20).any(|w| w == addr) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Get confirmations for a block
    pub async fn get_confirmations(&self, block_hash: &[u8; 32]) -> Option<u64> {
        let headers = self.headers.read().await;
        headers.get_by_hash(block_hash).map(|h| {
            headers.height().saturating_sub(h.height)
        })
    }
    
    /// Sync headers from peer
    pub async fn sync_headers(&self, headers: Vec<LightHeader>) -> Result<u64, String> {
        let mut chain = self.headers.write().await;
        let mut added = 0;
        
        for header in headers {
            if chain.add_header(header).is_ok() {
                added += 1;
            }
        }
        
        Ok(added)
    }
    
    /// Get header at height
    pub async fn get_header(&self, height: u64) -> Option<LightHeader> {
        self.headers.read().await.get_at_height(height).cloned()
    }
    
    /// Verify checkpoint
    pub fn verify_checkpoint(&self, height: u64, hash: &[u8; 32]) -> bool {
        self.config.checkpoints.iter().any(|(h, expected)| {
            *h == height && expected == hash
        })
    }
}
