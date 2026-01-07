//! Gas Fee Distribution
//!
//! Distributes gas fees across streams according to the PYRAX tokenomics:
//! - Stream A (ASIC miners): 20%
//! - Stream B (GPU miners): 40%
//! - Stream C (Stakers): 30%
//! - Treasury: 10%

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use crate::consensus::Stream;

/// Stream share percentages
pub const STREAM_A_SHARE_PERCENT: u8 = 20;
pub const STREAM_B_SHARE_PERCENT: u8 = 40;
pub const STREAM_C_SHARE_PERCENT: u8 = 30;
pub const TREASURY_SHARE_PERCENT: u8 = 10;

/// Stream share configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamShare {
    /// Stream identifier
    pub stream: Stream,
    /// Percentage share (0-100)
    pub percent: u8,
}

impl StreamShare {
    pub fn stream_a() -> Self {
        Self { stream: Stream::A, percent: STREAM_A_SHARE_PERCENT }
    }

    pub fn stream_b() -> Self {
        Self { stream: Stream::B, percent: STREAM_B_SHARE_PERCENT }
    }

    pub fn stream_c() -> Self {
        Self { stream: Stream::C, percent: STREAM_C_SHARE_PERCENT }
    }
}

/// Fee allocation for a single distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeAllocation {
    /// Block height (EVM)
    pub block_height: u64,
    /// Block hash
    pub block_hash: H256,
    /// Total fees collected
    pub total_fees: u64,
    /// Amount to Stream A miners
    pub stream_a_amount: u64,
    /// Amount to Stream B miners
    pub stream_b_amount: u64,
    /// Amount to Stream C stakers
    pub stream_c_amount: u64,
    /// Amount to treasury
    pub treasury_amount: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Whether distributed
    pub distributed: bool,
}

impl FeeAllocation {
    /// Create new allocation from total fees
    pub fn new(block_height: u64, block_hash: H256, total_fees: u64) -> Self {
        let stream_a_amount = (total_fees as u128 * STREAM_A_SHARE_PERCENT as u128 / 100) as u64;
        let stream_b_amount = (total_fees as u128 * STREAM_B_SHARE_PERCENT as u128 / 100) as u64;
        let stream_c_amount = (total_fees as u128 * STREAM_C_SHARE_PERCENT as u128 / 100) as u64;
        let treasury_amount = total_fees
            .saturating_sub(stream_a_amount)
            .saturating_sub(stream_b_amount)
            .saturating_sub(stream_c_amount);

        Self {
            block_height,
            block_hash,
            total_fees,
            stream_a_amount,
            stream_b_amount,
            stream_c_amount,
            treasury_amount,
            timestamp: current_timestamp(),
            distributed: false,
        }
    }

    /// Get amount for a specific stream
    pub fn amount_for_stream(&self, stream: Stream) -> u64 {
        match stream {
            Stream::A => self.stream_a_amount,
            Stream::B => self.stream_b_amount,
            Stream::C => self.stream_c_amount,
        }
    }

    /// Verify allocation sums to total
    pub fn verify(&self) -> bool {
        let sum = self.stream_a_amount
            .saturating_add(self.stream_b_amount)
            .saturating_add(self.stream_c_amount)
            .saturating_add(self.treasury_amount);
        sum == self.total_fees
    }
}

/// Distribution record for a recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionRecord {
    /// Recipient address
    pub recipient: Address,
    /// Stream type
    pub stream: Stream,
    /// Amount distributed
    pub amount: u64,
    /// Block height of source fees
    pub source_block: u64,
    /// Distribution transaction hash
    pub tx_hash: Option<H256>,
    /// Timestamp
    pub timestamp: u64,
    /// Whether claimed/paid
    pub claimed: bool,
}

/// Accumulated fees pending distribution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingFees {
    /// Stream A pending
    pub stream_a: u64,
    /// Stream B pending
    pub stream_b: u64,
    /// Stream C pending
    pub stream_c: u64,
    /// Treasury pending
    pub treasury: u64,
}

