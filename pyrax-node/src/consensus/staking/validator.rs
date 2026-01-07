//! Validator Management
//!
//! Defines validators and the active validator set

use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::{MIN_VALIDATOR_STAKE, MAX_ACTIVE_VALIDATORS, JAIL_DURATION_SECS};

/// Validator status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    /// Pending activation (stake bonding)
    Pending,
    /// Active in validator set
    Active,
    /// Inactive but not unbonding
    Inactive,
    /// Unbonding stake (waiting period)
    Unbonding,
    /// Jailed for misbehavior
    Jailed,
    /// Permanently tombstoned
    Tombstoned,
}

/// Validator node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Validator address (staking key)
    pub address: Address,
    /// Operator address (can be different from staking key)
    pub operator: Address,
    /// Consensus public key (BLS or Ed25519)
    pub consensus_pubkey: [u8; 32],
    /// Commission rate (basis points, 10000 = 100%)
    pub commission_bps: u16,
    /// Total self-stake
    pub self_stake: u64,
    /// Total delegated stake
    pub delegated_stake: u64,
    /// Current status
    pub status: ValidatorStatus,
    /// Jail release timestamp (if jailed)
    pub jail_until: Option<u64>,
    /// Number of blocks signed in current epoch
    pub blocks_signed: u64,
    /// Number of blocks proposed in current epoch
    pub blocks_proposed: u64,
    /// Total blocks missed (for uptime calculation)
    pub blocks_missed: u64,
    /// Accumulated rewards pending withdrawal
    pub pending_rewards: u64,
    /// Registration timestamp
    pub registered_at: u64,
    /// Last activity timestamp
    pub last_active: u64,
    /// Validator description/name
    pub description: String,
    /// Website URL
    pub website: String,
}

impl Validator {
    /// Create a new validator
    pub fn new(
        address: Address,
        operator: Address,
        consensus_pubkey: [u8; 32],
        self_stake: u64,
        commission_bps: u16,
        description: String,
    ) -> Self {
        let now = current_timestamp();
        Self {
            address,
            operator,
            consensus_pubkey,
            commission_bps: commission_bps.min(10000),
            self_stake,
            delegated_stake: 0,
            status: ValidatorStatus::Pending,
            jail_until: None,
            blocks_signed: 0,
            blocks_proposed: 0,
            blocks_missed: 0,
            pending_rewards: 0,
            registered_at: now,
            last_active: now,
            description,
            website: String::new(),
        }
    }

    /// Total stake (self + delegated)
    pub fn total_stake(&self) -> u64 {
        self.self_stake.saturating_add(self.delegated_stake)
    }

    /// Voting power (proportional to stake)
    pub fn voting_power(&self) -> u64 {
        self.total_stake()
    }

    /// Check if validator can be activated
    pub fn can_activate(&self) -> bool {
        self.status == ValidatorStatus::Pending 
            && self.self_stake >= MIN_VALIDATOR_STAKE
    }

    /// Check if validator is in jail
    pub fn is_jailed(&self) -> bool {
        self.status == ValidatorStatus::Jailed
    }

    /// Check if jail period has ended
    pub fn can_unjail(&self) -> bool {
        match self.jail_until {
            Some(release_time) => current_timestamp() >= release_time,
            None => false,
        }
    }

    /// Jail the validator
    pub fn jail(&mut self, duration_secs: u64) {
        self.status = ValidatorStatus::Jailed;
        self.jail_until = Some(current_timestamp() + duration_secs);
    }

