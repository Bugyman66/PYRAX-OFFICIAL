//! PYRAX Stream C - ZK Staking Service
//!
//! Production-ready staking service for Stream C (ZK-STARK + PoS):
//! - Stake management (deposit, withdraw, slash)
//! - ZK proof validation and reward distribution
//! - Checkpoint finality
//! - Validator registration and rotation
//!
//! Validators stake PYRAX tokens and generate ZK proofs for Stream A/B blocks.
//! Valid proofs earn 10 PYRAX per checkpoint, invalid proofs result in slashing.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc, broadcast};
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

use crate::types::{H256, Address, BlockNumber, Transaction, Block};
use crate::storage::ChainDB;
use crate::consensus::Stream;
use crate::zkrollup::{ZkProver, ZkVerifier, Proof, ProofType, VerificationResult};

/// Minimum stake to become a validator (10,000 PYRAX)
pub const MIN_VALIDATOR_STAKE: u64 = 10_000 * 100_000_000;

/// Maximum validators in active set
pub const MAX_VALIDATORS: usize = 100;

/// Checkpoint interval (every 100 Stream A blocks)
pub const CHECKPOINT_INTERVAL: u64 = 100;

/// Unbonding period (7 days in seconds)
pub const UNBONDING_PERIOD: u64 = 7 * 24 * 60 * 60;

/// Slashing percentage for invalid proofs (10%)
pub const SLASH_PERCENT: u64 = 10;

/// Reward per valid checkpoint (10 PYRAX)
pub const CHECKPOINT_REWARD: u64 = 10 * 100_000_000;

/// Staking service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingConfig {
    /// Enable staking service
    pub enabled: bool,
    /// RPC bind address for staking API
    pub rpc_bind: String,
    /// RPC port for staking API  
    pub rpc_port: u16,
    /// Minimum stake amount
    pub min_stake: u64,
    /// Maximum validators
    pub max_validators: usize,
    /// Checkpoint interval
    pub checkpoint_interval: u64,
    /// Enable ZK proof generation (requires GPU)
    pub enable_proving: bool,
    /// Proof type to use
    pub proof_type: ProofType,
}

impl Default for StakingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rpc_bind: "0.0.0.0".to_string(),
            rpc_port: 8547,
            min_stake: MIN_VALIDATOR_STAKE,
            max_validators: MAX_VALIDATORS,
            checkpoint_interval: CHECKPOINT_INTERVAL,
            enable_proving: true,
            proof_type: ProofType::Stark,
        }
    }
}

/// Validator state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Validator address
    pub address: Address,
    /// Staked amount (in base units)
    pub stake: u64,
    /// Commission rate (basis points, 0-10000)
    pub commission_bps: u16,
    /// Total rewards earned
    pub total_rewards: u64,
    /// Total slashed amount
    pub total_slashed: u64,
    /// Checkpoints validated
    pub checkpoints_validated: u64,
    /// Invalid proofs submitted (leads to slashing)
    pub invalid_proofs: u64,
    /// Registration timestamp
    pub registered_at: u64,
    /// Last activity timestamp
    pub last_active_at: u64,
    /// Is currently active in validator set
    pub is_active: bool,
    /// Unbonding start time (0 if not unbonding)
    pub unbonding_at: u64,
    /// Pending unbond amount
    pub unbonding_amount: u64,
}

impl Validator {
    /// Create new validator
    pub fn new(address: Address, stake: u64) -> Self {
        let now = current_timestamp();
        Self {
            address,
            stake,
            commission_bps: 500, // 5% default commission
            total_rewards: 0,
            total_slashed: 0,
            checkpoints_validated: 0,
            invalid_proofs: 0,
            registered_at: now,
            last_active_at: now,
            is_active: false,
            unbonding_at: 0,
            unbonding_amount: 0,
        }
    }

    /// Calculate effective stake (stake - unbonding)
    pub fn effective_stake(&self) -> u64 {
        self.stake.saturating_sub(self.unbonding_amount)
    }

    /// Check if can be slashed
    pub fn can_slash(&self) -> bool {
        self.effective_stake() > 0
    }

