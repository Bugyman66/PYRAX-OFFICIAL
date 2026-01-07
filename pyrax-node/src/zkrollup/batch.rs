//! Batch Management
//!
//! Transaction batches for L2 rollup

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::{MAX_BATCH_SIZE, BATCH_INTERVAL_SECS};

/// Batch status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BatchStatus {
    /// Batch is being built
    Building,
    /// Batch is sealed, awaiting proof
    Sealed,
    /// Proof is being generated
    Proving,
    /// Batch has valid proof
    Proven,
    /// Batch is submitted to L1
    Submitted,
    /// Batch is finalized on L1
    Finalized,
    /// Batch was rejected
    Rejected,
}

/// L2 Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Transaction {
    /// Transaction hash
    pub hash: H256,
    /// Sender address
    pub from: Address,
    /// Recipient address (None for contract creation)
    pub to: Option<Address>,
    /// Value transferred
    pub value: u64,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas price
    pub gas_price: u64,
    /// Nonce
    pub nonce: u64,
    /// Input data
    pub data: Vec<u8>,
    /// Signature
    pub signature: Vec<u8>,
    /// Transaction type (0 = transfer, 1 = contract call, 2 = deposit, 3 = withdrawal)
    pub tx_type: u8,
}

impl L2Transaction {
    /// Create transfer transaction
    pub fn transfer(from: Address, to: Address, value: u64, nonce: u64) -> Self {
        let mut tx = Self {
            hash: H256::zero(),
            from,
            to: Some(to),
            value,
            gas_limit: 21000,
            gas_price: 1_000_000,
            nonce,
            data: Vec::new(),
            signature: Vec::new(),
            tx_type: 0,
        };
        tx.hash = tx.compute_hash();
        tx
    }

    /// Compute transaction hash
    pub fn compute_hash(&self) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.from.0);
        if let Some(ref to) = self.to {
            hasher.update(&to.0);
        }
        hasher.update(&self.value.to_le_bytes());
        hasher.update(&self.gas_limit.to_le_bytes());
        hasher.update(&self.gas_price.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.data);
        hasher.update(&[self.tx_type]);
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Get gas cost
    pub fn gas_cost(&self) -> u64 {
        self.gas_limit * self.gas_price
    }

    /// Is deposit transaction
    pub fn is_deposit(&self) -> bool {
        self.tx_type == 2
    }

    /// Is withdrawal transaction
    pub fn is_withdrawal(&self) -> bool {
        self.tx_type == 3
    }
}

/// Batch header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchHeader {
    /// Batch number
    pub batch_number: u64,
    /// Previous batch hash
    pub parent_hash: H256,
    /// State root before batch
    pub pre_state_root: H256,
    /// State root after batch
    pub post_state_root: H256,
    /// Transactions merkle root
    pub tx_root: H256,
    /// Number of transactions
    pub tx_count: u32,
    /// Total gas used
    pub gas_used: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Sequencer address
    pub sequencer: Address,
    /// L1 block number (for ordering)
    pub l1_block_number: u64,
}

impl BatchHeader {
    /// Compute batch hash
    pub fn hash(&self) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.batch_number.to_le_bytes());
        hasher.update(&self.parent_hash.0);
        hasher.update(&self.pre_state_root.0);
        hasher.update(&self.post_state_root.0);
        hasher.update(&self.tx_root.0);
        hasher.update(&self.tx_count.to_le_bytes());
        hasher.update(&self.gas_used.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.sequencer.0);
        H256::from_slice(hasher.finalize().as_bytes())
    }
}

/// Transaction batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    /// Batch header
    pub header: BatchHeader,
    /// Transactions in batch
    pub transactions: Vec<L2Transaction>,
    /// Status
    pub status: BatchStatus,
    /// Proof (if proven)
    pub proof: Option<Vec<u8>>,
    /// L1 submission tx hash
    pub l1_tx_hash: Option<H256>,
    /// Created timestamp
    pub created_at: u64,
    /// Sealed timestamp
    pub sealed_at: Option<u64>,
    /// Proven timestamp
    pub proven_at: Option<u64>,
    /// Submitted timestamp
    pub submitted_at: Option<u64>,
    /// Finalized timestamp
    pub finalized_at: Option<u64>,
}

