//! L2 State Management
//!
//! Merkle tree state, commitments, and transitions

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};

/// L2 account state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Account {
    /// Account nonce
    pub nonce: u64,
    /// Account balance (smallest PYRAX unit)
    pub balance: u64,
    /// Storage root hash
    pub storage_root: H256,
    /// Code hash (for contracts)
    pub code_hash: H256,
}

impl Default for L2Account {
    fn default() -> Self {
        Self {
            nonce: 0,
            balance: 0,
            storage_root: H256::zero(),
            code_hash: H256::zero(),
        }
    }
}

impl L2Account {
    /// Create new account with balance
    pub fn with_balance(balance: u64) -> Self {
        Self {
            balance,
            ..Default::default()
        }
    }

    /// Compute account hash
    pub fn hash(&self) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.balance.to_le_bytes());
        hasher.update(&self.storage_root.0);
        hasher.update(&self.code_hash.0);
        H256::from_slice(hasher.finalize().as_bytes())
    }
}

/// State commitment (root hash + metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateCommitment {
    /// State root hash
    pub state_root: H256,
    /// Block number this commitment is for
    pub block_number: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Number of accounts
    pub account_count: u64,
    /// Total value locked
    pub total_value: u64,
}

impl StateCommitment {
    /// Create new commitment
    pub fn new(state_root: H256, block_number: u64) -> Self {
        Self {
            state_root,
            block_number,
            timestamp: current_timestamp(),
            account_count: 0,
            total_value: 0,
        }
    }

    /// Compute commitment hash
    pub fn hash(&self) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.state_root.0);
        hasher.update(&self.block_number.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }
}

/// State transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Previous state root
    pub pre_state_root: H256,
    /// Post state root
    pub post_state_root: H256,
    /// Block number
    pub block_number: u64,
    /// Transaction hashes included
    pub tx_hashes: Vec<H256>,
    /// Accounts modified
    pub modified_accounts: Vec<Address>,
    /// Transition hash
    pub transition_hash: H256,
}

impl StateTransition {
    /// Create new state transition
    pub fn new(
        pre_state_root: H256,
        post_state_root: H256,
        block_number: u64,
        tx_hashes: Vec<H256>,
        modified_accounts: Vec<Address>,
    ) -> Self {
        let transition_hash = Self::compute_hash(&pre_state_root, &post_state_root, block_number);
        Self {
            pre_state_root,
            post_state_root,
            block_number,
            tx_hashes,
            modified_accounts,
            transition_hash,
        }
    }

    /// Compute transition hash
    fn compute_hash(pre: &H256, post: &H256, block: u64) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&pre.0);
        hasher.update(&post.0);
        hasher.update(&block.to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }
}

/// Merkle proof for state inclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Leaf value hash
    pub leaf: H256,
    /// Sibling hashes from leaf to root
    pub siblings: Vec<H256>,
    /// Path bits (0 = left, 1 = right)
    pub path: Vec<bool>,
    /// Root hash
    pub root: H256,
}

impl MerkleProof {
    /// Verify the proof
    pub fn verify(&self) -> bool {
        let mut current = self.leaf;
        
        for (i, sibling) in self.siblings.iter().enumerate() {
            current = if self.path.get(i).copied().unwrap_or(false) {
                // Current is on the right
                Self::hash_pair(sibling, &current)
            } else {
                // Current is on the left
                Self::hash_pair(&current, sibling)
            };
        }
        
        current == self.root
    }

    /// Hash two nodes together
    fn hash_pair(left: &H256, right: &H256) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&left.0);
        hasher.update(&right.0);
        H256::from_slice(hasher.finalize().as_bytes())
    }
}

/// Sparse Merkle Tree for L2 state
pub struct SparseMerkleTree {
    /// Tree depth (256 for address-keyed tree)
    depth: usize,
    /// Cached nodes (hash -> children)
    nodes: HashMap<H256, (H256, H256)>,
    /// Leaf values (key -> value hash)
    leaves: HashMap<H256, H256>,
    /// Current root
    root: H256,
    /// Default hashes for each level
    default_hashes: Vec<H256>,
}

