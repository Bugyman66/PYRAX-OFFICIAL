use thiserror::Error;
use tracing::{debug, warn};

use crate::types::{Block, BlockHeader, SignedTransaction, H256, BlockNumber};
use crate::storage::ChainDB;
use crate::config::ConsensusConfig;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid parent hash: expected {expected}, got {got}")]
    InvalidParentHash { expected: String, got: String },
    #[error("Invalid block height: expected {expected}, got {got}")]
    InvalidHeight { expected: BlockNumber, got: BlockNumber },
    #[error("Invalid timestamp: block timestamp {block} not after parent {parent}")]
    InvalidTimestamp { block: u64, parent: u64 },
    #[error("Future block: timestamp {0} is in the future")]
    FutureBlock(u64),
    #[error("Invalid difficulty: expected {expected}, got {got}")]
    InvalidDifficulty { expected: u64, got: u64 },
    #[error("Invalid merkle root: expected {expected}, got {got}")]
    InvalidMerkleRoot { expected: String, got: String },
    #[error("PoW validation failed: hash does not meet target")]
    InvalidPoW,
    #[error("Invalid transaction at index {index}: {reason}")]
    InvalidTransaction { index: usize, reason: String },
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },
    #[error("Gas limit exceeded")]
    GasLimitExceeded,
    #[error("Duplicate transaction")]
    DuplicateTransaction,
}

pub fn validate_block(
    block: &Block,
    parent: Option<&Block>,
    config: &ConsensusConfig,
) -> Result<(), ValidationError> {
    debug!("Validating block at height {}", block.height());

    // Genesis block special case
    if block.height() == 0 {
        return validate_genesis(block);
    }

    let parent = parent.ok_or_else(|| ValidationError::InvalidParentHash {
        expected: "parent block".into(),
        got: "none".into(),
    })?;

    // Validate parent hash
    if block.header.parent_hash != parent.hash() {
        return Err(ValidationError::InvalidParentHash {
            expected: hex::encode(parent.hash().as_bytes()),
            got: hex::encode(block.header.parent_hash.as_bytes()),
        });
    }

    // Validate height
    let expected_height = parent.height() + 1;
    if block.height() != expected_height {
        return Err(ValidationError::InvalidHeight {
            expected: expected_height,
            got: block.height(),
        });
    }

    // Validate timestamp
    if block.header.timestamp <= parent.header.timestamp {
        return Err(ValidationError::InvalidTimestamp {
            block: block.header.timestamp,
            parent: parent.header.timestamp,
        });
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if block.header.timestamp > now + 15 {
        return Err(ValidationError::FutureBlock(block.header.timestamp));
    }

    // Validate merkle root
    let computed_merkle = block.body.merkle_root();
    if block.header.merkle_root != computed_merkle {
        return Err(ValidationError::InvalidMerkleRoot {
            expected: hex::encode(computed_merkle.as_bytes()),
            got: hex::encode(block.header.merkle_root.as_bytes()),
        });
    }

    // Validate PoW
    let block_hash = block.hash();
    if !block.header.meets_difficulty(&block_hash) {
        return Err(ValidationError::InvalidPoW);
    }

    // Validate all transactions
    for (idx, tx) in block.body.transactions.iter().enumerate() {
        validate_transaction(tx).map_err(|e| ValidationError::InvalidTransaction {
            index: idx,
            reason: e.to_string(),
        })?;
    }

    debug!("Block {} validated successfully", block.height());
    Ok(())
}

fn validate_genesis(block: &Block) -> Result<(), ValidationError> {
    if block.header.parent_hash != H256::zero() {
        return Err(ValidationError::InvalidParentHash {
            expected: "zero hash".into(),
            got: hex::encode(block.header.parent_hash.as_bytes()),
        });
    }

    if block.height() != 0 {
        return Err(ValidationError::InvalidHeight {
            expected: 0,
            got: block.height(),
        });
    }

    Ok(())
}

pub fn validate_transaction(tx: &SignedTransaction) -> Result<(), ValidationError> {
    // Verify signature
    if !tx.verify() {
        return Err(ValidationError::InvalidSignature);
    }

    // Check gas limit covers intrinsic gas
    let intrinsic_gas = tx.tx.intrinsic_gas();
    if tx.tx.gas_limit < intrinsic_gas {
        return Err(ValidationError::GasLimitExceeded);
    }

    Ok(())
}

pub fn validate_transaction_against_state(
    tx: &SignedTransaction,
    sender_balance: ethereum_types::U256,
    sender_nonce: u64,
) -> Result<(), ValidationError> {
    // Verify nonce
    if tx.tx.nonce != sender_nonce {
        return Err(ValidationError::InvalidNonce {
            expected: sender_nonce,
            got: tx.tx.nonce,
        });
    }

    // Verify balance covers value + max fee
    let max_fee = ethereum_types::U256::from(tx.tx.gas_price) * ethereum_types::U256::from(tx.tx.gas_limit);
    let total_cost = tx.tx.value + max_fee;

    if sender_balance < total_cost {
        return Err(ValidationError::InsufficientBalance);
    }

    Ok(())
}