impl PendingFees {
    pub fn add(&mut self, allocation: &FeeAllocation) {
        self.stream_a = self.stream_a.saturating_add(allocation.stream_a_amount);
        self.stream_b = self.stream_b.saturating_add(allocation.stream_b_amount);
        self.stream_c = self.stream_c.saturating_add(allocation.stream_c_amount);
        self.treasury = self.treasury.saturating_add(allocation.treasury_amount);
    }

    pub fn total(&self) -> u64 {
        self.stream_a
            .saturating_add(self.stream_b)
            .saturating_add(self.stream_c)
            .saturating_add(self.treasury)
    }

    pub fn take_stream(&mut self, stream: Stream) -> u64 {
        match stream {
            Stream::A => {
                let amount = self.stream_a;
                self.stream_a = 0;
                amount
            }
            Stream::B => {
                let amount = self.stream_b;
                self.stream_b = 0;
                amount
            }
            Stream::C => {
                let amount = self.stream_c;
                self.stream_c = 0;
                amount
            }
        }
    }

    pub fn take_treasury(&mut self) -> u64 {
        let amount = self.treasury;
        self.treasury = 0;
        amount
    }
}

/// Fee distributor - manages gas fee distribution across streams
pub struct FeeDistributor {
    /// Treasury address
    treasury_address: Address,
    /// Allocation history
    allocations: Arc<RwLock<Vec<FeeAllocation>>>,
    /// Distribution records by recipient
    distributions: Arc<RwLock<HashMap<Address, Vec<DistributionRecord>>>>,
    /// Pending fees awaiting distribution
    pending: Arc<RwLock<PendingFees>>,
    /// Total fees collected (all time)
    total_collected: Arc<RwLock<u64>>,
    /// Total fees distributed (all time)
    total_distributed: Arc<RwLock<u64>>,
    /// Stream A recipients (miners) and their shares
    stream_a_recipients: Arc<RwLock<HashMap<Address, u64>>>,
    /// Stream B recipients (GPU miners) and their shares
    stream_b_recipients: Arc<RwLock<HashMap<Address, u64>>>,
    /// Stream C recipients (stakers) and their shares
    stream_c_recipients: Arc<RwLock<HashMap<Address, u64>>>,
    /// Minimum distribution threshold
    min_distribution_threshold: u64,
    /// Distribution interval (blocks)
    distribution_interval: u64,
    /// Last distribution block
    last_distribution_block: Arc<RwLock<u64>>,
}

impl FeeDistributor {
    /// Create new fee distributor
    pub fn new(treasury_address: Address) -> Self {
        Self {
            treasury_address,
            allocations: Arc::new(RwLock::new(Vec::new())),
            distributions: Arc::new(RwLock::new(HashMap::new())),
            pending: Arc::new(RwLock::new(PendingFees::default())),
            total_collected: Arc::new(RwLock::new(0)),
            total_distributed: Arc::new(RwLock::new(0)),
            stream_a_recipients: Arc::new(RwLock::new(HashMap::new())),
            stream_b_recipients: Arc::new(RwLock::new(HashMap::new())),
            stream_c_recipients: Arc::new(RwLock::new(HashMap::new())),
            min_distribution_threshold: 100_000_000, // 1 PYRAX minimum
            distribution_interval: 100, // Every 100 blocks
            last_distribution_block: Arc::new(RwLock::new(0)),
        }
    }

    /// Set treasury address
    pub fn set_treasury(&mut self, address: Address) {
        self.treasury_address = address;
    }

    /// Collect fees from a block
    pub fn collect_fees(&self, block_height: u64, block_hash: H256, total_fees: u64) -> FeeAllocation {
        let allocation = FeeAllocation::new(block_height, block_hash, total_fees);
        
        self.pending.write().add(&allocation);
        self.allocations.write().push(allocation.clone());
        *self.total_collected.write() += total_fees;
        
        allocation
    }

