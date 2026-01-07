//! Upgrade Mechanism
//!
//! Production-grade on-chain upgrade system

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::UPGRADE_DELAY_BLOCKS;

/// Upgrade status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeStatus {
    /// Proposal submitted
    Proposed,
    /// Voting in progress
    Voting,
    /// Approved by governance
    Approved,
    /// Scheduled for activation
    Scheduled,
    /// Currently activating
    Activating,
    /// Successfully activated
    Activated,
    /// Upgrade failed
    Failed,
    /// Proposal rejected
    Rejected,
    /// Proposal cancelled
    Cancelled,
}

/// Upgrade type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeType {
    /// Soft fork (backward compatible)
    SoftFork,
    /// Hard fork (breaking changes)
    HardFork,
    /// Parameter change only
    ParameterChange,
    /// Emergency security fix
    EmergencyFix,
    /// Consensus rule change
    ConsensusChange,
    /// Protocol version bump
    ProtocolVersion,
}

impl UpgradeType {
    /// Minimum approval percentage required
    pub fn min_approval_percent(&self) -> u8 {
        match self {
            UpgradeType::SoftFork => 67,
            UpgradeType::HardFork => 75,
            UpgradeType::ParameterChange => 51,
            UpgradeType::EmergencyFix => 67,
            UpgradeType::ConsensusChange => 80,
            UpgradeType::ProtocolVersion => 67,
        }
    }

    /// Minimum voting period (blocks)
    pub fn min_voting_period(&self) -> u64 {
        match self {
            UpgradeType::SoftFork => 14400,       // ~24 hours
            UpgradeType::HardFork => 100800,      // ~7 days
            UpgradeType::ParameterChange => 7200, // ~12 hours
            UpgradeType::EmergencyFix => 1200,    // ~2 hours
            UpgradeType::ConsensusChange => 201600, // ~14 days
            UpgradeType::ProtocolVersion => 50400, // ~3.5 days
        }
    }
}

/// Upgrade proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeProposal {
    /// Proposal ID
    pub id: H256,
    /// Upgrade type
    pub upgrade_type: UpgradeType,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Proposer address
    pub proposer: Address,
    /// Target version
    pub target_version: String,
    /// Current status
    pub status: UpgradeStatus,
    /// Activation block height
    pub activation_height: Option<u64>,
    /// Code changes (hash of diff)
    pub code_hash: H256,
    /// Parameter changes
    pub parameter_changes: HashMap<String, String>,
    /// Votes for
    pub votes_for: u64,
    /// Votes against
    pub votes_against: u64,
    /// Total voting power participated
    pub total_voted: u64,
    /// Voters
    pub voters: HashMap<Address, Vote>,
    /// Created at block
    pub created_at_block: u64,
    /// Voting ends at block
    pub voting_ends_at: u64,
    /// Created timestamp
    pub created_at: u64,
    /// Updated timestamp
    pub updated_at: u64,
}

impl UpgradeProposal {
    /// Create new proposal
    pub fn new(
        upgrade_type: UpgradeType,
        title: String,
        description: String,
        proposer: Address,
        target_version: String,
        current_block: u64,
    ) -> Self {
        let id = Self::generate_id(&title, &proposer);
        let voting_period = upgrade_type.min_voting_period();

        Self {
            id,
            upgrade_type,
            title,
            description,
            proposer,
            target_version,
            status: UpgradeStatus::Proposed,
            activation_height: None,
            code_hash: H256::zero(),
            parameter_changes: HashMap::new(),
            votes_for: 0,
            votes_against: 0,
            total_voted: 0,
            voters: HashMap::new(),
            created_at_block: current_block,
            voting_ends_at: current_block + voting_period,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        }
    }

