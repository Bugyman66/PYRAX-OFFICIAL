//! Checkpoint Finality
//!
//! Implements checkpoint-based finality for Stream C

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use blake3::Hasher;

use crate::types::Address;

type H256 = crate::types::H256;
use super::validator::{ValidatorSet, ValidatorStatus};
use super::{CHECKPOINT_INTERVAL, CHECKPOINT_BASE_REWARD};

/// Checkpoint status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointStatus {
    /// Proposed, awaiting votes
    Proposed,
    /// Has quorum, pending finalization
    Justified,
    /// Finalized (irreversible)
    Finalized,
    /// Rejected (conflicting checkpoint finalized)
    Rejected,
}

/// Checkpoint vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointVote {
    /// Voter address
    pub voter: Address,
    /// Checkpoint hash being voted for
    pub checkpoint_hash: [u8; 32],
    /// Voter's voting power at time of vote
    pub voting_power: u64,
    /// Signature over checkpoint hash
    pub signature: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
}

impl CheckpointVote {
    /// Create a new vote
    pub fn new(voter: Address, checkpoint_hash: [u8; 32], voting_power: u64, signature: Vec<u8>) -> Self {
        Self {
            voter,
            checkpoint_hash,
            voting_power,
            signature,
            timestamp: current_timestamp(),
        }
    }

    /// Verify vote signature
    pub fn verify(&self, _pubkey: &[u8; 32]) -> bool {
        // In production, verify signature using ed25519 or BLS
        // For now, accept all votes (signature verification would be integrated with key management)
        self.voting_power > 0
    }
}

/// Finality checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint height (Stream A block number)
    pub height: u64,
    /// Epoch number
    pub epoch: u64,
    /// Hash of the checkpointed Stream A block
    pub block_hash: H256,
    /// State root at checkpoint
    pub state_root: H256,
    /// Hash of previous checkpoint
    pub parent_checkpoint: H256,
    /// Validator set hash at this checkpoint
    pub validator_set_hash: H256,
    /// Checkpoint status
    pub status: CheckpointStatus,
    /// Votes received
    pub votes: HashMap<Address, CheckpointVote>,
    /// Total voting power in votes
    pub vote_power: u64,
    /// Required voting power for quorum
    pub quorum: u64,
    /// Proposer address
    pub proposer: Address,
    /// Creation timestamp
    pub created_at: u64,
    /// Finalization timestamp
    pub finalized_at: Option<u64>,
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(
        height: u64,
        epoch: u64,
        block_hash: H256,
        state_root: H256,
        parent_checkpoint: H256,
        validator_set: &ValidatorSet,
        proposer: Address,
    ) -> Self {
        let validator_set_hash = Self::compute_validator_set_hash(validator_set);
        let quorum = validator_set.quorum();

        Self {
            height,
            epoch,
            block_hash,
            state_root,
            parent_checkpoint,
            validator_set_hash,
            status: CheckpointStatus::Proposed,
            votes: HashMap::new(),
            vote_power: 0,
            quorum,
            proposer,
            created_at: current_timestamp(),
            finalized_at: None,
        }
    }

    /// Compute checkpoint hash
    pub fn hash(&self) -> H256 {
        let mut hasher = Hasher::new();
        hasher.update(&self.height.to_be_bytes());
        hasher.update(&self.epoch.to_be_bytes());
        hasher.update(&self.block_hash.0);
        hasher.update(&self.state_root.0);
        hasher.update(&self.parent_checkpoint.0);
        hasher.update(&self.validator_set_hash.0);
        let result = hasher.finalize();
        H256::from_slice(result.as_bytes())
    }

    /// Add a vote
    pub fn add_vote(&mut self, vote: CheckpointVote) -> bool {
        if self.status != CheckpointStatus::Proposed {
            return false;
        }

        // Check vote is for this checkpoint
        let hash = self.hash();
        if vote.checkpoint_hash != hash.0 {
            return false;
        }

        // Check not already voted
        if self.votes.contains_key(&vote.voter) {
            return false;
        }

        self.vote_power += vote.voting_power;
        self.votes.insert(vote.voter, vote);

        // Check if we have quorum
        if self.vote_power >= self.quorum {
            self.status = CheckpointStatus::Justified;
        }

        true
    }

    /// Finalize the checkpoint
    pub fn finalize(&mut self) -> bool {
        if self.status != CheckpointStatus::Justified {
            return false;
        }

        self.status = CheckpointStatus::Finalized;
        self.finalized_at = Some(current_timestamp());
        true
    }

    /// Reject the checkpoint
    pub fn reject(&mut self) {
        if self.status != CheckpointStatus::Finalized {
            self.status = CheckpointStatus::Rejected;
        }
    }

    /// Check if checkpoint has quorum
    pub fn has_quorum(&self) -> bool {
        self.vote_power >= self.quorum
    }

    /// Get vote count
    pub fn vote_count(&self) -> usize {
        self.votes.len()
    }

    /// Compute hash of validator set
    fn compute_validator_set_hash(set: &ValidatorSet) -> H256 {
        let mut hasher = Hasher::new();
        hasher.update(&set.epoch.to_be_bytes());
        for addr in &set.validators {
            hasher.update(&addr.0);
            if let Some(&power) = set.voting_power.get(addr) {
                hasher.update(&power.to_be_bytes());
            }
        }
        let result = hasher.finalize();
        H256::from_slice(result.as_bytes())
    }
}

