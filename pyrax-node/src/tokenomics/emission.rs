//! Token Emission Schedules
//!
//! Mining rewards and ZK prover rewards with halving

use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::Address;
use super::{
    MINING_REWARDS_ALLOCATION, ZK_PROVER_ALLOCATION,
    HALVING_INTERVAL_BLOCKS, INITIAL_BLOCK_REWARD,
    ZK_PROVER_BURN_RATE_BPS, DECIMALS,
};

/// Emission schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionSchedule {
    /// Initial reward per unit (block or attestation)
    pub initial_reward: u64,
    /// Halving interval (in blocks)
    pub halving_interval: u64,
    /// Total allocation for this emission
    pub total_allocation: u64,
    /// Current epoch (halving count)
    pub current_epoch: u32,
    /// Total emitted
    pub total_emitted: u64,
    /// Burn rate (basis points, 0-10000)
    pub burn_rate_bps: u16,
}

impl EmissionSchedule {
    /// Calculate reward at given block height
    pub fn reward_at_height(&self, height: u64) -> u64 {
        let halvings = (height / self.halving_interval) as u32;
        self.initial_reward >> halvings
    }

    /// Calculate remaining allocation
    pub fn remaining(&self) -> u64 {
        self.total_allocation.saturating_sub(self.total_emitted)
    }

    /// Calculate burn amount from reward
    pub fn burn_amount(&self, reward: u64) -> u64 {
        if self.burn_rate_bps == 0 {
            return 0;
        }
        (reward as u128 * self.burn_rate_bps as u128 / 10000) as u64
    }

    /// Calculate net reward after burn
    pub fn net_reward(&self, reward: u64) -> u64 {
        reward - self.burn_amount(reward)
    }

    /// Is allocation exhausted
    pub fn is_exhausted(&self) -> bool {
        self.remaining() == 0
    }
}

/// Mining rewards manager
/// Handles Stream A (ASIC) and Stream B (CPU/GPU) rewards
pub struct MiningRewards {
    /// Total allocation
    pub total_allocation: u64,
    /// Stream A (ASIC) allocation (60%)
    pub stream_a_allocation: u64,
    /// Stream B (CPU/GPU) allocation (40%)
    pub stream_b_allocation: u64,
    /// Stream A emitted
    stream_a_emitted: Arc<RwLock<u64>>,
    /// Stream B emitted
    stream_b_emitted: Arc<RwLock<u64>>,
    /// Current block height
    current_height: Arc<RwLock<u64>>,
    /// Halving interval
    pub halving_interval: u64,
    /// Initial Stream A block reward
    pub initial_stream_a_reward: u64,
    /// Initial Stream B block reward
    pub initial_stream_b_reward: u64,
    /// Total blocks mined
    blocks_mined: Arc<RwLock<u64>>,
    /// Statistics
    stats: Arc<RwLock<MiningStats>>,
}

