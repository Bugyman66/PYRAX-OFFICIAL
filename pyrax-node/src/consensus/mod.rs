//! PYRAX Consensus - TriStream DAG with GHOSTDAG ordering
//!
//! Stream A: BLAKE3 PoW (10s blocks, ASIC-friendly)
//! Stream B: KAWPOW PoW (60s blocks, GPU mining)
//! Stream C: ZK-STARK + PoS (Finality checkpoints)

mod streams;
pub mod blake3_pow;
pub mod kawpow;
pub mod dag;
pub mod staking;

pub use streams::{Stream, TREASURY_GAS_SHARE_PERCENT, TOKEN_DECIMALS, MAX_SUPPLY};
pub use blake3_pow::Blake3Pow;
pub use kawpow::{kawpow_hash, generate_cache, compute_seed, get_epoch, get_cache_size, get_dag_size};
pub use dag::{DagManager, DagError};
pub use staking::{
    Validator, ValidatorStatus, ValidatorSet, ValidatorSelection,
    StakingRegistry, Stake, StakeStatus, StakingConfig, StakingError,
    Checkpoint, CheckpointManager, CheckpointStatus, FinalityError,
    SlashingOffense, SlashingRecord, SlashingManager,
    MIN_VALIDATOR_STAKE, MIN_DELEGATION_STAKE, MAX_ACTIVE_VALIDATORS,
    CHECKPOINT_INTERVAL, UNBONDING_PERIOD_SECS, CHECKPOINT_BASE_REWARD,
};