    /// Generate proposal ID
    fn generate_id(title: &str, proposer: &Address) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(title.as_bytes());
        hasher.update(&proposer.0);
        hasher.update(&current_timestamp().to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Set code hash
    pub fn with_code_hash(mut self, hash: H256) -> Self {
        self.code_hash = hash;
        self
    }

    /// Add parameter change
    pub fn with_parameter(mut self, key: String, value: String) -> Self {
        self.parameter_changes.insert(key, value);
        self
    }

    /// Record vote
    pub fn vote(&mut self, voter: Address, voting_power: u64, support: bool) -> Result<(), UpgradeError> {
        if self.status != UpgradeStatus::Voting && self.status != UpgradeStatus::Proposed {
            return Err(UpgradeError::VotingClosed);
        }

        if self.voters.contains_key(&voter) {
            return Err(UpgradeError::AlreadyVoted(voter));
        }

        let vote = Vote {
            voter,
            voting_power,
            support,
            timestamp: current_timestamp(),
        };

        self.voters.insert(voter, vote);
        self.total_voted += voting_power;

        if support {
            self.votes_for += voting_power;
        } else {
            self.votes_against += voting_power;
        }

        self.status = UpgradeStatus::Voting;
        self.updated_at = current_timestamp();

        Ok(())
    }

    /// Calculate approval percentage
    pub fn approval_percent(&self) -> u8 {
        if self.total_voted == 0 {
            return 0;
        }
        ((self.votes_for as f64 / self.total_voted as f64) * 100.0) as u8
    }

    /// Check if proposal passed
    pub fn is_passed(&self) -> bool {
        self.approval_percent() >= self.upgrade_type.min_approval_percent()
    }

    /// Check if voting period ended
    pub fn voting_ended(&self, current_block: u64) -> bool {
        current_block >= self.voting_ends_at
    }

    /// Finalize voting
    pub fn finalize(&mut self, current_block: u64) -> Result<(), UpgradeError> {
        if !self.voting_ended(current_block) {
            return Err(UpgradeError::VotingNotEnded);
        }

        if self.is_passed() {
            self.status = UpgradeStatus::Approved;
            self.activation_height = Some(current_block + UPGRADE_DELAY_BLOCKS);
        } else {
            self.status = UpgradeStatus::Rejected;
        }

        self.updated_at = current_timestamp();
        Ok(())
    }

    /// Schedule upgrade
    pub fn schedule(&mut self, activation_height: u64) -> Result<(), UpgradeError> {
        if self.status != UpgradeStatus::Approved {
            return Err(UpgradeError::NotApproved);
        }

        self.activation_height = Some(activation_height);
        self.status = UpgradeStatus::Scheduled;
        self.updated_at = current_timestamp();
        Ok(())
    }

    /// Check if should activate
    pub fn should_activate(&self, current_block: u64) -> bool {
        if self.status != UpgradeStatus::Scheduled {
            return false;
        }
        self.activation_height.map_or(false, |h| current_block >= h)
    }

    /// Activate upgrade
    pub fn activate(&mut self) -> Result<(), UpgradeError> {
        if self.status != UpgradeStatus::Scheduled {
            return Err(UpgradeError::NotScheduled);
        }

        self.status = UpgradeStatus::Activated;
        self.updated_at = current_timestamp();
        Ok(())
    }

    /// Cancel proposal
    pub fn cancel(&mut self, canceller: &Address) -> Result<(), UpgradeError> {
        if *canceller != self.proposer {
            return Err(UpgradeError::NotProposer);
        }

        if self.status == UpgradeStatus::Activated {
            return Err(UpgradeError::AlreadyActivated);
        }

        self.status = UpgradeStatus::Cancelled;
        self.updated_at = current_timestamp();
        Ok(())
    }
}

/// Vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Voter address
    pub voter: Address,
    /// Voting power
    pub voting_power: u64,
    /// Support the upgrade
    pub support: bool,
    /// Vote timestamp
    pub timestamp: u64,
}

/// Upgrade configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeConfig {
    /// Enable upgrades
    pub enabled: bool,
    /// Minimum proposer stake
    pub min_proposer_stake: u64,
    /// Default voting period (blocks)
    pub default_voting_period: u64,
    /// Upgrade delay after approval (blocks)
    pub upgrade_delay_blocks: u64,
    /// Allow emergency upgrades
    pub allow_emergency: bool,
    /// Require code audit
    pub require_audit: bool,
}

impl Default for UpgradeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_proposer_stake: 100_000 * 100_000_000, // 100,000 PYRAX
            default_voting_period: 50400, // ~3.5 days
            upgrade_delay_blocks: UPGRADE_DELAY_BLOCKS,
            allow_emergency: true,
            require_audit: true,
        }
    }
}

/// Upgrade manager
pub struct UpgradeManager {
    /// Configuration
    config: UpgradeConfig,
    /// Active proposals
    proposals: Arc<RwLock<HashMap<H256, UpgradeProposal>>>,
    /// Activated upgrades
    activated: Arc<RwLock<Vec<UpgradeProposal>>>,
    /// Pending upgrades (scheduled)
    pending: Arc<RwLock<Vec<H256>>>,
    /// Current protocol version
    current_version: Arc<RwLock<String>>,
    /// Current block height
    current_block: Arc<RwLock<u64>>,
    /// Statistics
    stats: Arc<RwLock<UpgradeStats>>,
}

