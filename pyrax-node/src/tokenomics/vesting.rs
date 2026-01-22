//! Vesting Schedules
//!
//! Token vesting with cliff and linear release

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::{TWELVE_MONTHS_SECS, FORTY_EIGHT_MONTHS_SECS, DECIMALS};

/// Vesting type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VestingType {
    /// Immediate release (100% at TGE)
    Immediate,
    /// Cliff then immediate
    CliffImmediate,
    /// Cliff then linear vesting
    CliffLinear,
    /// Linear vesting (no cliff)
    Linear,
    /// Milestone-based release
    Milestone,
    /// Per-block emission
    PerBlock,
    /// Per-attestation emission
    PerAttestation,
}

/// Vesting schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    /// Vesting type
    pub vesting_type: VestingType,
    /// Cliff duration (seconds)
    pub cliff_duration_secs: u64,
    /// Vesting duration after cliff (seconds)
    pub vesting_duration_secs: u64,
    /// TGE release percentage (0-100)
    pub tge_release_percent: u8,
    /// Description
    pub description: String,
}

impl VestingSchedule {
    /// Create immediate release schedule
    pub fn immediate() -> Self {
        Self {
            vesting_type: VestingType::Immediate,
            cliff_duration_secs: 0,
            vesting_duration_secs: 0,
            tge_release_percent: 100,
            description: "100% at TGE".to_string(),
        }
    }

    /// Create presale schedule (30% TGE, 1mo cliff, 12mo linear)
    pub fn presale() -> Self {
        Self {
            vesting_type: VestingType::CliffLinear,
            cliff_duration_secs: super::ONE_MONTH_SECS,
            vesting_duration_secs: super::TWELVE_MONTHS_SECS,
            tge_release_percent: 30,
            description: "30% at TGE, 1 month cliff, 12 month linear vesting".to_string(),
        }
    }

    /// Create BDAG Community schedule (12mo cliff, 12mo linear)
    pub fn bdag_community() -> Self {
        Self {
            vesting_type: VestingType::CliffLinear,
            cliff_duration_secs: TWELVE_MONTHS_SECS,
            vesting_duration_secs: TWELVE_MONTHS_SECS,
            tge_release_percent: 0,
            description: "12 month cliff, 12 month linear vesting".to_string(),
        }
    }

    /// Create Team/Advisors schedule (12mo cliff, 48mo linear)
    pub fn team_advisors() -> Self {
        Self {
            vesting_type: VestingType::CliffLinear,
            cliff_duration_secs: TWELVE_MONTHS_SECS,
            vesting_duration_secs: FORTY_EIGHT_MONTHS_SECS,
            tge_release_percent: 0,
            description: "12 month cliff, 48 month linear vesting".to_string(),
        }
    }

    /// Create milestone-based schedule
    pub fn milestone() -> Self {
        Self {
            vesting_type: VestingType::Milestone,
            cliff_duration_secs: 0,
            vesting_duration_secs: 0,
            tge_release_percent: 0,
            description: "Milestone-based, DAO approval".to_string(),
        }
    }

    /// Create per-block emission schedule
    pub fn per_block() -> Self {
        Self {
            vesting_type: VestingType::PerBlock,
            cliff_duration_secs: 0,
            vesting_duration_secs: 0,
            tge_release_percent: 0,
            description: "Released per block mined".to_string(),
        }
    }

    /// Create per-attestation emission schedule
    pub fn per_attestation() -> Self {
        Self {
            vesting_type: VestingType::PerAttestation,
            cliff_duration_secs: 0,
            vesting_duration_secs: 0,
            tge_release_percent: 0,
            description: "Released per attestation".to_string(),
        }
    }

