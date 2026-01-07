//! L2 ZK Rollup Infrastructure
//!
//! Zero-knowledge rollup layer for PYRAX with:
//! - ZK proof generation (prover pipeline)
//! - On-chain verification
//! - Decentralized sequencer
//! - Data availability guarantees
//! - L2 ↔ L1 bridge with withdrawals

mod prover;
mod verifier;
mod sequencer;
mod batch;
mod state;
mod bridge;
mod da;

pub use prover::{ZkProver, ProverConfig, Proof, ProofType, ProverError};
pub use verifier::{ZkVerifier, VerifierConfig, VerificationResult};
pub use sequencer::{Sequencer, SequencerConfig, SequencerMode, SequencerError};
pub use batch::{Batch, BatchHeader, BatchStatus, BatchBuilder};
pub use state::{L2State, StateCommitment, StateTransition, MerkleProof};
pub use bridge::{L2Bridge, Deposit, Withdrawal, WithdrawalStatus, BridgeError};
pub use da::{DataAvailability, DaConfig, DaCommitment, DaProvider};

/// L2 chain ID (distinct from L1)
pub const L2_CHAIN_ID: u64 = 0x505952_02; // "PYR" + 02 for L2

/// Maximum transactions per batch
pub const MAX_BATCH_SIZE: usize = 1000;

/// Batch submission interval (seconds)
pub const BATCH_INTERVAL_SECS: u64 = 60;

/// Challenge period for withdrawals (7 days)
pub const WITHDRAWAL_CHALLENGE_PERIOD: u64 = 7 * 24 * 60 * 60;

/// Minimum stake for sequencer (10,000 PYRAX)
pub const MIN_SEQUENCER_STAKE: u64 = 10_000 * 100_000_000;

/// Proof generation timeout (10 minutes)
pub const PROOF_TIMEOUT_SECS: u64 = 600;

/// Maximum L2 block gas limit
pub const L2_BLOCK_GAS_LIMIT: u64 = 30_000_000;

/// L2 block time (2 seconds)
pub const L2_BLOCK_TIME_SECS: u64 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(L2_CHAIN_ID, 0x505952_02);
        assert_eq!(MAX_BATCH_SIZE, 1000);
        assert_eq!(WITHDRAWAL_CHALLENGE_PERIOD, 604800);
    }
}
