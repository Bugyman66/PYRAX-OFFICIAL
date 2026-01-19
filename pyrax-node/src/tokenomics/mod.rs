//! PYRAX Tokenomics
//!
//! Total Supply: 100,000,000,000 PYRAX (100 Billion)
//! Smallest Unit: 1 PYRAX = 100,000,000 units (8 decimals)
//!
//! Pool Allocations:
//! - Presale: 15% (15B) - 30% at TGE, 1 month cliff, 12 month linear vesting
//! - BDAG Community: 10% (10B) - 12 month cliff, 12 month linear vesting
//! - Mining Rewards: 35% (35B) - Released per block mined
//! - ZK Prover Rewards: 5% (5B) - Released per attestation, 10% burned
//! - Team & Founders: 4% (4B) - 12 month cliff, 48 month linear vesting
//! - Advisors: 3% (3B) - 12 month cliff, 48 month linear vesting
//! - Ecosystem: 10% (10B) - Milestone-based, DAO approval
//! - Marketing: 5% (5B) - As needed, multi-sig approval
//! - Liquidity: 10% (10B) - 100% at TGE
//! - Treasury: 2% (2B) - 75% DAO approval required
//! - Reserve: 1% (1B) - Protocol buffer

#![allow(dead_code)]

mod allocation;
mod vesting;
mod emission;

pub use allocation::{TokenAllocation, AllocationPool, AllocationManager};
pub use vesting::{VestingSchedule, VestingType, VestingContract};
pub use emission::{EmissionSchedule, MiningRewards, ZkProverRewards};

/// Total supply of PYRAX tokens (smallest unit)
pub const TOTAL_SUPPLY: u64 = 100_000_000_000 * DECIMALS;

/// Decimal places (8 decimals like Bitcoin)
pub const DECIMALS: u64 = 100_000_000;

/// Token symbol
pub const SYMBOL: &str = "PYRAX";

/// Token name
pub const NAME: &str = "PYRAX";

/// Presale allocation: 15% (15,000,000,000 PYRAX)
pub const PRESALE_ALLOCATION: u64 = 15_000_000_000 * DECIMALS;
pub const PRESALE_PERCENT: u8 = 15;

/// BDAG Community allocation: 10% (10,000,000,000 PYRAX)
pub const BDAG_COMMUNITY_ALLOCATION: u64 = 10_000_000_000 * DECIMALS;
pub const BDAG_COMMUNITY_PERCENT: u8 = 10;

/// Mining rewards allocation: 35% (35,000,000,000 PYRAX)
pub const MINING_REWARDS_ALLOCATION: u64 = 35_000_000_000 * DECIMALS;
pub const MINING_REWARDS_PERCENT: u8 = 35;

/// ZK Prover rewards allocation: 5% (5,000,000,000 PYRAX)
pub const ZK_PROVER_ALLOCATION: u64 = 5_000_000_000 * DECIMALS;
pub const ZK_PROVER_PERCENT: u8 = 5;

/// Team & Founders allocation: 4% (4,000,000,000 PYRAX)
pub const TEAM_ALLOCATION: u64 = 4_000_000_000 * DECIMALS;
pub const TEAM_PERCENT: u8 = 4;

/// Advisors allocation: 3% (3,000,000,000 PYRAX)
pub const ADVISORS_ALLOCATION: u64 = 3_000_000_000 * DECIMALS;
pub const ADVISORS_PERCENT: u8 = 3;

/// Ecosystem allocation: 10% (10,000,000,000 PYRAX)
pub const ECOSYSTEM_ALLOCATION: u64 = 10_000_000_000 * DECIMALS;
pub const ECOSYSTEM_PERCENT: u8 = 10;

/// Marketing allocation: 5% (5,000,000,000 PYRAX)
pub const MARKETING_ALLOCATION: u64 = 5_000_000_000 * DECIMALS;
pub const MARKETING_PERCENT: u8 = 5;

/// Liquidity allocation: 10% (10,000,000,000 PYRAX)
pub const LIQUIDITY_ALLOCATION: u64 = 10_000_000_000 * DECIMALS;
pub const LIQUIDITY_PERCENT: u8 = 10;

/// Treasury allocation: 2% (2,000,000,000 PYRAX)
pub const TREASURY_ALLOCATION: u64 = 2_000_000_000 * DECIMALS;
pub const TREASURY_PERCENT: u8 = 2;

/// Reserve allocation: 1% (1,000,000,000 PYRAX)
pub const RESERVE_ALLOCATION: u64 = 1_000_000_000 * DECIMALS;
pub const RESERVE_PERCENT: u8 = 1;

