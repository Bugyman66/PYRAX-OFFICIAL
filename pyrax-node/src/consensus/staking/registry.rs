//! Staking Registry
//!
//! Manages stake deposits, withdrawals, and delegations

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::validator::{Validator, ValidatorStatus, ValidatorSet, ValidatorSelection};
use super::{
    MIN_VALIDATOR_STAKE, MIN_DELEGATION_STAKE, MAX_ACTIVE_VALIDATORS,
    UNBONDING_PERIOD_SECS, EPOCH_DURATION_SECS,
};

/// Stake status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StakeStatus {
    /// Actively staked
    Bonded,
    /// Unbonding (waiting period)
    Unbonding,
    /// Fully unbonded, ready to withdraw
    Unbonded,
}

/// Individual stake record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stake {
    /// Staker address
    pub staker: Address,
    /// Validator address (for delegations)
    pub validator: Address,
    /// Amount staked
    pub amount: u64,
    /// Current status
    pub status: StakeStatus,
    /// Timestamp when stake was created
    pub created_at: u64,
    /// Timestamp when unbonding started (if applicable)
    pub unbonding_at: Option<u64>,
    /// Accumulated rewards
    pub rewards: u64,
    /// Whether this is self-stake or delegation
    pub is_self_stake: bool,
}

impl Stake {
    /// Create new stake
    pub fn new(staker: Address, validator: Address, amount: u64, is_self_stake: bool) -> Self {
        Self {
            staker,
            validator,
            amount,
            status: StakeStatus::Bonded,
            created_at: current_timestamp(),
            unbonding_at: None,
            rewards: 0,
            is_self_stake,
        }
    }

    /// Start unbonding
    pub fn start_unbonding(&mut self) {
        if self.status == StakeStatus::Bonded {
            self.status = StakeStatus::Unbonding;
            self.unbonding_at = Some(current_timestamp());
        }
    }

    /// Check if unbonding is complete
    pub fn is_unbonding_complete(&self) -> bool {
        match self.unbonding_at {
            Some(start) => current_timestamp() >= start + UNBONDING_PERIOD_SECS,
            None => false,
        }
    }

    /// Complete unbonding
    pub fn complete_unbonding(&mut self) {
        if self.status == StakeStatus::Unbonding && self.is_unbonding_complete() {
            self.status = StakeStatus::Unbonded;
        }
    }

    /// Add rewards
    pub fn add_rewards(&mut self, amount: u64) {
        self.rewards = self.rewards.saturating_add(amount);
    }

    /// Withdraw rewards
    pub fn withdraw_rewards(&mut self) -> u64 {
        let rewards = self.rewards;
        self.rewards = 0;
        rewards
    }
}

/// Staking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingConfig {
    /// Minimum stake to become validator
    pub min_validator_stake: u64,
    /// Minimum delegation amount
    pub min_delegation: u64,
    /// Maximum validators in active set
    pub max_validators: usize,
    /// Unbonding period in seconds
    pub unbonding_period_secs: u64,
    /// Epoch duration in seconds
    pub epoch_duration_secs: u64,
    /// Maximum commission rate (basis points)
    pub max_commission_bps: u16,
    /// Maximum commission change per day (basis points)
    pub max_commission_change_bps: u16,
}

impl Default for StakingConfig {
    fn default() -> Self {
        Self {
            min_validator_stake: MIN_VALIDATOR_STAKE,
            min_delegation: MIN_DELEGATION_STAKE,
            max_validators: MAX_ACTIVE_VALIDATORS,
            unbonding_period_secs: UNBONDING_PERIOD_SECS,
            epoch_duration_secs: EPOCH_DURATION_SECS,
            max_commission_bps: 5000, // 50% max
            max_commission_change_bps: 100, // 1% per day max change
        }
    }
}

/// Staking errors
#[derive(Debug, thiserror::Error)]
pub enum StakingError {
    #[error("Validator not found: {0:?}")]
    ValidatorNotFound(Address),

    #[error("Validator already exists: {0:?}")]
    ValidatorExists(Address),

    #[error("Insufficient stake: required {required}, got {actual}")]
    InsufficientStake { required: u64, actual: u64 },