    /// Calculate vested amount at timestamp
    pub fn calculate_vested(
        &self,
        total_amount: u64,
        start_timestamp: u64,
        current_timestamp: u64,
    ) -> u64 {
        if current_timestamp < start_timestamp {
            return 0;
        }

        let elapsed = current_timestamp - start_timestamp;

        match self.vesting_type {
            VestingType::Immediate => total_amount,
            
            VestingType::CliffImmediate => {
                if elapsed < self.cliff_duration_secs {
                    0
                } else {
                    total_amount
                }
            }
            
            VestingType::CliffLinear => {
                if elapsed < self.cliff_duration_secs {
                    0
                } else {
                    let vesting_elapsed = elapsed - self.cliff_duration_secs;
                    if vesting_elapsed >= self.vesting_duration_secs {
                        total_amount
                    } else {
                        // Linear vesting
                        let vested_ratio = vesting_elapsed as u128 * 1_000_000 
                            / self.vesting_duration_secs as u128;
                        ((total_amount as u128 * vested_ratio) / 1_000_000) as u64
                    }
                }
            }
            
            VestingType::Linear => {
                if elapsed >= self.vesting_duration_secs {
                    total_amount
                } else {
                    let vested_ratio = elapsed as u128 * 1_000_000 
                        / self.vesting_duration_secs as u128;
                    ((total_amount as u128 * vested_ratio) / 1_000_000) as u64
                }
            }
            
            // Milestone and emission-based handled separately
            VestingType::Milestone | VestingType::PerBlock | VestingType::PerAttestation => 0,
        }
    }

    /// Calculate TGE release amount
    pub fn tge_amount(&self, total_amount: u64) -> u64 {
        (total_amount as u128 * self.tge_release_percent as u128 / 100) as u64
    }

    /// Get total vesting duration
    pub fn total_duration(&self) -> u64 {
        self.cliff_duration_secs + self.vesting_duration_secs
    }
}

/// Vesting contract for a beneficiary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingContract {
    /// Contract ID
    pub id: H256,
    /// Beneficiary address
    pub beneficiary: Address,
    /// Total amount
    pub total_amount: u64,
    /// Released amount
    pub released_amount: u64,
    /// Vesting schedule
    pub schedule: VestingSchedule,
    /// Start timestamp (TGE or creation time)
    pub start_timestamp: u64,
    /// Is revocable
    pub revocable: bool,
    /// Is revoked
    pub revoked: bool,
    /// Revoked timestamp
    pub revoked_at: Option<u64>,
    /// Created timestamp
    pub created_at: u64,
    /// Last claim timestamp
    pub last_claim_at: Option<u64>,
    /// Notes
    pub notes: String,
}

impl VestingContract {
    /// Create new vesting contract
    pub fn new(
        beneficiary: Address,
        total_amount: u64,
        schedule: VestingSchedule,
        start_timestamp: u64,
        revocable: bool,
        notes: String,
    ) -> Self {
        let id = Self::generate_id(&beneficiary, total_amount);
        Self {
            id,
            beneficiary,
            total_amount,
            released_amount: 0,
            schedule,
            start_timestamp,
            revocable,
            revoked: false,
            revoked_at: None,
            created_at: current_timestamp(),
            last_claim_at: None,
            notes,
        }
    }