    /// Apply slash
    pub fn slash(&mut self, percent: u64) -> u64 {
        let slash_amount = (self.effective_stake() * percent) / 100;
        self.stake = self.stake.saturating_sub(slash_amount);
        self.total_slashed += slash_amount;
        slash_amount
    }

    /// Add reward
    pub fn add_reward(&mut self, amount: u64) {
        self.total_rewards += amount;
        self.last_active_at = current_timestamp();
    }

    /// Start unbonding
    pub fn start_unbonding(&mut self, amount: u64) -> Result<(), String> {
        if amount > self.effective_stake() {
            return Err("Insufficient stake".to_string());
        }
        if self.unbonding_at > 0 {
            return Err("Already unbonding".to_string());
        }
        self.unbonding_at = current_timestamp();
        self.unbonding_amount = amount;
        Ok(())
    }

    /// Complete unbonding (after period)
    pub fn complete_unbonding(&mut self) -> Result<u64, String> {
        if self.unbonding_at == 0 {
            return Err("Not unbonding".to_string());
        }
        let elapsed = current_timestamp() - self.unbonding_at;
        if elapsed < UNBONDING_PERIOD {
            return Err(format!("Unbonding period not complete ({} seconds remaining)", 
                UNBONDING_PERIOD - elapsed));
        }
        let amount = self.unbonding_amount;
        self.stake = self.stake.saturating_sub(amount);
        self.unbonding_at = 0;
        self.unbonding_amount = 0;
        Ok(amount)
    }
}

/// Checkpoint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint ID
    pub id: u64,
    /// Stream A block height being checkpointed
    pub stream_a_height: BlockNumber,
    /// Stream B block height being checkpointed
    pub stream_b_height: BlockNumber,
    /// State root after checkpoint
    pub state_root: H256,
    /// UTXO commitment
    pub utxo_commitment: H256,
    /// ZK proof of validity
    pub proof: Option<Proof>,
    /// Validator who created this checkpoint
    pub validator: Address,
    /// Timestamp
    pub timestamp: u64,
    /// Is finalized
    pub finalized: bool,
    /// Attestations from other validators
    pub attestations: Vec<Attestation>,
}

impl Checkpoint {
    /// Create new checkpoint
    pub fn new(
        id: u64,
        stream_a_height: BlockNumber,
        stream_b_height: BlockNumber,
        state_root: H256,
        utxo_commitment: H256,
        validator: Address,
    ) -> Self {
        Self {
            id,
            stream_a_height,
            stream_b_height,
            state_root,
            utxo_commitment,
            proof: None,
            validator,
            timestamp: current_timestamp(),
            finalized: false,
            attestations: Vec::new(),
        }
    }

    /// Compute checkpoint hash
    pub fn hash(&self) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.id.to_le_bytes());
        hasher.update(&self.stream_a_height.to_le_bytes());
        hasher.update(&self.stream_b_height.to_le_bytes());
        hasher.update(&self.state_root.0);
        hasher.update(&self.utxo_commitment.0);
        hasher.update(&self.validator.0);
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Check if has enough attestations for finality (2/3 of validators)
    pub fn has_quorum(&self, total_validators: usize) -> bool {
        let required = (total_validators * 2) / 3 + 1;
        self.attestations.len() >= required
    }
}

/// Validator attestation for a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// Validator address
    pub validator: Address,
    /// Checkpoint hash being attested
    pub checkpoint_hash: H256,
    /// Signature (simplified - in production use BLS)
    pub signature: H256,
    /// Timestamp
    pub timestamp: u64,
}

impl Attestation {
    /// Create new attestation
    pub fn new(validator: Address, checkpoint_hash: H256) -> Self {
        // In production, this would be a proper cryptographic signature
        let signature = Self::compute_signature(&validator, &checkpoint_hash);
        Self {
            validator,
            checkpoint_hash,
            signature,
            timestamp: current_timestamp(),
        }
    }

    /// Compute attestation signature
    fn compute_signature(validator: &Address, checkpoint_hash: &H256) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&validator.0);
        hasher.update(&checkpoint_hash.0);
        hasher.update(&current_timestamp().to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }
}

