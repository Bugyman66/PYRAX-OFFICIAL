//! Token Allocation Management
//!
//! Manages allocation pools and distribution

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::*;

/// Allocation pool identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllocationPool {
    /// Presale - 15% - 100% at TGE
    Presale,
    /// BDAG Community - 10% - 12mo cliff, 12mo vesting
    BdagCommunity,
    /// Mining Rewards - 35% - Per block
    MiningRewards,
    /// ZK Prover Rewards - 5% - Per attestation
    ZkProverRewards,
    /// Team & Founders - 4% - 12mo cliff, 48mo vesting
    TeamFounders,
    /// Advisors - 3% - 12mo cliff, 48mo vesting
    Advisors,
    /// Ecosystem - 10% - Milestone-based, DAO
    Ecosystem,
    /// Marketing - 5% - Multi-sig approval
    Marketing,
    /// Liquidity - 10% - 100% at TGE
    Liquidity,
    /// Treasury - 2% - 75% DAO approval
    Treasury,
    /// Reserve - 1% - Protocol buffer
    Reserve,
}

impl AllocationPool {
    /// Get allocation amount for this pool
    pub fn allocation(&self) -> u64 {
        match self {
            AllocationPool::Presale => PRESALE_ALLOCATION,
            AllocationPool::BdagCommunity => BDAG_COMMUNITY_ALLOCATION,
            AllocationPool::MiningRewards => MINING_REWARDS_ALLOCATION,
            AllocationPool::ZkProverRewards => ZK_PROVER_ALLOCATION,
            AllocationPool::TeamFounders => TEAM_ALLOCATION,
            AllocationPool::Advisors => ADVISORS_ALLOCATION,
            AllocationPool::Ecosystem => ECOSYSTEM_ALLOCATION,
            AllocationPool::Marketing => MARKETING_ALLOCATION,
            AllocationPool::Liquidity => LIQUIDITY_ALLOCATION,
            AllocationPool::Treasury => TREASURY_ALLOCATION,
            AllocationPool::Reserve => RESERVE_ALLOCATION,
        }
    }

    /// Get allocation percentage
    pub fn percentage(&self) -> u8 {
        match self {
            AllocationPool::Presale => PRESALE_PERCENT,
            AllocationPool::BdagCommunity => BDAG_COMMUNITY_PERCENT,
            AllocationPool::MiningRewards => MINING_REWARDS_PERCENT,
            AllocationPool::ZkProverRewards => ZK_PROVER_PERCENT,
            AllocationPool::TeamFounders => TEAM_PERCENT,
            AllocationPool::Advisors => ADVISORS_PERCENT,
            AllocationPool::Ecosystem => ECOSYSTEM_PERCENT,
            AllocationPool::Marketing => MARKETING_PERCENT,
            AllocationPool::Liquidity => LIQUIDITY_PERCENT,
            AllocationPool::Treasury => TREASURY_PERCENT,
            AllocationPool::Reserve => RESERVE_PERCENT,
        }
    }

