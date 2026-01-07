//! L1-L2 Bridge
//!
//! Two-way peg between PYRAX L1 (UTXO) and L2 (EVM)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use blake3::Hasher;

use super::types::{Address, B256, U256};
use crate::types::H256;

/// Bridge deposit (L1 -> L2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    /// Deposit ID (hash of L1 tx + output index)
    pub id: B256,
    /// L1 transaction hash
    pub l1_tx_hash: H256,
    /// L1 UTXO output index
    pub l1_output_index: u32,
    /// L2 recipient address
    pub l2_recipient: Address,
    /// Amount in base units
    pub amount: u64,
    /// L1 block number where deposit was made
    pub l1_block_number: u64,
    /// L1 block hash
    pub l1_block_hash: H256,
    /// Number of L1 confirmations required
    pub confirmations_required: u32,
    /// Current confirmation count
    pub confirmations: u32,
    /// Status
    pub status: DepositStatus,
    /// L2 mint transaction hash (if minted)
    pub l2_mint_tx: Option<B256>,
    /// Timestamp
    pub timestamp: u64,
}

/// Deposit status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepositStatus {
    /// Waiting for L1 confirmations
    Pending,
    /// Confirmed on L1, ready to mint on L2
    Confirmed,
    /// Minted on L2
    Minted,
    /// Failed or invalid
    Failed,
}

/// Bridge withdrawal (L2 -> L1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withdrawal {
    /// Withdrawal ID
    pub id: B256,
    /// L2 transaction hash
    pub l2_tx_hash: B256,
    /// L2 sender address
    pub l2_sender: Address,
    /// L1 recipient address (PYRAX address)
    pub l1_recipient: [u8; 25], // Base58 decoded PYRAX address
    /// Amount in base units
    pub amount: u64,
    /// L2 block number where withdrawal was initiated
    pub l2_block_number: u64,
    /// Challenge period end timestamp
    pub challenge_end: u64,
    /// Status
    pub status: WithdrawalStatus,
    /// Merkle proof for withdrawal (for L1 claim)
    pub merkle_proof: Option<Vec<B256>>,
    /// L1 claim transaction hash (if claimed)
    pub l1_claim_tx: Option<H256>,
    /// Timestamp
    pub timestamp: u64,
}

/// Withdrawal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WithdrawalStatus {
    /// Initiated on L2, in challenge period
    Pending,
    /// Challenge period passed, ready to claim on L1
    Ready,
    /// Claimed on L1
    Claimed,
    /// Challenged and invalidated
    Challenged,
    /// Failed
    Failed,
}

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Minimum deposit amount
    pub min_deposit: u64,
    /// Maximum deposit amount
    pub max_deposit: u64,
    /// Required L1 confirmations for deposits
    pub deposit_confirmations: u32,
    /// Challenge period duration in seconds
    pub challenge_period_secs: u64,
    /// Bridge fee percentage (in basis points, 100 = 1%)
    pub fee_bps: u16,
    /// L1 bridge contract address (lock address)
    pub l1_bridge_address: [u8; 25],
    /// L2 bridge contract address
    pub l2_bridge_address: Address,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            min_deposit: 100_000, // 0.001 PYRAX
            max_deposit: 1_000_000_000_000, // 10,000 PYRAX
            deposit_confirmations: 10,
            challenge_period_secs: 7 * 24 * 60 * 60, // 7 days
            fee_bps: 10, // 0.1%
            l1_bridge_address: [0u8; 25],
            l2_bridge_address: [0u8; 20],
        }
    }
}

/// L1-L2 Bridge
pub struct Bridge {
    config: BridgeConfig,
    /// Pending deposits
    deposits: Arc<RwLock<HashMap<B256, Deposit>>>,
    /// Pending withdrawals
    withdrawals: Arc<RwLock<HashMap<B256, Withdrawal>>>,
    /// Total deposited (L1 locked)
    total_deposited: Arc<RwLock<u64>>,
    /// Total withdrawn (L1 unlocked)
    total_withdrawn: Arc<RwLock<u64>>,
    /// Withdrawal merkle tree root (updated per L2 block)
    withdrawal_root: Arc<RwLock<B256>>,
}