    /// Unjail the validator
    pub fn unjail(&mut self) -> Result<(), &'static str> {
        if !self.can_unjail() {
            return Err("Jail period not ended");
        }
        if self.status == ValidatorStatus::Tombstoned {
            return Err("Validator is tombstoned");
        }
        self.status = ValidatorStatus::Inactive;
        self.jail_until = None;
        Ok(())
    }

    /// Permanently tombstone (cannot unjail)
    pub fn tombstone(&mut self) {
        self.status = ValidatorStatus::Tombstoned;
    }

    /// Calculate uptime percentage
    pub fn uptime_percent(&self) -> f64 {
        let total = self.blocks_signed + self.blocks_missed;
        if total == 0 {
            return 100.0;
        }
        (self.blocks_signed as f64 / total as f64) * 100.0
    }

    /// Add delegation
    pub fn add_delegation(&mut self, amount: u64) {
        self.delegated_stake = self.delegated_stake.saturating_add(amount);
    }

    /// Remove delegation
    pub fn remove_delegation(&mut self, amount: u64) {
        self.delegated_stake = self.delegated_stake.saturating_sub(amount);
    }

    /// Add rewards
    pub fn add_rewards(&mut self, amount: u64) {
        self.pending_rewards = self.pending_rewards.saturating_add(amount);
    }

    /// Withdraw rewards
    pub fn withdraw_rewards(&mut self) -> u64 {
        let rewards = self.pending_rewards;
        self.pending_rewards = 0;
        rewards
    }

    /// Slash stake by percentage
    pub fn slash(&mut self, percent: u8) -> u64 {
        let slash_amount = (self.self_stake as u128 * percent as u128 / 100) as u64;
        self.self_stake = self.self_stake.saturating_sub(slash_amount);
        slash_amount
    }

    /// Record block signed
    pub fn record_signed(&mut self) {
        self.blocks_signed += 1;
        self.last_active = current_timestamp();
    }

    /// Record block missed
    pub fn record_missed(&mut self) {
        self.blocks_missed += 1;
    }

    /// Record block proposed
    pub fn record_proposed(&mut self) {
        self.blocks_proposed += 1;
        self.last_active = current_timestamp();
    }
}

/// Validator selection algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorSelection {
    /// Weighted random by stake
    WeightedRandom,
    /// Round-robin among active validators
    RoundRobin,
    /// Highest stake first
    HighestStake,
}

/// Active validator set for an epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    /// Epoch number
    pub epoch: u64,
    /// Epoch start timestamp
    pub epoch_start: u64,
    /// Active validators (sorted by voting power)
    pub validators: Vec<Address>,
    /// Voting power per validator
    pub voting_power: HashMap<Address, u64>,
    /// Total voting power
    pub total_voting_power: u64,
    /// Current proposer index (for round-robin)
    pub proposer_index: usize,
    /// Selection algorithm
    pub selection: ValidatorSelection,
}

impl ValidatorSet {
    /// Create a new validator set
    pub fn new(epoch: u64, selection: ValidatorSelection) -> Self {
        Self {
            epoch,
            epoch_start: current_timestamp(),
            validators: Vec::new(),
            voting_power: HashMap::new(),
            total_voting_power: 0,
            proposer_index: 0,
            selection,
        }
    }

    /// Build validator set from registry
    pub fn from_validators(
        epoch: u64,
        validators: Vec<Validator>,
        selection: ValidatorSelection,
    ) -> Self {
        let mut set = Self::new(epoch, selection);
        
        // Sort by voting power descending
        let mut sorted: Vec<_> = validators.into_iter()
            .filter(|v| v.status == ValidatorStatus::Active)
            .collect();
        sorted.sort_by(|a, b| b.voting_power().cmp(&a.voting_power()));
        
        // Take top MAX_ACTIVE_VALIDATORS
        for validator in sorted.into_iter().take(MAX_ACTIVE_VALIDATORS) {
            let power = validator.voting_power();
            set.voting_power.insert(validator.address, power);
            set.validators.push(validator.address);
            set.total_voting_power += power;
        }
        
        set
    }

    /// Get current proposer
    pub fn current_proposer(&self) -> Option<Address> {
        self.validators.get(self.proposer_index).copied()
    }

    /// Advance to next proposer
    pub fn next_proposer(&mut self) -> Option<Address> {
        if self.validators.is_empty() {
            return None;
        }
        self.proposer_index = (self.proposer_index + 1) % self.validators.len();
        self.current_proposer()
    }

    /// Select proposer for a given block
    pub fn select_proposer(&self, block_number: u64) -> Option<Address> {
        if self.validators.is_empty() {
            return None;
        }
        
        match self.selection {
            ValidatorSelection::RoundRobin => {
                let index = (block_number as usize) % self.validators.len();
                self.validators.get(index).copied()
            }
            ValidatorSelection::WeightedRandom => {
                // Deterministic weighted selection based on block number
                let seed = block_number;
                self.weighted_select(seed)
            }
            ValidatorSelection::HighestStake => {
                self.validators.first().copied()
            }
        }
    }