/// Finality errors
#[derive(Debug, thiserror::Error)]
pub enum FinalityError {
    #[error("Checkpoint not found: height {0}")]
    CheckpointNotFound(u64),

    #[error("Invalid checkpoint height: expected {expected}, got {actual}")]
    InvalidHeight { expected: u64, actual: u64 },

    #[error("Parent checkpoint not found: {0:?}")]
    ParentNotFound(H256),

    #[error("Checkpoint already exists at height {0}")]
    CheckpointExists(u64),

    #[error("Invalid vote: {0}")]
    InvalidVote(String),

    #[error("Not in validator set")]
    NotValidator,

    #[error("Checkpoint not justified")]
    NotJustified,

    #[error("Conflicting checkpoint")]
    ConflictingCheckpoint,

    #[error("Invalid signature")]
    InvalidSignature,
}

/// Checkpoint manager
pub struct CheckpointManager {
    /// All checkpoints by height
    checkpoints: Arc<RwLock<HashMap<u64, Checkpoint>>>,
    /// Checkpoint by hash
    by_hash: Arc<RwLock<HashMap<H256, u64>>>,
    /// Last finalized checkpoint height
    last_finalized: Arc<RwLock<u64>>,
    /// Last justified checkpoint height
    last_justified: Arc<RwLock<u64>>,
    /// Pending checkpoints awaiting votes
    pending: Arc<RwLock<Vec<u64>>>,
}

impl CheckpointManager {
    /// Create new checkpoint manager
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
            by_hash: Arc::new(RwLock::new(HashMap::new())),
            last_finalized: Arc::new(RwLock::new(0)),
            last_justified: Arc::new(RwLock::new(0)),
            pending: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create genesis checkpoint
    pub fn create_genesis(&self, genesis_hash: H256, state_root: H256) -> Checkpoint {
        let checkpoint = Checkpoint {
            height: 0,
            epoch: 0,
            block_hash: genesis_hash,
            state_root,
            parent_checkpoint: H256::zero(),
            validator_set_hash: H256::zero(),
            status: CheckpointStatus::Finalized,
            votes: HashMap::new(),
            vote_power: 0,
            quorum: 0,
            proposer: Address::ZERO,
            created_at: current_timestamp(),
            finalized_at: Some(current_timestamp()),
        };

        let hash = checkpoint.hash();
        self.checkpoints.write().insert(0, checkpoint.clone());
        self.by_hash.write().insert(hash, 0);

        checkpoint
    }