    /// Register a recipient for a stream (miner/staker)
    pub fn register_recipient(&self, stream: Stream, address: Address, weight: u64) {
        let recipients = match stream {
            Stream::A => &self.stream_a_recipients,
            Stream::B => &self.stream_b_recipients,
            Stream::C => &self.stream_c_recipients,
        };
        
        let mut map = recipients.write();
        let current = map.entry(address).or_insert(0);
        *current = current.saturating_add(weight);
    }

    /// Update recipient weight (e.g., based on hashrate or stake)
    pub fn update_recipient_weight(&self, stream: Stream, address: Address, weight: u64) {
        let recipients = match stream {
            Stream::A => &self.stream_a_recipients,
            Stream::B => &self.stream_b_recipients,
            Stream::C => &self.stream_c_recipients,
        };
        
        recipients.write().insert(address, weight);
    }

    /// Remove recipient
    pub fn remove_recipient(&self, stream: Stream, address: &Address) {
        let recipients = match stream {
            Stream::A => &self.stream_a_recipients,
            Stream::B => &self.stream_b_recipients,
            Stream::C => &self.stream_c_recipients,
        };
        
        recipients.write().remove(address);
    }

    /// Check if distribution is due
    pub fn should_distribute(&self, current_block: u64) -> bool {
        let last = *self.last_distribution_block.read();
        let pending = self.pending.read();
        
        current_block >= last + self.distribution_interval 
            && pending.total() >= self.min_distribution_threshold
    }

    /// Calculate distributions for a stream
    pub fn calculate_stream_distribution(
        &self,
        stream: Stream,
        amount: u64,
    ) -> Vec<(Address, u64)> {
        let recipients = match stream {
            Stream::A => &self.stream_a_recipients,
            Stream::B => &self.stream_b_recipients,
            Stream::C => &self.stream_c_recipients,
        };
        
        let map = recipients.read();
        let total_weight: u64 = map.values().sum();
        
        if total_weight == 0 || map.is_empty() {
            return Vec::new();
        }
        
        map.iter()
            .map(|(addr, weight)| {
                let share = (amount as u128 * *weight as u128 / total_weight as u128) as u64;
                (*addr, share)
            })
            .filter(|(_, share)| *share > 0)
            .collect()
    }

    /// Execute distribution for all streams
    pub fn distribute(&self, current_block: u64) -> DistributionResult {
        if !self.should_distribute(current_block) {
            return DistributionResult::default();
        }

        let mut pending = self.pending.write();
        let mut result = DistributionResult::default();
        
        // Distribute Stream A
        let stream_a_amount = pending.take_stream(Stream::A);
        if stream_a_amount > 0 {
            let distributions = self.calculate_stream_distribution(Stream::A, stream_a_amount);
            for (addr, amount) in distributions {
                self.record_distribution(addr, Stream::A, amount, current_block);
                result.stream_a_distributed += amount;
                result.stream_a_recipients += 1;
            }
        }
        
        // Distribute Stream B
        let stream_b_amount = pending.take_stream(Stream::B);
        if stream_b_amount > 0 {
            let distributions = self.calculate_stream_distribution(Stream::B, stream_b_amount);
            for (addr, amount) in distributions {
                self.record_distribution(addr, Stream::B, amount, current_block);
                result.stream_b_distributed += amount;
                result.stream_b_recipients += 1;
            }
        }
        
        // Distribute Stream C
        let stream_c_amount = pending.take_stream(Stream::C);
        if stream_c_amount > 0 {
            let distributions = self.calculate_stream_distribution(Stream::C, stream_c_amount);
            for (addr, amount) in distributions {
                self.record_distribution(addr, Stream::C, amount, current_block);
                result.stream_c_distributed += amount;
                result.stream_c_recipients += 1;
            }
        }
        
        // Treasury
        let treasury_amount = pending.take_treasury();
        if treasury_amount > 0 {
            self.record_distribution(self.treasury_address, Stream::A, treasury_amount, current_block);
            result.treasury_distributed = treasury_amount;
        }
        
        result.total_distributed = result.stream_a_distributed
            + result.stream_b_distributed
            + result.stream_c_distributed
            + result.treasury_distributed;
        
        *self.total_distributed.write() += result.total_distributed;
        *self.last_distribution_block.write() = current_block;
        
        result
    }