impl Batch {
    /// Create new batch
    pub fn new(
        batch_number: u64,
        parent_hash: H256,
        pre_state_root: H256,
        sequencer: Address,
        l1_block_number: u64,
    ) -> Self {
        let header = BatchHeader {
            batch_number,
            parent_hash,
            pre_state_root,
            post_state_root: pre_state_root, // Will be updated
            tx_root: H256::zero(),
            tx_count: 0,
            gas_used: 0,
            timestamp: current_timestamp(),
            sequencer,
            l1_block_number,
        };

        Self {
            header,
            transactions: Vec::new(),
            status: BatchStatus::Building,
            proof: None,
            l1_tx_hash: None,
            created_at: current_timestamp(),
            sealed_at: None,
            proven_at: None,
            submitted_at: None,
            finalized_at: None,
        }
    }

    /// Get batch hash
    pub fn hash(&self) -> H256 {
        self.header.hash()
    }

    /// Add transaction to batch
    pub fn add_transaction(&mut self, tx: L2Transaction) -> Result<(), BatchError> {
        if self.status != BatchStatus::Building {
            return Err(BatchError::BatchSealed);
        }
        if self.transactions.len() >= MAX_BATCH_SIZE {
            return Err(BatchError::BatchFull);
        }

        self.header.gas_used += tx.gas_limit;
        self.transactions.push(tx);
        self.header.tx_count = self.transactions.len() as u32;

        Ok(())
    }

    /// Seal batch (no more transactions)
    pub fn seal(&mut self, post_state_root: H256) -> Result<(), BatchError> {
        if self.status != BatchStatus::Building {
            return Err(BatchError::InvalidState(self.status));
        }

        self.header.post_state_root = post_state_root;
        self.header.tx_root = self.compute_tx_root();
        self.status = BatchStatus::Sealed;
        self.sealed_at = Some(current_timestamp());

        Ok(())
    }

    /// Compute transactions merkle root
    fn compute_tx_root(&self) -> H256 {
        if self.transactions.is_empty() {
            return H256::zero();
        }

        let mut hashes: Vec<H256> = self.transactions.iter()
            .map(|tx| tx.hash)
            .collect();

        // Pad to power of 2
        while hashes.len() & (hashes.len() - 1) != 0 {
            hashes.push(H256::zero());
        }

        // Build merkle tree
        while hashes.len() > 1 {
            let mut next = Vec::new();
            for chunk in hashes.chunks(2) {
                let left = chunk[0];
                let right = chunk.get(1).copied().unwrap_or(H256::zero());
                next.push(Self::hash_pair(&left, &right));
            }
            hashes = next;
        }

        hashes[0]
    }

    /// Hash two nodes
    fn hash_pair(left: &H256, right: &H256) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&left.0);
        hasher.update(&right.0);
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Set proof
    pub fn set_proof(&mut self, proof: Vec<u8>) -> Result<(), BatchError> {
        if self.status != BatchStatus::Sealed && self.status != BatchStatus::Proving {
            return Err(BatchError::InvalidState(self.status));
        }

        self.proof = Some(proof);
        self.status = BatchStatus::Proven;
        self.proven_at = Some(current_timestamp());

        Ok(())
    }

    /// Mark as submitted
    pub fn mark_submitted(&mut self, l1_tx_hash: H256) -> Result<(), BatchError> {
        if self.status != BatchStatus::Proven {
            return Err(BatchError::InvalidState(self.status));
        }

        self.l1_tx_hash = Some(l1_tx_hash);
        self.status = BatchStatus::Submitted;
        self.submitted_at = Some(current_timestamp());

        Ok(())
    }

    /// Mark as finalized
    pub fn mark_finalized(&mut self) -> Result<(), BatchError> {
        if self.status != BatchStatus::Submitted {
            return Err(BatchError::InvalidState(self.status));
        }

        self.status = BatchStatus::Finalized;
        self.finalized_at = Some(current_timestamp());

        Ok(())
    }

    /// Check if batch should be sealed (time-based)
    pub fn should_seal(&self) -> bool {
        if self.status != BatchStatus::Building {
            return false;
        }
        
        let elapsed = current_timestamp() - self.created_at;
        elapsed >= BATCH_INTERVAL_SECS || self.transactions.len() >= MAX_BATCH_SIZE
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, hash: &H256) -> Option<&L2Transaction> {
        self.transactions.iter().find(|tx| tx.hash == *hash)
    }
}