/// Staking event for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StakingEvent {
    ValidatorRegistered { address: Address, stake: u64 },
    ValidatorDeactivated { address: Address },
    StakeDeposited { address: Address, amount: u64 },
    UnbondingStarted { address: Address, amount: u64 },
    UnbondingCompleted { address: Address, amount: u64 },
    ValidatorSlashed { address: Address, amount: u64, reason: String },
    CheckpointCreated { id: u64, validator: Address },
    CheckpointFinalized { id: u64, state_root: H256 },
    RewardDistributed { address: Address, amount: u64 },
}

/// Production Staking Service
pub struct StakingService {
    config: StakingConfig,
    db: Arc<ChainDB>,
    validators: Arc<RwLock<HashMap<Address, Validator>>>,
    active_set: Arc<RwLock<Vec<Address>>>,
    checkpoints: Arc<RwLock<Vec<Checkpoint>>>,
    last_checkpoint_id: Arc<RwLock<u64>>,
    event_tx: broadcast::Sender<StakingEvent>,
    running: Arc<RwLock<bool>>,
    prover: Option<Arc<ZkProver>>,
    verifier: Arc<ZkVerifier>,
}

impl StakingService {
    /// Create new staking service
    pub fn new(config: StakingConfig, db: Arc<ChainDB>) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        
        let prover = if config.enable_proving {
            // Use service address as default prover address
            Some(Arc::new(ZkProver::new(Default::default(), Address::ZERO)))
        } else {
            None
        };

