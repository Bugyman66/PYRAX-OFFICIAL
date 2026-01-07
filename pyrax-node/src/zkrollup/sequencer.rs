//! Decentralized Sequencer
//!
//! Transaction ordering and batch production with censorship resistance

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::batch::{Batch, BatchBuilder, L2Transaction, BatchError};
use super::state::L2State;
use super::{MIN_SEQUENCER_STAKE, L2_BLOCK_TIME_SECS, MAX_BATCH_SIZE};

/// Sequencer operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequencerMode {
    /// Single sequencer (centralized, faster)
    Single,
    /// Rotating sequencer set
    Rotating,
    /// Decentralized with leader election
    Decentralized,
    /// Based Sequencing (L1-driven ordering)
    Based,
}

/// Sequencer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequencerStatus {
    /// Sequencer is active
    Active,
    /// Sequencer is syncing
    Syncing,
    /// Sequencer is paused
    Paused,
    /// Sequencer is slashed
    Slashed,
    /// Sequencer is unbonding
    Unbonding,
}

/// Sequencer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerConfig {
    /// Operation mode
    pub mode: SequencerMode,
    /// Block time (seconds)
    pub block_time_secs: u64,
    /// Maximum transactions per batch
    pub max_batch_size: usize,
    /// Minimum stake required
    pub min_stake: u64,
    /// Force inclusion delay (L1 blocks)
    pub force_inclusion_delay: u64,
    /// Sequencer rotation period (L2 blocks)
    pub rotation_period: u64,
    /// Enable priority ordering
    pub priority_ordering: bool,
    /// Maximum gas per batch
    pub max_batch_gas: u64,
}

impl Default for SequencerConfig {
    fn default() -> Self {
        Self {
            mode: SequencerMode::Decentralized,
            block_time_secs: L2_BLOCK_TIME_SECS,
            max_batch_size: MAX_BATCH_SIZE,
            min_stake: MIN_SEQUENCER_STAKE,
            force_inclusion_delay: 100, // ~20 minutes on L1
            rotation_period: 1000,
            priority_ordering: true,
            max_batch_gas: 30_000_000,
        }
    }
}

/// Sequencer node info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerInfo {
    /// Sequencer address
    pub address: Address,
    /// Staked amount
    pub stake: u64,
    /// Status
    pub status: SequencerStatus,
    /// Endpoint URL
    pub endpoint: String,
    /// Batches produced
    pub batches_produced: u64,
    /// Transactions sequenced
    pub txs_sequenced: u64,
    /// Total fees earned
    pub fees_earned: u64,
    /// Registration timestamp
    pub registered_at: u64,
    /// Last active timestamp
    pub last_active: u64,
    /// Reputation score (0-100)
    pub reputation: u32,
    /// Consecutive missed slots
    pub missed_slots: u32,
}

impl SequencerInfo {
    /// Create new sequencer info
    pub fn new(address: Address, stake: u64, endpoint: String) -> Self {
        Self {
            address,
            stake,
            status: SequencerStatus::Active,
            endpoint,
            batches_produced: 0,
            txs_sequenced: 0,
            fees_earned: 0,
            registered_at: current_timestamp(),
            last_active: current_timestamp(),
            reputation: 50,
            missed_slots: 0,
        }
    }

    /// Record batch production
    pub fn record_batch(&mut self, tx_count: u64, fees: u64) {
        self.batches_produced += 1;
        self.txs_sequenced += tx_count;
        self.fees_earned += fees;
        self.last_active = current_timestamp();
        self.missed_slots = 0;
        self.reputation = (self.reputation + 1).min(100);
    }

    /// Record missed slot
    pub fn record_missed_slot(&mut self) {
        self.missed_slots += 1;
        self.reputation = self.reputation.saturating_sub(2);
    }

    /// Slash sequencer
    pub fn slash(&mut self, slash_percent: u8) -> u64 {
        let slash_amount = (self.stake as u128 * slash_percent as u128 / 100) as u64;
        self.stake = self.stake.saturating_sub(slash_amount);
        self.status = SequencerStatus::Slashed;
        self.reputation = 0;
        slash_amount
    }
}

/// Transaction in mempool with priority
#[derive(Debug, Clone)]
struct PendingTransaction {
    tx: L2Transaction,
    received_at: u64,
    priority: u64,
    l1_inclusion_block: Option<u64>, // For force inclusion
}

