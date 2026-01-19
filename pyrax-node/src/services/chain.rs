//! Chain Service - Core blockchain coordination service
//!
//! Production-ready chain service that coordinates:
//! - Block building with proper merkle roots
//! - Block validation before commitment
//! - UTXO state management
//! - Chain tip tracking
//! - Reorg handling
//!
//! This is the single source of truth for chain operations.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use tracing::{info, warn, error, debug};

use crate::types::{
    H256, Address, Block, BlockHeader, Transaction, BlockNumber, 
    OutPoint, Utxo, ChainTip, TxOutput,
};
use crate::storage::ChainDB;
use crate::mempool::Mempool;
use crate::validation::{BlockValidator, ValidationError};

/// Chain service configuration
#[derive(Debug, Clone)]
pub struct ChainServiceConfig {
    /// Maximum block size in bytes
    pub max_block_size: usize,
    /// Maximum transactions per block
    pub max_txs_per_block: usize,
    /// Block time target in seconds
    pub block_time_secs: u64,
    /// Initial difficulty
    pub initial_difficulty: u64,
    /// Coinbase maturity (blocks before spendable)
    pub coinbase_maturity: u64,
    /// Stream A block reward (50 PYRAX)
    pub stream_a_block_reward: u64,
    /// Stream B block reward (100 PYRAX)
    pub stream_b_block_reward: u64,
}

impl Default for ChainServiceConfig {
    fn default() -> Self {
        Self {
            max_block_size: 1_000_000,        // 1 MB
            max_txs_per_block: 5000,
            block_time_secs: 10,              // 10 seconds for Stream A
            initial_difficulty: 1,
            coinbase_maturity: 100,
            stream_a_block_reward: 50 * 100_000_000,  // 50 PYRAX
            stream_b_block_reward: 100 * 100_000_000, // 100 PYRAX
        }
    }
}

/// Chain service error types
#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Invalid merkle root: computed {computed}, header {header}")]
    InvalidMerkleRoot { computed: H256, header: H256 },
    #[error("Block not found: {0}")]
    BlockNotFound(H256),
    #[error("Parent block not found: {0}")]
    ParentNotFound(H256),
    #[error("Chain tip mismatch")]
    TipMismatch,
}

/// Block template for mining
#[derive(Debug, Clone)]
pub struct ChainBlockTemplate {
    pub height: BlockNumber,
    pub parent_hash: H256,
    pub merkle_root: H256,
    pub timestamp: u64,
    pub difficulty: u64,
    pub coinbase_tx: Transaction,
    pub transactions: Vec<Transaction>,
    pub coinbase_value: u64,
}

/// Production Chain Service
pub struct ChainService {
    db: Arc<ChainDB>,
    mempool: Arc<Mempool>,
    config: ChainServiceConfig,
    validator: BlockValidator,
    /// Cached chain tip for fast access
    cached_tip: Arc<RwLock<ChainTip>>,
}

impl ChainService {
    /// Create a new chain service
    pub fn new(db: Arc<ChainDB>, mempool: Arc<Mempool>, config: ChainServiceConfig) -> Self {
        let tip = db.get_tip();
        Self {
            db,
            mempool,
            config,
            validator: BlockValidator::new(),
            cached_tip: Arc::new(RwLock::new(tip)),
        }
    }
    
    /// Get current chain tip
    pub fn get_tip(&self) -> ChainTip {
        self.cached_tip.read().clone()
    }
    
    /// Get chain height
    pub fn height(&self) -> BlockNumber {
        self.cached_tip.read().height
    }
    
    /// Get tip hash
    pub fn tip_hash(&self) -> H256 {
        self.cached_tip.read().hash
    }
    
    /// Get total difficulty
    pub fn total_difficulty(&self) -> u64 {
        self.cached_tip.read().total_difficulty
    }
    
    /// Get block by hash
    pub fn get_block(&self, hash: &H256) -> Result<Option<Block>, ChainError> {
        self.db.get_block(hash)
            .map_err(|e| ChainError::Storage(e.to_string()))
    }
    
