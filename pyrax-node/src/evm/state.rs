//! EVM State Database
//!
//! Implements account state management with Merkle Patricia Trie

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use blake3::Hasher;

use super::types::{Address, B256, U256};

/// Storage slot key-value pair
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageSlot {
    pub original_value: U256,
    pub current_value: U256,
}

/// Account state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub nonce: u64,
    pub balance: U256,
    pub code_hash: B256,
    pub code: Vec<u8>,
    pub storage: HashMap<B256, StorageSlot>,
}

impl Default for AccountState {
    fn default() -> Self {
        Self {
            nonce: 0,
            balance: U256::ZERO,
            code_hash: EMPTY_CODE_HASH,
            code: Vec::new(),
            storage: HashMap::new(),
        }
    }
}

impl AccountState {
    /// Create a new account with balance
    pub fn new_with_balance(balance: U256) -> Self {
        Self {
            balance,
            ..Default::default()
        }
    }

    /// Create a contract account with code
    pub fn new_contract(code: Vec<u8>, balance: U256) -> Self {
        let code_hash = hash_code(&code);
        Self {
            nonce: 1,
            balance,
            code_hash,
            code,
            storage: HashMap::new(),
        }
    }

    /// Check if account is empty (can be pruned)
    pub fn is_empty(&self) -> bool {
        self.nonce == 0 && self.balance.is_zero() && self.code.is_empty()
    }

    /// Check if account has code (is a contract)
    pub fn has_code(&self) -> bool {
        !self.code.is_empty() && self.code_hash != EMPTY_CODE_HASH
    }

    /// Get storage value
    pub fn get_storage(&self, key: &B256) -> U256 {
        self.storage
            .get(key)
            .map(|slot| slot.current_value)
            .unwrap_or(U256::ZERO)
    }

    /// Set storage value
    pub fn set_storage(&mut self, key: B256, value: U256) {
        let slot = self.storage.entry(key).or_default();
        slot.current_value = value;
    }
}

/// Empty code hash (keccak256 of empty bytes)
pub const EMPTY_CODE_HASH: B256 = [
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c,
    0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b,
    0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
];

/// Hash code using BLAKE3
fn hash_code(code: &[u8]) -> B256 {
    if code.is_empty() {
        return EMPTY_CODE_HASH;
    }
    let mut hasher = Hasher::new();
    hasher.update(code);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(result.as_bytes());
    hash
}

/// State database for EVM execution
#[derive(Debug)]
pub struct StateDB {
    /// Account states
    accounts: Arc<RwLock<HashMap<Address, AccountState>>>,
    /// Cached state root
    state_root: Arc<RwLock<B256>>,
    /// Block number this state is at
    block_number: u64,
    /// Pending changes (not yet committed)
    pending_changes: Arc<RwLock<HashMap<Address, AccountState>>>,
    /// Deleted accounts
    deleted_accounts: Arc<RwLock<Vec<Address>>>,
}

impl StateDB {
    /// Create a new empty state database
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            state_root: Arc::new(RwLock::new([0u8; 32])),
            block_number: 0,
            pending_changes: Arc::new(RwLock::new(HashMap::new())),
            deleted_accounts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create state database with genesis accounts
    pub fn with_genesis(genesis_accounts: HashMap<Address, AccountState>) -> Self {
        let db = Self::new();
        {
            let mut accounts = db.accounts.write();
            *accounts = genesis_accounts;
        }
        db.compute_state_root();
        db
    }

    /// Get account state (or default if not exists)
    pub fn get_account(&self, address: &Address) -> AccountState {
        // Check pending changes first
        if let Some(account) = self.pending_changes.read().get(address) {
            return account.clone();
        }
        // Then check committed state
        self.accounts
            .read()
            .get(address)
            .cloned()
            .unwrap_or_default()
    }

    /// Get account balance
    pub fn get_balance(&self, address: &Address) -> U256 {
        self.get_account(address).balance
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &Address) -> u64 {
        self.get_account(address).nonce
    }

    /// Get account code
    pub fn get_code(&self, address: &Address) -> Vec<u8> {
        self.get_account(address).code
    }

    /// Get account code hash
    pub fn get_code_hash(&self, address: &Address) -> B256 {
        self.get_account(address).code_hash
    }

    /// Get storage value
    pub fn get_storage(&self, address: &Address, key: &B256) -> U256 {
        self.get_account(address).get_storage(key)
    }

    /// Check if account exists
    pub fn account_exists(&self, address: &Address) -> bool {
        if self.pending_changes.read().contains_key(address) {
            return true;
        }
        self.accounts.read().contains_key(address)
    }

    /// Set account state (pending)
    pub fn set_account(&self, address: Address, account: AccountState) {
        self.pending_changes.write().insert(address, account);
    }

    /// Update account balance
    pub fn add_balance(&self, address: &Address, amount: U256) {
        let mut account = self.get_account(address);
        account.balance = account.balance.saturating_add(amount);
        self.set_account(*address, account);
    }

    /// Subtract account balance
    pub fn sub_balance(&self, address: &Address, amount: U256) -> bool {
        let mut account = self.get_account(address);
        if let Some(new_balance) = account.balance.checked_sub(amount) {
            account.balance = new_balance;
            self.set_account(*address, account);
            true
        } else {
            false
        }
    }

    /// Increment account nonce
    pub fn increment_nonce(&self, address: &Address) {
        let mut account = self.get_account(address);
        account.nonce = account.nonce.saturating_add(1);
        self.set_account(*address, account);
    }

    /// Set account code (for contract creation)
    pub fn set_code(&self, address: &Address, code: Vec<u8>) {
        let mut account = self.get_account(address);
        account.code_hash = hash_code(&code);
        account.code = code;
        self.set_account(*address, account);
    }

