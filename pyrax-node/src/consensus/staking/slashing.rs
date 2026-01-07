//! Slashing Conditions
//!
//! Implements slashing for validator misbehavior

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::Address;

type H256 = [u8; 32];
use super::validator::Validator;
use super::{DOUBLE_SIGN_SLASH_PERCENT, DOWNTIME_SLASH_PERCENT_TENTHS, JAIL_DURATION_SECS};

/// Slashing offense types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashingOffense {
    /// Signed two different blocks at same height
    DoubleSign,
    /// Signed two different checkpoints at same height
    DoubleVote,
    /// Extended downtime (missed too many blocks)
    Downtime,
    /// Proposed invalid block
    InvalidBlock,
    /// Equivocation in consensus messages
    Equivocation,
}

impl SlashingOffense {
    /// Get slash percentage for offense
    pub fn slash_percent(&self) -> u8 {
        match self {
            SlashingOffense::DoubleSign => DOUBLE_SIGN_SLASH_PERCENT,
            SlashingOffense::DoubleVote => DOUBLE_SIGN_SLASH_PERCENT,
            SlashingOffense::Downtime => 0, // Handled separately (0.1%)
            SlashingOffense::InvalidBlock => 1,
            SlashingOffense::Equivocation => DOUBLE_SIGN_SLASH_PERCENT,
        }
    }

    /// Whether offense results in tombstoning
    pub fn is_tombstonable(&self) -> bool {
        matches!(self, 
            SlashingOffense::DoubleSign | 
            SlashingOffense::DoubleVote |
            SlashingOffense::Equivocation
        )
    }

    /// Get jail duration for offense
    pub fn jail_duration(&self) -> u64 {
        match self {
            SlashingOffense::DoubleSign => JAIL_DURATION_SECS * 30, // 30 days
            SlashingOffense::DoubleVote => JAIL_DURATION_SECS * 30,
            SlashingOffense::Downtime => JAIL_DURATION_SECS, // 1 day
            SlashingOffense::InvalidBlock => JAIL_DURATION_SECS * 7, // 7 days
            SlashingOffense::Equivocation => JAIL_DURATION_SECS * 30,
        }
    }

    /// Description of the offense
    pub fn description(&self) -> &'static str {
        match self {
            SlashingOffense::DoubleSign => "Signed conflicting blocks at same height",
            SlashingOffense::DoubleVote => "Voted for conflicting checkpoints",
            SlashingOffense::Downtime => "Extended downtime - missed blocks",
            SlashingOffense::InvalidBlock => "Proposed an invalid block",
            SlashingOffense::Equivocation => "Sent conflicting consensus messages",
        }
    }
}

/// Evidence of slashable offense
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvidence {
    /// Type of offense
    pub offense: SlashingOffense,
    /// Validator address
    pub validator: Address,
    /// Block height where offense occurred
    pub height: u64,
    /// First conflicting item hash (e.g., block hash)
    pub evidence_a: [u8; 32],
    /// Second conflicting item hash
    pub evidence_b: [u8; 32],
    /// Signature over first item
    pub signature_a: Vec<u8>,
    /// Signature over second item
    pub signature_b: Vec<u8>,
    /// Timestamp when evidence was submitted
    pub submitted_at: u64,
    /// Reporter address (who submitted evidence)
    pub reporter: Address,
}

impl SlashingEvidence {
    /// Create double-sign evidence
    pub fn double_sign(
        validator: Address,
        height: u64,
        block_a: [u8; 32],
        block_b: [u8; 32],
        sig_a: Vec<u8>,
        sig_b: Vec<u8>,
        reporter: Address,
    ) -> Self {
        Self {
            offense: SlashingOffense::DoubleSign,
            validator,
            height,
            evidence_a: block_a,
            evidence_b: block_b,
            signature_a: sig_a,
            signature_b: sig_b,
            submitted_at: current_timestamp(),
            reporter,
        }
    }

    /// Create double-vote evidence
    pub fn double_vote(
        validator: Address,
        height: u64,
        checkpoint_a: [u8; 32],
        checkpoint_b: [u8; 32],
        sig_a: Vec<u8>,
        sig_b: Vec<u8>,
        reporter: Address,
    ) -> Self {
        Self {
            offense: SlashingOffense::DoubleVote,
            validator,
            height,
            evidence_a: checkpoint_a,
            evidence_b: checkpoint_b,
            signature_a: sig_a,
            signature_b: sig_b,
            submitted_at: current_timestamp(),
            reporter,
        }
    }