impl UpgradeManager {
    /// Create new upgrade manager
    pub fn new(config: UpgradeConfig, initial_version: String) -> Self {
        Self {
            config,
            proposals: Arc::new(RwLock::new(HashMap::new())),
            activated: Arc::new(RwLock::new(Vec::new())),
            pending: Arc::new(RwLock::new(Vec::new())),
            current_version: Arc::new(RwLock::new(initial_version)),
            current_block: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(UpgradeStats::default())),
        }
    }

    /// Submit upgrade proposal
    pub fn submit_proposal(
        &self,
        upgrade_type: UpgradeType,
        title: String,
        description: String,
        proposer: Address,
        target_version: String,
    ) -> Result<H256, UpgradeError> {
        if !self.config.enabled {
            return Err(UpgradeError::UpgradesDisabled);
        }

        let current_block = *self.current_block.read();
        let proposal = UpgradeProposal::new(
            upgrade_type,
            title,
            description,
            proposer,
            target_version,
            current_block,
        );

        let id = proposal.id;
        self.proposals.write().insert(id, proposal);
        self.stats.write().proposals_submitted += 1;

        Ok(id)
    }

    /// Vote on proposal
    pub fn vote(
        &self,
        proposal_id: &H256,
        voter: Address,
        voting_power: u64,
        support: bool,
    ) -> Result<(), UpgradeError> {
        let mut proposals = self.proposals.write();
        let proposal = proposals.get_mut(proposal_id)
            .ok_or_else(|| UpgradeError::ProposalNotFound(*proposal_id))?;

        proposal.vote(voter, voting_power, support)?;
        self.stats.write().votes_cast += 1;

        Ok(())
    }

    /// Finalize voting
    pub fn finalize_proposal(&self, proposal_id: &H256) -> Result<UpgradeStatus, UpgradeError> {
        let current_block = *self.current_block.read();
        let mut proposals = self.proposals.write();
        let proposal = proposals.get_mut(proposal_id)
            .ok_or_else(|| UpgradeError::ProposalNotFound(*proposal_id))?;

        proposal.finalize(current_block)?;

        if proposal.status == UpgradeStatus::Approved {
            self.pending.write().push(*proposal_id);
            self.stats.write().proposals_approved += 1;
        } else {
            self.stats.write().proposals_rejected += 1;
        }

        Ok(proposal.status)
    }

    /// Process block (check for pending activations)
    pub fn process_block(&self, block_height: u64) -> Vec<UpgradeProposal> {
        *self.current_block.write() = block_height;

        let mut activated = Vec::new();
        let mut to_remove = Vec::new();

        // Check pending upgrades
        for proposal_id in self.pending.read().iter() {
            let mut proposals = self.proposals.write();
            if let Some(proposal) = proposals.get_mut(proposal_id) {
                if proposal.should_activate(block_height) {
                    if proposal.activate().is_ok() {
                        *self.current_version.write() = proposal.target_version.clone();
                        activated.push(proposal.clone());
                        to_remove.push(*proposal_id);
                        self.stats.write().upgrades_activated += 1;
                    }
                }
            }
        }

        // Move activated to history
        {
            let mut pending = self.pending.write();
            pending.retain(|id| !to_remove.contains(id));
        }

        for proposal in &activated {
            self.activated.write().push(proposal.clone());
        }

        activated
    }

    /// Get proposal
    pub fn get_proposal(&self, id: &H256) -> Option<UpgradeProposal> {
        self.proposals.read().get(id).cloned()
    }

    /// Get all active proposals
    pub fn get_active_proposals(&self) -> Vec<UpgradeProposal> {
        self.proposals.read()
            .values()
            .filter(|p| matches!(p.status, 
                UpgradeStatus::Proposed | 
                UpgradeStatus::Voting |
                UpgradeStatus::Approved |
                UpgradeStatus::Scheduled
            ))
            .cloned()
            .collect()
    }

    /// Get pending upgrades
    pub fn get_pending_upgrades(&self) -> Vec<UpgradeProposal> {
        let pending = self.pending.read();
        let proposals = self.proposals.read();
        
        pending.iter()
            .filter_map(|id| proposals.get(id).cloned())
            .collect()
    }

    /// Get upgrade history
    pub fn get_history(&self) -> Vec<UpgradeProposal> {
        self.activated.read().clone()
    }

    /// Get current version
    pub fn current_version(&self) -> String {
        self.current_version.read().clone()
    }

    /// Cancel proposal
    pub fn cancel_proposal(&self, proposal_id: &H256, canceller: &Address) -> Result<(), UpgradeError> {
        let mut proposals = self.proposals.write();
        let proposal = proposals.get_mut(proposal_id)
            .ok_or_else(|| UpgradeError::ProposalNotFound(*proposal_id))?;

        proposal.cancel(canceller)?;
        
        // Remove from pending if scheduled
        self.pending.write().retain(|id| id != proposal_id);

        Ok(())
    }

    /// Check if version is compatible
    pub fn is_compatible(&self, version: &str) -> bool {
        let current = self.current_version.read();
        // Simple version check - production would use semver
        version >= current.as_str()
    }

    /// Get statistics
    pub fn stats(&self) -> UpgradeStats {
        self.stats.read().clone()
    }
}

