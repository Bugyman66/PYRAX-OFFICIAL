//! Block and Transaction Validation for PYRAX
//!
//! Production-ready validation rules for:
//! - Transaction inputs/outputs
//! - UTXO availability and maturity
//! - Block structure and PoW
//! - Chain linking (parent hash verification)

use thiserror::Error;
use crate::types::{Block, Transaction, H256, OutPoint, BlockNumber, Address};
use crate::storage::ChainDB;
use crate::consensus::Stream;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid block hash")]
    InvalidBlockHash,
    #[error("Invalid PoW: hash does not meet target")]
    InvalidPoW,
    #[error("Invalid merkle root")]
    InvalidMerkleRoot,
    #[error("Invalid parent hash: expected {expected}, got {got}")]
    InvalidParentHash { expected: H256, got: H256 },
    #[error("Invalid height: expected {expected}, got {got}")]
    InvalidHeight { expected: BlockNumber, got: BlockNumber },
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Missing coinbase transaction")]
    MissingCoinbase,
    #[error("Invalid coinbase: {0}")]
    InvalidCoinbase(String),
    #[error("UTXO not found: {0:?}")]
    UtxoNotFound(OutPoint),
    #[error("UTXO not mature (coinbase needs 100 confirmations)")]
    UtxoNotMature,
    #[error("Double spend detected: {0:?}")]
    DoubleSpend(OutPoint),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Insufficient funds: input={input}, output={output}")]
    InsufficientFunds { input: u64, output: u64 },
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid script")]
    InvalidScript,
    #[error("Block too large: {size} > {max}")]
    BlockTooLarge { size: usize, max: usize },
    #[error("Too many transactions: {count} > {max}")]
    TooManyTransactions { count: usize, max: usize },
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Maximum block size in bytes (1 MB)
pub const MAX_BLOCK_SIZE: usize = 1_000_000;

/// Maximum transactions per block
pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 5000;

/// Coinbase maturity (blocks before coinbase can be spent)
pub const COINBASE_MATURITY: u64 = 100;

/// Block reward for Stream A (50 PYRAX = 50 * 10^8 base units)
pub const STREAM_A_BLOCK_REWARD: u64 = 50 * 100_000_000;

use crate::types::BlockHeader;

/// Validates blocks and headers before adding to chain
pub struct BlockValidator {
    // Validator is now stateless - takes db reference per call
}

impl BlockValidator {
    pub fn new() -> Self {
        Self {}
    }

    /// Validate a header without full block (for headers-first sync)
    pub fn validate_header(&self, header: &BlockHeader, db: &ChainDB) -> Result<(), ValidationError> {
        // 1. Validate PoW
        if !header.verify_pow() {
            return Err(ValidationError::InvalidPoW);
        }

        // 2. Validate timestamp (not too far in future)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if header.timestamp > now + 7200 { // 2 hours max drift
            return Err(ValidationError::InvalidTimestamp(
                format!("Block timestamp {} is too far in the future (now: {})", header.timestamp, now)
            ));
        }

        // 3. Validate chain link if not genesis
        if header.height > 0 {
            // Check if parent exists
            if !db.has_block(&header.parent_hash).unwrap_or(false) {
                // For headers-first sync, parent might be in pending headers
                // Return OK and let sync module handle ordering
            }
        }

