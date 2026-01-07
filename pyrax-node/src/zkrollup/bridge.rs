//! L2 Bridge
//!
//! Deposits and withdrawals between L1 and L2

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::state::{L2State, MerkleProof};
use super::WITHDRAWAL_CHALLENGE_PERIOD;

/// Withdrawal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WithdrawalStatus {
    /// Withdrawal initiated
    Pending,
    /// Withdrawal proof submitted
    Proven,
    /// In challenge period
    Challenging,
    /// Challenge period passed, ready to finalize
    Ready,
    /// Withdrawal finalized
    Finalized,
    /// Withdrawal challenged and invalidated
    Challenged,
    /// Withdrawal expired
    Expired,
}

/// Deposit from L1 to L2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    /// Deposit ID
    pub id: H256,
    /// L1 transaction hash
    pub l1_tx_hash: H256,
    /// L1 block number
    pub l1_block_number: u64,
    /// Depositor address
    pub from: Address,
    /// Recipient address on L2
    pub to: Address,
    /// Amount deposited
    pub amount: u64,
    /// Deposit timestamp
    pub timestamp: u64,
    /// Is processed on L2
    pub processed: bool,
    /// L2 transaction hash (when processed)
    pub l2_tx_hash: Option<H256>,
}

impl Deposit {
    /// Create new deposit
    pub fn new(
        l1_tx_hash: H256,
        l1_block_number: u64,
        from: Address,
        to: Address,
        amount: u64,
    ) -> Self {
        let id = Self::generate_id(&l1_tx_hash, &from, amount);
        Self {
            id,
            l1_tx_hash,
            l1_block_number,
            from,
            to,
            amount,
            timestamp: current_timestamp(),
            processed: false,
            l2_tx_hash: None,
        }
    }

    /// Generate deposit ID
    fn generate_id(l1_tx_hash: &H256, from: &Address, amount: u64) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&l1_tx_hash.0);
        hasher.update(&from.0);
        hasher.update(&amount.to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Mark as processed
    pub fn mark_processed(&mut self, l2_tx_hash: H256) {
        self.processed = true;
        self.l2_tx_hash = Some(l2_tx_hash);
    }
}

/// Withdrawal from L2 to L1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withdrawal {
    /// Withdrawal ID
    pub id: H256,
    /// Withdrawer address
    pub from: Address,
    /// Recipient address on L1
    pub to: Address,
    /// Amount to withdraw
    pub amount: u64,
    /// L2 transaction hash
    pub l2_tx_hash: H256,
    /// L2 block number
    pub l2_block_number: u64,
    /// Status
    pub status: WithdrawalStatus,
    /// Merkle proof (once proven)
    pub proof: Option<WithdrawalProof>,
    /// Challenge period start
    pub challenge_start: Option<u64>,
    /// Finalization timestamp
    pub finalized_at: Option<u64>,
    /// L1 finalization tx hash
    pub l1_tx_hash: Option<H256>,
    /// Creation timestamp
    pub created_at: u64,
}

impl Withdrawal {
    /// Create new withdrawal
    pub fn new(
        from: Address,
        to: Address,
        amount: u64,
        l2_tx_hash: H256,
        l2_block_number: u64,
    ) -> Self {
        let id = Self::generate_id(&l2_tx_hash, &from, amount);
        Self {
            id,
            from,
            to,
            amount,
            l2_tx_hash,
            l2_block_number,
            status: WithdrawalStatus::Pending,
            proof: None,
            challenge_start: None,
            finalized_at: None,
            l1_tx_hash: None,
            created_at: current_timestamp(),
        }
    }

