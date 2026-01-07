//! Contract Storage
//!
//! Persistent storage layer for WASM contracts

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};

/// Storage key type
pub type StorageKey = Vec<u8>;

/// Storage value type
pub type StorageValue = Vec<u8>;

/// Contract code record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCodeRecord {
    /// Contract bytecode
    pub code: Vec<u8>,
    /// Code hash
    pub code_hash: H256,
    /// Deployment timestamp
    pub deployed_at: u64,
    /// Deployer address
    pub deployer: Address,
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Contract address
    pub address: Address,
    /// Code hash
    pub code_hash: H256,
    /// Version
    pub version: u32,
    /// Creator address
    pub creator: Address,
    /// Creation block
    pub creation_block: u64,
    /// Last update block
    pub last_update_block: u64,
    /// Is active
    pub is_active: bool,
}

/// Contract storage implementation
pub struct ContractStorage {
    /// Contract code by address
    code: Arc<RwLock<HashMap<Address, ContractCodeRecord>>>,
    /// Contract state by (address, key)
    state: Arc<RwLock<HashMap<Address, HashMap<StorageKey, StorageValue>>>>,
    /// Contract metadata by address
    metadata: Arc<RwLock<HashMap<Address, ContractMetadata>>>,
    /// Code hash to addresses mapping
    code_addresses: Arc<RwLock<HashMap<H256, Vec<Address>>>>,
}