impl PendingTransaction {
    fn new(tx: L2Transaction) -> Self {
        let priority = tx.gas_price; // Simple priority = gas price
        Self {
            tx,
            received_at: current_timestamp(),
            priority,
            l1_inclusion_block: None,
        }
    }

    fn with_force_inclusion(tx: L2Transaction, l1_block: u64) -> Self {
        Self {
            tx,
            received_at: current_timestamp(),
            priority: u64::MAX, // Maximum priority for force inclusion
            l1_inclusion_block: Some(l1_block),
        }
    }
}

/// Decentralized Sequencer
pub struct Sequencer {
    /// Configuration
    config: SequencerConfig,
    /// This sequencer's address
    address: Address,
    /// This sequencer's info
    info: Arc<RwLock<SequencerInfo>>,
    /// Registered sequencers
    sequencers: Arc<RwLock<HashMap<Address, SequencerInfo>>>,
    /// Current leader
    current_leader: Arc<RwLock<Option<Address>>>,
    /// Pending transactions (mempool)
    mempool: Arc<RwLock<VecDeque<PendingTransaction>>>,
    /// L2 state
    state: Arc<L2State>,
    /// Batch builder
    batch_builder: Arc<RwLock<BatchBuilder>>,
    /// Current L1 block number
    l1_block_number: Arc<RwLock<u64>>,
    /// Current L2 block number
    l2_block_number: Arc<RwLock<u64>>,
    /// Statistics
    stats: Arc<RwLock<SequencerStats>>,
}