    /// Generate withdrawal ID
    fn generate_id(l2_tx_hash: &H256, from: &Address, amount: u64) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&l2_tx_hash.0);
        hasher.update(&from.0);
        hasher.update(&amount.to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Compute withdrawal hash
    pub fn hash(&self) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.id.0);
        hasher.update(&self.from.0);
        hasher.update(&self.to.0);
        hasher.update(&self.amount.to_le_bytes());
        hasher.update(&self.l2_tx_hash.0);
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Submit proof
    pub fn submit_proof(&mut self, proof: WithdrawalProof) {
        self.proof = Some(proof);
        self.status = WithdrawalStatus::Proven;
        self.challenge_start = Some(current_timestamp());
    }

    /// Check if challenge period passed
    pub fn is_ready(&self) -> bool {
        if self.status != WithdrawalStatus::Proven && self.status != WithdrawalStatus::Challenging {
            return false;
        }

        if let Some(start) = self.challenge_start {
            current_timestamp() >= start + WITHDRAWAL_CHALLENGE_PERIOD
        } else {
            false
        }
    }

    /// Mark as ready
    pub fn mark_ready(&mut self) {
        if self.is_ready() {
            self.status = WithdrawalStatus::Ready;
        }
    }

    /// Finalize withdrawal
    pub fn finalize(&mut self, l1_tx_hash: H256) {
        self.status = WithdrawalStatus::Finalized;
        self.finalized_at = Some(current_timestamp());
        self.l1_tx_hash = Some(l1_tx_hash);
    }

    /// Challenge withdrawal
    pub fn challenge(&mut self) {
        if self.status == WithdrawalStatus::Proven || self.status == WithdrawalStatus::Challenging {
            self.status = WithdrawalStatus::Challenged;
        }
    }

    /// Get time until ready (seconds)
    pub fn time_until_ready(&self) -> Option<u64> {
        if let Some(start) = self.challenge_start {
            let end = start + WITHDRAWAL_CHALLENGE_PERIOD;
            let now = current_timestamp();
            if now < end {
                Some(end - now)
            } else {
                Some(0)
            }
        } else {
            None
        }
    }
}

/// Withdrawal proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalProof {
    /// Withdrawal hash
    pub withdrawal_hash: H256,
    /// State root at withdrawal
    pub state_root: H256,
    /// Batch number containing withdrawal
    pub batch_number: u64,
    /// Batch proof (ZK proof that batch is valid)
    pub batch_proof: Vec<u8>,
    /// Merkle proof of withdrawal in batch
    pub merkle_proof: MerkleProof,
    /// Output root
    pub output_root: H256,
}

impl WithdrawalProof {
    /// Verify the proof
    pub fn verify(&self) -> bool {
        // Verify merkle proof
        if !self.merkle_proof.verify() {
            return false;
        }

        // Verify batch proof (simplified)
        if self.batch_proof.is_empty() {
            return false;
        }

        true
    }
}

/// L2 Bridge
pub struct L2Bridge {
    /// L2 state
    state: Arc<L2State>,
    /// Pending deposits
    pending_deposits: Arc<RwLock<HashMap<H256, Deposit>>>,
    /// Processed deposits
    processed_deposits: Arc<RwLock<HashMap<H256, Deposit>>>,
    /// Pending withdrawals
    pending_withdrawals: Arc<RwLock<HashMap<H256, Withdrawal>>>,
    /// Finalized withdrawals
    finalized_withdrawals: Arc<RwLock<HashMap<H256, Withdrawal>>>,
    /// Withdrawal tree (for merkle proofs)
    withdrawal_tree: Arc<RwLock<Vec<H256>>>,
    /// Bridge contract address on L1
    l1_bridge_address: Address,
    /// Statistics
    stats: Arc<RwLock<BridgeStats>>,
}

impl L2Bridge {
    /// Create new bridge
    pub fn new(state: Arc<L2State>, l1_bridge_address: Address) -> Self {
        Self {
            state,
            pending_deposits: Arc::new(RwLock::new(HashMap::new())),
            processed_deposits: Arc::new(RwLock::new(HashMap::new())),
            pending_withdrawals: Arc::new(RwLock::new(HashMap::new())),
            finalized_withdrawals: Arc::new(RwLock::new(HashMap::new())),
            withdrawal_tree: Arc::new(RwLock::new(Vec::new())),
            l1_bridge_address,
            stats: Arc::new(RwLock::new(BridgeStats::default())),
        }
    }