impl ContractStorage {
    /// Create new contract storage
    pub fn new() -> Self {
        Self {
            code: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            code_addresses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store contract code
    pub fn store_code(&self, address: Address, code: Vec<u8>, code_hash: H256) {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let record = ContractCodeRecord {
            code,
            code_hash,
            deployed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            deployer: Address::ZERO,
        };

        self.code.write().insert(address, record);
        self.code_addresses.write()
            .entry(code_hash)
            .or_insert_with(Vec::new)
            .push(address);
    }

    /// Store code with deployer info
    pub fn store_code_with_deployer(
        &self,
        address: Address,
        code: Vec<u8>,
        code_hash: H256,
        deployer: Address,
        block: u64,
    ) {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let record = ContractCodeRecord {
            code,
            code_hash,
            deployed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            deployer,
        };

        self.code.write().insert(address, record);
        
        let metadata = ContractMetadata {
            address,
            code_hash,
            version: 1,
            creator: deployer,
            creation_block: block,
            last_update_block: block,
            is_active: true,
        };
        self.metadata.write().insert(address, metadata);
        
        self.code_addresses.write()
            .entry(code_hash)
            .or_insert_with(Vec::new)
            .push(address);
    }

    /// Get contract code
    pub fn get_code(&self, address: &Address) -> Option<Vec<u8>> {
        self.code.read().get(address).map(|r| r.code.clone())
    }

    /// Get code hash
    pub fn get_code_hash(&self, address: &Address) -> Option<H256> {
        self.code.read().get(address).map(|r| r.code_hash)
    }

    /// Check if contract exists
    pub fn has_code(&self, address: &Address) -> bool {
        self.code.read().contains_key(address)
    }

    /// Remove contract code
    pub fn remove_code(&self, address: &Address) {
        if let Some(record) = self.code.write().remove(address) {
            // Remove from code_addresses
            if let Some(addresses) = self.code_addresses.write().get_mut(&record.code_hash) {
                addresses.retain(|a| a != address);
            }
        }
        self.state.write().remove(address);
        self.metadata.write().remove(address);
    }

    /// Get storage value
    pub fn get(&self, address: &Address, key: &[u8]) -> Option<StorageValue> {
        self.state.read()
            .get(address)
            .and_then(|m| m.get(key).cloned())
    }

    /// Set storage value
    pub fn set(&self, address: &Address, key: &[u8], value: StorageValue) {
        self.state.write()
            .entry(*address)
            .or_insert_with(HashMap::new)
            .insert(key.to_vec(), value);
    }

    /// Delete storage value
    pub fn delete(&self, address: &Address, key: &[u8]) {
        if let Some(contract_state) = self.state.write().get_mut(address) {
            contract_state.remove(key);
        }
    }

    /// Get all storage for contract
    pub fn get_all(&self, address: &Address) -> HashMap<StorageKey, StorageValue> {
        self.state.read()
            .get(address)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear all storage for contract
    pub fn clear(&self, address: &Address) {
        self.state.write().remove(address);
    }

    /// Get storage size for contract
    pub fn storage_size(&self, address: &Address) -> usize {
        self.state.read()
            .get(address)
            .map(|m| m.len())
            .unwrap_or(0)
    }

    /// Get contract metadata
    pub fn get_metadata(&self, address: &Address) -> Option<ContractMetadata> {
        self.metadata.read().get(address).cloned()
    }

    /// Update contract metadata
    pub fn update_metadata(&self, address: &Address, block: u64) {
        if let Some(meta) = self.metadata.write().get_mut(address) {
            meta.last_update_block = block;
            meta.version += 1;
        }
    }

    /// Deactivate contract
    pub fn deactivate(&self, address: &Address) {
        if let Some(meta) = self.metadata.write().get_mut(address) {
            meta.is_active = false;
        }
    }

    /// Get contracts by code hash
    pub fn get_contracts_by_code(&self, code_hash: &H256) -> Vec<Address> {
        self.code_addresses.read()
            .get(code_hash)
            .cloned()
            .unwrap_or_default()
    }

    /// Get storage statistics
    pub fn stats(&self) -> StorageStats {
        let code = self.code.read();
        let state = self.state.read();
        
        let total_code_size: usize = code.values().map(|r| r.code.len()).sum();
        let total_state_entries: usize = state.values().map(|m| m.len()).sum();
        let total_state_size: usize = state.values()
            .flat_map(|m| m.values())
            .map(|v| v.len())
            .sum();

        StorageStats {
            contract_count: code.len() as u64,
            total_code_size: total_code_size as u64,
            total_state_entries: total_state_entries as u64,
            total_state_size: total_state_size as u64,
        }
    }
}

impl Default for ContractStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub contract_count: u64,
    pub total_code_size: u64,
    pub total_state_entries: u64,
    pub total_state_size: u64,
}

/// Storage proof for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    /// Contract address
    pub address: Address,
    /// Storage key
    pub key: StorageKey,
    /// Storage value
    pub value: Option<StorageValue>,
    /// Merkle proof (placeholder for future)
    pub proof: Vec<[u8; 32]>,
}

impl StorageProof {
    pub fn new(address: Address, key: StorageKey, value: Option<StorageValue>) -> Self {
        Self {
            address,
            key,
            value,
            proof: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let storage = ContractStorage::new();
        let stats = storage.stats();
        assert_eq!(stats.contract_count, 0);
    }

    #[test]
    fn test_code_storage() {
        let storage = ContractStorage::new();
        let address = Address([1u8; 20]);
        let code = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic
        let code_hash = H256([2u8; 32]);

        storage.store_code(address, code.clone(), code_hash);

        assert!(storage.has_code(&address));
        assert_eq!(storage.get_code(&address), Some(code));
        assert_eq!(storage.get_code_hash(&address), Some(code_hash));
    }

    #[test]
    fn test_state_storage() {
        let storage = ContractStorage::new();
        let address = Address([1u8; 20]);
        let key = vec![1, 2, 3];
        let value = vec![4, 5, 6];

        storage.set(&address, &key, value.clone());

        assert_eq!(storage.get(&address, &key), Some(value));
        assert_eq!(storage.storage_size(&address), 1);
    }

    #[test]
    fn test_storage_delete() {
        let storage = ContractStorage::new();
        let address = Address([1u8; 20]);
        let key = vec![1, 2, 3];

        storage.set(&address, &key, vec![4, 5, 6]);
        storage.delete(&address, &key);

        assert_eq!(storage.get(&address, &key), None);
    }

    #[test]
    fn test_storage_clear() {
        let storage = ContractStorage::new();
        let address = Address([1u8; 20]);

        storage.set(&address, &[1], vec![1]);
        storage.set(&address, &[2], vec![2]);
        storage.set(&address, &[3], vec![3]);

        assert_eq!(storage.storage_size(&address), 3);

        storage.clear(&address);
        assert_eq!(storage.storage_size(&address), 0);
    }

    #[test]
    fn test_contracts_by_code() {
        let storage = ContractStorage::new();
        let code_hash = H256([1u8; 32]);
        let addr1 = Address([1u8; 20]);
        let addr2 = Address([2u8; 20]);

        storage.store_code(addr1, vec![1], code_hash);
        storage.store_code(addr2, vec![1], code_hash);

        let contracts = storage.get_contracts_by_code(&code_hash);
        assert_eq!(contracts.len(), 2);
        assert!(contracts.contains(&addr1));
        assert!(contracts.contains(&addr2));
    }
}