impl MiningRewards {
    /// Create new mining rewards manager
    pub fn new() -> Self {
        // 60% ASIC (Stream A), 40% CPU/GPU (Stream B)
        let stream_a = (MINING_REWARDS_ALLOCATION as u128 * 60 / 100) as u64;
        let stream_b = MINING_REWARDS_ALLOCATION - stream_a;

        // Initial rewards split 60/40
        let initial_a = (INITIAL_BLOCK_REWARD as u128 * 60 / 100) as u64;
        let initial_b = INITIAL_BLOCK_REWARD - initial_a;

        Self {
            total_allocation: MINING_REWARDS_ALLOCATION,
            stream_a_allocation: stream_a,
            stream_b_allocation: stream_b,
            stream_a_emitted: Arc::new(RwLock::new(0)),
            stream_b_emitted: Arc::new(RwLock::new(0)),
            current_height: Arc::new(RwLock::new(0)),
            halving_interval: HALVING_INTERVAL_BLOCKS,
            initial_stream_a_reward: initial_a,
            initial_stream_b_reward: initial_b,
            blocks_mined: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(MiningStats::default())),
        }
    }

    /// Get current halving epoch
    pub fn current_epoch(&self) -> u32 {
        (*self.current_height.read() / self.halving_interval) as u32
    }

    /// Get Stream A reward at height
    pub fn stream_a_reward(&self, height: u64) -> u64 {
        let halvings = (height / self.halving_interval) as u32;
        let reward = self.initial_stream_a_reward >> halvings;
        
        // Cap at remaining allocation
        let remaining = self.stream_a_allocation.saturating_sub(*self.stream_a_emitted.read());
        reward.min(remaining)
    }

    /// Get Stream B reward at height
    pub fn stream_b_reward(&self, height: u64) -> u64 {
        let halvings = (height / self.halving_interval) as u32;
        let reward = self.initial_stream_b_reward >> halvings;
        
        // Cap at remaining allocation
        let remaining = self.stream_b_allocation.saturating_sub(*self.stream_b_emitted.read());
        reward.min(remaining)
    }

    /// Get total block reward at height
    pub fn total_reward(&self, height: u64) -> u64 {
        self.stream_a_reward(height) + self.stream_b_reward(height)
    }

    /// Process block reward
    pub fn process_block(
        &self,
        height: u64,
        asic_miner: Option<Address>,
        gpu_miner: Option<Address>,
    ) -> BlockReward {
        *self.current_height.write() = height;
        *self.blocks_mined.write() += 1;

        let stream_a = if asic_miner.is_some() {
            let reward = self.stream_a_reward(height);
            *self.stream_a_emitted.write() += reward;
            reward
        } else {
            0
        };

        let stream_b = if gpu_miner.is_some() {
            let reward = self.stream_b_reward(height);
            *self.stream_b_emitted.write() += reward;
            reward
        } else {
            0
        };

        let total = stream_a + stream_b;
        
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.total_emitted += total;
            stats.blocks_processed += 1;
        }

        BlockReward {
            height,
            stream_a_reward: stream_a,
            stream_b_reward: stream_b,
            stream_a_miner: asic_miner,
            stream_b_miner: gpu_miner,
            total_reward: total,
            epoch: self.current_epoch(),
        }
    }

    /// Get total emitted
    pub fn total_emitted(&self) -> u64 {
        *self.stream_a_emitted.read() + *self.stream_b_emitted.read()
    }

    /// Get remaining allocation
    pub fn remaining(&self) -> u64 {
        self.total_allocation.saturating_sub(self.total_emitted())
    }

    /// Get emission percentage
    pub fn emission_percent(&self) -> f64 {
        (self.total_emitted() as f64 / self.total_allocation as f64) * 100.0
    }

    /// Estimate blocks until next halving
    pub fn blocks_until_halving(&self) -> u64 {
        let current = *self.current_height.read();
        let next_halving = ((current / self.halving_interval) + 1) * self.halving_interval;
        next_halving - current
    }

    /// Get statistics
    pub fn stats(&self) -> MiningStats {
        self.stats.read().clone()
    }
}

impl Default for MiningRewards {
    fn default() -> Self {
        Self::new()
    }
}

/// Block reward details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockReward {
    /// Block height
    pub height: u64,
    /// Stream A (ASIC) reward
    pub stream_a_reward: u64,
    /// Stream B (CPU/GPU) reward
    pub stream_b_reward: u64,
    /// Stream A miner
    pub stream_a_miner: Option<Address>,
    /// Stream B miner
    pub stream_b_miner: Option<Address>,
    /// Total reward
    pub total_reward: u64,
    /// Halving epoch
    pub epoch: u32,
}

/// Mining statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiningStats {
    pub blocks_processed: u64,
    pub total_emitted: u64,
    pub stream_a_total: u64,
    pub stream_b_total: u64,
}

/// ZK Prover rewards manager
/// Handles Stream C rewards for zero-knowledge proof verification
pub struct ZkProverRewards {
    /// Total allocation
    pub total_allocation: u64,
    /// Total emitted (before burn)
    total_emitted: Arc<RwLock<u64>>,
    /// Total burned
    total_burned: Arc<RwLock<u64>>,
    /// Burn rate (basis points)
    pub burn_rate_bps: u16,
    /// Base reward per attestation
    pub base_attestation_reward: u64,
    /// Attestations processed
    attestations: Arc<RwLock<u64>>,
    /// Statistics
    stats: Arc<RwLock<ZkProverStats>>,
}