impl Sequencer {
    /// Create new sequencer
    pub fn new(
        config: SequencerConfig,
        address: Address,
        stake: u64,
        endpoint: String,
        state: Arc<L2State>,
    ) -> Self {
        let info = SequencerInfo::new(address, stake, endpoint);
        
        Self {
            config,
            address,
            info: Arc::new(RwLock::new(info)),
            sequencers: Arc::new(RwLock::new(HashMap::new())),
            current_leader: Arc::new(RwLock::new(None)),
            mempool: Arc::new(RwLock::new(VecDeque::new())),
            state,
            batch_builder: Arc::new(RwLock::new(BatchBuilder::new(address))),
            l1_block_number: Arc::new(RwLock::new(0)),
            l2_block_number: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(SequencerStats::default())),
        }
    }

    /// Register as sequencer
    pub fn register(&self) -> Result<(), SequencerError> {
        let info = self.info.read().clone();
        
        if info.stake < self.config.min_stake {
            return Err(SequencerError::InsufficientStake {
                required: self.config.min_stake,
                available: info.stake,
            });
        }

        self.sequencers.write().insert(self.address, info);
        Ok(())
    }

    /// Submit transaction to mempool
    pub fn submit_transaction(&self, tx: L2Transaction) -> Result<H256, SequencerError> {
        // Validate transaction
        self.validate_transaction(&tx)?;

        let tx_hash = tx.hash;
        let pending = PendingTransaction::new(tx);
        
        // Add to mempool with priority ordering
        let mut mempool = self.mempool.write();
        
        if self.config.priority_ordering {
            // Insert in priority order
            let pos = mempool.iter()
                .position(|p| p.priority < pending.priority)
                .unwrap_or(mempool.len());
            mempool.insert(pos, pending);
        } else {
            mempool.push_back(pending);
        }

        self.stats.write().txs_received += 1;
        Ok(tx_hash)
    }

    /// Force include transaction via L1
    pub fn force_include(&self, tx: L2Transaction, l1_block: u64) -> Result<H256, SequencerError> {
        self.validate_transaction(&tx)?;

        let tx_hash = tx.hash;
        let pending = PendingTransaction::with_force_inclusion(tx, l1_block);
        
        // Force inclusions go to front of queue
        self.mempool.write().push_front(pending);
        self.stats.write().force_inclusions += 1;

        Ok(tx_hash)
    }

    /// Validate transaction
    fn validate_transaction(&self, tx: &L2Transaction) -> Result<(), SequencerError> {
        // Check nonce
        let expected_nonce = self.state.get_nonce(&tx.from);
        if tx.nonce != expected_nonce {
            return Err(SequencerError::InvalidNonce {
                expected: expected_nonce,
                actual: tx.nonce,
            });
        }

        // Check balance for gas
        let balance = self.state.get_balance(&tx.from);
        let required = tx.value + tx.gas_cost();
        if balance < required {
            return Err(SequencerError::InsufficientBalance {
                available: balance,
                required,
            });
        }

        // Check signature (simplified)
        if tx.signature.is_empty() {
            return Err(SequencerError::InvalidSignature);
        }

        Ok(())
    }

    /// Produce next batch
    pub fn produce_batch(&self) -> Result<Batch, SequencerError> {
        // Check if we are the current leader
        if !self.is_leader() {
            return Err(SequencerError::NotLeader);
        }

        let l1_block = *self.l1_block_number.read();
        let pre_state_root = self.state.state_root();

        // Start new batch
        {
            let mut builder = self.batch_builder.write();
            builder.start_batch(pre_state_root, l1_block)
                .map_err(|e| SequencerError::BatchError(e))?;
        }

        // Process transactions from mempool
        let mut processed_txs = Vec::new();
        let mut total_gas = 0u64;

        while total_gas < self.config.max_batch_gas {
            let pending = {
                let mut mempool = self.mempool.write();
                if mempool.is_empty() {
                    break;
                }
                mempool.pop_front()
            };

            if let Some(pending) = pending {
                // Check force inclusion deadline
                if let Some(l1_block_req) = pending.l1_inclusion_block {
                    if l1_block < l1_block_req + self.config.force_inclusion_delay {
                        // Not yet required, can skip
                        continue;
                    }
                }

                // Execute transaction
                let tx_clone = pending.tx.clone();
                match self.execute_transaction(&pending.tx) {
                    Ok(gas_used) => {
                        total_gas += gas_used;
                        processed_txs.push(tx_clone.clone());
                        
                        let mut builder = self.batch_builder.write();
                        if builder.add_transaction(tx_clone).is_err() {
                            break; // Batch full
                        }
                    }
                    Err(_) => {
                        // Transaction failed, skip it
                        self.stats.write().txs_failed += 1;
                    }
                }

                if processed_txs.len() >= self.config.max_batch_size {
                    break;
                }
            }
        }

        // Seal batch
        let post_state_root = self.state.state_root();
        let batch = {
            let mut builder = self.batch_builder.write();
            builder.seal_batch(post_state_root)
                .map_err(|e| SequencerError::BatchError(e))?
        };

        // Update sequencer info
        let fees: u64 = processed_txs.iter().map(|tx| tx.gas_cost()).sum();
        self.info.write().record_batch(processed_txs.len() as u64, fees);

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.batches_produced += 1;
            stats.txs_processed += processed_txs.len() as u64;
            stats.total_gas_used += total_gas;
        }

        *self.l2_block_number.write() += 1;

        Ok(batch)
    }

    /// Execute transaction and return gas used
    fn execute_transaction(&self, tx: &L2Transaction) -> Result<u64, SequencerError> {
        // Execute based on transaction type
        match tx.tx_type {
            0 => {
                // Transfer
                if let Some(to) = tx.to {
                    self.state.transfer(&tx.from, &to, tx.value)
                        .map_err(|_| SequencerError::ExecutionFailed)?;
                }
            }
            1 => {
                // Contract call (simplified)
                self.state.increment_nonce(&tx.from);
            }
            2 => {
                // Deposit - credit account
                self.state.credit(&tx.from, tx.value);
            }
            3 => {
                // Withdrawal - debit account
                self.state.debit(&tx.from, tx.value)
                    .map_err(|_| SequencerError::ExecutionFailed)?;
            }
            _ => {
                return Err(SequencerError::InvalidTransactionType);
            }
        }

        Ok(tx.gas_limit)
    }

    /// Check if this sequencer is current leader
    pub fn is_leader(&self) -> bool {
        match self.config.mode {
            SequencerMode::Single => true,
            SequencerMode::Rotating | SequencerMode::Decentralized => {
                let leader = self.current_leader.read();
                leader.map_or(false, |l| l == self.address)
            }
            SequencerMode::Based => {
                // Based sequencing - L1 determines order
                true
            }
        }
    }

    /// Rotate leader (for rotating/decentralized modes)
    pub fn rotate_leader(&self) {
        let sequencers = self.sequencers.read();
        let active: Vec<_> = sequencers.values()
            .filter(|s| s.status == SequencerStatus::Active)
            .collect();

        if active.is_empty() {
            *self.current_leader.write() = None;
            return;
        }

        // Simple round-robin based on L2 block number
        let block = *self.l2_block_number.read();
        let index = (block / self.config.rotation_period) as usize % active.len();
        
        *self.current_leader.write() = Some(active[index].address);
    }

    /// Update L1 block number
    pub fn update_l1_block(&self, block: u64) {
        *self.l1_block_number.write() = block;
        
        // Check for force inclusions that must be processed
        self.process_force_inclusions(block);
    }

    /// Process pending force inclusions
    fn process_force_inclusions(&self, l1_block: u64) {
        let deadline = l1_block.saturating_sub(self.config.force_inclusion_delay);
        
        let mempool = self.mempool.read();
        let overdue = mempool.iter()
            .filter(|p| {
                p.l1_inclusion_block
                    .map_or(false, |b| b <= deadline)
            })
            .count();

        if overdue > 0 {
            self.stats.write().overdue_force_inclusions += overdue as u64;
        }
    }

    /// Get mempool size
    pub fn mempool_size(&self) -> usize {
        self.mempool.read().len()
    }

    /// Get current leader
    pub fn get_leader(&self) -> Option<Address> {
        *self.current_leader.read()
    }

    /// Get sequencer info
    pub fn get_sequencer(&self, address: &Address) -> Option<SequencerInfo> {
        self.sequencers.read().get(address).cloned()
    }

    /// Get all active sequencers
    pub fn get_active_sequencers(&self) -> Vec<SequencerInfo> {
        self.sequencers.read()
            .values()
            .filter(|s| s.status == SequencerStatus::Active)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> SequencerStats {
        self.stats.read().clone()
    }
}