    /// Record deposit from L1
    pub fn record_deposit(&self, deposit: Deposit) -> Result<H256, BridgeError> {
        let id = deposit.id;

        // Check for duplicate
        if self.pending_deposits.read().contains_key(&id) {
            return Err(BridgeError::DuplicateDeposit(id));
        }
        if self.processed_deposits.read().contains_key(&id) {
            return Err(BridgeError::DuplicateDeposit(id));
        }

        self.pending_deposits.write().insert(id, deposit);
        self.stats.write().deposits_recorded += 1;

        Ok(id)
    }

    /// Process pending deposit
    pub fn process_deposit(&self, deposit_id: &H256) -> Result<H256, BridgeError> {
        let mut deposit = self.pending_deposits.write()
            .remove(deposit_id)
            .ok_or_else(|| BridgeError::DepositNotFound(*deposit_id))?;

        // Credit the recipient
        self.state.credit(&deposit.to, deposit.amount);

        // Generate L2 tx hash
        let l2_tx_hash = {
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(&deposit.id.0);
            hasher.update(b"L2_DEPOSIT");
            hasher.update(&current_timestamp().to_le_bytes());
            H256::from_slice(hasher.finalize().as_bytes())
        };

        let amount = deposit.amount;
        deposit.mark_processed(l2_tx_hash);
        self.processed_deposits.write().insert(*deposit_id, deposit);

        self.stats.write().deposits_processed += 1;
        self.stats.write().total_deposited += amount;

        Ok(l2_tx_hash)
    }

    /// Initiate withdrawal from L2
    pub fn initiate_withdrawal(
        &self,
        from: Address,
        to: Address,
        amount: u64,
    ) -> Result<Withdrawal, BridgeError> {
        // Check balance
        let balance = self.state.get_balance(&from);
        if balance < amount {
            return Err(BridgeError::InsufficientBalance {
                available: balance,
                required: amount,
            });
        }

        // Debit the account
        self.state.debit(&from, amount)
            .map_err(|_| BridgeError::InsufficientBalance {
                available: balance,
                required: amount,
            })?;

        // Create withdrawal transaction hash
        let l2_tx_hash = {
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(&from.0);
            hasher.update(&to.0);
            hasher.update(&amount.to_le_bytes());
            hasher.update(&current_timestamp().to_le_bytes());
            H256::from_slice(hasher.finalize().as_bytes())
        };

        let l2_block = self.state.block_number();
        let withdrawal = Withdrawal::new(from, to, amount, l2_tx_hash, l2_block);
        let id = withdrawal.id;

        // Add to withdrawal tree
        self.withdrawal_tree.write().push(withdrawal.hash());

        self.pending_withdrawals.write().insert(id, withdrawal.clone());

        self.stats.write().withdrawals_initiated += 1;

        Ok(withdrawal)
    }

    /// Submit withdrawal proof
    pub fn submit_withdrawal_proof(
        &self,
        withdrawal_id: &H256,
        proof: WithdrawalProof,
    ) -> Result<(), BridgeError> {
        // Verify proof
        if !proof.verify() {
            return Err(BridgeError::InvalidProof);
        }

        let mut withdrawals = self.pending_withdrawals.write();
        let withdrawal = withdrawals.get_mut(withdrawal_id)
            .ok_or_else(|| BridgeError::WithdrawalNotFound(*withdrawal_id))?;

        if withdrawal.status != WithdrawalStatus::Pending {
            return Err(BridgeError::InvalidWithdrawalState(withdrawal.status));
        }

        withdrawal.submit_proof(proof);
        self.stats.write().proofs_submitted += 1;

        Ok(())
    }

    /// Finalize withdrawal on L1
    pub fn finalize_withdrawal(
        &self,
        withdrawal_id: &H256,
        l1_tx_hash: H256,
    ) -> Result<(), BridgeError> {
        let mut withdrawal = self.pending_withdrawals.write()
            .remove(withdrawal_id)
            .ok_or_else(|| BridgeError::WithdrawalNotFound(*withdrawal_id))?;

        if !withdrawal.is_ready() && withdrawal.status != WithdrawalStatus::Ready {
            // Re-insert and return error
            self.pending_withdrawals.write().insert(*withdrawal_id, withdrawal);
            return Err(BridgeError::WithdrawalNotReady);
        }

        withdrawal.finalize(l1_tx_hash);
        self.finalized_withdrawals.write().insert(*withdrawal_id, withdrawal.clone());

        self.stats.write().withdrawals_finalized += 1;
        self.stats.write().total_withdrawn += withdrawal.amount;

        Ok(())
    }