    /// Create downtime evidence
    pub fn downtime(
        validator: Address,
        start_height: u64,
        reporter: Address,
    ) -> Self {
        Self {
            offense: SlashingOffense::Downtime,
            validator,
            height: start_height,
            evidence_a: [0u8; 32],
            evidence_b: [0u8; 32],
            signature_a: Vec::new(),
            signature_b: Vec::new(),
            submitted_at: current_timestamp(),
            reporter,
        }
    }

    /// Verify the evidence is valid
    pub fn verify(&self) -> bool {
        match self.offense {
            SlashingOffense::DoubleSign | SlashingOffense::DoubleVote => {
                // Evidence items must be different
                if self.evidence_a == self.evidence_b {
                    return false;
                }
                // In production, verify signatures
                true
            }
            SlashingOffense::Downtime => {
                // Downtime evidence doesn't need conflicting signatures
                true
            }
            SlashingOffense::InvalidBlock | SlashingOffense::Equivocation => {
                self.evidence_a != [0u8; 32]
            }
        }
    }

    /// Compute hash of evidence
    pub fn hash(&self) -> [u8; 32] {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&[self.offense as u8]);
        hasher.update(&self.validator.0);
        hasher.update(&self.height.to_be_bytes());
        hasher.update(&self.evidence_a);
        hasher.update(&self.evidence_b);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(result.as_bytes());
        hash
    }
}

/// Record of a slashing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingRecord {
    /// Evidence hash
    pub evidence_hash: [u8; 32],
    /// Offense type
    pub offense: SlashingOffense,
    /// Validator slashed
    pub validator: Address,
    /// Amount slashed
    pub amount_slashed: u64,
    /// Whether validator was jailed
    pub jailed: bool,
    /// Whether validator was tombstoned
    pub tombstoned: bool,
    /// Jail release time (if jailed)
    pub jail_until: Option<u64>,
    /// Reporter reward
    pub reporter_reward: u64,
    /// Reporter address
    pub reporter: Address,
    /// Timestamp
    pub slashed_at: u64,
    /// Height at which slashing occurred
    pub slashed_height: u64,
}

/// Downtime tracking for a validator
#[derive(Debug, Clone, Default)]
pub struct DowntimeTracker {
    /// Consecutive blocks missed
    pub consecutive_missed: u64,
    /// Total blocks in window
    pub window_blocks: u64,
    /// Missed blocks in window
    pub window_missed: u64,
    /// Last block height checked
    pub last_height: u64,
}

impl DowntimeTracker {
    /// Record a signed block
    pub fn record_signed(&mut self, height: u64) {
        self.consecutive_missed = 0;
        self.last_height = height;
        self.window_blocks += 1;
    }

    /// Record a missed block
    pub fn record_missed(&mut self, height: u64) {
        self.consecutive_missed += 1;
        self.window_missed += 1;
        self.window_blocks += 1;
        self.last_height = height;
    }

    /// Check if should be slashed for downtime
    pub fn should_slash(&self, max_consecutive: u64, max_window_percent: u8) -> bool {
        // Check consecutive missed first
        if self.consecutive_missed >= max_consecutive {
            return true;
        }
        // Only check window percentage after minimum window size (100 blocks)
        if self.window_blocks >= 100 {
            let missed_percent = (self.window_missed * 100) / self.window_blocks;
            if missed_percent >= max_window_percent as u64 {
                return true;
            }
        }
        false
    }

    /// Reset window
    pub fn reset_window(&mut self) {
        self.window_blocks = 0;
        self.window_missed = 0;
    }
}

/// Slashing manager
pub struct SlashingManager {
    /// Slashing records
    records: Arc<RwLock<Vec<SlashingRecord>>>,
    /// Evidence by hash (to prevent duplicates)
    evidence_hashes: Arc<RwLock<HashMap<H256, bool>>>,
    /// Downtime trackers per validator
    downtime: Arc<RwLock<HashMap<Address, DowntimeTracker>>>,
    /// Maximum consecutive missed blocks before slash
    max_consecutive_missed: u64,
    /// Maximum missed percentage in window before slash
    max_missed_percent: u8,
    /// Downtime window size (in blocks)
    downtime_window: u64,
    /// Reporter reward percentage (of slashed amount)
    reporter_reward_percent: u8,
}

