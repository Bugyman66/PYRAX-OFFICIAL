//! Genesis block configuration and validation
//!
//! Canonical genesis blocks for mainnet, testnet, and devnet.
//! Each network has a deterministic genesis block with a fixed hash.

use serde::{Deserialize, Serialize};
use crate::types::{Block, BlockHeader, Transaction, TxInput, TxOutput, OutPoint, H256, Address, NetworkId};

/// Genesis configuration loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub network: String,
    pub chain_id: u32,
    pub timestamp: u64,
    pub difficulty: u64,
    pub block_reward: u64,
    pub version: u32,
    pub stream: u8,
    pub beneficiary: String,
    pub extra_data: String,
    #[serde(default)]
    pub allocations: Vec<GenesisAllocation>,
    pub expected_hash: Option<String>,
}

/// Pre-allocated balance at genesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAllocation {
    pub address: String,
    pub balance: u64,
}

/// Canonical genesis block hashes (computed deterministically)
pub mod hashes {
    use super::*;
    use std::sync::LazyLock;

    /// Mainnet genesis hash - FROZEN after first mainnet launch
    pub static MAINNET_GENESIS_HASH: LazyLock<H256> = LazyLock::new(|| {
        genesis_block(NetworkId::MAINNET).hash()
    });

    /// Testnet genesis hash
    pub static TESTNET_GENESIS_HASH: LazyLock<H256> = LazyLock::new(|| {
        genesis_block(NetworkId::TESTNET).hash()
    });

    /// Devnet genesis hash
    pub static DEVNET_GENESIS_HASH: LazyLock<H256> = LazyLock::new(|| {
        genesis_block(NetworkId::DEVNET).hash()
    });

    /// Get expected genesis hash for a network
    pub fn expected_genesis_hash(network: NetworkId) -> H256 {
        match network {
            NetworkId::MAINNET => *MAINNET_GENESIS_HASH,
            NetworkId::TESTNET => *TESTNET_GENESIS_HASH,
            NetworkId::DEVNET => *DEVNET_GENESIS_HASH,
            _ => *DEVNET_GENESIS_HASH,
        }
    }
}

/// Genesis block parameters for each network
pub mod params {
    use super::*;

    /// Mainnet genesis timestamp: 2025-01-01 00:00:00 UTC
    pub const MAINNET_TIMESTAMP: u64 = 1735689600;
    
    /// Testnet genesis timestamp: same as mainnet
    pub const TESTNET_TIMESTAMP: u64 = 1735689600;
    
    /// Devnet genesis timestamp: same as mainnet
    pub const DEVNET_TIMESTAMP: u64 = 1735689600;

    /// Mainnet initial difficulty
    pub const MAINNET_DIFFICULTY: u64 = 1_000_000;
    
    /// Testnet initial difficulty (lower for faster blocks)
    pub const TESTNET_DIFFICULTY: u64 = 1_000;
    
    /// Devnet initial difficulty (minimal for development)
    pub const DEVNET_DIFFICULTY: u64 = 1;

    /// Block reward: 50 PYRAX = 50 * 10^8 satoshis
    pub const BLOCK_REWARD: u64 = 50 * 100_000_000;

    /// Genesis extra data
    pub const MAINNET_EXTRA_DATA: &[u8] = b"PYRAX Genesis - TriStream DAG Blockchain";
    pub const TESTNET_EXTRA_DATA: &[u8] = b"PYRAX Testnet Genesis";
    pub const DEVNET_EXTRA_DATA: &[u8] = b"PYRAX Devnet Genesis";
}