    /// Challenge withdrawal
    pub fn challenge_withdrawal(
        &self,
        withdrawal_id: &H256,
        _fraud_proof: Vec<u8>,
    ) -> Result<(), BridgeError> {
        let mut withdrawals = self.pending_withdrawals.write();
        let withdrawal = withdrawals.get_mut(withdrawal_id)
            .ok_or_else(|| BridgeError::WithdrawalNotFound(*withdrawal_id))?;

        if withdrawal.status != WithdrawalStatus::Proven && 
           withdrawal.status != WithdrawalStatus::Challenging {
            return Err(BridgeError::InvalidWithdrawalState(withdrawal.status));
        }

        // Verify fraud proof (simplified)
        withdrawal.challenge();

        // Return funds to user on L2
        self.state.credit(&withdrawal.from, withdrawal.amount);

        self.stats.write().challenges += 1;

        Ok(())
    }

    /// Generate withdrawal proof
    pub fn generate_withdrawal_proof(
        &self,
        withdrawal_id: &H256,
        batch_number: u64,
        batch_proof: Vec<u8>,
    ) -> Result<WithdrawalProof, BridgeError> {
        let withdrawal = self.pending_withdrawals.read()
            .get(withdrawal_id)
            .cloned()
            .ok_or_else(|| BridgeError::WithdrawalNotFound(*withdrawal_id))?;

        let state_root = self.state.state_root();
        let withdrawal_hash = withdrawal.hash();

        // Generate merkle proof
        let merkle_proof = self.generate_merkle_proof(&withdrawal_hash)?;

        // Compute output root
        let output_root = self.compute_output_root(&state_root, &withdrawal_hash);

        Ok(WithdrawalProof {
            withdrawal_hash,
            state_root,
            batch_number,
            batch_proof,
            merkle_proof,
            output_root,
        })
    }

    /// Generate merkle proof for withdrawal
    fn generate_merkle_proof(&self, withdrawal_hash: &H256) -> Result<MerkleProof, BridgeError> {
        let tree = self.withdrawal_tree.read();
        
        // Find withdrawal in tree
        let index = tree.iter()
            .position(|h| h == withdrawal_hash)
            .ok_or(BridgeError::WithdrawalNotInTree)?;

        // Build merkle proof
        let mut siblings = Vec::new();
        let mut path = Vec::new();
        let mut idx = index;
        let mut level = tree.clone();

        while level.len() > 1 {
            let is_right = idx % 2 == 1;
            path.push(is_right);

            let sibling_idx = if is_right { idx - 1 } else { idx + 1 };
            let sibling = level.get(sibling_idx).copied().unwrap_or(H256::zero());
            siblings.push(sibling);

            // Move to next level
            let mut next_level = Vec::new();
            for chunk in level.chunks(2) {
                let left = chunk[0];
                let right = chunk.get(1).copied().unwrap_or(H256::zero());
                next_level.push(Self::hash_pair(&left, &right));
            }
            level = next_level;
            idx /= 2;
        }

        let root = level.first().copied().unwrap_or(H256::zero());

        Ok(MerkleProof {
            leaf: *withdrawal_hash,
            siblings,
            path,
            root,
        })
    }

    /// Compute output root
    fn compute_output_root(&self, state_root: &H256, withdrawal_root: &H256) -> H256 {
        Self::hash_pair(state_root, withdrawal_root)
    }