    /// Propose a new checkpoint
    pub fn propose_checkpoint(
        &self,
        height: u64,
        epoch: u64,
        block_hash: H256,
        state_root: H256,
        validator_set: &ValidatorSet,
        proposer: Address,
    ) -> Result<Checkpoint, FinalityError> {
        // Validate height is at checkpoint interval
        if height % CHECKPOINT_INTERVAL != 0 {
            return Err(FinalityError::InvalidHeight {
                expected: (height / CHECKPOINT_INTERVAL + 1) * CHECKPOINT_INTERVAL,
                actual: height,
            });
        }

        // Check doesn't already exist
        if self.checkpoints.read().contains_key(&height) {
            return Err(FinalityError::CheckpointExists(height));
        }

        // Get parent checkpoint
        let parent_height = height.saturating_sub(CHECKPOINT_INTERVAL);
        let parent_hash = if parent_height == 0 {
            self.checkpoints.read()
                .get(&0)
                .map(|c| c.hash())
                .unwrap_or(H256::zero())
        } else {
            self.checkpoints.read()
                .get(&parent_height)
                .ok_or(FinalityError::ParentNotFound(H256::zero()))?
                .hash()
        };

        let checkpoint = Checkpoint::new(
            height,
            epoch,
            block_hash,
            state_root,
            parent_hash,
            validator_set,
            proposer,
        );

        let hash = checkpoint.hash();
        self.checkpoints.write().insert(height, checkpoint.clone());
        self.by_hash.write().insert(hash, height);
        self.pending.write().push(height);

        Ok(checkpoint)
    }

    /// Add vote to checkpoint
    pub fn add_vote(
        &self,
        height: u64,
        vote: CheckpointVote,
        validator_set: &ValidatorSet,
    ) -> Result<bool, FinalityError> {
        // Verify voter is in validator set
        if !validator_set.contains(&vote.voter) {
            return Err(FinalityError::NotValidator);
        }

        let mut checkpoints = self.checkpoints.write();
        let checkpoint = checkpoints.get_mut(&height)
            .ok_or(FinalityError::CheckpointNotFound(height))?;

        let added = checkpoint.add_vote(vote);

        // Check if newly justified
        if checkpoint.status == CheckpointStatus::Justified {
            *self.last_justified.write() = height;
        }

        Ok(added)
    }

    /// Finalize a justified checkpoint
    pub fn finalize_checkpoint(&self, height: u64) -> Result<u64, FinalityError> {
        let mut checkpoints = self.checkpoints.write();
        let checkpoint = checkpoints.get_mut(&height)
            .ok_or(FinalityError::CheckpointNotFound(height))?;

        if checkpoint.status != CheckpointStatus::Justified {
            return Err(FinalityError::NotJustified);
        }

        checkpoint.finalize();
        *self.last_finalized.write() = height;

        // Remove from pending
        self.pending.write().retain(|&h| h != height);

        // Reject any conflicting checkpoints at same height
        // (In a more complex implementation, we'd track forks)

        // Calculate rewards
        let reward = CHECKPOINT_BASE_REWARD;
        Ok(reward)
    }

    /// Get checkpoint by height
    pub fn get_checkpoint(&self, height: u64) -> Option<Checkpoint> {
        self.checkpoints.read().get(&height).cloned()
    }

    /// Get checkpoint by hash
    pub fn get_checkpoint_by_hash(&self, hash: &H256) -> Option<Checkpoint> {
        let height = self.by_hash.read().get(hash).copied()?;
        self.get_checkpoint(height)
    }

    /// Get last finalized height
    pub fn last_finalized_height(&self) -> u64 {
        *self.last_finalized.read()
    }

    /// Get last justified height
    pub fn last_justified_height(&self) -> u64 {
        *self.last_justified.read()
    }

    /// Get pending checkpoints
    pub fn pending_checkpoints(&self) -> Vec<u64> {
        self.pending.read().clone()
    }

    /// Check if a block is finalized
    pub fn is_finalized(&self, block_height: u64) -> bool {
        let last_finalized = *self.last_finalized.read();
        block_height <= last_finalized
    }

    /// Get finality depth (blocks since last finalized)
    pub fn finality_depth(&self, current_height: u64) -> u64 {
        let last_finalized = *self.last_finalized.read();
        current_height.saturating_sub(last_finalized)
    }

    /// Should create checkpoint at this height?
    pub fn should_checkpoint(&self, height: u64) -> bool {
        height > 0 && height % CHECKPOINT_INTERVAL == 0
    }

    /// Get all checkpoints in range
    pub fn get_checkpoints_in_range(&self, start: u64, end: u64) -> Vec<Checkpoint> {
        let checkpoints = self.checkpoints.read();
        (start..=end)
            .filter(|&h| h % CHECKPOINT_INTERVAL == 0)
            .filter_map(|h| checkpoints.get(&h).cloned())
            .collect()
    }