/// Create the canonical genesis block for a network
pub fn genesis_block(network: NetworkId) -> Block {
    let (timestamp, difficulty, extra_data) = match network {
        NetworkId::MAINNET => (
            params::MAINNET_TIMESTAMP,
            params::MAINNET_DIFFICULTY,
            params::MAINNET_EXTRA_DATA,
        ),
        NetworkId::TESTNET => (
            params::TESTNET_TIMESTAMP,
            params::TESTNET_DIFFICULTY,
            params::TESTNET_EXTRA_DATA,
        ),
        NetworkId::DEVNET | _ => (
            params::DEVNET_TIMESTAMP,
            params::DEVNET_DIFFICULTY,
            params::DEVNET_EXTRA_DATA,
        ),
    };

    // Genesis coinbase transaction
    let coinbase_tx = Transaction {
        version: 1,
        inputs: vec![TxInput {
            previous_output: OutPoint::coinbase(),
            script_sig: extra_data.to_vec(),
            sequence: 0xFFFFFFFF,
        }],
        outputs: vec![TxOutput {
            value: 0, // Genesis coinbase has no output value
            script_pubkey: vec![],
        }],
        lock_time: 0,
    };

    let merkle_root = coinbase_tx.txid();

    let header = BlockHeader {
        version: 1,
        stream: 0, // Stream A (BLAKE3 PoW)
        parent_hash: H256::zero(),
        merkle_root,
        utxo_commitment: H256::zero(),
        timestamp,
        difficulty,
        nonce: 0,
        extra_nonce: 0,
        height: 0,
        beneficiary: Address::ZERO,
    };

    Block {
        header,
        transactions: vec![coinbase_tx],
    }
}

/// Validate that a block matches the expected genesis for a network
pub fn validate_genesis(block: &Block, network: NetworkId) -> Result<(), GenesisError> {
    let expected = genesis_block(network);
    let expected_hash = expected.hash();
    let actual_hash = block.hash();

    // Check hash matches
    if actual_hash != expected_hash {
        return Err(GenesisError::HashMismatch {
            expected: expected_hash,
            actual: actual_hash,
        });
    }

    // Check height is 0
    if block.height() != 0 {
        return Err(GenesisError::InvalidHeight(block.height()));
    }

    // Check parent is zero
    if block.header.parent_hash != H256::zero() {
        return Err(GenesisError::InvalidParentHash);
    }

    // Check timestamp
    if block.header.timestamp != expected.header.timestamp {
        return Err(GenesisError::InvalidTimestamp {
            expected: expected.header.timestamp,
            actual: block.header.timestamp,
        });
    }

    Ok(())
}

/// Genesis validation errors
#[derive(Debug, Clone)]
pub enum GenesisError {
    HashMismatch { expected: H256, actual: H256 },
    InvalidHeight(u64),
    InvalidParentHash,
    InvalidTimestamp { expected: u64, actual: u64 },
    InvalidDifficulty { expected: u64, actual: u64 },
}

impl std::fmt::Display for GenesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenesisError::HashMismatch { expected, actual } => {
                write!(f, "Genesis hash mismatch: expected {}, got {}", expected, actual)
            }
            GenesisError::InvalidHeight(h) => {
                write!(f, "Genesis block must have height 0, got {}", h)
            }
            GenesisError::InvalidParentHash => {
                write!(f, "Genesis block must have zero parent hash")
            }
            GenesisError::InvalidTimestamp { expected, actual } => {
                write!(f, "Genesis timestamp mismatch: expected {}, got {}", expected, actual)
            }
            GenesisError::InvalidDifficulty { expected, actual } => {
                write!(f, "Genesis difficulty mismatch: expected {}, got {}", expected, actual)
            }
        }
    }
}

impl std::error::Error for GenesisError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block_deterministic() {
        // Genesis block must be deterministic
        let genesis1 = genesis_block(NetworkId::DEVNET);
        let genesis2 = genesis_block(NetworkId::DEVNET);
        assert_eq!(genesis1.hash(), genesis2.hash());
    }

    #[test]
    fn test_genesis_validation() {
        let genesis = genesis_block(NetworkId::DEVNET);
        assert!(validate_genesis(&genesis, NetworkId::DEVNET).is_ok());
    }

    #[test]
    fn test_genesis_hash_differs_per_network() {
        let mainnet = genesis_block(NetworkId::MAINNET);
        let testnet = genesis_block(NetworkId::TESTNET);
        let devnet = genesis_block(NetworkId::DEVNET);
        
        // Each network has a unique genesis (due to different difficulty)
        assert_ne!(mainnet.hash(), devnet.hash());
        assert_ne!(testnet.hash(), devnet.hash());
    }

    #[test]
    fn test_genesis_has_coinbase() {
        let genesis = genesis_block(NetworkId::DEVNET);
        assert_eq!(genesis.transactions.len(), 1);
        assert!(genesis.transactions[0].is_coinbase());
    }
}