    /// Get pool name
    pub fn name(&self) -> &'static str {
        match self {
            AllocationPool::Presale => "Presale",
            AllocationPool::BdagCommunity => "BDAG Community",
            AllocationPool::MiningRewards => "Mining Rewards",
            AllocationPool::ZkProverRewards => "ZK Prover Rewards",
            AllocationPool::TeamFounders => "Team & Founders",
            AllocationPool::Advisors => "Advisors",
            AllocationPool::Ecosystem => "Ecosystem",
            AllocationPool::Marketing => "Marketing",
            AllocationPool::Liquidity => "Liquidity",
            AllocationPool::Treasury => "Treasury",
            AllocationPool::Reserve => "Reserve",
        }
    }

    /// Get pool description
    pub fn description(&self) -> &'static str {
        match self {
            AllocationPool::Presale => "Public token sale across 4 phases aligned with testnet launches",
            AllocationPool::BdagCommunity => "BlockDAG community migration program, 31.25% of original investment",
            AllocationPool::MiningRewards => "Stream A (ASIC) and Stream B (CPU/GPU) mining incentives",
            AllocationPool::ZkProverRewards => "Stream C zero-knowledge proof verification rewards",
            AllocationPool::TeamFounders => "Core team compensation for development and operations",
            AllocationPool::Advisors => "Strategic, technical, and legal advisors",
            AllocationPool::Ecosystem => "Developer grants, partnerships, incentives, bug bounties",
            AllocationPool::Marketing => "Digital marketing, community rewards, events, influencers",
            AllocationPool::Liquidity => "CEX listings (60%), DEX pools (30%), market making (10%)",
            AllocationPool::Treasury => "Emergency fund and long-term protocol sustainability",
            AllocationPool::Reserve => "Protocol buffer for unforeseen needs",
        }
    }

    /// Get release schedule description
    pub fn release_schedule(&self) -> &'static str {
        match self {
            AllocationPool::Presale => "30% at TGE, 1 month cliff, 12 month linear vesting",
            AllocationPool::BdagCommunity => "12 month cliff, 12 month linear vesting",
            AllocationPool::MiningRewards => "Released per block mined",
            AllocationPool::ZkProverRewards => "Released per attestation, 10% burned",
            AllocationPool::TeamFounders => "12 month cliff, 48 month linear vesting",
            AllocationPool::Advisors => "12 month cliff, 48 month linear vesting",
            AllocationPool::Ecosystem => "Milestone-based, DAO approval",
            AllocationPool::Marketing => "As needed, multi-sig approval",
            AllocationPool::Liquidity => "100% at TGE",
            AllocationPool::Treasury => "75% DAO approval required",
            AllocationPool::Reserve => "DAO governance controlled",
        }
    }

    /// Is TGE release (immediate 100%)
    pub fn is_tge_release(&self) -> bool {
        matches!(self, AllocationPool::Liquidity)
    }

    /// Has vesting schedule
    pub fn has_vesting(&self) -> bool {
        matches!(self, 
            AllocationPool::Presale |
            AllocationPool::BdagCommunity | 
            AllocationPool::TeamFounders | 
            AllocationPool::Advisors
        )
    }

    /// Get TGE release percentage
    pub fn tge_percent(&self) -> u8 {
        match self {
            AllocationPool::Presale => 30,  // 30% at TGE
            AllocationPool::Liquidity => 100, // 100% at TGE
            _ => 0, // Others have no TGE release
        }
    }

    /// Is emission-based (per block/attestation)
    pub fn is_emission_based(&self) -> bool {
        matches!(self, 
            AllocationPool::MiningRewards | 
            AllocationPool::ZkProverRewards
        )
    }

    /// Requires DAO approval
    pub fn requires_dao(&self) -> bool {
        matches!(self, 
            AllocationPool::Ecosystem | 
            AllocationPool::Treasury |
            AllocationPool::Reserve
        )
    }

    /// Requires multi-sig
    pub fn requires_multisig(&self) -> bool {
        matches!(self, AllocationPool::Marketing)
    }

    /// Get all pools
    pub fn all() -> Vec<AllocationPool> {
        vec![
            AllocationPool::Presale,
            AllocationPool::BdagCommunity,
            AllocationPool::MiningRewards,
            AllocationPool::ZkProverRewards,
            AllocationPool::TeamFounders,
            AllocationPool::Advisors,
            AllocationPool::Ecosystem,
            AllocationPool::Marketing,
            AllocationPool::Liquidity,
            AllocationPool::Treasury,
            AllocationPool::Reserve,
        ]
    }
}

/// Token allocation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAllocation {
    /// Allocation ID
    pub id: H256,
    /// Pool
    pub pool: AllocationPool,
    /// Recipient address
    pub recipient: Address,
    /// Total amount allocated
    pub total_amount: u64,
    /// Amount released
    pub released_amount: u64,
    /// Amount remaining
    pub remaining_amount: u64,
    /// Allocation timestamp
    pub allocated_at: u64,
    /// Last release timestamp
    pub last_release_at: Option<u64>,
    /// Is fully released
    pub fully_released: bool,
    /// Notes/reason
    pub notes: String,
}

impl TokenAllocation {
    /// Create new allocation
    pub fn new(pool: AllocationPool, recipient: Address, amount: u64, notes: String) -> Self {
        let id = Self::generate_id(&pool, &recipient, amount);
        Self {
            id,
            pool,
            recipient,
            total_amount: amount,
            released_amount: 0,
            remaining_amount: amount,
            allocated_at: current_timestamp(),
            last_release_at: None,
            fully_released: false,
            notes,
        }
    }