    /// Weighted selection by stake
    fn weighted_select(&self, seed: u64) -> Option<Address> {
        if self.total_voting_power == 0 {
            return None;
        }
        
        // Simple deterministic selection
        let target = seed % self.total_voting_power;
        let mut cumulative = 0u64;
        
        for addr in &self.validators {
            if let Some(&power) = self.voting_power.get(addr) {
                cumulative += power;
                if cumulative > target {
                    return Some(*addr);
                }
            }
        }
        
        self.validators.first().copied()
    }

    /// Check if address is in validator set
    pub fn contains(&self, address: &Address) -> bool {
        self.validators.contains(address)
    }

    /// Get voting power for address
    pub fn get_voting_power(&self, address: &Address) -> u64 {
        self.voting_power.get(address).copied().unwrap_or(0)
    }

    /// Number of validators
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// Check if set is empty
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Calculate quorum (2/3 of voting power)
    pub fn quorum(&self) -> u64 {
        (self.total_voting_power * 2) / 3 + 1
    }

    /// Check if votes meet quorum
    pub fn has_quorum(&self, votes: &HashMap<Address, u64>) -> bool {
        let total_votes: u64 = votes.values().sum();
        total_votes >= self.quorum()
    }

    /// Get validators as slice
    pub fn as_slice(&self) -> &[Address] {
        &self.validators
    }
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

    fn make_validator(addr_byte: u8, stake: u64) -> Validator {
        let mut addr_bytes = [0u8; 20];
        addr_bytes[0] = addr_byte;
        let addr = Address(addr_bytes);
        let mut v = Validator::new(
            addr,
            addr,
            [0u8; 32],
            stake,
            500, // 5% commission
            format!("Validator {}", addr_byte),
        );
        v.status = ValidatorStatus::Active;
        v
    }

    #[test]
    fn test_validator_creation() {
        let addr = Address([1u8; 20]);
        let v = Validator::new(addr, addr, [0u8; 32], 1_000_000, 500, "Test".to_string());
        assert_eq!(v.address, addr);
        assert_eq!(v.self_stake, 1_000_000);
        assert_eq!(v.commission_bps, 500);
        assert_eq!(v.status, ValidatorStatus::Pending);
    }

    #[test]
    fn test_validator_stake() {
        let mut v = make_validator(1, 1_000_000);
        v.add_delegation(500_000);
        assert_eq!(v.total_stake(), 1_500_000);
        assert_eq!(v.voting_power(), 1_500_000);
    }

    #[test]
    fn test_validator_slash() {
        let mut v = make_validator(1, 1_000_000);
        let slashed = v.slash(5); // 5%
        assert_eq!(slashed, 50_000);
        assert_eq!(v.self_stake, 950_000);
    }

    #[test]
    fn test_validator_set_creation() {
        let validators = vec![
            make_validator(1, 1_000_000),
            make_validator(2, 2_000_000),
            make_validator(3, 500_000),
        ];
        
        let set = ValidatorSet::from_validators(1, validators, ValidatorSelection::HighestStake);
        
        assert_eq!(set.len(), 3);
        assert_eq!(set.total_voting_power, 3_500_000);
        
        // Should be sorted by stake (highest first)
        let mut addr2_bytes = [0u8; 20];
        addr2_bytes[0] = 2;
        assert_eq!(set.validators[0], Address(addr2_bytes));
    }

    #[test]
    fn test_validator_set_proposer() {
        let validators = vec![
            make_validator(1, 1_000_000),
            make_validator(2, 1_000_000),
            make_validator(3, 1_000_000),
        ];
        
        let set = ValidatorSet::from_validators(1, validators, ValidatorSelection::RoundRobin);
        
        // Round-robin selection
        assert!(set.select_proposer(0).is_some());
        assert!(set.select_proposer(1).is_some());
        assert!(set.select_proposer(2).is_some());
        
        // Should cycle
        assert_eq!(set.select_proposer(0), set.select_proposer(3));
    }

    #[test]
    fn test_quorum_calculation() {
        let validators = vec![
            make_validator(1, 100),
            make_validator(2, 100),
            make_validator(3, 100),
        ];
        
        let set = ValidatorSet::from_validators(1, validators, ValidatorSelection::RoundRobin);
        
        // 2/3 of 300 = 200, so quorum = 201
        assert_eq!(set.quorum(), 201);
    }
}