/// Batch builder for constructing batches
pub struct BatchBuilder {
    /// Current batch being built
    current: Option<Batch>,
    /// Completed batches
    batches: Arc<RwLock<HashMap<H256, Batch>>>,
    /// Batches by number
    by_number: Arc<RwLock<HashMap<u64, H256>>>,
    /// Last batch number
    last_batch_number: Arc<RwLock<u64>>,
    /// Sequencer address
    sequencer: Address,
}

impl BatchBuilder {
    /// Create new batch builder
    pub fn new(sequencer: Address) -> Self {
        Self {
            current: None,
            batches: Arc::new(RwLock::new(HashMap::new())),
            by_number: Arc::new(RwLock::new(HashMap::new())),
            last_batch_number: Arc::new(RwLock::new(0)),
            sequencer,
        }
    }

    /// Start new batch
    pub fn start_batch(
        &mut self,
        pre_state_root: H256,
        l1_block_number: u64,
    ) -> Result<(), BatchError> {
        if self.current.is_some() {
            return Err(BatchError::BatchInProgress);
        }

        let batch_number = *self.last_batch_number.read() + 1;
        let parent_hash = self.by_number.read()
            .get(&(batch_number - 1))
            .copied()
            .unwrap_or(H256::zero());

        self.current = Some(Batch::new(
            batch_number,
            parent_hash,
            pre_state_root,
            self.sequencer,
            l1_block_number,
        ));

        Ok(())
    }

    /// Add transaction to current batch
    pub fn add_transaction(&mut self, tx: L2Transaction) -> Result<(), BatchError> {
        let batch = self.current.as_mut()
            .ok_or(BatchError::NoBatchInProgress)?;
        batch.add_transaction(tx)
    }

    /// Seal current batch
    pub fn seal_batch(&mut self, post_state_root: H256) -> Result<Batch, BatchError> {
        let mut batch = self.current.take()
            .ok_or(BatchError::NoBatchInProgress)?;
        
        batch.seal(post_state_root)?;

        let hash = batch.hash();
        let number = batch.header.batch_number;

        self.batches.write().insert(hash, batch.clone());
        self.by_number.write().insert(number, hash);
        *self.last_batch_number.write() = number;

        Ok(batch)
    }

    /// Get batch by hash
    pub fn get_batch(&self, hash: &H256) -> Option<Batch> {
        self.batches.read().get(hash).cloned()
    }

    /// Get batch by number
    pub fn get_batch_by_number(&self, number: u64) -> Option<Batch> {
        let hash = self.by_number.read().get(&number).copied()?;
        self.get_batch(&hash)
    }

    /// Get current batch
    pub fn current_batch(&self) -> Option<&Batch> {
        self.current.as_ref()
    }

    /// Check if should seal current batch
    pub fn should_seal(&self) -> bool {
        self.current.as_ref().map(|b| b.should_seal()).unwrap_or(false)
    }

    /// Get last batch number
    pub fn last_batch_number(&self) -> u64 {
        *self.last_batch_number.read()
    }

    /// Update batch status
    pub fn update_batch(&self, hash: &H256, f: impl FnOnce(&mut Batch)) -> Result<(), BatchError> {
        let mut batches = self.batches.write();
        let batch = batches.get_mut(hash)
            .ok_or_else(|| BatchError::BatchNotFound(*hash))?;
        f(batch);
        Ok(())
    }