    /// Get checkpoint statistics
    pub fn stats(&self) -> CheckpointStats {
        let checkpoints = self.checkpoints.read();
        let pending = self.pending.read();

        CheckpointStats {
            total_checkpoints: checkpoints.len() as u64,
            pending_checkpoints: pending.len() as u64,
            last_finalized: *self.last_finalized.read(),
            last_justified: *self.last_justified.read(),
            finalized_count: checkpoints.values()
                .filter(|c| c.status == CheckpointStatus::Finalized)
                .count() as u64,
        }
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Checkpoint statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStats {
    pub total_checkpoints: u64,
    pub pending_checkpoints: u64,
    pub last_finalized: u64,
    pub last_justified: u64,
    pub finalized_count: u64,
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
    use super::super::validator::ValidatorSelection;

    fn make_validator_set() -> ValidatorSet {
        let mut set = ValidatorSet::new(1, ValidatorSelection::RoundRobin);
        let addr1 = Address([1u8; 20]);
        let addr2 = Address([2u8; 20]);
        let addr3 = Address([3u8; 20]);
        
        set.validators = vec![addr1, addr2, addr3];
        set.voting_power.insert(addr1, 100);
        set.voting_power.insert(addr2, 100);
        set.voting_power.insert(addr3, 100);
        set.total_voting_power = 300;
        
        set
    }

    #[test]
    fn test_checkpoint_creation() {
        let set = make_validator_set();
        let checkpoint = Checkpoint::new(
            100,
            1,
            crate::types::H256::zero(),
            crate::types::H256::zero(),
            crate::types::H256::zero(),
            &set,
            Address([1u8; 20]),
        );

        assert_eq!(checkpoint.height, 100);
        assert_eq!(checkpoint.status, CheckpointStatus::Proposed);
        assert_eq!(checkpoint.quorum, 201); // 2/3 of 300 + 1
    }

    #[test]
    fn test_checkpoint_voting() {
        let set = make_validator_set();
        let mut checkpoint = Checkpoint::new(
            100,
            1,
            crate::types::H256::zero(),
            crate::types::H256::zero(),
            crate::types::H256::zero(),
            &set,
            Address([1u8; 20]),
        );

        let hash = checkpoint.hash();

        // Add votes
        let vote1 = CheckpointVote::new(Address([1u8; 20]), hash.0, 100, vec![0u8; 64]);
        assert!(checkpoint.add_vote(vote1));
        assert_eq!(checkpoint.status, CheckpointStatus::Proposed);

        let vote2 = CheckpointVote::new(Address([2u8; 20]), hash.0, 100, vec![0u8; 64]);
        assert!(checkpoint.add_vote(vote2));
        assert_eq!(checkpoint.status, CheckpointStatus::Proposed);

        // Third vote should reach quorum
        let vote3 = CheckpointVote::new(Address([3u8; 20]), hash.0, 100, vec![0u8; 64]);
        assert!(checkpoint.add_vote(vote3));
        assert_eq!(checkpoint.status, CheckpointStatus::Justified);
    }

    #[test]
    fn test_checkpoint_manager() {
        let manager = CheckpointManager::new();
        let genesis = manager.create_genesis(H256::zero(), H256::zero());
        
        assert_eq!(genesis.height, 0);
        assert_eq!(genesis.status, CheckpointStatus::Finalized);
        assert_eq!(manager.last_finalized_height(), 0);
    }

    #[test]
    fn test_checkpoint_interval() {
        let manager = CheckpointManager::new();
        
        assert!(!manager.should_checkpoint(0));
        assert!(!manager.should_checkpoint(50));
        assert!(manager.should_checkpoint(100));
        assert!(manager.should_checkpoint(200));
    }

    #[test]
    fn test_propose_checkpoint() {
        let manager = CheckpointManager::new();
        manager.create_genesis(crate::types::H256::zero(), crate::types::H256::zero());
        
        let set = make_validator_set();
        let result = manager.propose_checkpoint(
            100,
            1,
            crate::types::H256([1u8; 32]),
            crate::types::H256([2u8; 32]),
            &set,
            Address([1u8; 20]),
        );

        assert!(result.is_ok());
        let checkpoint = result.unwrap();
        assert_eq!(checkpoint.height, 100);
        assert!(manager.get_checkpoint(100).is_some());
    }
}