    /// Generate contract ID
    fn generate_id(beneficiary: &Address, amount: u64) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&beneficiary.0);
        hasher.update(&amount.to_le_bytes());
        hasher.update(&current_timestamp().to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Calculate currently vested amount
    pub fn vested_amount(&self) -> u64 {
        if self.revoked {
            return self.released_amount;
        }
        self.schedule.calculate_vested(
            self.total_amount,
            self.start_timestamp,
            current_timestamp(),
        )
    }

    /// Calculate claimable amount
    pub fn claimable_amount(&self) -> u64 {
        self.vested_amount().saturating_sub(self.released_amount)
    }

    /// Claim vested tokens
    pub fn claim(&mut self) -> Result<u64, VestingError> {
        if self.revoked {
            return Err(VestingError::ContractRevoked);
        }

        let claimable = self.claimable_amount();
        if claimable == 0 {
            return Err(VestingError::NothingToClaim);
        }

        self.released_amount += claimable;
        self.last_claim_at = Some(current_timestamp());

        Ok(claimable)
    }

    /// Revoke contract (admin only)
    pub fn revoke(&mut self) -> Result<u64, VestingError> {
        if !self.revocable {
            return Err(VestingError::NotRevocable);
        }
        if self.revoked {
            return Err(VestingError::AlreadyRevoked);
        }

        // Release vested amount, return unvested
        let vested = self.vested_amount();
        let unvested = self.total_amount - vested;

        self.revoked = true;
        self.revoked_at = Some(current_timestamp());
        self.total_amount = vested;

        Ok(unvested)
    }

    /// Get vesting progress (0-100)
    pub fn progress(&self) -> u8 {
        if self.total_amount == 0 {
            return 100;
        }
        ((self.vested_amount() as u128 * 100) / self.total_amount as u128) as u8
    }

    /// Is fully vested
    pub fn is_fully_vested(&self) -> bool {
        self.vested_amount() >= self.total_amount
    }

    /// Is cliff passed
    pub fn cliff_passed(&self) -> bool {
        let elapsed = current_timestamp().saturating_sub(self.start_timestamp);
        elapsed >= self.schedule.cliff_duration_secs
    }

    /// Time until cliff ends (seconds)
    pub fn time_until_cliff(&self) -> u64 {
        if self.cliff_passed() {
            return 0;
        }
        let elapsed = current_timestamp().saturating_sub(self.start_timestamp);
        self.schedule.cliff_duration_secs.saturating_sub(elapsed)
    }

    /// Time until fully vested (seconds)
    pub fn time_until_vested(&self) -> u64 {
        if self.is_fully_vested() {
            return 0;
        }
        let elapsed = current_timestamp().saturating_sub(self.start_timestamp);
        self.schedule.total_duration().saturating_sub(elapsed)
    }
}

/// Vesting manager
pub struct VestingManager {
    /// Vesting contracts by ID
    contracts: Arc<RwLock<HashMap<H256, VestingContract>>>,
    /// Contracts by beneficiary
    by_beneficiary: Arc<RwLock<HashMap<Address, Vec<H256>>>>,
    /// TGE timestamp
    tge_timestamp: Arc<RwLock<u64>>,
    /// Statistics
    stats: Arc<RwLock<VestingStats>>,
}

impl VestingManager {
    /// Create new vesting manager
    pub fn new() -> Self {
        Self {
            contracts: Arc::new(RwLock::new(HashMap::new())),
            by_beneficiary: Arc::new(RwLock::new(HashMap::new())),
            tge_timestamp: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(VestingStats::default())),
        }
    }

    /// Set TGE timestamp
    pub fn set_tge(&self, timestamp: u64) {
        *self.tge_timestamp.write() = timestamp;
    }

    /// Create vesting contract
    pub fn create_contract(
        &self,
        beneficiary: Address,
        total_amount: u64,
        schedule: VestingSchedule,
        revocable: bool,
        notes: String,
    ) -> Result<H256, VestingError> {
        let start = *self.tge_timestamp.read();
        if start == 0 {
            return Err(VestingError::TgeNotSet);
        }

        let contract = VestingContract::new(
            beneficiary,
            total_amount,
            schedule,
            start,
            revocable,
            notes,
        );
        let id = contract.id;

        self.contracts.write().insert(id, contract);
        self.by_beneficiary.write()
            .entry(beneficiary)
            .or_insert_with(Vec::new)
            .push(id);

        self.stats.write().contracts_created += 1;
        self.stats.write().total_locked += total_amount;

        Ok(id)
    }

    /// Claim from contract
    pub fn claim(&self, contract_id: &H256) -> Result<u64, VestingError> {
        let mut contracts = self.contracts.write();
        let contract = contracts.get_mut(contract_id)
            .ok_or_else(|| VestingError::ContractNotFound(*contract_id))?;

        let claimed = contract.claim()?;
        
        self.stats.write().total_claimed += claimed;

        Ok(claimed)
    }

    /// Claim all for beneficiary
    pub fn claim_all(&self, beneficiary: &Address) -> Result<u64, VestingError> {
        let contract_ids = self.by_beneficiary.read()
            .get(beneficiary)
            .cloned()
            .unwrap_or_default();

        let mut total_claimed = 0u64;
        for id in contract_ids {
            if let Ok(claimed) = self.claim(&id) {
                total_claimed += claimed;
            }
        }

        Ok(total_claimed)
    }

    /// Revoke contract
    pub fn revoke(&self, contract_id: &H256) -> Result<u64, VestingError> {
        let mut contracts = self.contracts.write();
        let contract = contracts.get_mut(contract_id)
            .ok_or_else(|| VestingError::ContractNotFound(*contract_id))?;

        let unvested = contract.revoke()?;
        self.stats.write().contracts_revoked += 1;

        Ok(unvested)
    }

    /// Get contract
    pub fn get_contract(&self, id: &H256) -> Option<VestingContract> {
        self.contracts.read().get(id).cloned()
    }

    /// Get beneficiary contracts
    pub fn get_beneficiary_contracts(&self, beneficiary: &Address) -> Vec<VestingContract> {
        let ids = self.by_beneficiary.read()
            .get(beneficiary)
            .cloned()
            .unwrap_or_default();

        let contracts = self.contracts.read();
        ids.iter()
            .filter_map(|id| contracts.get(id).cloned())
            .collect()
    }

    /// Get total claimable for beneficiary
    pub fn total_claimable(&self, beneficiary: &Address) -> u64 {
        self.get_beneficiary_contracts(beneficiary)
            .iter()
            .map(|c| c.claimable_amount())
            .sum()
    }

    /// Get statistics
    pub fn stats(&self) -> VestingStats {
        self.stats.read().clone()
    }
}

