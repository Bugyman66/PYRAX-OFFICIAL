//! AI/ML Compute Infrastructure
//!
//! Decentralized compute network for AI/ML workloads on PYRAX.
//!
//! Key features:
//! - Job scheduling and lifecycle management
//! - Model registry with versioning
//! - Compute node management and reputation
//! - AI marketplace with pricing
//! - Result verification and dispute resolution

mod job;
mod model;
mod node;
mod marketplace;
mod scheduler;
mod verification;

pub use job::{ComputeJob, JobSpec, JobStatus, JobResult, JobError};
pub use model::{ModelRegistry, ModelInfo, ModelVersion, ModelType};
pub use node::{ComputeNode, NodeSpec, NodeStatus, NodeCapability, ComputePool};
pub use marketplace::{AIMarketplace, Listing, ListingType, Bid, Order, OrderStatus};
pub use scheduler::{JobScheduler, SchedulingPolicy, ResourceAllocation};
pub use verification::{ResultVerifier, VerificationMethod, VerificationResult};

/// Maximum job duration (24 hours)
pub const MAX_JOB_DURATION_SECS: u64 = 24 * 60 * 60;

/// Minimum compute stake (1000 PYRAX)
pub const MIN_COMPUTE_STAKE: u64 = 1000 * 100_000_000;

/// Maximum concurrent jobs per node
pub const MAX_CONCURRENT_JOBS: u32 = 10;

/// Job result submission timeout (1 hour)
pub const RESULT_TIMEOUT_SECS: u64 = 60 * 60;

/// Verification quorum (3 nodes minimum)
pub const VERIFICATION_QUORUM: u32 = 3;

/// Platform fee percentage (2%)
pub const PLATFORM_FEE_PERCENT: u8 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MAX_JOB_DURATION_SECS, 86400);
        assert_eq!(MIN_COMPUTE_STAKE, 100_000_000_000);
        assert_eq!(VERIFICATION_QUORUM, 3);
    }
}