    #[error("Stake not found")]
    StakeNotFound,

    #[error("Invalid status: {0:?}")]
    InvalidStatus(StakeStatus),

    #[error("Validator is jailed")]
    ValidatorJailed,

    #[error("Validator is tombstoned")]
    ValidatorTombstoned,

    #[error("Unbonding not complete")]
    UnbondingNotComplete,

    #[error("Commission too high: {0}")]
    CommissionTooHigh(u16),

    #[error("Self-stake required")]
    SelfStakeRequired,

    #[error("Cannot unstake below minimum")]
    BelowMinimumStake,
}

/// Staking Registry - manages all validators and stakes
pub struct StakingRegistry {
    config: StakingConfig,
    /// All validators by address
    validators: Arc<RwLock<HashMap<Address, Validator>>>,
    /// All stakes (staker -> validator -> Stake)
    stakes: Arc<RwLock<HashMap<Address, HashMap<Address, Stake>>>>,
    /// Current epoch
    current_epoch: Arc<RwLock<u64>>,
    /// Current validator set
    validator_set: Arc<RwLock<ValidatorSet>>,
    /// Total staked amount
    total_staked: Arc<RwLock<u64>>,
    /// Unbonding queue (completion_time -> stakes)
    unbonding_queue: Arc<RwLock<HashMap<u64, Vec<(Address, Address)>>>>,
}