    /// Record a distribution
    fn record_distribution(&self, recipient: Address, stream: Stream, amount: u64, source_block: u64) {
        let record = DistributionRecord {
            recipient,
            stream,
            amount,
            source_block,
            tx_hash: None,
            timestamp: current_timestamp(),
            claimed: false,
        };
        
        self.distributions.write()
            .entry(recipient)
            .or_insert_with(Vec::new)
            .push(record);
    }

    /// Get pending distributions for an address
    pub fn get_pending_distributions(&self, address: &Address) -> Vec<DistributionRecord> {
        self.distributions.read()
            .get(address)
            .map(|v| v.iter().filter(|r| !r.claimed).cloned().collect())
            .unwrap_or_default()
    }

    /// Mark distributions as claimed
    pub fn mark_claimed(&self, address: &Address, tx_hash: H256) {
        if let Some(records) = self.distributions.write().get_mut(address) {
            for record in records.iter_mut() {
                if !record.claimed {
                    record.claimed = true;
                    record.tx_hash = Some(tx_hash);
                }
            }
        }
    }

    /// Get distribution statistics
    pub fn stats(&self) -> FeeDistributorStats {
        let pending = self.pending.read();
        
        FeeDistributorStats {
            total_collected: *self.total_collected.read(),
            total_distributed: *self.total_distributed.read(),
            pending_stream_a: pending.stream_a,
            pending_stream_b: pending.stream_b,
            pending_stream_c: pending.stream_c,
            pending_treasury: pending.treasury,
            stream_a_recipients: self.stream_a_recipients.read().len() as u64,
            stream_b_recipients: self.stream_b_recipients.read().len() as u64,
            stream_c_recipients: self.stream_c_recipients.read().len() as u64,
            last_distribution_block: *self.last_distribution_block.read(),
        }
    }

    /// Get allocation history
    pub fn get_allocations(&self, limit: usize) -> Vec<FeeAllocation> {
        let allocations = self.allocations.read();
        allocations.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for FeeDistributor {
    fn default() -> Self {
        Self::new(Address::ZERO)
    }
}

/// Distribution result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistributionResult {
    pub stream_a_distributed: u64,
    pub stream_a_recipients: u64,
    pub stream_b_distributed: u64,
    pub stream_b_recipients: u64,
    pub stream_c_distributed: u64,
    pub stream_c_recipients: u64,
    pub treasury_distributed: u64,
    pub total_distributed: u64,
}

/// Fee distributor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeDistributorStats {
    pub total_collected: u64,
    pub total_distributed: u64,
    pub pending_stream_a: u64,
    pub pending_stream_b: u64,
    pub pending_stream_c: u64,
    pub pending_treasury: u64,
    pub stream_a_recipients: u64,
    pub stream_b_recipients: u64,
    pub stream_c_recipients: u64,
    pub last_distribution_block: u64,
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
    fn test_fee_allocation() {
        let allocation = FeeAllocation::new(100, H256([1u8; 32]), 1_000_000_000);
        
        assert_eq!(allocation.stream_a_amount, 200_000_000); // 20%
        assert_eq!(allocation.stream_b_amount, 400_000_000); // 40%
        assert_eq!(allocation.stream_c_amount, 300_000_000); // 30%
        assert_eq!(allocation.treasury_amount, 100_000_000); // 10%
        assert!(allocation.verify());
    }