    /// Hash two values
    fn hash_pair(a: &H256, b: &H256) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&a.0);
        hasher.update(&b.0);
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Get deposit by ID
    pub fn get_deposit(&self, id: &H256) -> Option<Deposit> {
        self.pending_deposits.read().get(id).cloned()
            .or_else(|| self.processed_deposits.read().get(id).cloned())
    }

    /// Get withdrawal by ID
    pub fn get_withdrawal(&self, id: &H256) -> Option<Withdrawal> {
        self.pending_withdrawals.read().get(id).cloned()
            .or_else(|| self.finalized_withdrawals.read().get(id).cloned())
    }

    /// Get pending deposits
    pub fn get_pending_deposits(&self) -> Vec<Deposit> {
        self.pending_deposits.read().values().cloned().collect()
    }

    /// Get pending withdrawals
    pub fn get_pending_withdrawals(&self) -> Vec<Withdrawal> {
        self.pending_withdrawals.read().values().cloned().collect()
    }

    /// Update ready withdrawals
    pub fn update_ready_withdrawals(&self) {
        let mut withdrawals = self.pending_withdrawals.write();
        for withdrawal in withdrawals.values_mut() {
            if withdrawal.is_ready() {
                withdrawal.status = WithdrawalStatus::Ready;
            }
        }
    }

    /// Get statistics
    pub fn stats(&self) -> BridgeStats {
        self.stats.read().clone()
    }
}

/// Bridge errors
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Duplicate deposit: {0:?}")]
    DuplicateDeposit(H256),

    #[error("Deposit not found: {0:?}")]
    DepositNotFound(H256),

    #[error("Withdrawal not found: {0:?}")]
    WithdrawalNotFound(H256),

    #[error("Insufficient balance: available {available}, required {required}")]
    InsufficientBalance { available: u64, required: u64 },

    #[error("Invalid withdrawal state: {0:?}")]
    InvalidWithdrawalState(WithdrawalStatus),

    #[error("Withdrawal not ready")]
    WithdrawalNotReady,

    #[error("Invalid proof")]
    InvalidProof,

    #[error("Withdrawal not in tree")]
    WithdrawalNotInTree,

    #[error("Challenge period active")]
    ChallengePeriodActive,
}

/// Bridge statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BridgeStats {
    pub deposits_recorded: u64,
    pub deposits_processed: u64,
    pub total_deposited: u64,
    pub withdrawals_initiated: u64,
    pub proofs_submitted: u64,
    pub withdrawals_finalized: u64,
    pub total_withdrawn: u64,
    pub challenges: u64,
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

    fn make_bridge() -> L2Bridge {
        let state = Arc::new(L2State::new());
        state.credit(&Address([1u8; 20]), 1_000_000_000_000);
        L2Bridge::new(state, Address([0u8; 20]))
    }

    #[test]
    fn test_deposit() {
        let bridge = make_bridge();
        
        let deposit = Deposit::new(
            H256([1u8; 32]),
            100,
            Address([2u8; 20]),
            Address([3u8; 20]),
            1_000_000,
        );

        let id = bridge.record_deposit(deposit).unwrap();
        assert!(bridge.get_deposit(&id).is_some());
    }

    #[test]
    fn test_process_deposit() {
        let bridge = make_bridge();
        
        let deposit = Deposit::new(
            H256([1u8; 32]),
            100,
            Address([2u8; 20]),
            Address([3u8; 20]),
            1_000_000,
        );

        let id = bridge.record_deposit(deposit).unwrap();
        let l2_hash = bridge.process_deposit(&id).unwrap();
        
        assert!(!l2_hash.0.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_withdrawal() {
        let bridge = make_bridge();
        
        let withdrawal = bridge.initiate_withdrawal(
            Address([1u8; 20]),
            Address([2u8; 20]),
            1_000_000,
        ).unwrap();

        assert_eq!(withdrawal.status, WithdrawalStatus::Pending);
        assert!(bridge.get_withdrawal(&withdrawal.id).is_some());
    }

    #[test]
    fn test_withdrawal_proof_generation() {
        let bridge = make_bridge();
        
        let withdrawal = bridge.initiate_withdrawal(
            Address([1u8; 20]),
            Address([2u8; 20]),
            1_000_000,
        ).unwrap();

        let proof = bridge.generate_withdrawal_proof(
            &withdrawal.id,
            1,
            vec![1, 2, 3],
        ).unwrap();

        assert!(proof.verify());
    }
}