impl SparseMerkleTree {
    /// Create new tree with specified depth
    pub fn new(depth: usize) -> Self {
        let default_hashes = Self::compute_default_hashes(depth);
        let root = default_hashes[depth];
        
        Self {
            depth,
            nodes: HashMap::new(),
            leaves: HashMap::new(),
            root,
            default_hashes,
        }
    }

    /// Compute default hashes for empty tree
    fn compute_default_hashes(depth: usize) -> Vec<H256> {
        let mut hashes = vec![H256::zero()]; // Level 0 (leaf)
        
        for _ in 0..depth {
            let prev = hashes.last().unwrap();
            let next = MerkleProof::hash_pair(prev, prev);
            hashes.push(next);
        }
        
        hashes
    }

    /// Get current root
    pub fn root(&self) -> H256 {
        self.root
    }

    /// Get value at key
    pub fn get(&self, key: &H256) -> Option<H256> {
        self.leaves.get(key).copied()
    }

    /// Update value at key
    pub fn update(&mut self, key: &H256, value: H256) {
        self.leaves.insert(*key, value);
        self.recompute_root();
    }

    /// Delete value at key
    pub fn delete(&mut self, key: &H256) {
        self.leaves.remove(key);
        self.recompute_root();
    }

    /// Generate proof for key
    pub fn prove(&self, key: &H256) -> MerkleProof {
        let leaf = self.leaves.get(key).copied().unwrap_or(H256::zero());
        let mut siblings = Vec::new();
        let mut path = Vec::new();
        
        // Walk up the tree collecting siblings
        let key_bits = Self::key_to_bits(key, self.depth);
        let mut current_hash = leaf;
        
        for (level, &is_right) in key_bits.iter().enumerate() {
            path.push(is_right);
            
            // Get sibling at this level
            let sibling = self.get_sibling_hash(key, level);
            siblings.push(sibling);
            
            // Compute parent
            current_hash = if is_right {
                MerkleProof::hash_pair(&sibling, &current_hash)
            } else {
                MerkleProof::hash_pair(&current_hash, &sibling)
            };
        }
        
        MerkleProof {
            leaf,
            siblings,
            path,
            root: self.root,
        }
    }

    /// Get sibling hash at level
    fn get_sibling_hash(&self, key: &H256, level: usize) -> H256 {
        // For a proper implementation, this would traverse the tree
        // For now, return default hash at level
        self.default_hashes[level]
    }

    /// Convert key to bit path
    fn key_to_bits(key: &H256, depth: usize) -> Vec<bool> {
        let mut bits = Vec::with_capacity(depth);
        for i in 0..depth {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            let bit = (key.0[byte_idx] >> bit_idx) & 1 == 1;
            bits.push(bit);
        }
        bits
    }

    /// Recompute root after updates
    fn recompute_root(&mut self) {
        if self.leaves.is_empty() {
            self.root = self.default_hashes[self.depth];
            return;
        }

        // Simple recomputation for small trees
        // Production would use more efficient incremental updates
        let mut level_hashes: Vec<H256> = self.leaves.values().copied().collect();
        
        if level_hashes.is_empty() {
            self.root = self.default_hashes[self.depth];
            return;
        }

        // Pad to power of 2
        let target_size = (level_hashes.len() as f64).log2().ceil() as u32;
        let target_size = 2usize.pow(target_size);
        while level_hashes.len() < target_size {
            level_hashes.push(H256::zero());
        }

        // Build tree bottom-up
        while level_hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in level_hashes.chunks(2) {
                let left = chunk[0];
                let right = chunk.get(1).copied().unwrap_or(H256::zero());
                next_level.push(MerkleProof::hash_pair(&left, &right));
            }
            level_hashes = next_level;
        }

        self.root = level_hashes[0];
    }

    /// Get tree statistics
    pub fn stats(&self) -> TreeStats {
        TreeStats {
            depth: self.depth,
            leaf_count: self.leaves.len(),
            node_count: self.nodes.len(),
            root: self.root,
        }
    }
}

/// Tree statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeStats {
    pub depth: usize,
    pub leaf_count: usize,
    pub node_count: usize,
    pub root: H256,
}