impl Bridge {
    /// Create a new bridge
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            config,
            deposits: Arc::new(RwLock::new(HashMap::new())),
            withdrawals: Arc::new(RwLock::new(HashMap::new())),
            total_deposited: Arc::new(RwLock::new(0)),
            total_withdrawn: Arc::new(RwLock::new(0)),
            withdrawal_root: Arc::new(RwLock::new([0u8; 32])),
        }
    }

    /// Create a deposit entry when L1 lock is detected
    pub fn create_deposit(
        &self,
        l1_tx_hash: H256,
        l1_output_index: u32,
        l2_recipient: Address,
        amount: u64,
        l1_block_number: u64,
        l1_block_hash: H256,
    ) -> Result<B256, BridgeError> {
        // Validate amount
        if amount < self.config.min_deposit {
            return Err(BridgeError::AmountTooSmall(self.config.min_deposit));
        }
        if amount > self.config.max_deposit {
            return Err(BridgeError::AmountTooLarge(self.config.max_deposit));
        }

        // Calculate deposit ID
        let deposit_id = self.compute_deposit_id(&l1_tx_hash, l1_output_index);

        // Check for duplicate
        if self.deposits.read().contains_key(&deposit_id) {
            return Err(BridgeError::DuplicateDeposit(deposit_id));
        }

        let deposit = Deposit {
            id: deposit_id,
            l1_tx_hash,
            l1_output_index,
            l2_recipient,
            amount,
            l1_block_number,
            l1_block_hash,
            confirmations_required: self.config.deposit_confirmations,
            confirmations: 0,
            status: DepositStatus::Pending,
            l2_mint_tx: None,
            timestamp: current_timestamp(),
        };

        self.deposits.write().insert(deposit_id, deposit);

        Ok(deposit_id)
    }

    /// Update deposit confirmations
    pub fn update_deposit_confirmations(
        &self,
        deposit_id: &B256,
        current_l1_height: u64,
    ) -> Result<DepositStatus, BridgeError> {
        let mut deposits = self.deposits.write();
        let deposit = deposits.get_mut(deposit_id)
            .ok_or(BridgeError::DepositNotFound(*deposit_id))?;

        if deposit.status != DepositStatus::Pending {
            return Ok(deposit.status);
        }

        let confirmations = current_l1_height.saturating_sub(deposit.l1_block_number) as u32;
        deposit.confirmations = confirmations;

        if confirmations >= deposit.confirmations_required {
            deposit.status = DepositStatus::Confirmed;
        }

        Ok(deposit.status)
    }

    /// Mark deposit as minted on L2
    pub fn complete_deposit(
        &self,
        deposit_id: &B256,
        l2_mint_tx: B256,
    ) -> Result<(), BridgeError> {
        let mut deposits = self.deposits.write();
        let deposit = deposits.get_mut(deposit_id)
            .ok_or(BridgeError::DepositNotFound(*deposit_id))?;

        if deposit.status != DepositStatus::Confirmed {
            return Err(BridgeError::InvalidDepositStatus(deposit.status));
        }

        deposit.status = DepositStatus::Minted;
        deposit.l2_mint_tx = Some(l2_mint_tx);

        // Update total deposited
        *self.total_deposited.write() += deposit.amount;

        Ok(())
    }

    /// Initiate a withdrawal (L2 -> L1)
    pub fn initiate_withdrawal(
        &self,
        l2_tx_hash: B256,
        l2_sender: Address,
        l1_recipient: [u8; 25],
        amount: u64,
        l2_block_number: u64,
    ) -> Result<B256, BridgeError> {
        // Validate amount
        if amount < self.config.min_deposit {
            return Err(BridgeError::AmountTooSmall(self.config.min_deposit));
        }

        // Calculate fee
        let fee = (amount as u128 * self.config.fee_bps as u128 / 10000) as u64;
        let net_amount = amount.saturating_sub(fee);

        // Calculate withdrawal ID
        let withdrawal_id = self.compute_withdrawal_id(&l2_tx_hash);

        // Check for duplicate
        if self.withdrawals.read().contains_key(&withdrawal_id) {
            return Err(BridgeError::DuplicateWithdrawal(withdrawal_id));
        }

        let timestamp = current_timestamp();
        let withdrawal = Withdrawal {
            id: withdrawal_id,
            l2_tx_hash,
            l2_sender,
            l1_recipient,
            amount: net_amount,
            l2_block_number,
            challenge_end: timestamp + self.config.challenge_period_secs,
            status: WithdrawalStatus::Pending,
            merkle_proof: None,
            l1_claim_tx: None,
            timestamp,
        };

        self.withdrawals.write().insert(withdrawal_id, withdrawal);

        // Update withdrawal merkle root
        self.update_withdrawal_root();

        Ok(withdrawal_id)
    }

    /// Check if withdrawal is ready to claim
    pub fn check_withdrawal_ready(&self, withdrawal_id: &B256) -> Result<bool, BridgeError> {
        // First check status and challenge end without holding lock during proof generation
        let should_update = {
            let withdrawals = self.withdrawals.read();
            let withdrawal = withdrawals.get(withdrawal_id)
                .ok_or(BridgeError::WithdrawalNotFound(*withdrawal_id))?;

            if withdrawal.status != WithdrawalStatus::Pending {
                return Ok(withdrawal.status == WithdrawalStatus::Ready);
            }

            let now = current_timestamp();
            now >= withdrawal.challenge_end
        };

        if should_update {
            // Generate proof without holding lock
            let proof = self.generate_merkle_proof(withdrawal_id);
            
            // Now update with write lock
            let mut withdrawals = self.withdrawals.write();
            if let Some(withdrawal) = withdrawals.get_mut(withdrawal_id) {
                withdrawal.status = WithdrawalStatus::Ready;
                withdrawal.merkle_proof = Some(proof);
            }
            return Ok(true);
        }

        Ok(false)
    }

    /// Complete withdrawal claim on L1
    pub fn complete_withdrawal(
        &self,
        withdrawal_id: &B256,
        l1_claim_tx: H256,
    ) -> Result<(), BridgeError> {
        let mut withdrawals = self.withdrawals.write();
        let withdrawal = withdrawals.get_mut(withdrawal_id)
            .ok_or(BridgeError::WithdrawalNotFound(*withdrawal_id))?;

        if withdrawal.status != WithdrawalStatus::Ready {
            return Err(BridgeError::InvalidWithdrawalStatus(withdrawal.status));
        }

        withdrawal.status = WithdrawalStatus::Claimed;
        withdrawal.l1_claim_tx = Some(l1_claim_tx);

        // Update total withdrawn
        *self.total_withdrawn.write() += withdrawal.amount;

        Ok(())
    }

    /// Challenge a withdrawal (fraud proof)
    pub fn challenge_withdrawal(
        &self,
        withdrawal_id: &B256,
        _proof: Vec<u8>,
    ) -> Result<(), BridgeError> {
        let mut withdrawals = self.withdrawals.write();
        let withdrawal = withdrawals.get_mut(withdrawal_id)
            .ok_or(BridgeError::WithdrawalNotFound(*withdrawal_id))?;

        if withdrawal.status != WithdrawalStatus::Pending {
            return Err(BridgeError::InvalidWithdrawalStatus(withdrawal.status));
        }

        // Verify fraud proof
        // For production, implement actual fraud proof verification

        withdrawal.status = WithdrawalStatus::Challenged;

        Ok(())
    }

    /// Get deposit by ID
    pub fn get_deposit(&self, deposit_id: &B256) -> Option<Deposit> {
        self.deposits.read().get(deposit_id).cloned()
    }

    /// Get withdrawal by ID
    pub fn get_withdrawal(&self, withdrawal_id: &B256) -> Option<Withdrawal> {
        self.withdrawals.read().get(withdrawal_id).cloned()
    }

    /// Get all pending deposits
    pub fn get_pending_deposits(&self) -> Vec<Deposit> {
        self.deposits.read()
            .values()
            .filter(|d| d.status == DepositStatus::Pending || d.status == DepositStatus::Confirmed)
            .cloned()
            .collect()
    }

    /// Get all pending withdrawals
    pub fn get_pending_withdrawals(&self) -> Vec<Withdrawal> {
        self.withdrawals.read()
            .values()
            .filter(|w| w.status == WithdrawalStatus::Pending || w.status == WithdrawalStatus::Ready)
            .cloned()
            .collect()
    }

    /// Get bridge statistics
    pub fn get_stats(&self) -> BridgeStats {
        let deposits = self.deposits.read();
        let withdrawals = self.withdrawals.read();

        BridgeStats {
            total_deposits: deposits.len() as u64,
            pending_deposits: deposits.values().filter(|d| d.status == DepositStatus::Pending).count() as u64,
            total_withdrawals: withdrawals.len() as u64,
            pending_withdrawals: withdrawals.values().filter(|w| w.status == WithdrawalStatus::Pending).count() as u64,
            total_deposited: *self.total_deposited.read(),
            total_withdrawn: *self.total_withdrawn.read(),
            net_locked: self.total_deposited.read().saturating_sub(*self.total_withdrawn.read()),
        }
    }

    /// Get current withdrawal root
    pub fn withdrawal_root(&self) -> B256 {
        *self.withdrawal_root.read()
    }

    /// Get bridge config
    pub fn config(&self) -> &BridgeConfig {
        &self.config
    }

    // Internal helpers

    fn compute_deposit_id(&self, l1_tx_hash: &H256, output_index: u32) -> B256 {
        let mut hasher = Hasher::new();
        hasher.update(&l1_tx_hash.0);
        hasher.update(&output_index.to_be_bytes());
        hasher.update(b"deposit");
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(result.as_bytes());
        id
    }

    fn compute_withdrawal_id(&self, l2_tx_hash: &B256) -> B256 {
        let mut hasher = Hasher::new();
        hasher.update(l2_tx_hash);
        hasher.update(b"withdrawal");
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(result.as_bytes());
        id
    }

    fn update_withdrawal_root(&self) {
        let withdrawals = self.withdrawals.read();
        let mut pending: Vec<_> = withdrawals.values()
            .filter(|w| w.status == WithdrawalStatus::Pending || w.status == WithdrawalStatus::Ready)
            .collect();
        pending.sort_by_key(|w| w.timestamp);

        let mut hasher = Hasher::new();
        for w in pending {
            hasher.update(&w.id);
            hasher.update(&w.amount.to_be_bytes());
            hasher.update(&w.l1_recipient);
        }
        let result = hasher.finalize();
        let mut root = [0u8; 32];
        root.copy_from_slice(result.as_bytes());
        *self.withdrawal_root.write() = root;
    }

    fn generate_merkle_proof(&self, withdrawal_id: &B256) -> Vec<B256> {
        // Simplified proof generation - in production, build actual Merkle tree
        let withdrawals = self.withdrawals.read();
        let pending: Vec<_> = withdrawals.values()
            .filter(|w| w.status == WithdrawalStatus::Pending || w.status == WithdrawalStatus::Ready)
            .collect();

        let mut proof = Vec::new();
        for w in pending {
            if &w.id != withdrawal_id {
                proof.push(w.id);
            }
        }
        proof
    }
}