    /// Get block by height
    pub fn get_block_by_height(&self, height: BlockNumber) -> Result<Option<Block>, ChainError> {
        self.db.get_block_by_height(height)
            .map_err(|e| ChainError::Storage(e.to_string()))
    }
    
    /// Get current network difficulty
    pub fn get_difficulty(&self) -> u64 {
        // For devnet, use simple difficulty
        // In production, implement difficulty adjustment algorithm
        self.config.initial_difficulty
    }
    
    /// Get block reward for a given height and stream
    pub fn get_block_reward(&self, _height: BlockNumber, stream: u8) -> u64 {
        match stream {
            0 => self.config.stream_a_block_reward, // Stream A (BLAKE3)
            1 => self.config.stream_b_block_reward, // Stream B (KAWPOW)
            _ => self.config.stream_a_block_reward,
        }
    }
    
    /// Compute merkle root from transactions
    /// 
    /// Uses BLAKE3 hashing for merkle tree construction.
    /// This is the canonical merkle root computation used throughout the chain.
    pub fn compute_merkle_root(transactions: &[Transaction]) -> H256 {
        if transactions.is_empty() {
            return H256::zero();
        }
        
        // Get transaction IDs as leaves
        let mut hashes: Vec<H256> = transactions.iter().map(|tx| tx.txid()).collect();
        
        // Build merkle tree bottom-up
        while hashes.len() > 1 {
            // If odd number of hashes, duplicate the last one
            if hashes.len() % 2 == 1 {
                hashes.push(*hashes.last().unwrap());
            }
            
            // Compute next level
            hashes = hashes.chunks(2).map(|pair| {
                let mut combined = [0u8; 64];
                combined[..32].copy_from_slice(pair[0].as_bytes());
                combined[32..].copy_from_slice(pair[1].as_bytes());
                H256::from_slice(blake3::hash(&combined).as_bytes())
            }).collect();
        }
        
        hashes[0]
    }
    
    /// Verify merkle root matches transactions
    pub fn verify_merkle_root(header: &BlockHeader, transactions: &[Transaction]) -> bool {
        let computed = Self::compute_merkle_root(transactions);
        header.merkle_root == computed
    }
    
    /// Create a block template for mining
    pub fn create_block_template(
        &self,
        beneficiary: Address,
        stream: u8,
    ) -> Result<ChainBlockTemplate, ChainError> {
        let tip = self.get_tip();
        let height = tip.height + 1;
        let parent_hash = tip.hash;
        
        // Get pending transactions from mempool
        let pending_txs = self.mempool.get_pending(
            self.config.max_txs_per_block - 1 // Leave room for coinbase
        );
        
        // Calculate coinbase value (reward + fees)
        // Note: For UTXO model, fees are calculated as (sum of inputs - sum of outputs)
        // At template creation time, we use just the block reward
        // Fees will be added when the block is actually built with validated transactions
        let reward = self.get_block_reward(height, stream);
        let coinbase_value = reward; // TODO: Add fee calculation when input values are available
        
        // Create coinbase transaction
        let coinbase_tx = Transaction::coinbase(height, coinbase_value, &beneficiary);
        
        // Build full transaction list
        let mut all_txs = vec![coinbase_tx.clone()];
        all_txs.extend(pending_txs.clone());
        
        // Compute merkle root
        let merkle_root = Self::compute_merkle_root(&all_txs);
        
        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Get difficulty
        let difficulty = self.get_difficulty();
        
        debug!("Created block template: height={}, parent={}, merkle={}, txs={}",
            height, parent_hash, merkle_root, all_txs.len());
        
        Ok(ChainBlockTemplate {
            height,
            parent_hash,
            merkle_root,
            timestamp,
            difficulty,
            coinbase_tx,
            transactions: pending_txs,
            coinbase_value,
        })
    }
    