    #[test]
    fn test_pending_fees() {
        let mut pending = PendingFees::default();
        
        let alloc = FeeAllocation::new(1, H256([1u8; 32]), 1_000_000_000);
        pending.add(&alloc);
        
        assert_eq!(pending.stream_a, 200_000_000);
        assert_eq!(pending.stream_b, 400_000_000);
        assert_eq!(pending.total(), 1_000_000_000);
        
        let taken = pending.take_stream(Stream::A);
        assert_eq!(taken, 200_000_000);
        assert_eq!(pending.stream_a, 0);
    }

    #[test]
    fn test_fee_distributor() {
        let treasury = Address([0xff; 20]);
        let distributor = FeeDistributor::new(treasury);
        
        // Register recipients
        let miner_a = Address([1u8; 20]);
        let miner_b = Address([2u8; 20]);
        
        distributor.register_recipient(Stream::A, miner_a, 100);
        distributor.register_recipient(Stream::A, miner_b, 100);
        
        // Collect fees
        let alloc = distributor.collect_fees(100, H256([1u8; 32]), 1_000_000_000);
        assert!(alloc.verify());
        
        let stats = distributor.stats();
        assert_eq!(stats.total_collected, 1_000_000_000);
        assert_eq!(stats.pending_stream_a, 200_000_000);
    }

    #[test]
    fn test_distribution_calculation() {
        let distributor = FeeDistributor::new(Address::ZERO);
        
        let addr1 = Address([1u8; 20]);
        let addr2 = Address([2u8; 20]);
        
        distributor.register_recipient(Stream::B, addr1, 300); // 75%
        distributor.register_recipient(Stream::B, addr2, 100); // 25%
        
        let distributions = distributor.calculate_stream_distribution(Stream::B, 1000);
        
        assert_eq!(distributions.len(), 2);
        
        let d1 = distributions.iter().find(|(a, _)| *a == addr1).unwrap();
        let d2 = distributions.iter().find(|(a, _)| *a == addr2).unwrap();
        
        assert_eq!(d1.1, 750); // 75%
        assert_eq!(d2.1, 250); // 25%
    }

    #[test]
    fn test_full_distribution() {
        let treasury = Address([0xff; 20]);
        let mut distributor = FeeDistributor::new(treasury);
        distributor.min_distribution_threshold = 0; // No minimum for test
        distributor.distribution_interval = 0; // Immediate distribution
        
        // Register recipients for all streams
        let miner_a = Address([1u8; 20]);
        let gpu_miner = Address([2u8; 20]);
        let staker = Address([3u8; 20]);
        
        distributor.register_recipient(Stream::A, miner_a, 100);
        distributor.register_recipient(Stream::B, gpu_miner, 100);
        distributor.register_recipient(Stream::C, staker, 100);
        
        // Collect fees
        distributor.collect_fees(100, H256([1u8; 32]), 1_000_000_000);
        
        // Distribute
        let result = distributor.distribute(100);
        
        assert_eq!(result.stream_a_distributed, 200_000_000);
        assert_eq!(result.stream_b_distributed, 400_000_000);
        assert_eq!(result.stream_c_distributed, 300_000_000);
        assert_eq!(result.treasury_distributed, 100_000_000);
        assert_eq!(result.total_distributed, 1_000_000_000);
        
        // Check pending is zero
        let stats = distributor.stats();
        assert_eq!(stats.pending_stream_a, 0);
        assert_eq!(stats.pending_stream_b, 0);
        assert_eq!(stats.pending_stream_c, 0);
    }

    #[test]
    fn test_stream_share_percentages() {
        // Verify percentages add up to 100
        let total = STREAM_A_SHARE_PERCENT 
            + STREAM_B_SHARE_PERCENT 
            + STREAM_C_SHARE_PERCENT 
            + TREASURY_SHARE_PERCENT;
        assert_eq!(total, 100);
    }
}