impl SlashingManager {
    /// Create new slashing manager
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            evidence_hashes: Arc::new(RwLock::new(HashMap::new())),
            downtime: Arc::new(RwLock::new(HashMap::new())),
            max_consecutive_missed: 100, // 100 consecutive blocks
            max_missed_percent: 50, // 50% in window
            downtime_window: 1000, // 1000 block window
            reporter_reward_percent: 5, // 5% of slashed amount
        }
    }

    /// Submit slashing evidence
    pub fn submit_evidence(
        &self,
        evidence: SlashingEvidence,
        validator: &mut Validator,
    ) -> Result<SlashingRecord, SlashingError> {
        // Verify evidence
        if !evidence.verify() {
            return Err(SlashingError::InvalidEvidence);
        }

        // Check for duplicate
        let evidence_hash = evidence.hash();
        {
            let mut hashes = self.evidence_hashes.write();
            if hashes.contains_key(&evidence_hash) {
                return Err(SlashingError::DuplicateEvidence);
            }
            hashes.insert(evidence_hash, true);
        }

        // Calculate slash amount
        let slash_percent = if evidence.offense == SlashingOffense::Downtime {
            // 0.1% for downtime
            let slash = (validator.self_stake as u128 * DOWNTIME_SLASH_PERCENT_TENTHS as u128 / 1000) as u64;
            slash
        } else {
            let percent = evidence.offense.slash_percent();
            validator.slash(percent)
        };

        // Jail the validator
        let jail_duration = evidence.offense.jail_duration();
        validator.jail(jail_duration);

        // Tombstone if severe offense
        let tombstoned = evidence.offense.is_tombstonable();
        if tombstoned {
            validator.tombstone();
        }

        // Calculate reporter reward
        let reporter_reward = (slash_percent as u128 * self.reporter_reward_percent as u128 / 100) as u64;

        let record = SlashingRecord {
            evidence_hash,
            offense: evidence.offense,
            validator: evidence.validator,
            amount_slashed: slash_percent,
            jailed: true,
            tombstoned,
            jail_until: validator.jail_until,
            reporter_reward,
            reporter: evidence.reporter,
            slashed_at: current_timestamp(),
            slashed_height: evidence.height,
        };

        self.records.write().push(record.clone());

        Ok(record)
    }

    /// Record block signature (or miss) for downtime tracking
    pub fn record_block(
        &self,
        validator: Address,
        height: u64,
        signed: bool,
    ) -> Option<SlashingEvidence> {
        let mut downtime = self.downtime.write();
        let tracker = downtime.entry(validator).or_insert_with(DowntimeTracker::default);

        if signed {
            tracker.record_signed(height);
        } else {
            tracker.record_missed(height);
        }

        // Check if should slash
        if tracker.should_slash(self.max_consecutive_missed, self.max_missed_percent) {
            // Reset tracker
            let evidence = SlashingEvidence::downtime(
                validator,
                height.saturating_sub(tracker.consecutive_missed),
                Address::ZERO, // System-generated
            );
            tracker.consecutive_missed = 0;
            tracker.reset_window();
            return Some(evidence);
        }

        // Reset window periodically
        if tracker.window_blocks >= self.downtime_window {
            tracker.reset_window();
        }

        None
    }

    /// Get all slashing records
    pub fn get_records(&self) -> Vec<SlashingRecord> {
        self.records.read().clone()
    }

    /// Get records for a validator
    pub fn get_records_for_validator(&self, validator: &Address) -> Vec<SlashingRecord> {
        self.records.read()
            .iter()
            .filter(|r| &r.validator == validator)
            .cloned()
            .collect()
    }

    /// Get total slashed amount
    pub fn total_slashed(&self) -> u64 {
        self.records.read().iter().map(|r| r.amount_slashed).sum()
    }

    /// Get downtime tracker for validator
    pub fn get_downtime(&self, validator: &Address) -> Option<DowntimeTracker> {
        self.downtime.read().get(validator).cloned()
    }

    /// Get slashing statistics
    pub fn stats(&self) -> SlashingStats {
        let records = self.records.read();
        
        SlashingStats {
            total_slashing_events: records.len() as u64,
            total_amount_slashed: records.iter().map(|r| r.amount_slashed).sum(),
            double_sign_count: records.iter().filter(|r| r.offense == SlashingOffense::DoubleSign).count() as u64,
            downtime_count: records.iter().filter(|r| r.offense == SlashingOffense::Downtime).count() as u64,
            tombstoned_count: records.iter().filter(|r| r.tombstoned).count() as u64,
        }
    }
}

