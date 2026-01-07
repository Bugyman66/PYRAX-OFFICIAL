//! Host Functions for WASM Contracts
//!
//! Provides the interface between WASM contracts and the blockchain

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::types::Address;
use super::runtime::{ContractLog, StateChange};
use super::storage::ContractStorage;

/// Host context passed to WASM execution
pub struct HostContext {
    /// Contract being executed
    pub contract_address: Address,
    /// Caller address
    pub caller: Address,
    /// Value transferred
    pub value: u64,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas used so far
    pub gas_used: u64,
    /// Current block height
    pub block_height: u64,
    /// Current block timestamp
    pub block_timestamp: u64,
    /// Storage interface
    storage: Arc<ContractStorage>,
    /// Pending storage writes (key -> value)
    pending_writes: Vec<(Vec<u8>, Vec<u8>)>,
    /// Emitted logs
    pub logs: Vec<ContractLog>,
    /// State changes for this execution
    pub state_changes: Vec<StateChange>,
    /// Call depth
    pub depth: u32,
    /// Read-only flag
    pub read_only: bool,
}

impl HostContext {
    /// Create new host context
    pub fn new(
        contract_address: Address,
        caller: Address,
        value: u64,
        gas_limit: u64,
        storage: Arc<ContractStorage>,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        Self {
            contract_address,
            caller,
            value,
            gas_limit,
            gas_used: 0,
            block_height: 0,
            block_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            storage,
            pending_writes: Vec::new(),
            logs: Vec::new(),
            state_changes: Vec::new(),
            depth: 0,
            read_only: false,
        }
    }

    /// Set block context
    pub fn with_block(mut self, height: u64, timestamp: u64) -> Self {
        self.block_height = height;
        self.block_timestamp = timestamp;
        self
    }

    /// Set read-only mode
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Read from storage
    pub fn storage_read(&self, key: &[u8]) -> Option<Vec<u8>> {
        // Check pending writes first
        for (k, v) in self.pending_writes.iter().rev() {
            if k == key {
                return Some(v.clone());
            }
        }
        
        // Read from persistent storage
        self.storage.get(&self.contract_address, key)
    }

    /// Write to storage
    pub fn storage_write(&mut self, key: Vec<u8>, value: Vec<u8>) {
        if self.read_only {
            return;
        }

        // Record state change
        let old_value = self.storage_read(&key);
        self.state_changes.push(StateChange {
            address: self.contract_address,
            key: {
                let mut k = [0u8; 32];
                let len = key.len().min(32);
                k[..len].copy_from_slice(&key[..len]);
                k
            },
            old_value: old_value.clone(),
            new_value: Some(value.clone()),
        });

        // Add to pending writes
        self.pending_writes.push((key, value));
    }

    /// Delete from storage
    pub fn storage_delete(&mut self, key: &[u8]) {
        if self.read_only {
            return;
        }

        let old_value = self.storage_read(key);
        self.state_changes.push(StateChange {
            address: self.contract_address,
            key: {
                let mut k = [0u8; 32];
                let len = key.len().min(32);
                k[..len].copy_from_slice(&key[..len]);
                k
            },
            old_value,
            new_value: None,
        });

        self.pending_writes.push((key.to_vec(), Vec::new()));
    }

    /// Emit a log event
    pub fn emit_log(&mut self, topics: Vec<[u8; 32]>, data: Vec<u8>) {
        if self.read_only {
            return;
        }

        self.logs.push(ContractLog {
            address: self.contract_address,
            topics,
            data,
        });
    }

    /// Commit pending writes to storage
    pub fn commit(&self) {
        for (key, value) in &self.pending_writes {
            if value.is_empty() {
                self.storage.delete(&self.contract_address, key);
            } else {
                self.storage.set(&self.contract_address, key, value.clone());
            }
        }
    }

    /// Consume gas
    pub fn consume_gas(&mut self, amount: u64) -> bool {
        if self.gas_used + amount > self.gas_limit {
            false
        } else {
            self.gas_used += amount;
            true
        }
    }

    /// Get remaining gas
    pub fn remaining_gas(&self) -> u64 {
        self.gas_limit.saturating_sub(self.gas_used)
    }
}

/// Host function definitions
pub struct HostFunctions;

impl HostFunctions {
    /// Storage read cost (base + per byte)
    pub const STORAGE_READ_BASE: u64 = 200;
    pub const STORAGE_READ_PER_BYTE: u64 = 3;

    /// Storage write cost (base + per byte)
    pub const STORAGE_WRITE_BASE: u64 = 5000;
    pub const STORAGE_WRITE_PER_BYTE: u64 = 10;