impl ZkProverRewards {
    /// Create new ZK prover rewards manager
    pub fn new() -> Self {
        // Base attestation reward (~100 PYRAX)
        let base_reward = 100 * DECIMALS;

        Self {
            total_allocation: ZK_PROVER_ALLOCATION,
            total_emitted: Arc::new(RwLock::new(0)),
            total_burned: Arc::new(RwLock::new(0)),
            burn_rate_bps: ZK_PROVER_BURN_RATE_BPS,
            base_attestation_reward: base_reward,
            attestations: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(ZkProverStats::default())),
        }
    }

    /// Calculate reward for attestation
    pub fn calculate_reward(&self, proof_complexity: u64) -> u64 {
        // Base reward scaled by complexity (1-10)
        let complexity_multiplier = proof_complexity.clamp(1, 10);
        let reward = self.base_attestation_reward * complexity_multiplier;
        
        // Cap at remaining allocation
        let remaining = self.remaining();
        reward.min(remaining)
    }

    /// Calculate burn amount
    pub fn burn_amount(&self, reward: u64) -> u64 {
        (reward as u128 * self.burn_rate_bps as u128 / 10000) as u64
    }

    /// Calculate net reward after burn
    pub fn net_reward(&self, reward: u64) -> u64 {
        reward - self.burn_amount(reward)
    }

    /// Process attestation reward
    pub fn process_attestation(
        &self,
        prover: Address,
        proof_complexity: u64,
    ) -> AttestationReward {
        let gross_reward = self.calculate_reward(proof_complexity);
        let burn = self.burn_amount(gross_reward);
        let net = gross_reward - burn;

        // Update totals
        *self.total_emitted.write() += gross_reward;
        *self.total_burned.write() += burn;
        *self.attestations.write() += 1;

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.attestations_processed += 1;
            stats.total_rewarded += net;
            stats.total_burned += burn;
        }

        AttestationReward {
            prover,
            gross_reward,
            burn_amount: burn,
            net_reward: net,
            proof_complexity,
        }
    }

    /// Get total emitted
    pub fn total_emitted(&self) -> u64 {
        *self.total_emitted.read()
    }

    /// Get total burned
    pub fn total_burned(&self) -> u64 {
        *self.total_burned.read()
    }

    /// Get remaining allocation
    pub fn remaining(&self) -> u64 {
        self.total_allocation.saturating_sub(self.total_emitted())
    }

    /// Get emission percentage
    pub fn emission_percent(&self) -> f64 {
        (self.total_emitted() as f64 / self.total_allocation as f64) * 100.0
    }

    /// Get attestation count
    pub fn attestation_count(&self) -> u64 {
        *self.attestations.read()
    }

    /// Get statistics
    pub fn stats(&self) -> ZkProverStats {
        self.stats.read().clone()
    }
}

impl Default for ZkProverRewards {
    fn default() -> Self {
        Self::new()
    }
}

/// Attestation reward details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationReward {
    /// Prover address
    pub prover: Address,
    /// Gross reward (before burn)
    pub gross_reward: u64,
    /// Burn amount (10%)
    pub burn_amount: u64,
    /// Net reward (after burn)
    pub net_reward: u64,
    /// Proof complexity
    pub proof_complexity: u64,
}

/// ZK Prover statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZkProverStats {
    pub attestations_processed: u64,
    pub total_rewarded: u64,
    pub total_burned: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_rewards() {
        let rewards = MiningRewards::new();
        
        // Initial reward
        let reward = rewards.total_reward(0);
        assert_eq!(reward, INITIAL_BLOCK_REWARD);

        // After one halving
        let reward = rewards.total_reward(HALVING_INTERVAL_BLOCKS);
        assert_eq!(reward, INITIAL_BLOCK_REWARD / 2);

        // After two halvings
        let reward = rewards.total_reward(HALVING_INTERVAL_BLOCKS * 2);
        assert_eq!(reward, INITIAL_BLOCK_REWARD / 4);
    }

    #[test]
    fn test_stream_split() {
        let rewards = MiningRewards::new();
        
        let stream_a = rewards.stream_a_reward(0);
        let stream_b = rewards.stream_b_reward(0);
        
        // 60/40 split
        assert!(stream_a > stream_b);
        assert_eq!(stream_a + stream_b, INITIAL_BLOCK_REWARD);
    }

    #[test]
    fn test_process_block() {
        let rewards = MiningRewards::new();
        
        let block_reward = rewards.process_block(
            1,
            Some(Address([1u8; 20])),
            Some(Address([2u8; 20])),
        );

        assert_eq!(block_reward.height, 1);
        assert!(block_reward.total_reward > 0);
    }

    #[test]
    fn test_zk_prover_rewards() {
        let rewards = ZkProverRewards::new();
        
        let base = rewards.base_attestation_reward;
        assert_eq!(base, 100 * DECIMALS);
    }

    #[test]
    fn test_zk_burn() {
        let rewards = ZkProverRewards::new();
        
        let reward = 1000 * DECIMALS;
        let burn = rewards.burn_amount(reward);
        
        // 10% burn
        assert_eq!(burn, 100 * DECIMALS);
    }

    #[test]
    fn test_attestation_reward() {
        let rewards = ZkProverRewards::new();
        
        let attestation = rewards.process_attestation(
            Address([1u8; 20]),
            5, // complexity
        );

        assert!(attestation.gross_reward > 0);
        assert_eq!(attestation.net_reward, attestation.gross_reward - attestation.burn_amount);
    }

    #[test]
    fn test_complexity_scaling() {
        let rewards = ZkProverRewards::new();
        
        let low = rewards.calculate_reward(1);
        let high = rewards.calculate_reward(10);
        
        assert_eq!(high, low * 10);
    }
}