    /// Build a complete block from template and mining solution
    pub fn build_block(
        &self,
        template: &ChainBlockTemplate,
        nonce: u64,
        extra_nonce: u64,
        beneficiary: Address,
        stream: u8,
    ) -> Block {
        // Build full transaction list
        let mut all_txs = vec![template.coinbase_tx.clone()];
        all_txs.extend(template.transactions.clone());
        
        // Create header
        let header = BlockHeader {
            version: 1,
            stream,
            parent_hash: template.parent_hash,
            merkle_root: template.merkle_root,
            utxo_commitment: H256::zero(), // TODO: Implement UTXO commitment
            timestamp: template.timestamp,
            difficulty: template.difficulty,
            nonce,
            extra_nonce,
            height: template.height,
            beneficiary,
        };
        
        Block::new(header, all_txs)
    }
    
    /// Build a block with proper merkle root from transactions
    pub fn build_block_with_merkle(
        &self,
        parent_hash: H256,
        height: BlockNumber,
        transactions: Vec<Transaction>,
        timestamp: u64,
        difficulty: u64,
        nonce: u64,
        extra_nonce: u64,
        beneficiary: Address,
        stream: u8,
    ) -> Block {
        // Compute merkle root
        let merkle_root = Self::compute_merkle_root(&transactions);
        
        // Create header
        let header = BlockHeader {
            version: 1,
            stream,
            parent_hash,
            merkle_root,
            utxo_commitment: H256::zero(),
            timestamp,
            difficulty,
            nonce,
            extra_nonce,
            height,
            beneficiary,
        };
        
        Block::new(header, transactions)
    }
    
    /// Validate a block before processing
    pub fn validate_block(&self, block: &Block) -> Result<(), ChainError> {
        // 1. Verify merkle root
        if !Self::verify_merkle_root(&block.header, &block.transactions) {
            let computed = Self::compute_merkle_root(&block.transactions);
            return Err(ChainError::InvalidMerkleRoot {
                computed,
                header: block.header.merkle_root,
            });
        }
        
        // 2. Full validation
        self.validator.validate_block(block, &self.db)?;
        
        Ok(())
    }
    
    /// Process and commit a new block
    /// 
    /// This is the main entry point for adding blocks to the chain.
    /// It validates the block, updates UTXO set, and updates chain tip.
    pub fn process_block(&self, block: Block) -> Result<H256, ChainError> {
        let hash = block.hash();
        let height = block.height();
        
        info!("Processing block {} at height {}", hash, height);
        
        // Validate block
        self.validate_block(&block)?;
        
        // Commit to storage
        self.db.commit_block(&block)
            .map_err(|e| ChainError::Storage(e.to_string()))?;
        
        // Update cached tip
        let new_tip = ChainTip {
            height,
            hash,
            total_difficulty: self.total_difficulty() + block.header.difficulty,
        };
        *self.cached_tip.write() = new_tip;
        
        // Remove included transactions from mempool
        for tx in &block.transactions {
            if !tx.is_coinbase() {
                self.mempool.remove(&tx.txid());
            }
        }
        
        info!("Block {} committed at height {}", hash, height);
        
        Ok(hash)
    }
    
    /// Submit a mined block
    /// 
    /// Convenience method that builds and processes a block from mining results.
    pub fn submit_mined_block(
        &self,
        template: &ChainBlockTemplate,
        nonce: u64,
        extra_nonce: u64,
        beneficiary: Address,
        stream: u8,
    ) -> Result<H256, ChainError> {
        let block = self.build_block(template, nonce, extra_nonce, beneficiary, stream);
        
        // Verify PoW before processing
        if !block.header.verify_pow() {
            return Err(ChainError::InvalidBlock("PoW verification failed".to_string()));
        }
        
        self.process_block(block)
    }
    
    /// Get UTXO for an outpoint
    pub fn get_utxo(&self, outpoint: &OutPoint) -> Result<Option<Utxo>, ChainError> {
        self.db.get_utxo(outpoint)
            .map_err(|e| ChainError::Storage(e.to_string()))
    }
    
    /// Get all UTXOs for an address
    pub fn get_utxos_for_address(&self, address: &Address) -> Result<Vec<(OutPoint, Utxo)>, ChainError> {
        self.db.get_utxos_for_address(address)
            .map_err(|e| ChainError::Storage(e.to_string()))
    }
    