    /// Generate allocation ID
    fn generate_id(pool: &AllocationPool, recipient: &Address, amount: u64) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&[*pool as u8]);
        hasher.update(&recipient.0);
        hasher.update(&amount.to_le_bytes());
        hasher.update(&current_timestamp().to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Release tokens
    pub fn release(&mut self, amount: u64) -> Result<u64, AllocationError> {
        if amount > self.remaining_amount {
            return Err(AllocationError::InsufficientBalance {
                requested: amount,
                available: self.remaining_amount,
            });
        }

        self.released_amount += amount;
        self.remaining_amount -= amount;
        self.last_release_at = Some(current_timestamp());

        if self.remaining_amount == 0 {
            self.fully_released = true;
        }

        Ok(amount)
    }

    /// Get release percentage
    pub fn release_percentage(&self) -> f64 {
        if self.total_amount == 0 {
            return 0.0;
        }
        (self.released_amount as f64 / self.total_amount as f64) * 100.0
    }
}

/// Pool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    /// Pool
    pub pool: AllocationPool,
    /// Total allocation
    pub total_allocation: u64,
    /// Total distributed
    pub total_distributed: u64,
    /// Total remaining
    pub total_remaining: u64,
    /// Number of allocations
    pub allocation_count: u64,
    /// Is active
    pub active: bool,
}

impl PoolState {
    /// Create new pool state
    pub fn new(pool: AllocationPool) -> Self {
        Self {
            pool,
            total_allocation: pool.allocation(),
            total_distributed: 0,
            total_remaining: pool.allocation(),
            allocation_count: 0,
            active: true,
        }
    }

    /// Allocate from pool
    pub fn allocate(&mut self, amount: u64) -> Result<(), AllocationError> {
        if amount > self.total_remaining {
            return Err(AllocationError::PoolExhausted {
                pool: self.pool,
                requested: amount,
                available: self.total_remaining,
            });
        }

        self.total_distributed += amount;
        self.total_remaining -= amount;
        self.allocation_count += 1;

        Ok(())
    }

    /// Distribution percentage
    pub fn distribution_percentage(&self) -> f64 {
        if self.total_allocation == 0 {
            return 0.0;
        }
        (self.total_distributed as f64 / self.total_allocation as f64) * 100.0
    }
}

/// Allocation manager
pub struct AllocationManager {
    /// Pool states
    pools: Arc<RwLock<HashMap<AllocationPool, PoolState>>>,
    /// Allocations by ID
    allocations: Arc<RwLock<HashMap<H256, TokenAllocation>>>,
    /// Allocations by recipient
    by_recipient: Arc<RwLock<HashMap<Address, Vec<H256>>>>,
    /// TGE timestamp
    tge_timestamp: Arc<RwLock<u64>>,
    /// Statistics
    stats: Arc<RwLock<AllocationStats>>,
}