    /// Get pending batches (sealed but not finalized)
    pub fn get_pending_batches(&self) -> Vec<Batch> {
        self.batches.read()
            .values()
            .filter(|b| {
                matches!(b.status, 
                    BatchStatus::Sealed | 
                    BatchStatus::Proving | 
                    BatchStatus::Proven |
                    BatchStatus::Submitted
                )
            })
            .cloned()
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> BatchBuilderStats {
        let batches = self.batches.read();
        let mut by_status = HashMap::new();
        let mut total_txs = 0u64;
        let mut total_gas = 0u64;

        for batch in batches.values() {
            *by_status.entry(batch.status).or_insert(0u64) += 1;
            total_txs += batch.transactions.len() as u64;
            total_gas += batch.header.gas_used;
        }

        BatchBuilderStats {
            total_batches: batches.len() as u64,
            total_transactions: total_txs,
            total_gas_used: total_gas,
            last_batch_number: *self.last_batch_number.read(),
            current_batch_txs: self.current.as_ref().map(|b| b.transactions.len()).unwrap_or(0),
            by_status,
        }
    }
}

/// Batch errors
#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("Batch is sealed, cannot add transactions")]
    BatchSealed,

    #[error("Batch is full")]
    BatchFull,

    #[error("Invalid batch state: {0:?}")]
    InvalidState(BatchStatus),

    #[error("Batch not found: {0:?}")]
    BatchNotFound(H256),

    #[error("Batch already in progress")]
    BatchInProgress,

    #[error("No batch in progress")]
    NoBatchInProgress,

    #[error("Invalid proof: {0}")]
    InvalidProof(String),
}

/// Batch builder statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchBuilderStats {
    pub total_batches: u64,
    pub total_transactions: u64,
    pub total_gas_used: u64,
    pub last_batch_number: u64,
    pub current_batch_txs: usize,
    pub by_status: HashMap<BatchStatus, u64>,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l2_transaction() {
        let tx = L2Transaction::transfer(
            Address([1u8; 20]),
            Address([2u8; 20]),
            1000,
            0,
        );

        assert_eq!(tx.value, 1000);
        assert_eq!(tx.tx_type, 0);
        assert!(!tx.is_deposit());
    }

    #[test]
    fn test_batch_creation() {
        let batch = Batch::new(
            1,
            H256::zero(),
            H256([1u8; 32]),
            Address([1u8; 20]),
            100,
        );

        assert_eq!(batch.status, BatchStatus::Building);
        assert_eq!(batch.header.batch_number, 1);
    }

    #[test]
    fn test_batch_add_transaction() {
        let mut batch = Batch::new(
            1,
            H256::zero(),
            H256([1u8; 32]),
            Address([1u8; 20]),
            100,
        );

        let tx = L2Transaction::transfer(
            Address([1u8; 20]),
            Address([2u8; 20]),
            1000,
            0,
        );

        assert!(batch.add_transaction(tx).is_ok());
        assert_eq!(batch.transactions.len(), 1);
    }

    #[test]
    fn test_batch_seal() {
        let mut batch = Batch::new(
            1,
            H256::zero(),
            H256([1u8; 32]),
            Address([1u8; 20]),
            100,
        );

        let tx = L2Transaction::transfer(
            Address([1u8; 20]),
            Address([2u8; 20]),
            1000,
            0,
        );
        batch.add_transaction(tx).unwrap();

        let post_state = H256([2u8; 32]);
        assert!(batch.seal(post_state).is_ok());
        assert_eq!(batch.status, BatchStatus::Sealed);
    }

    #[test]
    fn test_batch_builder() {
        let mut builder = BatchBuilder::new(Address([1u8; 20]));

        builder.start_batch(H256([1u8; 32]), 100).unwrap();

        let tx = L2Transaction::transfer(
            Address([1u8; 20]),
            Address([2u8; 20]),
            1000,
            0,
        );
        builder.add_transaction(tx).unwrap();

        let batch = builder.seal_batch(H256([2u8; 32])).unwrap();
        assert_eq!(batch.header.batch_number, 1);
        assert_eq!(builder.last_batch_number(), 1);
    }
}