/// Sequencer errors
#[derive(Debug, thiserror::Error)]
pub enum SequencerError {
    #[error("Insufficient stake: required {required}, available {available}")]
    InsufficientStake { required: u64, available: u64 },

    #[error("Not the current leader")]
    NotLeader,

    #[error("Invalid nonce: expected {expected}, actual {actual}")]
    InvalidNonce { expected: u64, actual: u64 },

    #[error("Insufficient balance: available {available}, required {required}")]
    InsufficientBalance { available: u64, required: u64 },

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid transaction type")]
    InvalidTransactionType,

    #[error("Execution failed")]
    ExecutionFailed,

    #[error("Batch error: {0}")]
    BatchError(BatchError),

    #[error("Sequencer not found: {0:?}")]
    NotFound(Address),
}

/// Sequencer statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SequencerStats {
    pub batches_produced: u64,
    pub txs_received: u64,
    pub txs_processed: u64,
    pub txs_failed: u64,
    pub force_inclusions: u64,
    pub overdue_force_inclusions: u64,
    pub total_gas_used: u64,
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

    fn make_sequencer() -> Sequencer {
        let config = SequencerConfig::default();
        let state = Arc::new(L2State::new());
        
        // Fund an account for testing
        state.credit(&Address([1u8; 20]), 1_000_000_000_000);
        
        Sequencer::new(
            config,
            Address([1u8; 20]),
            MIN_SEQUENCER_STAKE,
            "https://seq.example.com".to_string(),
            state,
        )
    }

    #[test]
    fn test_sequencer_creation() {
        let seq = make_sequencer();
        assert_eq!(seq.mempool_size(), 0);
    }

    #[test]
    fn test_submit_transaction() {
        let seq = make_sequencer();
        
        let tx = L2Transaction::transfer(
            Address([1u8; 20]),
            Address([2u8; 20]),
            1000,
            0,
        );
        // Add signature for validation
        let mut tx = tx;
        tx.signature = vec![0u8; 65];

        let result = seq.submit_transaction(tx);
        assert!(result.is_ok());
        assert_eq!(seq.mempool_size(), 1);
    }

    #[test]
    fn test_sequencer_registration() {
        let seq = make_sequencer();
        assert!(seq.register().is_ok());
    }

    #[test]
    fn test_leader_rotation() {
        let seq = make_sequencer();
        seq.register().unwrap();
        seq.rotate_leader();
        
        assert!(seq.get_leader().is_some());
    }
}