impl AllocationManager {
    /// Create new allocation manager
    pub fn new() -> Self {
        let mut pools = HashMap::new();
        for pool in AllocationPool::all() {
            pools.insert(pool, PoolState::new(pool));
        }

        Self {
            pools: Arc::new(RwLock::new(pools)),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            by_recipient: Arc::new(RwLock::new(HashMap::new())),
            tge_timestamp: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(AllocationStats::default())),
        }
    }

    /// Set TGE timestamp
    pub fn set_tge(&self, timestamp: u64) {
        *self.tge_timestamp.write() = timestamp;
    }

    /// Get TGE timestamp
    pub fn tge(&self) -> u64 {
        *self.tge_timestamp.read()
    }

    /// Create allocation
    pub fn create_allocation(
        &self,
        pool: AllocationPool,
        recipient: Address,
        amount: u64,
        notes: String,
    ) -> Result<H256, AllocationError> {
        // Check pool has sufficient funds
        {
            let mut pools = self.pools.write();
            let pool_state = pools.get_mut(&pool)
                .ok_or(AllocationError::PoolNotFound(pool))?;
            pool_state.allocate(amount)?;
        }

        // Create allocation
        let allocation = TokenAllocation::new(pool, recipient, amount, notes);
        let id = allocation.id;

        // Store
        self.allocations.write().insert(id, allocation);
        self.by_recipient.write()
            .entry(recipient)
            .or_insert_with(Vec::new)
            .push(id);

        self.stats.write().allocations_created += 1;

        Ok(id)
    }

    /// Release tokens from allocation
    pub fn release_tokens(
        &self,
        allocation_id: &H256,
        amount: u64,
    ) -> Result<u64, AllocationError> {
        let mut allocations = self.allocations.write();
        let allocation = allocations.get_mut(allocation_id)
            .ok_or_else(|| AllocationError::AllocationNotFound(*allocation_id))?;

        let released = allocation.release(amount)?;
        self.stats.write().tokens_released += released;

        Ok(released)
    }

    /// Get allocation
    pub fn get_allocation(&self, id: &H256) -> Option<TokenAllocation> {
        self.allocations.read().get(id).cloned()
    }

    /// Get allocations for recipient
    pub fn get_recipient_allocations(&self, recipient: &Address) -> Vec<TokenAllocation> {
        let ids = self.by_recipient.read()
            .get(recipient)
            .cloned()
            .unwrap_or_default();

        let allocations = self.allocations.read();
        ids.iter()
            .filter_map(|id| allocations.get(id).cloned())
            .collect()
    }

    /// Get pool state
    pub fn get_pool_state(&self, pool: AllocationPool) -> Option<PoolState> {
        self.pools.read().get(&pool).cloned()
    }

    /// Get all pool states
    pub fn get_all_pools(&self) -> Vec<PoolState> {
        self.pools.read().values().cloned().collect()
    }

    /// Get statistics
    pub fn stats(&self) -> AllocationStats {
        self.stats.read().clone()
    }
}

impl Default for AllocationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Allocation errors
#[derive(Debug, thiserror::Error)]
pub enum AllocationError {
    #[error("Pool not found: {0:?}")]
    PoolNotFound(AllocationPool),

    #[error("Pool exhausted: {pool:?} - requested {requested}, available {available}")]
    PoolExhausted {
        pool: AllocationPool,
        requested: u64,
        available: u64,
    },

    #[error("Allocation not found: {0:?}")]
    AllocationNotFound(H256),

    #[error("Insufficient balance: requested {requested}, available {available}")]
    InsufficientBalance { requested: u64, available: u64 },

    #[error("Vesting not started")]
    VestingNotStarted,

    #[error("DAO approval required")]
    DaoApprovalRequired,

    #[error("Multi-sig approval required")]
    MultisigRequired,
}

/// Allocation statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllocationStats {
    pub allocations_created: u64,
    pub tokens_released: u64,
    pub pools_exhausted: u64,
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
    fn test_allocation_pool() {
        assert_eq!(AllocationPool::Presale.allocation(), PRESALE_ALLOCATION);
        assert_eq!(AllocationPool::Presale.percentage(), 15);
    }

    #[test]
    fn test_all_pools() {
        let pools = AllocationPool::all();
        assert_eq!(pools.len(), 11);
    }

    #[test]
    fn test_pool_state() {
        let mut state = PoolState::new(AllocationPool::Presale);
        assert_eq!(state.total_allocation, PRESALE_ALLOCATION);
        
        state.allocate(1000).unwrap();
        assert_eq!(state.allocation_count, 1);
    }

    #[test]
    fn test_allocation_manager() {
        let manager = AllocationManager::new();
        
        let id = manager.create_allocation(
            AllocationPool::Presale,
            Address([1u8; 20]),
            1000,
            "Test allocation".into(),
        ).unwrap();

        assert!(manager.get_allocation(&id).is_some());
    }

    #[test]
    fn test_release_tokens() {
        let manager = AllocationManager::new();
        
        let id = manager.create_allocation(
            AllocationPool::Presale,
            Address([1u8; 20]),
            1000,
            "Test".into(),
        ).unwrap();

        let released = manager.release_tokens(&id, 500).unwrap();
        assert_eq!(released, 500);

        let allocation = manager.get_allocation(&id).unwrap();
        assert_eq!(allocation.remaining_amount, 500);
    }
}
