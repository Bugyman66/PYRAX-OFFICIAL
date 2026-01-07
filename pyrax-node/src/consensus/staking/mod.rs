//! Stream C: Staking and Validation
//!
//! Implements Proof-of-Stake consensus with:
//! - Validator registration and staking
//! - Validator set rotation
//! - Checkpoint finality
//! - Slashing for misbehavior
//! - Reward distribution

mod validator;
mod registry;
mod checkpoint;
mod slashing;

pub use validator::{Validator, ValidatorStatus, ValidatorSet, ValidatorSelection};
pub use registry::{StakingRegistry, Stake, StakeStatus, StakingConfig, StakingError};
pub use checkpoint::{Checkpoint, CheckpointManager, CheckpointStatus, FinalityError};
pub use slashing::{SlashingOffense, SlashingRecord, SlashingManager};

use crate::types::{Address, H256};

/// Minimum stake required to become a validator (100,000 PYRAX)
pub const MIN_VALIDATOR_STAKE: u64 = 100_000 * 100_000_000;

/// Minimum stake for delegation (100 PYRAX)
pub const MIN_DELEGATION_STAKE: u64 = 100 * 100_000_000;

/// Maximum validators in active set
pub const MAX_ACTIVE_VALIDATORS: usize = 100;

/// Checkpoint interval in Stream A blocks
pub const CHECKPOINT_INTERVAL: u64 = 100;

/// Unbonding period in seconds (7 days)
pub const UNBONDING_PERIOD_SECS: u64 = 7 * 24 * 60 * 60;

/// Epoch duration in seconds (1 day)
pub const EPOCH_DURATION_SECS: u64 = 24 * 60 * 60;

/// Reward per checkpoint (10 PYRAX base + gas share)
pub const CHECKPOINT_BASE_REWARD: u64 = 10 * 100_000_000;

/// Slashing percentage for double signing (5%)
pub const DOUBLE_SIGN_SLASH_PERCENT: u8 = 5;

/// Slashing percentage for downtime (0.1%)
pub const DOWNTIME_SLASH_PERCENT_TENTHS: u8 = 1; // 0.1%

/// Jail duration for slashed validators (1 day)
pub const JAIL_DURATION_SECS: u64 = 24 * 60 * 60;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MIN_VALIDATOR_STAKE, 100_000 * 100_000_000); // 100k PYRAX
        assert_eq!(MAX_ACTIVE_VALIDATORS, 100);
        assert_eq!(CHECKPOINT_INTERVAL, 100);
    }
}