    /// Storage delete refund
    pub const STORAGE_DELETE_REFUND: u64 = 15000;

    /// Log cost (base + per topic + per byte)
    pub const LOG_BASE: u64 = 375;
    pub const LOG_PER_TOPIC: u64 = 375;
    pub const LOG_PER_BYTE: u64 = 8;

    /// Hash costs
    pub const BLAKE3_BASE: u64 = 30;
    pub const BLAKE3_PER_WORD: u64 = 6;
    pub const KECCAK256_BASE: u64 = 30;
    pub const KECCAK256_PER_WORD: u64 = 6;

    /// Call costs
    pub const CALL_BASE: u64 = 700;
    pub const CALL_VALUE_TRANSFER: u64 = 9000;
    pub const CALL_NEW_ACCOUNT: u64 = 25000;

    /// Calculate storage read cost
    pub fn storage_read_cost(value_len: usize) -> u64 {
        Self::STORAGE_READ_BASE + (value_len as u64 * Self::STORAGE_READ_PER_BYTE)
    }

    /// Calculate storage write cost
    pub fn storage_write_cost(value_len: usize) -> u64 {
        Self::STORAGE_WRITE_BASE + (value_len as u64 * Self::STORAGE_WRITE_PER_BYTE)
    }

    /// Calculate log cost
    pub fn log_cost(topics: usize, data_len: usize) -> u64 {
        Self::LOG_BASE 
            + (topics as u64 * Self::LOG_PER_TOPIC)
            + (data_len as u64 * Self::LOG_PER_BYTE)
    }

    /// Calculate hash cost
    pub fn hash_cost(input_len: usize) -> u64 {
        let words = (input_len + 31) / 32;
        Self::BLAKE3_BASE + (words as u64 * Self::BLAKE3_PER_WORD)
    }
}

/// Call frame for nested calls
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Contract address
    pub contract_address: Address,
    /// Caller address
    pub caller: Address,
    /// Value transferred
    pub value: u64,
    /// Gas allocated
    pub gas: u64,
    /// Input data
    pub input: Vec<u8>,
    /// Call depth
    pub depth: u32,
    /// Static call flag
    pub is_static: bool,
}

impl CallFrame {
    pub fn new(
        contract_address: Address,
        caller: Address,
        value: u64,
        gas: u64,
        input: Vec<u8>,
    ) -> Self {
        Self {
            contract_address,
            caller,
            value,
            gas,
            input,
            depth: 0,
            is_static: false,
        }
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn static_call(mut self) -> Self {
        self.is_static = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_functions_costs() {
        assert_eq!(HostFunctions::storage_read_cost(32), 200 + 96);
        assert_eq!(HostFunctions::storage_write_cost(32), 5000 + 320);
        assert_eq!(HostFunctions::log_cost(2, 64), 375 + 750 + 512);
    }

    #[test]
    fn test_host_context() {
        let storage = Arc::new(ContractStorage::new());
        let ctx = HostContext::new(
            Address([1u8; 20]),
            Address([2u8; 20]),
            1000,
            100000,
            storage,
        );

        assert_eq!(ctx.contract_address, Address([1u8; 20]));
        assert_eq!(ctx.caller, Address([2u8; 20]));
        assert_eq!(ctx.value, 1000);
        assert_eq!(ctx.remaining_gas(), 100000);
    }

    #[test]
    fn test_storage_operations() {
        let storage = Arc::new(ContractStorage::new());
        let mut ctx = HostContext::new(
            Address([1u8; 20]),
            Address([2u8; 20]),
            0,
            100000,
            storage,
        );

        // Write
        ctx.storage_write(vec![1, 2, 3], vec![4, 5, 6]);
        
        // Read pending
        let value = ctx.storage_read(&[1, 2, 3]);
        assert_eq!(value, Some(vec![4, 5, 6]));

        // State change recorded
        assert_eq!(ctx.state_changes.len(), 1);
    }

    #[test]
    fn test_gas_consumption() {
        let storage = Arc::new(ContractStorage::new());
        let mut ctx = HostContext::new(
            Address([1u8; 20]),
            Address([2u8; 20]),
            0,
            1000,
            storage,
        );

        assert!(ctx.consume_gas(500));
        assert_eq!(ctx.remaining_gas(), 500);
        
        assert!(ctx.consume_gas(500));
        assert_eq!(ctx.remaining_gas(), 0);
        
        assert!(!ctx.consume_gas(1)); // Should fail
    }
}