    /// Get balance for an address
    pub fn get_balance(&self, address: &Address) -> Result<u64, ChainError> {
        self.db.get_balance_for_address(address)
            .map_err(|e| ChainError::Storage(e.to_string()))
    }
    
    /// Get transaction by hash
    pub fn get_transaction(&self, txid: &H256) -> Result<Option<Transaction>, ChainError> {
        self.db.get_transaction(txid)
            .map_err(|e| ChainError::Storage(e.to_string()))
    }
    
    /// Rollback chain to a specific height (for reorgs)
    pub fn rollback_to(&self, height: BlockNumber) -> Result<(), ChainError> {
        warn!("Rolling back chain to height {}", height);
        
        self.db.rollback_to(height)
            .map_err(|e| ChainError::Storage(e.to_string()))?;
        
        // Update cached tip
        if let Some(block) = self.get_block_by_height(height)? {
            let tip = ChainTip {
                height,
                hash: block.hash(),
                total_difficulty: self.db.total_difficulty(),
            };
            *self.cached_tip.write() = tip;
        }
        
        Ok(())
    }
    
    /// Check if chain is synchronized
    pub fn is_synced(&self) -> bool {
        let tip = self.get_tip();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Consider synced if tip is within 5 minutes
        if let Ok(Some(block)) = self.get_block(&tip.hash) {
            now.saturating_sub(block.header.timestamp) < 300
        } else {
            false
        }
    }
    
    /// Get chain statistics
    pub fn get_stats(&self) -> ChainStats {
        let tip = self.get_tip();
        let utxo_count = self.db.utxo_count().unwrap_or(0);
        
        ChainStats {
            height: tip.height,
            tip_hash: tip.hash,
            total_difficulty: tip.total_difficulty,
            utxo_count,
            mempool_size: self.mempool.len(),
            is_synced: self.is_synced(),
        }
    }
}

/// Chain statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChainStats {
    pub height: BlockNumber,
    pub tip_hash: H256,
    pub total_difficulty: u64,
    pub utxo_count: u64,
    pub mempool_size: usize,
    pub is_synced: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merkle_root_empty() {
        let root = ChainService::compute_merkle_root(&[]);
        assert!(root.is_zero());
    }
    
    #[test]
    fn test_merkle_root_single() {
        let tx = Transaction::coinbase(1, 50_00000000, &Address::ZERO);
        let root = ChainService::compute_merkle_root(&[tx.clone()]);
        assert_eq!(root, tx.txid());
    }
    
    #[test]
    fn test_merkle_root_multiple() {
        let tx1 = Transaction::coinbase(1, 50_00000000, &Address::ZERO);
        let tx2 = Transaction::coinbase(2, 50_00000000, &Address::ZERO);
        
        let root = ChainService::compute_merkle_root(&[tx1.clone(), tx2.clone()]);
        
        // Manually compute expected root
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(tx1.txid().as_bytes());
        combined[32..].copy_from_slice(tx2.txid().as_bytes());
        let expected = H256::from_slice(blake3::hash(&combined).as_bytes());
        
        assert_eq!(root, expected);
    }
    
    #[test]
    fn test_merkle_root_deterministic() {
        let txs = vec![
            Transaction::coinbase(1, 50_00000000, &Address::ZERO),
            Transaction::coinbase(2, 50_00000000, &Address::ZERO),
            Transaction::coinbase(3, 50_00000000, &Address::ZERO),
        ];
        
        let root1 = ChainService::compute_merkle_root(&txs);
        let root2 = ChainService::compute_merkle_root(&txs);
        
        assert_eq!(root1, root2);
    }
    
    #[test]
    fn test_config_defaults() {
        let config = ChainServiceConfig::default();
        assert_eq!(config.stream_a_block_reward, 50 * 100_000_000);
        assert_eq!(config.stream_b_block_reward, 100 * 100_000_000);
        assert_eq!(config.coinbase_maturity, 100);
    }
}