    /// Set storage value
    pub fn set_storage(&self, address: &Address, key: B256, value: U256) {
        let mut account = self.get_account(address);
        account.set_storage(key, value);
        self.set_account(*address, account);
    }

    /// Mark account for deletion (SELFDESTRUCT)
    pub fn delete_account(&self, address: &Address) {
        self.deleted_accounts.write().push(*address);
        self.pending_changes.write().remove(address);
    }

    /// Commit pending changes to main state
    pub fn commit(&self) -> B256 {
        // Apply pending changes
        {
            let mut accounts = self.accounts.write();
            let pending = self.pending_changes.read();
            for (address, account) in pending.iter() {
                if account.is_empty() {
                    accounts.remove(address);
                } else {
                    accounts.insert(*address, account.clone());
                }
            }
        }

        // Remove deleted accounts
        {
            let mut accounts = self.accounts.write();
            let deleted = self.deleted_accounts.read();
            for address in deleted.iter() {
                accounts.remove(address);
            }
        }

        // Clear pending
        self.pending_changes.write().clear();
        self.deleted_accounts.write().clear();

        // Compute new state root
        self.compute_state_root()
    }

    /// Revert pending changes
    pub fn revert(&self) {
        self.pending_changes.write().clear();
        self.deleted_accounts.write().clear();
    }

    /// Compute state root from current state
    fn compute_state_root(&self) -> B256 {
        let accounts = self.accounts.read();
        
        // Sort accounts by address for deterministic hashing
        let mut sorted_accounts: Vec<_> = accounts.iter().collect();
        sorted_accounts.sort_by_key(|(addr, _)| *addr);

        let mut hasher = Hasher::new();
        for (address, account) in sorted_accounts {
            hasher.update(address);
            hasher.update(&account.nonce.to_be_bytes());
            hasher.update(&account.balance.to_be_bytes());
            hasher.update(&account.code_hash);
            
            // Hash storage
            let mut storage_entries: Vec<_> = account.storage.iter().collect();
            storage_entries.sort_by_key(|(k, _)| *k);
            for (key, slot) in storage_entries {
                hasher.update(key);
                hasher.update(&slot.current_value.to_be_bytes());
            }
        }

        let result = hasher.finalize();
        let mut root = [0u8; 32];
        root.copy_from_slice(result.as_bytes());
        
        *self.state_root.write() = root;
        root
    }

    /// Get current state root
    pub fn state_root(&self) -> B256 {
        *self.state_root.read()
    }

    /// Get block number
    pub fn block_number(&self) -> u64 {
        self.block_number
    }

    /// Set block number
    pub fn set_block_number(&mut self, number: u64) {
        self.block_number = number;
    }

    /// Create a snapshot for reverting
    pub fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            pending_changes: self.pending_changes.read().clone(),
            deleted_accounts: self.deleted_accounts.read().clone(),
        }
    }

    /// Restore from snapshot
    pub fn restore(&self, snapshot: StateSnapshot) {
        *self.pending_changes.write() = snapshot.pending_changes;
        *self.deleted_accounts.write() = snapshot.deleted_accounts;
    }

    /// Get total number of accounts
    pub fn account_count(&self) -> usize {
        self.accounts.read().len()
    }
}

impl Default for StateDB {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StateDB {
    fn clone(&self) -> Self {
        Self {
            accounts: Arc::new(RwLock::new(self.accounts.read().clone())),
            state_root: Arc::new(RwLock::new(*self.state_root.read())),
            block_number: self.block_number,
            pending_changes: Arc::new(RwLock::new(self.pending_changes.read().clone())),
            deleted_accounts: Arc::new(RwLock::new(self.deleted_accounts.read().clone())),
        }
    }
}

/// State snapshot for reverting
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pending_changes: HashMap<Address, AccountState>,
    deleted_accounts: Vec<Address>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_state_default() {
        let account = AccountState::default();
        assert_eq!(account.nonce, 0);
        assert!(account.balance.is_zero());
        assert!(account.is_empty());
    }

    #[test]
    fn test_account_with_balance() {
        let account = AccountState::new_with_balance(U256::from_u64(1000));
        assert_eq!(account.balance.low_u64(), 1000);
        assert!(!account.is_empty());
    }

    #[test]
    fn test_state_db_basic() {
        let db = StateDB::new();
        let address = [1u8; 20];

        // Initially empty
        assert!(!db.account_exists(&address));
        assert!(db.get_balance(&address).is_zero());

        // Add balance
        db.add_balance(&address, U256::from_u64(1000));
        assert_eq!(db.get_balance(&address).low_u64(), 1000);

        // Commit
        let root = db.commit();
        assert!(!root.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_state_db_storage() {
        let db = StateDB::new();
        let address = [2u8; 20];
        let key = [3u8; 32];
        let value = U256::from_u64(42);

        db.set_storage(&address, key, value);
        assert_eq!(db.get_storage(&address, &key).low_u64(), 42);
    }

    #[test]
    fn test_state_db_revert() {
        let db = StateDB::new();
        let address = [4u8; 20];

        db.add_balance(&address, U256::from_u64(1000));
        assert_eq!(db.get_balance(&address).low_u64(), 1000);

        db.revert();
        assert!(db.get_balance(&address).is_zero());
    }

    #[test]
    fn test_state_snapshot() {
        let db = StateDB::new();
        let address = [5u8; 20];

        db.add_balance(&address, U256::from_u64(500));
        let snapshot = db.snapshot();

        db.add_balance(&address, U256::from_u64(500));
        assert_eq!(db.get_balance(&address).low_u64(), 1000);

        db.restore(snapshot);
        assert_eq!(db.get_balance(&address).low_u64(), 500);
    }
}