impl Default for VestingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Vesting errors
#[derive(Debug, thiserror::Error)]
pub enum VestingError {
    #[error("TGE timestamp not set")]
    TgeNotSet,

    #[error("Contract not found: {0:?}")]
    ContractNotFound(H256),

    #[error("Nothing to claim")]
    NothingToClaim,

    #[error("Contract is not revocable")]
    NotRevocable,

    #[error("Contract already revoked")]
    AlreadyRevoked,

    #[error("Contract revoked")]
    ContractRevoked,

    #[error("Cliff not passed")]
    CliffNotPassed,
}

/// Vesting statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VestingStats {
    pub contracts_created: u64,
    pub contracts_revoked: u64,
    pub total_locked: u64,
    pub total_claimed: u64,
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
    fn test_vesting_schedule_immediate() {
        let schedule = VestingSchedule::immediate();
        let vested = schedule.calculate_vested(1000, 0, 1);
        assert_eq!(vested, 1000);
    }

    #[test]
    fn test_vesting_schedule_cliff_linear() {
        let schedule = VestingSchedule::team_advisors();
        
        // Before cliff
        let vested = schedule.calculate_vested(1000, 0, TWELVE_MONTHS_SECS - 1);
        assert_eq!(vested, 0);
        
        // At cliff
        let vested = schedule.calculate_vested(1000, 0, TWELVE_MONTHS_SECS);
        assert_eq!(vested, 0);
        
        // Halfway through vesting
        let vested = schedule.calculate_vested(
            1000, 
            0, 
            TWELVE_MONTHS_SECS + FORTY_EIGHT_MONTHS_SECS / 2
        );
        assert_eq!(vested, 500);
        
        // Fully vested
        let vested = schedule.calculate_vested(
            1000, 
            0, 
            TWELVE_MONTHS_SECS + FORTY_EIGHT_MONTHS_SECS
        );
        assert_eq!(vested, 1000);
    }

    #[test]
    fn test_vesting_contract() {
        let contract = VestingContract::new(
            Address([1u8; 20]),
            1000 * DECIMALS,
            VestingSchedule::immediate(),
            0,
            false,
            "Test".into(),
        );

        assert_eq!(contract.vested_amount(), 1000 * DECIMALS);
    }

    #[test]
    fn test_vesting_manager() {
        let manager = VestingManager::new();
        manager.set_tge(1000);

        let id = manager.create_contract(
            Address([1u8; 20]),
            1000,
            VestingSchedule::immediate(),
            false,
            "Test".into(),
        ).unwrap();

        assert!(manager.get_contract(&id).is_some());
    }
}