impl StakingRegistry {
    /// Create new staking registry
    pub fn new(config: StakingConfig) -> Self {
        Self {
            config,
            validators: Arc::new(RwLock::new(HashMap::new())),
            stakes: Arc::new(RwLock::new(HashMap::new())),
            current_epoch: Arc::new(RwLock::new(0)),
            validator_set: Arc::new(RwLock::new(ValidatorSet::new(0, ValidatorSelection::WeightedRandom))),
            total_staked: Arc::new(RwLock::new(0)),
            unbonding_queue: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::new(StakingConfig::default())
    }

    /// Register a new validator
    pub fn register_validator(
        &self,
        address: Address,
        operator: Address,
        consensus_pubkey: [u8; 32],
        initial_stake: u64,
        commission_bps: u16,
        description: String,
    ) -> Result<(), StakingError> {
        // Validate stake
        if initial_stake < self.config.min_validator_stake {
            return Err(StakingError::InsufficientStake {
                required: self.config.min_validator_stake,
                actual: initial_stake,
            });
        }

        // Validate commission
        if commission_bps > self.config.max_commission_bps {
            return Err(StakingError::CommissionTooHigh(commission_bps));
        }

        let mut validators = self.validators.write();
        
        // Check if already exists
        if validators.contains_key(&address) {
            return Err(StakingError::ValidatorExists(address));
        }

        // Create validator
        let validator = Validator::new(
            address,
            operator,
            consensus_pubkey,
            initial_stake,
            commission_bps,
            description,
        );

        validators.insert(address, validator);

        // Create self-stake record
        let mut stakes = self.stakes.write();
        let staker_stakes = stakes.entry(address).or_insert_with(HashMap::new);
        staker_stakes.insert(address, Stake::new(address, address, initial_stake, true));

        // Update total
        *self.total_staked.write() += initial_stake;

        Ok(())
    }

    /// Activate a pending validator
    pub fn activate_validator(&self, address: &Address) -> Result<(), StakingError> {
        let mut validators = self.validators.write();
        let validator = validators.get_mut(address)
            .ok_or(StakingError::ValidatorNotFound(*address))?;

        if !validator.can_activate() {
            return Err(StakingError::InsufficientStake {
                required: self.config.min_validator_stake,
                actual: validator.self_stake,
            });
        }

        validator.status = ValidatorStatus::Active;
        Ok(())
    }

    /// Delegate stake to a validator
    pub fn delegate(
        &self,
        delegator: Address,
        validator_addr: Address,
        amount: u64,
    ) -> Result<(), StakingError> {
        // Validate amount
        if amount < self.config.min_delegation {
            return Err(StakingError::InsufficientStake {
                required: self.config.min_delegation,
                actual: amount,
            });
        }

        // Check validator exists and is valid
        {
            let validators = self.validators.read();
            let validator = validators.get(&validator_addr)
                .ok_or(StakingError::ValidatorNotFound(validator_addr))?;

            if validator.status == ValidatorStatus::Jailed {
                return Err(StakingError::ValidatorJailed);
            }
            if validator.status == ValidatorStatus::Tombstoned {
                return Err(StakingError::ValidatorTombstoned);
            }
        }

        // Update validator's delegated stake
        {
            let mut validators = self.validators.write();
            if let Some(v) = validators.get_mut(&validator_addr) {
                v.add_delegation(amount);
            }
        }

        // Create or update stake record
        {
            let mut stakes = self.stakes.write();
            let staker_stakes = stakes.entry(delegator).or_insert_with(HashMap::new);
            
            if let Some(existing) = staker_stakes.get_mut(&validator_addr) {
                existing.amount = existing.amount.saturating_add(amount);
            } else {
                staker_stakes.insert(
                    validator_addr,
                    Stake::new(delegator, validator_addr, amount, false),
                );
            }
        }

        // Update total
        *self.total_staked.write() += amount;

        Ok(())
    }

    /// Begin unstaking (starts unbonding period)
    pub fn begin_unstake(
        &self,
        staker: Address,
        validator_addr: Address,
        amount: u64,
    ) -> Result<u64, StakingError> {
        let completion_time = current_timestamp() + self.config.unbonding_period_secs;

        // Update stake record
        {
            let mut stakes = self.stakes.write();
            let staker_stakes = stakes.get_mut(&staker)
                .ok_or(StakingError::StakeNotFound)?;
            
            let stake = staker_stakes.get_mut(&validator_addr)
                .ok_or(StakingError::StakeNotFound)?;

            if stake.status != StakeStatus::Bonded {
                return Err(StakingError::InvalidStatus(stake.status));
            }

            if amount > stake.amount {
                return Err(StakingError::InsufficientStake {
                    required: amount,
                    actual: stake.amount,
                });
            }

            // Check minimum stake for self-stake
            if stake.is_self_stake {
                let remaining = stake.amount.saturating_sub(amount);
                if remaining > 0 && remaining < self.config.min_validator_stake {
                    return Err(StakingError::BelowMinimumStake);
                }
            }

            stake.amount = stake.amount.saturating_sub(amount);
            if stake.amount == 0 {
                stake.start_unbonding();
            }
        }

        // Update validator
        {
            let mut validators = self.validators.write();
            if let Some(v) = validators.get_mut(&validator_addr) {
                if staker == validator_addr {
                    v.self_stake = v.self_stake.saturating_sub(amount);
                    
                    // Deactivate if below minimum
                    if v.self_stake < self.config.min_validator_stake {
                        v.status = ValidatorStatus::Unbonding;
                    }
                } else {
                    v.remove_delegation(amount);
                }
            }
        }

        // Add to unbonding queue
        {
            let mut queue = self.unbonding_queue.write();
            queue.entry(completion_time)
                .or_insert_with(Vec::new)
                .push((staker, validator_addr));
        }

        Ok(completion_time)
    }

    /// Complete unbonding and withdraw stake
    pub fn complete_unbonding(
        &self,
        staker: Address,
        validator_addr: Address,
    ) -> Result<u64, StakingError> {
        let mut stakes = self.stakes.write();
        let staker_stakes = stakes.get_mut(&staker)
            .ok_or(StakingError::StakeNotFound)?;
        
        let stake = staker_stakes.get_mut(&validator_addr)
            .ok_or(StakingError::StakeNotFound)?;

        if stake.status != StakeStatus::Unbonding {
            return Err(StakingError::InvalidStatus(stake.status));
        }

        if !stake.is_unbonding_complete() {
            return Err(StakingError::UnbondingNotComplete);
        }

        stake.complete_unbonding();
        
        // Return amount (actual withdrawal handled by caller)
        let amount = stake.amount;
        
        // Remove stake record if fully unbonded
        if stake.amount == 0 {
            staker_stakes.remove(&validator_addr);
        }

        *self.total_staked.write() -= amount;

        Ok(amount)
    }

    /// Process unbonding queue (call periodically)
    pub fn process_unbonding_queue(&self) -> Vec<(Address, Address, u64)> {
        let now = current_timestamp();
        let mut completed = Vec::new();
        
        let mut queue = self.unbonding_queue.write();
        let mut to_remove = Vec::new();
        
        for (&completion_time, entries) in queue.iter() {
            if completion_time <= now {
                for (staker, validator) in entries {
                    if let Ok(amount) = self.complete_unbonding(*staker, *validator) {
                        completed.push((*staker, *validator, amount));
                    }
                }
                to_remove.push(completion_time);
            }
        }
        
        for time in to_remove {
            queue.remove(&time);
        }
        
        completed
    }

    /// Get validator by address
    pub fn get_validator(&self, address: &Address) -> Option<Validator> {
        self.validators.read().get(address).cloned()
    }

    /// Get all validators
    pub fn get_all_validators(&self) -> Vec<Validator> {
        self.validators.read().values().cloned().collect()
    }

    /// Get active validators
    pub fn get_active_validators(&self) -> Vec<Validator> {
        self.validators.read()
            .values()
            .filter(|v| v.status == ValidatorStatus::Active)
            .cloned()
            .collect()
    }

    /// Get stake for staker -> validator
    pub fn get_stake(&self, staker: &Address, validator: &Address) -> Option<Stake> {
        self.stakes.read()
            .get(staker)
            .and_then(|m| m.get(validator))
            .cloned()
    }

    /// Get all stakes for a staker
    pub fn get_stakes_for_staker(&self, staker: &Address) -> Vec<Stake> {
        self.stakes.read()
            .get(staker)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get total staked amount
    pub fn total_staked(&self) -> u64 {
        *self.total_staked.read()
    }

    /// Get current epoch
    pub fn current_epoch(&self) -> u64 {
        *self.current_epoch.read()
    }

    /// Get current validator set
    pub fn validator_set(&self) -> ValidatorSet {
        self.validator_set.read().clone()
    }

    /// Advance to next epoch
    pub fn advance_epoch(&self) -> ValidatorSet {
        let new_epoch = {
            let mut epoch = self.current_epoch.write();
            *epoch += 1;
            *epoch
        };

        // Build new validator set from active validators
        let validators = self.get_active_validators();
        let new_set = ValidatorSet::from_validators(
            new_epoch,
            validators,
            ValidatorSelection::WeightedRandom,
        );

        *self.validator_set.write() = new_set.clone();
        new_set
    }

    /// Distribute rewards to validators
    pub fn distribute_rewards(&self, total_rewards: u64) {
        // First, collect reward calculations
        let reward_data: Vec<(Address, u64, u64)> = {
            let validators = self.validators.read();
            let set = self.validator_set.read();
            
            if set.total_voting_power == 0 {
                return;
            }

            set.validators.iter()
                .filter_map(|addr| {
                    let power = set.voting_power.get(addr)?;
                    let share = (total_rewards as u128 * *power as u128 
                        / set.total_voting_power as u128) as u64;
                    let validator = validators.get(addr)?;
                    let commission = (share as u128 * validator.commission_bps as u128 / 10000) as u64;
                    let delegator_share = share.saturating_sub(commission);
                    Some((*addr, commission, delegator_share))
                })
                .collect()
        };

        // Apply rewards
        for (addr, commission, delegator_share) in reward_data {
            {
                let mut validators = self.validators.write();
                if let Some(v) = validators.get_mut(&addr) {
                    v.add_rewards(commission);
                }
            }
            self.distribute_to_delegators(&addr, delegator_share);
        }
    }

    /// Distribute rewards to delegators of a validator
    fn distribute_to_delegators(&self, validator_addr: &Address, amount: u64) {
        let stakes = self.stakes.read();
        
        // Calculate total delegations
        let mut total_delegated = 0u64;
        let mut delegators = Vec::new();
        
        for (staker, staker_stakes) in stakes.iter() {
            if let Some(stake) = staker_stakes.get(validator_addr) {
                if stake.status == StakeStatus::Bonded {
                    total_delegated += stake.amount;
                    delegators.push((*staker, stake.amount));
                }
            }
        }
        
        if total_delegated == 0 {
            return;
        }
        
        drop(stakes);
        
        // Distribute proportionally
        let mut stakes = self.stakes.write();
        for (staker, stake_amount) in delegators {
            let share = (amount as u128 * stake_amount as u128 / total_delegated as u128) as u64;
            if let Some(staker_stakes) = stakes.get_mut(&staker) {
                if let Some(stake) = staker_stakes.get_mut(validator_addr) {
                    stake.add_rewards(share);
                }
            }
        }
    }

    /// Claim rewards for a stake
    pub fn claim_rewards(&self, staker: Address, validator_addr: Address) -> Result<u64, StakingError> {
        let mut stakes = self.stakes.write();
        let staker_stakes = stakes.get_mut(&staker)
            .ok_or(StakingError::StakeNotFound)?;
        
        let stake = staker_stakes.get_mut(&validator_addr)
            .ok_or(StakingError::StakeNotFound)?;

        Ok(stake.withdraw_rewards())
    }

    /// Get config
    pub fn config(&self) -> &StakingConfig {
        &self.config
    }

    /// Get statistics
    pub fn stats(&self) -> StakingStats {
        let validators = self.validators.read();
        let set = self.validator_set.read();
        
        StakingStats {
            total_validators: validators.len() as u64,
            active_validators: validators.values()
                .filter(|v| v.status == ValidatorStatus::Active)
                .count() as u64,
            total_staked: *self.total_staked.read(),
            current_epoch: *self.current_epoch.read(),
            total_voting_power: set.total_voting_power,
        }
    }
}

/// Staking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStats {
    pub total_validators: u64,
    pub active_validators: u64,
    pub total_staked: u64,
    pub current_epoch: u64,
    pub total_voting_power: u64,
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
    fn test_registry_creation() {
        let registry = StakingRegistry::with_defaults();
        assert_eq!(registry.total_staked(), 0);
        assert_eq!(registry.current_epoch(), 0);
    }

    #[test]
    fn test_register_validator() {
        let registry = StakingRegistry::with_defaults();
        let addr = Address([1u8; 20]);
        
        let result = registry.register_validator(
            addr,
            addr,
            [0u8; 32],
            MIN_VALIDATOR_STAKE,
            500,
            "Test Validator".to_string(),
        );
        
        assert!(result.is_ok());
        assert!(registry.get_validator(&addr).is_some());
        assert_eq!(registry.total_staked(), MIN_VALIDATOR_STAKE);
    }

    #[test]
    fn test_insufficient_stake() {
        let registry = StakingRegistry::with_defaults();
        let addr = Address([2u8; 20]);
        
        let result = registry.register_validator(
            addr,
            addr,
            [0u8; 32],
            1000, // Too low
            500,
            "Test".to_string(),
        );
        
        assert!(matches!(result, Err(StakingError::InsufficientStake { .. })));
    }

    #[test]
    fn test_delegation() {
        let registry = StakingRegistry::with_defaults();
        let validator_addr = Address([1u8; 20]);
        let delegator_addr = Address([2u8; 20]);
        
        // Register validator
        registry.register_validator(
            validator_addr,
            validator_addr,
            [0u8; 32],
            MIN_VALIDATOR_STAKE,
            500,
            "Validator".to_string(),
        ).unwrap();
        
        registry.activate_validator(&validator_addr).unwrap();
        
        // Delegate
        let result = registry.delegate(delegator_addr, validator_addr, MIN_DELEGATION_STAKE);
        assert!(result.is_ok());
        
        let validator = registry.get_validator(&validator_addr).unwrap();
        assert_eq!(validator.delegated_stake, MIN_DELEGATION_STAKE);
    }

    #[test]
    fn test_epoch_advance() {
        let registry = StakingRegistry::with_defaults();
        let addr = Address([1u8; 20]);
        
        registry.register_validator(
            addr,
            addr,
            [0u8; 32],
            MIN_VALIDATOR_STAKE,
            500,
            "Test".to_string(),
        ).unwrap();
        
        registry.activate_validator(&addr).unwrap();
        
        let set = registry.advance_epoch();
        assert_eq!(set.epoch, 1);
        assert_eq!(set.len(), 1);
        assert!(set.contains(&addr));
    }
}