impl Default for UpgradeManager {
    fn default() -> Self {
        Self::new(UpgradeConfig::default(), "1.0.0".to_string())
    }
}

/// Upgrade statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpgradeStats {
    pub proposals_submitted: u64,
    pub votes_cast: u64,
    pub proposals_approved: u64,
    pub proposals_rejected: u64,
    pub upgrades_activated: u64,
}

/// Upgrade errors
#[derive(Debug, thiserror::Error)]
pub enum UpgradeError {
    #[error("Upgrades are disabled")]
    UpgradesDisabled,

    #[error("Proposal not found: {0:?}")]
    ProposalNotFound(H256),

    #[error("Voting is closed")]
    VotingClosed,

    #[error("Already voted: {0:?}")]
    AlreadyVoted(Address),

    #[error("Voting period not ended")]
    VotingNotEnded,

    #[error("Proposal not approved")]
    NotApproved,

    #[error("Proposal not scheduled")]
    NotScheduled,

    #[error("Not the proposer")]
    NotProposer,

    #[error("Already activated")]
    AlreadyActivated,

    #[error("Insufficient stake")]
    InsufficientStake,

    #[error("Invalid version: {0}")]
    InvalidVersion(String),
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
    fn test_upgrade_type() {
        assert_eq!(UpgradeType::HardFork.min_approval_percent(), 75);
        assert!(UpgradeType::HardFork.min_voting_period() > UpgradeType::SoftFork.min_voting_period());
    }

    #[test]
    fn test_upgrade_proposal() {
        let proposal = UpgradeProposal::new(
            UpgradeType::SoftFork,
            "Test Upgrade".to_string(),
            "Test description".to_string(),
            Address([1u8; 20]),
            "1.1.0".to_string(),
            100,
        );

        assert_eq!(proposal.status, UpgradeStatus::Proposed);
        assert_eq!(proposal.votes_for, 0);
    }

    #[test]
    fn test_voting() {
        let mut proposal = UpgradeProposal::new(
            UpgradeType::ParameterChange,
            "Param Change".to_string(),
            "Change gas limit".to_string(),
            Address([1u8; 20]),
            "1.0.1".to_string(),
            100,
        );

        proposal.vote(Address([2u8; 20]), 1000, true).unwrap();
        proposal.vote(Address([3u8; 20]), 500, false).unwrap();

        assert_eq!(proposal.votes_for, 1000);
        assert_eq!(proposal.votes_against, 500);
        assert_eq!(proposal.approval_percent(), 66);
    }

    #[test]
    fn test_upgrade_manager() {
        let manager = UpgradeManager::default();
        
        let id = manager.submit_proposal(
            UpgradeType::ParameterChange,
            "Test".to_string(),
            "Test".to_string(),
            Address([1u8; 20]),
            "1.1.0".to_string(),
        ).unwrap();

        assert!(manager.get_proposal(&id).is_some());
    }

    #[test]
    fn test_vote_on_proposal() {
        let manager = UpgradeManager::default();
        
        let id = manager.submit_proposal(
            UpgradeType::ParameterChange,
            "Test".to_string(),
            "Test".to_string(),
            Address([1u8; 20]),
            "1.1.0".to_string(),
        ).unwrap();

        manager.vote(&id, Address([2u8; 20]), 1000, true).unwrap();
        
        let proposal = manager.get_proposal(&id).unwrap();
        assert_eq!(proposal.votes_for, 1000);
    }

    #[test]
    fn test_version_compatibility() {
        let manager = UpgradeManager::default();
        assert!(manager.is_compatible("1.0.0"));
        assert!(manager.is_compatible("1.1.0"));
    }
}