        Self {
            config,
            db,
            validators: Arc::new(RwLock::new(HashMap::new())),
            active_set: Arc::new(RwLock::new(Vec::new())),
            checkpoints: Arc::new(RwLock::new(Vec::new())),
            last_checkpoint_id: Arc::new(RwLock::new(0)),
            event_tx,
            running: Arc::new(RwLock::new(false)),
            prover,
            verifier: Arc::new(ZkVerifier::new(Default::default())),
        }
    }

    /// Start the staking service
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("Starting PYRAX Stream C Staking Service...");
        info!("RPC endpoint: {}:{}", self.config.rpc_bind, self.config.rpc_port);
        info!("Min stake: {} PYRAX", self.config.min_stake / 100_000_000);
        info!("Checkpoint interval: {} blocks", self.config.checkpoint_interval);
        
        *self.running.write().await = true;

        // Start checkpoint monitor
        self.start_checkpoint_monitor().await;

        // Start validator rotation
        self.start_validator_rotation().await;

        info!("Staking Service started successfully");
        Ok(())
    }

    /// Stop the staking service
    pub async fn stop(&self) {
        info!("Stopping Staking Service...");
        *self.running.write().await = false;
    }

    /// Register a new validator
    pub async fn register_validator(&self, address: Address, stake: u64) -> Result<(), String> {
        if stake < self.config.min_stake {
            return Err(format!(
                "Insufficient stake: {} PYRAX required, {} provided",
                self.config.min_stake / 100_000_000,
                stake / 100_000_000
            ));
        }

        let mut validators = self.validators.write().await;
        
        if validators.contains_key(&address) {
            return Err("Validator already registered".to_string());
        }

        if validators.len() >= self.config.max_validators {
            // Check if new validator has more stake than minimum in active set
            let min_stake = self.get_min_active_stake().await;
            if stake <= min_stake {
                return Err("Stake too low to enter active set".to_string());
            }
        }

        let validator = Validator::new(address, stake);
        validators.insert(address, validator);

        drop(validators);

        // Update active set
        self.update_active_set().await;

        let _ = self.event_tx.send(StakingEvent::ValidatorRegistered { address, stake });
        
        info!("Validator registered: {} with {} PYRAX stake", address, stake / 100_000_000);
        Ok(())
    }

    /// Deposit additional stake
    pub async fn deposit_stake(&self, address: Address, amount: u64) -> Result<u64, String> {
        let mut validators = self.validators.write().await;
        
        let validator = validators.get_mut(&address)
            .ok_or("Validator not found")?;

        validator.stake += amount;
        let new_stake = validator.stake;

        drop(validators);

        self.update_active_set().await;

        let _ = self.event_tx.send(StakingEvent::StakeDeposited { address, amount });
        
        info!("Stake deposited: {} added {} PYRAX (total: {})", 
            address, amount / 100_000_000, new_stake / 100_000_000);
        
        Ok(new_stake)
    }

    /// Start unbonding stake
    pub async fn start_unbonding(&self, address: Address, amount: u64) -> Result<(), String> {
        let mut validators = self.validators.write().await;
        
        let validator = validators.get_mut(&address)
            .ok_or("Validator not found")?;

        validator.start_unbonding(amount)?;

        drop(validators);

        self.update_active_set().await;

        let _ = self.event_tx.send(StakingEvent::UnbondingStarted { address, amount });
        
        info!("Unbonding started: {} for {} PYRAX", address, amount / 100_000_000);
        Ok(())
    }

    /// Complete unbonding and withdraw
    pub async fn complete_unbonding(&self, address: Address) -> Result<u64, String> {
        let mut validators = self.validators.write().await;
        
        let validator = validators.get_mut(&address)
            .ok_or("Validator not found")?;

        let amount = validator.complete_unbonding()?;

        let _ = self.event_tx.send(StakingEvent::UnbondingCompleted { address, amount });
        
        info!("Unbonding completed: {} withdrew {} PYRAX", address, amount / 100_000_000);
        Ok(amount)
    }

    /// Submit a checkpoint with ZK proof
    pub async fn submit_checkpoint(
        &self,
        validator: Address,
        stream_a_height: BlockNumber,
        stream_b_height: BlockNumber,
        state_root: H256,
        utxo_commitment: H256,
        proof: Proof,
    ) -> Result<u64, String> {
        // Verify validator is active
        let validators = self.validators.read().await;
        let val = validators.get(&validator)
            .ok_or("Validator not found")?;
        
        if !val.is_active {
            return Err("Validator not in active set".to_string());
        }
        drop(validators);

        // Verify the ZK proof
        let verification = self.verifier.verify(&proof, validator)
            .map_err(|e| format!("Proof verification error: {:?}", e))?;
        
        if !verification.valid {
            // Slash the validator for invalid proof
            self.slash_validator(validator, "Invalid ZK proof".to_string()).await;
            return Err("Invalid ZK proof - validator slashed".to_string());
        }

        // Create checkpoint
        let mut checkpoint_id = self.last_checkpoint_id.write().await;
        *checkpoint_id += 1;
        let id = *checkpoint_id;
        drop(checkpoint_id);

        let mut checkpoint = Checkpoint::new(
            id,
            stream_a_height,
            stream_b_height,
            state_root,
            utxo_commitment,
            validator,
        );
        checkpoint.proof = Some(proof);

        // Add to checkpoints
        let mut checkpoints = self.checkpoints.write().await;
        checkpoints.push(checkpoint);
        drop(checkpoints);

        // Update validator stats
        let mut validators = self.validators.write().await;
        if let Some(v) = validators.get_mut(&validator) {
            v.checkpoints_validated += 1;
            v.add_reward(CHECKPOINT_REWARD);
        }
        drop(validators);

        let _ = self.event_tx.send(StakingEvent::CheckpointCreated { id, validator });
        let _ = self.event_tx.send(StakingEvent::RewardDistributed { 
            address: validator, 
            amount: CHECKPOINT_REWARD 
        });

        info!("Checkpoint {} created by {} at Stream A height {}", id, validator, stream_a_height);
        Ok(id)
    }

    /// Attest to a checkpoint
    pub async fn attest_checkpoint(&self, validator: Address, checkpoint_id: u64) -> Result<(), String> {
        // Verify validator is active
        let validators = self.validators.read().await;
        let val = validators.get(&validator)
            .ok_or("Validator not found")?;
        
        if !val.is_active {
            return Err("Validator not in active set".to_string());
        }
        drop(validators);

        // Find and update checkpoint
        let mut checkpoints = self.checkpoints.write().await;
        let checkpoint = checkpoints.iter_mut()
            .find(|c| c.id == checkpoint_id)
            .ok_or("Checkpoint not found")?;

        // Check not already attested
        if checkpoint.attestations.iter().any(|a| a.validator == validator) {
            return Err("Already attested".to_string());
        }

        let checkpoint_hash = checkpoint.hash();
        let attestation = Attestation::new(validator, checkpoint_hash);
        checkpoint.attestations.push(attestation);

        // Check for finality
        let active_set = self.active_set.read().await;
        if checkpoint.has_quorum(active_set.len()) && !checkpoint.finalized {
            checkpoint.finalized = true;
            let state_root = checkpoint.state_root;
            let id = checkpoint.id;
            drop(checkpoints);
            drop(active_set);
            
            let _ = self.event_tx.send(StakingEvent::CheckpointFinalized { id, state_root });
            info!("Checkpoint {} finalized with state root {}", id, state_root);
        }

        Ok(())
    }

    /// Slash a validator
    async fn slash_validator(&self, address: Address, reason: String) {
        let mut validators = self.validators.write().await;
        
        if let Some(validator) = validators.get_mut(&address) {
            let slash_amount = validator.slash(SLASH_PERCENT);
            validator.invalid_proofs += 1;
            
            let _ = self.event_tx.send(StakingEvent::ValidatorSlashed { 
                address, 
                amount: slash_amount,
                reason: reason.clone(),
            });
            
            warn!("Validator {} slashed {} PYRAX: {}", 
                address, slash_amount / 100_000_000, reason);
        }

        drop(validators);
        self.update_active_set().await;
    }

    /// Update the active validator set based on stake
    async fn update_active_set(&self) {
        let validators = self.validators.read().await;
        
        // Get all validators sorted by stake
        let mut sorted: Vec<_> = validators.iter()
            .filter(|(_, v)| v.effective_stake() >= self.config.min_stake)
            .collect();
        
        sorted.sort_by(|a, b| b.1.effective_stake().cmp(&a.1.effective_stake()));

        // Take top N validators
        let new_active: Vec<Address> = sorted.iter()
            .take(self.config.max_validators)
            .map(|(addr, _)| **addr)
            .collect();

        drop(validators);

        // Update active status
        let mut validators = self.validators.write().await;
        for (addr, validator) in validators.iter_mut() {
            validator.is_active = new_active.contains(addr);
        }
        drop(validators);

        // Update active set
        let mut active_set = self.active_set.write().await;
        *active_set = new_active;

        debug!("Active validator set updated: {} validators", active_set.len());
    }

    /// Get minimum stake in active set
    async fn get_min_active_stake(&self) -> u64 {
        let validators = self.validators.read().await;
        let active_set = self.active_set.read().await;
        
        active_set.iter()
            .filter_map(|addr| validators.get(addr))
            .map(|v| v.effective_stake())
            .min()
            .unwrap_or(0)
    }

    /// Start checkpoint monitoring task
    async fn start_checkpoint_monitor(&self) {
        let running = Arc::clone(&self.running);
        let db = Arc::clone(&self.db);
        let checkpoints = Arc::clone(&self.checkpoints);
        let checkpoint_interval = self.config.checkpoint_interval;
        let last_checkpoint_id = Arc::clone(&self.last_checkpoint_id);

        tokio::spawn(async move {
            let mut last_checked_height = 0u64;
            
            loop {
                if !*running.read().await {
                    break;
                }

                // Check current chain height
                let tip = db.get_tip();
                let current_height = tip.height;

                // Check if we need a new checkpoint
                if current_height >= last_checked_height + checkpoint_interval {
                    let checkpoint_height = (current_height / checkpoint_interval) * checkpoint_interval;
                    
                    // Check if checkpoint already exists for this height
                    let existing = checkpoints.read().await;
                    let exists = existing.iter()
                        .any(|c| c.stream_a_height == checkpoint_height);
                    drop(existing);

                    if !exists {
                        debug!("Checkpoint needed at height {}", checkpoint_height);
                        // In production, this would trigger proof generation
                        // For now, validators submit proofs via RPC
                    }

                    last_checked_height = checkpoint_height;
                }

                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
    }

    /// Start validator rotation task
    async fn start_validator_rotation(&self) {
        let running = Arc::clone(&self.running);
        let validators = Arc::clone(&self.validators);
        let active_set = Arc::clone(&self.active_set);

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                // Check for validators that should be deactivated (inactive > 1 hour)
                let now = current_timestamp();
                let mut to_deactivate = Vec::new();

                {
                    let vals = validators.read().await;
                    for (addr, v) in vals.iter() {
                        if v.is_active && (now - v.last_active_at) > 3600 {
                            to_deactivate.push(*addr);
                        }
                    }
                }

                for addr in to_deactivate {
                    let mut vals = validators.write().await;
                    if let Some(v) = vals.get_mut(&addr) {
                        v.is_active = false;
                        warn!("Validator {} deactivated due to inactivity", addr);
                    }
                }

                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    /// Get validator info
    pub async fn get_validator(&self, address: &Address) -> Option<Validator> {
        self.validators.read().await.get(address).cloned()
    }

    /// Get all validators
    pub async fn get_all_validators(&self) -> Vec<Validator> {
        self.validators.read().await.values().cloned().collect()
    }

    /// Get active validators
    pub async fn get_active_validators(&self) -> Vec<Address> {
        self.active_set.read().await.clone()
    }

    /// Get latest checkpoint
    pub async fn get_latest_checkpoint(&self) -> Option<Checkpoint> {
        self.checkpoints.read().await.last().cloned()
    }

    /// Get checkpoint by ID
    pub async fn get_checkpoint(&self, id: u64) -> Option<Checkpoint> {
        self.checkpoints.read().await.iter()
            .find(|c| c.id == id)
            .cloned()
    }

    /// Get staking statistics
    pub async fn get_stats(&self) -> StakingStats {
        let validators = self.validators.read().await;
        let active_set = self.active_set.read().await;
        let checkpoints = self.checkpoints.read().await;

        let total_staked: u64 = validators.values()
            .map(|v| v.stake)
            .sum();

        let total_rewards: u64 = validators.values()
            .map(|v| v.total_rewards)
            .sum();

        let total_slashed: u64 = validators.values()
            .map(|v| v.total_slashed)
            .sum();

        let finalized_checkpoints = checkpoints.iter()
            .filter(|c| c.finalized)
            .count() as u64;

        StakingStats {
            total_validators: validators.len() as u64,
            active_validators: active_set.len() as u64,
            total_staked,
            total_rewards,
            total_slashed,
            total_checkpoints: checkpoints.len() as u64,
            finalized_checkpoints,
            min_stake: self.config.min_stake,
            checkpoint_interval: self.config.checkpoint_interval,
        }
    }

    /// Subscribe to staking events
    pub fn subscribe(&self) -> broadcast::Receiver<StakingEvent> {
        self.event_tx.subscribe()
    }
}

/// Staking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStats {
    pub total_validators: u64,
    pub active_validators: u64,
    pub total_staked: u64,
    pub total_rewards: u64,
    pub total_slashed: u64,
    pub total_checkpoints: u64,
    pub finalized_checkpoints: u64,
    pub min_stake: u64,
    pub checkpoint_interval: u64,
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = Validator::new(Address::ZERO, MIN_VALIDATOR_STAKE);
        assert_eq!(validator.stake, MIN_VALIDATOR_STAKE);
        assert_eq!(validator.effective_stake(), MIN_VALIDATOR_STAKE);
        assert!(!validator.is_active);
    }

    #[test]
    fn test_validator_slash() {
        let mut validator = Validator::new(Address::ZERO, 100_000 * 100_000_000);
        let slashed = validator.slash(10); // 10%
        assert_eq!(slashed, 10_000 * 100_000_000);
        assert_eq!(validator.stake, 90_000 * 100_000_000);
    }

    #[test]
    fn test_checkpoint_hash() {
        let checkpoint = Checkpoint::new(
            1,
            100,
            10,
            H256::zero(),
            H256::zero(),
            Address::ZERO,
        );
        let hash = checkpoint.hash();
        assert!(!hash.is_zero());
    }

    #[test]
    fn test_attestation() {
        let attestation = Attestation::new(Address::ZERO, H256::zero());
        assert!(!attestation.signature.is_zero());
    }
}