impl Default for SlashingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Slashing errors
#[derive(Debug, thiserror::Error)]
pub enum SlashingError {
    #[error("Invalid evidence")]
    InvalidEvidence,

    #[error("Duplicate evidence")]
    DuplicateEvidence,

    #[error("Evidence expired")]
    EvidenceExpired,

    #[error("Validator not found")]
    ValidatorNotFound,
}

/// Slashing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingStats {
    pub total_slashing_events: u64,
    pub total_amount_slashed: u64,
    pub double_sign_count: u64,
    pub downtime_count: u64,
    pub tombstoned_count: u64,
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
    use super::super::validator::ValidatorStatus;

    fn make_validator() -> Validator {
        let addr = Address([1u8; 20]);
        let mut v = Validator::new(
            addr,
            addr,
            [0u8; 32],
            1_000_000_000_000, // 10k PYRAX
            500,
            "Test Validator".to_string(),
        );
        v.status = ValidatorStatus::Active;
        v
    }

    #[test]
    fn test_slashing_offense() {
        assert_eq!(SlashingOffense::DoubleSign.slash_percent(), 5);
        assert!(SlashingOffense::DoubleSign.is_tombstonable());
        assert!(!SlashingOffense::Downtime.is_tombstonable());
    }

    #[test]
    fn test_evidence_creation() {
        let addr = Address([1u8; 20]);
        let reporter = Address([2u8; 20]);
        
        let evidence = SlashingEvidence::double_sign(
            addr,
            100,
            [1u8; 32],
            [2u8; 32],
            vec![0u8; 64],
            vec![0u8; 64],
            reporter,
        );

        assert_eq!(evidence.offense, SlashingOffense::DoubleSign);
        assert!(evidence.verify());
    }

    #[test]
    fn test_invalid_evidence() {
        let addr = Address([1u8; 20]);
        
        // Same block hash for both = invalid
        let evidence = SlashingEvidence::double_sign(
            addr,
            100,
            [1u8; 32],
            [1u8; 32], // Same as evidence_a
            vec![0u8; 64],
            vec![0u8; 64],
            Address([0u8; 20]),
        );

        assert!(!evidence.verify());
    }

    #[test]
    fn test_slashing_execution() {
        let manager = SlashingManager::new();
        let mut validator = make_validator();
        let reporter = Address([2u8; 20]);
        
        let evidence = SlashingEvidence::double_sign(
            validator.address,
            100,
            [1u8; 32],
            [2u8; 32],
            vec![0u8; 64],
            vec![0u8; 64],
            reporter,
        );

        let original_stake = validator.self_stake;
        let result = manager.submit_evidence(evidence, &mut validator);
        
        assert!(result.is_ok());
        let record = result.unwrap();
        
        assert!(record.jailed);
        assert!(record.tombstoned);
        assert!(validator.self_stake < original_stake);
        assert_eq!(validator.status, ValidatorStatus::Tombstoned);
    }

    #[test]
    fn test_downtime_tracking() {
        let manager = SlashingManager::new();
        let addr = Address([1u8; 20]);
        
        // Record missed blocks until threshold (100 consecutive)
        let mut triggered = false;
        for i in 0..150 {
            let evidence = manager.record_block(addr, i, false);
            if evidence.is_some() {
                triggered = true;
                // Should trigger around 100 consecutive missed
                assert!(i >= 99, "Triggered too early at {}", i);
                break;
            }
        }
        assert!(triggered, "Should have triggered slashing for downtime");
    }

    #[test]
    fn test_duplicate_evidence() {
        let manager = SlashingManager::new();
        let mut validator = make_validator();
        let reporter = Address([2u8; 20]);
        
        let evidence = SlashingEvidence::double_sign(
            validator.address,
            100,
            [1u8; 32],
            [2u8; 32],
            vec![0u8; 64],
            vec![0u8; 64],
            reporter,
        );

        // First submission should succeed
        assert!(manager.submit_evidence(evidence.clone(), &mut validator).is_ok());
        
        // Second submission should fail
        let mut validator2 = make_validator();
        assert!(matches!(
            manager.submit_evidence(evidence, &mut validator2),
            Err(SlashingError::DuplicateEvidence)
        ));
    }
}