/// Mining halving interval (approximately 4 years in blocks at 6 sec block time)
pub const HALVING_INTERVAL_BLOCKS: u64 = 21_000_000; // ~4 years at 6 sec blocks

/// Initial block reward (before halving)
pub const INITIAL_BLOCK_REWARD: u64 = 1666 * DECIMALS; // ~1666 PYRAX per block

/// ZK Prover burn rate (10% of rewards burned)
pub const ZK_PROVER_BURN_RATE_BPS: u16 = 1000; // 10% in basis points

/// Treasury DAO approval threshold
pub const TREASURY_DAO_APPROVAL_BPS: u16 = 7500; // 75%

/// Vesting cliff periods (in seconds)
pub const ONE_MONTH_SECS: u64 = 30 * 24 * 60 * 60;
pub const THREE_MONTHS_SECS: u64 = 90 * 24 * 60 * 60;
pub const SIX_MONTHS_SECS: u64 = 180 * 24 * 60 * 60;
pub const NINE_MONTHS_SECS: u64 = 270 * 24 * 60 * 60;
pub const TWELVE_MONTHS_SECS: u64 = 365 * 24 * 60 * 60;
pub const FORTY_EIGHT_MONTHS_SECS: u64 = 4 * 365 * 24 * 60 * 60;

/// Presale TGE release percentage
pub const PRESALE_TGE_PERCENT: u8 = 30;

/// Presale cliff (1 month)
pub const PRESALE_CLIFF_SECS: u64 = ONE_MONTH_SECS;

/// Presale vesting duration (12 months after cliff)
pub const PRESALE_VESTING_SECS: u64 = TWELVE_MONTHS_SECS;

/// Get total of all allocations (should equal TOTAL_SUPPLY)
pub fn total_allocations() -> u64 {
    PRESALE_ALLOCATION
        + BDAG_COMMUNITY_ALLOCATION
        + MINING_REWARDS_ALLOCATION
        + ZK_PROVER_ALLOCATION
        + TEAM_ALLOCATION
        + ADVISORS_ALLOCATION
        + ECOSYSTEM_ALLOCATION
        + MARKETING_ALLOCATION
        + LIQUIDITY_ALLOCATION
        + TREASURY_ALLOCATION
        + RESERVE_ALLOCATION
}

/// Verify tokenomics integrity
pub fn verify_tokenomics() -> bool {
    total_allocations() == TOTAL_SUPPLY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_supply() {
        assert_eq!(TOTAL_SUPPLY, 100_000_000_000 * DECIMALS);
    }

    #[test]
    fn test_allocations_sum() {
        assert_eq!(total_allocations(), TOTAL_SUPPLY);
    }

    #[test]
    fn test_percentages_sum() {
        let total_percent = PRESALE_PERCENT
            + BDAG_COMMUNITY_PERCENT
            + MINING_REWARDS_PERCENT
            + ZK_PROVER_PERCENT
            + TEAM_PERCENT
            + ADVISORS_PERCENT
            + ECOSYSTEM_PERCENT
            + MARKETING_PERCENT
            + LIQUIDITY_PERCENT
            + TREASURY_PERCENT
            + RESERVE_PERCENT;
        assert_eq!(total_percent, 100);
    }

    #[test]
    fn test_verify_tokenomics() {
        assert!(verify_tokenomics());
    }

    #[test]
    fn test_individual_allocations() {
        assert_eq!(PRESALE_ALLOCATION, 15_000_000_000 * DECIMALS);
        assert_eq!(BDAG_COMMUNITY_ALLOCATION, 10_000_000_000 * DECIMALS);
        assert_eq!(MINING_REWARDS_ALLOCATION, 35_000_000_000 * DECIMALS);
        assert_eq!(ZK_PROVER_ALLOCATION, 5_000_000_000 * DECIMALS);
        assert_eq!(TEAM_ALLOCATION, 4_000_000_000 * DECIMALS);
        assert_eq!(ADVISORS_ALLOCATION, 3_000_000_000 * DECIMALS);
        assert_eq!(ECOSYSTEM_ALLOCATION, 10_000_000_000 * DECIMALS);
        assert_eq!(MARKETING_ALLOCATION, 5_000_000_000 * DECIMALS);
        assert_eq!(LIQUIDITY_ALLOCATION, 10_000_000_000 * DECIMALS);
        assert_eq!(TREASURY_ALLOCATION, 2_000_000_000 * DECIMALS);
        assert_eq!(RESERVE_ALLOCATION, 1_000_000_000 * DECIMALS);
    }
}
