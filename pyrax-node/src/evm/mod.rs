//! EVM Sidechain Module
//!
//! Implements an EVM-compatible execution layer for PYRAX using revm.
//! Features:
//! - Full EVM opcode support
//! - EIP-1559 gas metering
//! - Account state trie (MPT-based)
//! - Precompiled contracts
//! - L1-L2 bridge integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod state;
pub mod executor;
pub mod types;
pub mod precompiles;
pub mod bridge;
pub mod chain;
pub mod rpc;

pub use state::{AccountState, StateDB, StorageSlot};
pub use executor::{EvmExecutor, ExecutionResult, TransactionContext};
pub use types::*;

/// EVM Chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain ID (unique identifier)
    pub chain_id: u64,
    /// Block gas limit
    pub block_gas_limit: u64,
    /// Base fee per gas (EIP-1559)
    pub base_fee_per_gas: u64,
    /// Target block time in milliseconds
    pub block_time_ms: u64,
    /// Whether London (EIP-1559) is active
    pub london_active: bool,
    /// Whether Shanghai is active
    pub shanghai_active: bool,
    /// Whether Cancun is active
    pub cancun_active: bool,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_id: 7777, // PYRAX EVM chain ID
            block_gas_limit: 30_000_000,
            base_fee_per_gas: 1_000_000_000, // 1 cinder
            block_time_ms: 2000, // 2 second blocks
            london_active: true,
            shanghai_active: true,
            cancun_active: true,
        }
    }
}

/// EVM Block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmBlockHeader {
    pub number: u64,
    pub hash: [u8; 32],
    pub parent_hash: [u8; 32],
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
    #[serde(with = "logs_bloom_serde")]
    pub logs_bloom: Vec<u8>,
    pub difficulty: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub extra_data: Vec<u8>,
    pub mix_hash: [u8; 32],
    pub nonce: u64,
    pub base_fee_per_gas: u64,
    pub withdrawals_root: Option<[u8; 32]>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
}

mod logs_bloom_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    
    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        hex::encode(data).serialize(serializer)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex::decode(&s).map_err(serde::de::Error::custom)
    }
}

impl Default for EvmBlockHeader {
    fn default() -> Self {
        Self {
            number: 0,
            hash: [0u8; 32],
            parent_hash: [0u8; 32],
            state_root: [0u8; 32],
            transactions_root: [0u8; 32],
            receipts_root: [0u8; 32],
            logs_bloom: vec![0u8; 256],
            difficulty: 0,
            gas_limit: 30_000_000,
            gas_used: 0,
            timestamp: 0,
            extra_data: Vec::new(),
            mix_hash: [0u8; 32],
            nonce: 0,
            base_fee_per_gas: 1_000_000_000,
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
        }
    }
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_hash: [u8; 32],
    pub transaction_index: u64,
    pub block_hash: [u8; 32],
    pub block_number: u64,
    pub from: [u8; 20],
    pub to: Option<[u8; 20]>,
    pub cumulative_gas_used: u64,
    pub gas_used: u64,
    pub contract_address: Option<[u8; 20]>,
    pub logs: Vec<Log>,
    #[serde(with = "logs_bloom_serde")]
    pub logs_bloom: Vec<u8>,
    pub status: u8, // 1 = success, 0 = failure
    pub effective_gas_price: u64,
}

/// Event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub address: [u8; 20],
    pub topics: Vec<[u8; 32]>,
    pub data: Vec<u8>,
    pub block_number: u64,
    pub transaction_hash: [u8; 32],
    pub transaction_index: u64,
    pub block_hash: [u8; 32],
    pub log_index: u64,
    pub removed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_config_default() {
        let config = ChainConfig::default();
        assert_eq!(config.chain_id, 7777);
        assert_eq!(config.block_time_ms, 2000);
        assert!(config.london_active);
    }

    #[test]
    fn test_block_header_default() {
        let header = EvmBlockHeader::default();
        assert_eq!(header.number, 0);
        assert_eq!(header.gas_limit, 30_000_000);
    }
}