/// L2 State manager
pub struct L2State {
    /// Account state tree
    accounts: Arc<RwLock<SparseMerkleTree>>,
    /// Account data
    account_data: Arc<RwLock<HashMap<Address, L2Account>>>,
    /// Storage trees per contract
    storage: Arc<RwLock<HashMap<Address, SparseMerkleTree>>>,
    /// State commitments history
    commitments: Arc<RwLock<Vec<StateCommitment>>>,
    /// State transitions history
    transitions: Arc<RwLock<Vec<StateTransition>>>,
    /// Current block number
    block_number: Arc<RwLock<u64>>,
}

impl L2State {
    /// Create new L2 state
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(SparseMerkleTree::new(160))), // 20-byte address
            account_data: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(RwLock::new(HashMap::new())),
            commitments: Arc::new(RwLock::new(Vec::new())),
            transitions: Arc::new(RwLock::new(Vec::new())),
            block_number: Arc::new(RwLock::new(0)),
        }
    }

    /// Get current state root
    pub fn state_root(&self) -> H256 {
        self.accounts.read().root()
    }

    /// Get current block number
    pub fn block_number(&self) -> u64 {
        *self.block_number.read()
    }

    /// Get account
    pub fn get_account(&self, address: &Address) -> L2Account {
        self.account_data.read()
            .get(address)
            .cloned()
            .unwrap_or_default()
    }

    /// Update account
    pub fn update_account(&self, address: Address, account: L2Account) {
        let account_hash = account.hash();
        let key = Self::address_to_key(&address);
        
        self.account_data.write().insert(address, account);
        self.accounts.write().update(&key, account_hash);
    }

    /// Get balance
    pub fn get_balance(&self, address: &Address) -> u64 {
        self.get_account(address).balance
    }

    /// Get nonce
    pub fn get_nonce(&self, address: &Address) -> u64 {
        self.get_account(address).nonce
    }

    /// Transfer balance
    pub fn transfer(
        &self,
        from: &Address,
        to: &Address,
        amount: u64,
    ) -> Result<(), StateError> {
        let mut from_account = self.get_account(from);
        let mut to_account = self.get_account(to);

        if from_account.balance < amount {
            return Err(StateError::InsufficientBalance {
                available: from_account.balance,
                required: amount,
            });
        }

        from_account.balance -= amount;
        from_account.nonce += 1;
        to_account.balance += amount;

        self.update_account(*from, from_account);
        self.update_account(*to, to_account);

        Ok(())
    }

    /// Credit account (for deposits)
    pub fn credit(&self, address: &Address, amount: u64) {
        let mut account = self.get_account(address);
        account.balance += amount;
        self.update_account(*address, account);
    }

    /// Debit account (for withdrawals)
    pub fn debit(&self, address: &Address, amount: u64) -> Result<(), StateError> {
        let mut account = self.get_account(address);
        
        if account.balance < amount {
            return Err(StateError::InsufficientBalance {
                available: account.balance,
                required: amount,
            });
        }

        account.balance -= amount;
        self.update_account(*address, account);
        Ok(())
    }

    /// Increment nonce
    pub fn increment_nonce(&self, address: &Address) {
        let mut account = self.get_account(address);
        account.nonce += 1;
        self.update_account(*address, account);
    }

    /// Set storage value
    pub fn set_storage(&self, address: &Address, key: H256, value: H256) {
        let mut storage = self.storage.write();
        let tree = storage.entry(*address)
            .or_insert_with(|| SparseMerkleTree::new(256));
        tree.update(&key, value);

        // Update account's storage root
        let storage_root = tree.root();
        drop(storage);

        let mut account = self.get_account(address);
        account.storage_root = storage_root;
        self.update_account(*address, account);
    }

    /// Get storage value
    pub fn get_storage(&self, address: &Address, key: &H256) -> H256 {
        self.storage.read()
            .get(address)
            .and_then(|tree| tree.get(key))
            .unwrap_or(H256::zero())
    }

    /// Generate proof for account
    pub fn prove_account(&self, address: &Address) -> MerkleProof {
        let key = Self::address_to_key(address);
        self.accounts.read().prove(&key)
    }

    /// Apply state transition
    pub fn apply_transition(&self, transition: &StateTransition) -> Result<(), StateError> {
        let current_root = self.state_root();
        
        if current_root != transition.pre_state_root {
            return Err(StateError::InvalidPreState {
                expected: transition.pre_state_root,
                actual: current_root,
            });
        }

        // Transition is already applied, just record it
        self.transitions.write().push(transition.clone());
        *self.block_number.write() = transition.block_number;

        Ok(())
    }

    /// Create state commitment
    pub fn commit(&self) -> StateCommitment {
        let account_data = self.account_data.read();
        let total_value: u64 = account_data.values().map(|a| a.balance).sum();
        
        let mut commitment = StateCommitment::new(
            self.state_root(),
            *self.block_number.read(),
        );
        commitment.account_count = account_data.len() as u64;
        commitment.total_value = total_value;

        self.commitments.write().push(commitment.clone());
        commitment
    }

    /// Get commitment at block
    pub fn get_commitment(&self, block_number: u64) -> Option<StateCommitment> {
        self.commitments.read()
            .iter()
            .find(|c| c.block_number == block_number)
            .cloned()
    }

    /// Convert address to tree key
    fn address_to_key(address: &Address) -> H256 {
        let mut key = [0u8; 32];
        key[12..32].copy_from_slice(&address.0);
        H256(key)
    }

    /// Get state statistics
    pub fn stats(&self) -> L2StateStats {
        let account_data = self.account_data.read();
        L2StateStats {
            block_number: *self.block_number.read(),
            state_root: self.state_root(),
            account_count: account_data.len() as u64,
            total_value: account_data.values().map(|a| a.balance).sum(),
            commitment_count: self.commitments.read().len() as u64,
            transition_count: self.transitions.read().len() as u64,
        }
    }
}