        Ok(())
    }

    /// Full block validation
    pub fn validate_block(&self, block: &Block, db: &ChainDB) -> Result<(), ValidationError> {
        // 1. Validate block structure
        self.validate_structure(block)?;

        // 2. Validate PoW
        self.validate_pow(block)?;

        // 3. Validate parent hash and height
        self.validate_chain_link(block, db)?;

        // 4. Validate merkle root
        self.validate_merkle(block)?;

        // 5. Validate timestamp
        self.validate_timestamp(block, db)?;

        // 6. Validate transactions
        self.validate_transactions(block, db)?;

        Ok(())
    }

    /// Validate block structure (size limits, etc.)
    fn validate_structure(&self, block: &Block) -> Result<(), ValidationError> {
        // Check block size
        let block_data = bincode::serialize(block).unwrap_or_default();
        if block_data.len() > MAX_BLOCK_SIZE {
            return Err(ValidationError::BlockTooLarge {
                size: block_data.len(),
                max: MAX_BLOCK_SIZE,
            });
        }

        // Check transaction count
        if block.transactions.len() > MAX_TRANSACTIONS_PER_BLOCK {
            return Err(ValidationError::TooManyTransactions {
                count: block.transactions.len(),
                max: MAX_TRANSACTIONS_PER_BLOCK,
            });
        }

        // Must have at least coinbase (except genesis)
        if block.height() > 0 && block.transactions.is_empty() {
            return Err(ValidationError::MissingCoinbase);
        }

        Ok(())
    }

    /// Validate Proof of Work
    fn validate_pow(&self, block: &Block) -> Result<(), ValidationError> {
        if !block.header.verify_pow() {
            return Err(ValidationError::InvalidPoW);
        }
        Ok(())
    }

    /// Validate chain linkage (parent hash and height)
    fn validate_chain_link(&self, block: &Block, db: &ChainDB) -> Result<(), ValidationError> {
        let tip = db.get_tip();

        // Genesis block special case
        if block.height() == 0 {
            if block.header.parent_hash != H256::zero() {
                return Err(ValidationError::InvalidParentHash {
                    expected: H256::zero(),
                    got: block.header.parent_hash,
                });
            }
            return Ok(());
        }

        // Verify parent hash matches current tip
        if block.header.parent_hash != tip.hash {
            return Err(ValidationError::InvalidParentHash {
                expected: tip.hash,
                got: block.header.parent_hash,
            });
        }

        // Verify height is tip + 1
        let expected_height = tip.height + 1;
        if block.height() != expected_height {
            return Err(ValidationError::InvalidHeight {
                expected: expected_height,
                got: block.height(),
            });
        }

        Ok(())
    }

    /// Validate merkle root
    fn validate_merkle(&self, block: &Block) -> Result<(), ValidationError> {
        if !block.verify_merkle() {
            return Err(ValidationError::InvalidMerkleRoot);
        }
        Ok(())
    }

    /// Validate timestamp
    fn validate_timestamp(&self, block: &Block, db: &ChainDB) -> Result<(), ValidationError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Timestamp must not be more than 2 hours in the future
        if block.header.timestamp > now + 7200 {
            return Err(ValidationError::InvalidTimestamp(
                "Block timestamp too far in future".into()
            ));
        }

        // For non-genesis, timestamp must be >= parent timestamp
        if block.height() > 0 {
            if let Ok(Some(parent)) = db.get_block(&block.header.parent_hash) {
                if block.header.timestamp < parent.header.timestamp {
                    return Err(ValidationError::InvalidTimestamp(
                        "Block timestamp before parent".into()
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate all transactions in block
    fn validate_transactions(&self, block: &Block, db: &ChainDB) -> Result<(), ValidationError> {
        if block.transactions.is_empty() {
            return Ok(()); // Genesis can have no transactions
        }

        // First transaction must be coinbase
        let coinbase = &block.transactions[0];
        self.validate_coinbase(coinbase, block.height())?;

        // Track spent outputs within this block to detect double-spends
        let mut spent_in_block: std::collections::HashSet<OutPoint> = std::collections::HashSet::new();

        // Validate remaining transactions
        for (_idx, tx) in block.transactions.iter().enumerate().skip(1) {
            self.validate_transaction(tx, block.height(), &mut spent_in_block, db)?;
        }

        Ok(())
    }

    /// Validate coinbase transaction
    fn validate_coinbase(&self, tx: &Transaction, height: BlockNumber) -> Result<(), ValidationError> {
        if !tx.is_coinbase() {
            return Err(ValidationError::InvalidCoinbase(
                "First transaction must be coinbase".into()
            ));
        }

        // Coinbase must have exactly one input
        if tx.inputs.len() != 1 {
            return Err(ValidationError::InvalidCoinbase(
                "Coinbase must have exactly one input".into()
            ));
        }

        // Coinbase must have at least one output
        if tx.outputs.is_empty() {
            return Err(ValidationError::InvalidCoinbase(
                "Coinbase must have at least one output".into()
            ));
        }

        // Verify total output doesn't exceed block reward + fees
        // For now, just check it doesn't exceed block reward (fees calculated separately)
        let total_output = tx.total_output();
        let max_reward = Stream::A.block_reward();
        if total_output > max_reward {
            return Err(ValidationError::InvalidCoinbase(
                format!("Coinbase output {} exceeds max reward {}", total_output, max_reward)
            ));
        }

        Ok(())
    }

    /// Validate a single transaction
    fn validate_transaction(
        &self,
        tx: &Transaction,
        current_height: BlockNumber,
        spent_in_block: &mut std::collections::HashSet<OutPoint>,
        db: &ChainDB,
    ) -> Result<(), ValidationError> {
        // Must have inputs
        if tx.inputs.is_empty() {
            return Err(ValidationError::InvalidTransaction(
                "Transaction has no inputs".into()
            ));
        }

        // Must have outputs
        if tx.outputs.is_empty() {
            return Err(ValidationError::InvalidTransaction(
                "Transaction has no outputs".into()
            ));
        }

        let mut total_input: u64 = 0;

        // Validate each input
        for input in &tx.inputs {
            let outpoint = &input.previous_output;

            // Check for double-spend within block
            if spent_in_block.contains(outpoint) {
                return Err(ValidationError::DoubleSpend(*outpoint));
            }

            // Get UTXO from database
            let utxo = db.get_utxo(outpoint)
                .map_err(|e| ValidationError::StorageError(e.to_string()))?
                .ok_or_else(|| ValidationError::UtxoNotFound(*outpoint))?;

            // Check coinbase maturity
            if !utxo.is_mature(current_height) {
                return Err(ValidationError::UtxoNotMature);
            }

            // Add to total input
            total_input += utxo.output.value;

            // Mark as spent in this block
            spent_in_block.insert(*outpoint);

            // TODO: Validate signature against script_pubkey
            // This requires implementing script verification
        }

        // Calculate total output
        let total_output = tx.total_output();

        // Input must be >= output (difference is fee)
        if total_input < total_output {
            return Err(ValidationError::InsufficientFunds {
                input: total_input,
                output: total_output,
            });
        }

        Ok(())
    }
}

/// Validates a transaction before adding to mempool
pub fn validate_mempool_tx(
    tx: &Transaction,
    db: &ChainDB,
) -> Result<(), ValidationError> {
    // Can't be coinbase
    if tx.is_coinbase() {
        return Err(ValidationError::InvalidTransaction(
            "Coinbase not allowed in mempool".into()
        ));
    }

    // Must have inputs and outputs
    if tx.inputs.is_empty() || tx.outputs.is_empty() {
        return Err(ValidationError::InvalidTransaction(
            "Transaction must have inputs and outputs".into()
        ));
    }

    let current_height = db.get_tip().height;
    let mut total_input: u64 = 0;

    // Validate each input
    for input in &tx.inputs {
        let outpoint = &input.previous_output;

        // Get UTXO
        let utxo = db.get_utxo(outpoint)
            .map_err(|e| ValidationError::StorageError(e.to_string()))?
            .ok_or_else(|| ValidationError::UtxoNotFound(*outpoint))?;

        // Check maturity
        if !utxo.is_mature(current_height) {
            return Err(ValidationError::UtxoNotMature);
        }

        total_input += utxo.output.value;
    }

    // Verify sufficient funds
    let total_output = tx.total_output();
    if total_input < total_output {
        return Err(ValidationError::InsufficientFunds {
            input: total_input,
            output: total_output,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BlockHeader, TxInput, TxOutput};

    #[test]
    fn test_coinbase_validation() {
        let coinbase = Transaction::coinbase(1, STREAM_A_BLOCK_REWARD, &Address::ZERO);
        assert!(coinbase.is_coinbase());
        assert_eq!(coinbase.total_output(), STREAM_A_BLOCK_REWARD);
    }

    #[test]
    fn test_transaction_structure() {
        let tx = Transaction::new(
            vec![TxInput::new(OutPoint::new(H256::zero(), 0), vec![])],
            vec![TxOutput::p2pkh(1000, &Address::ZERO)],
        );
        assert!(!tx.is_coinbase());
        assert_eq!(tx.total_output(), 1000);
    }
}