impl Default for Bridge {
    fn default() -> Self {
        Self::new(BridgeConfig::default())
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    pub total_deposits: u64,
    pub pending_deposits: u64,
    pub total_withdrawals: u64,
    pub pending_withdrawals: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub net_locked: u64,
}

/// Bridge errors
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Amount too small: minimum is {0}")]
    AmountTooSmall(u64),

    #[error("Amount too large: maximum is {0}")]
    AmountTooLarge(u64),

    #[error("Duplicate deposit: {0:?}")]
    DuplicateDeposit(B256),

    #[error("Deposit not found: {0:?}")]
    DepositNotFound(B256),

    #[error("Invalid deposit status: {0:?}")]
    InvalidDepositStatus(DepositStatus),

    #[error("Duplicate withdrawal: {0:?}")]
    DuplicateWithdrawal(B256),

    #[error("Withdrawal not found: {0:?}")]
    WithdrawalNotFound(B256),

    #[error("Invalid withdrawal status: {0:?}")]
    InvalidWithdrawalStatus(WithdrawalStatus),

    #[error("Challenge period not ended")]
    ChallengePeriodActive,

    #[error("Invalid proof")]
    InvalidProof,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = Bridge::default();
        let stats = bridge.get_stats();
        assert_eq!(stats.total_deposits, 0);
        assert_eq!(stats.total_withdrawals, 0);
    }

    #[test]
    fn test_deposit_flow() {
        let bridge = Bridge::default();
        
        let l1_tx_hash = H256([1u8; 32]);
        let l2_recipient = [2u8; 20];
        let amount = 1_000_000;
        let l1_block = 100;
        let l1_block_hash = H256([3u8; 32]);

        // Create deposit
        let deposit_id = bridge.create_deposit(
            l1_tx_hash,
            0,
            l2_recipient,
            amount,
            l1_block,
            l1_block_hash,
        ).unwrap();

        // Check pending
        let deposit = bridge.get_deposit(&deposit_id).unwrap();
        assert_eq!(deposit.status, DepositStatus::Pending);

        // Update confirmations
        bridge.update_deposit_confirmations(&deposit_id, l1_block + 10).unwrap();
        let deposit = bridge.get_deposit(&deposit_id).unwrap();
        assert_eq!(deposit.status, DepositStatus::Confirmed);

        // Complete deposit
        let l2_mint_tx = [4u8; 32];
        bridge.complete_deposit(&deposit_id, l2_mint_tx).unwrap();
        let deposit = bridge.get_deposit(&deposit_id).unwrap();
        assert_eq!(deposit.status, DepositStatus::Minted);
    }

    #[test]
    fn test_withdrawal_flow() {
        let bridge = Bridge::new(BridgeConfig {
            challenge_period_secs: 0, // Instant for testing
            ..Default::default()
        });

        let l2_tx_hash = [5u8; 32];
        let l2_sender = [6u8; 20];
        let l1_recipient = [7u8; 25];
        let amount = 500_000;
        let l2_block = 200;

        // Initiate withdrawal
        let withdrawal_id = bridge.initiate_withdrawal(
            l2_tx_hash,
            l2_sender,
            l1_recipient,
            amount,
            l2_block,
        ).unwrap();

        // Check ready immediately (no challenge period)
        assert!(bridge.check_withdrawal_ready(&withdrawal_id).unwrap());

        // Complete withdrawal
        let l1_claim_tx = H256([8u8; 32]);
        bridge.complete_withdrawal(&withdrawal_id, l1_claim_tx).unwrap();
        let withdrawal = bridge.get_withdrawal(&withdrawal_id).unwrap();
        assert_eq!(withdrawal.status, WithdrawalStatus::Claimed);
    }

    #[test]
    fn test_amount_validation() {
        let bridge = Bridge::default();
        
        // Too small
        let result = bridge.create_deposit(
            H256([0u8; 32]),
            0,
            [0u8; 20],
            10, // Below minimum
            0,
            H256([0u8; 32]),
        );
        assert!(matches!(result, Err(BridgeError::AmountTooSmall(_))));

        // Too large
        let result = bridge.create_deposit(
            H256([0u8; 32]),
            0,
            [0u8; 20],
            10_000_000_000_000, // Above maximum
            0,
            H256([0u8; 32]),
        );
        assert!(matches!(result, Err(BridgeError::AmountTooLarge(_))));
    }
}