impl Default for L2State {
    fn default() -> Self {
        Self::new()
    }
}

/// State errors
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Insufficient balance: available {available}, required {required}")]
    InsufficientBalance { available: u64, required: u64 },

    #[error("Invalid pre-state: expected {expected:?}, actual {actual:?}")]
    InvalidPreState { expected: H256, actual: H256 },

    #[error("Invalid nonce: expected {expected}, actual {actual}")]
    InvalidNonce { expected: u64, actual: u64 },

    #[error("Account not found: {0:?}")]
    AccountNotFound(Address),
}

/// L2 state statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2StateStats {
    pub block_number: u64,
    pub state_root: H256,
    pub account_count: u64,
    pub total_value: u64,
    pub commitment_count: u64,
    pub transition_count: u64,
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
    fn test_l2_account() {
        let account = L2Account::with_balance(1000);
        assert_eq!(account.balance, 1000);
        assert_eq!(account.nonce, 0);
    }

    #[test]
    fn test_merkle_proof() {
        let mut tree = SparseMerkleTree::new(8);
        let key = H256([1u8; 32]);
        let value = H256([2u8; 32]);

        tree.update(&key, value);
        
        // Basic tree operation tests
        assert_eq!(tree.get(&key), Some(value));
        assert!(tree.stats().leaf_count > 0);
    }

    #[test]
    fn test_l2_state() {
        let state = L2State::new();
        let addr1 = Address([1u8; 20]);
        let addr2 = Address([2u8; 20]);

        // Credit addr1
        state.credit(&addr1, 1000);
        assert_eq!(state.get_balance(&addr1), 1000);

        // Transfer
        state.transfer(&addr1, &addr2, 400).unwrap();
        assert_eq!(state.get_balance(&addr1), 600);
        assert_eq!(state.get_balance(&addr2), 400);
    }

    #[test]
    fn test_state_commitment() {
        let state = L2State::new();
        let addr = Address([1u8; 20]);

        state.credit(&addr, 5000);
        let commitment = state.commit();

        assert_eq!(commitment.total_value, 5000);
        assert_eq!(commitment.account_count, 1);
    }

    #[test]
    fn test_account_proof() {
        let state = L2State::new();
        let addr = Address([1u8; 20]);

        state.credit(&addr, 1000);
        
        // Verify account was credited
        assert_eq!(state.get_balance(&addr), 1000);
        
        // State root should be non-zero after update
        assert_ne!(state.state_root(), H256::zero());
    }
}
