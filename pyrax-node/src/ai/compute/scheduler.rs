//! Job Scheduler
//!
//! Intelligent job scheduling and resource allocation

use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::job::{ComputeJob, JobSpec, JobStatus, ResourceRequirements};
use super::node::{ComputeNode, NodeStatus};

/// Scheduling policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulingPolicy {
    /// First come, first served
    FCFS,
    /// Shortest job first
    ShortestFirst,
    /// Highest priority first
    PriorityBased,
    /// Lowest price first
    LowestPrice,
    /// Best reputation first
    BestReputation,
    /// Geographic locality
    Locality,
    /// Balanced (default)
    Balanced,
}

/// Resource allocation for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Job ID
    pub job_id: H256,
    /// Assigned node address
    pub node: Address,
    /// Allocated GPU memory (MB)
    pub gpu_memory_mb: u64,
    /// Allocated GPU compute units
    pub gpu_compute_units: u32,
    /// Allocated CPU cores
    pub cpu_cores: u32,
    /// Allocated RAM (MB)
    pub ram_mb: u64,
    /// Allocated storage (MB)
    pub storage_mb: u64,
    /// Agreed price
    pub price: u64,
    /// Allocation timestamp
    pub allocated_at: u64,
    /// Expected duration (seconds)
    pub expected_duration: u64,
}

/// Job queue entry with priority
#[derive(Debug, Clone)]
struct QueuedJob {
    job: ComputeJob,
    priority_score: u64,
    queued_at: u64,
}

impl PartialEq for QueuedJob {
    fn eq(&self, other: &Self) -> bool {
        self.job.id == other.job.id
    }
}

impl Eq for QueuedJob {}

impl PartialOrd for QueuedJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier queue time
        self.priority_score.cmp(&other.priority_score)
            .then_with(|| other.queued_at.cmp(&self.queued_at))
    }
}

/// Node score for scheduling decisions
#[derive(Debug, Clone)]
struct NodeScore {
    node: ComputeNode,
    score: u64,
    price: u64,
}

impl PartialEq for NodeScore {
    fn eq(&self, other: &Self) -> bool {
        self.node.address == other.node.address
    }
}

impl Eq for NodeScore {}

impl PartialOrd for NodeScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

/// Job scheduler
pub struct JobScheduler {
    /// Scheduling policy
    policy: SchedulingPolicy,
    /// Job queue
    queue: Arc<RwLock<BinaryHeap<QueuedJob>>>,
    /// Active allocations
    allocations: Arc<RwLock<HashMap<H256, ResourceAllocation>>>,
    /// Node assignments (node -> job IDs)
    node_jobs: Arc<RwLock<HashMap<Address, Vec<H256>>>>,
    /// Scheduling statistics
    stats: Arc<RwLock<SchedulerStats>>,
    /// Price weight (0-100)
    price_weight: u32,
    /// Reputation weight (0-100)
    reputation_weight: u32,
    /// Locality weight (0-100)
    locality_weight: u32,
}

impl JobScheduler {
    /// Create new scheduler with policy
    pub fn new(policy: SchedulingPolicy) -> Self {
        Self {
            policy,
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            node_jobs: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(SchedulerStats::default())),
            price_weight: 30,
            reputation_weight: 40,
            locality_weight: 30,
        }
    }

    /// Create with default policy (Balanced)
    pub fn with_defaults() -> Self {
        Self::new(SchedulingPolicy::Balanced)
    }

    /// Set policy
    pub fn set_policy(&mut self, policy: SchedulingPolicy) {
        self.policy = policy;
    }

    /// Enqueue job for scheduling
    pub fn enqueue(&self, job: ComputeJob) {
        let priority_score = self.calculate_priority(&job);
        let queued = QueuedJob {
            job,
            priority_score,
            queued_at: current_timestamp(),
        };
        self.queue.write().push(queued);
        self.stats.write().jobs_queued += 1;
    }

    /// Calculate job priority score
    fn calculate_priority(&self, job: &ComputeJob) -> u64 {
        let base_priority = job.spec.priority as u64 * 1000;
        let price_bonus = (job.spec.max_price / 1_000_000) as u64; // Higher paying jobs get priority
        let wait_penalty = 0; // Could add wait time factor

        base_priority + price_bonus - wait_penalty
    }

    /// Schedule next job from queue to available nodes
    pub fn schedule_next(&self, available_nodes: &[ComputeNode]) -> Option<ResourceAllocation> {
        // Get next job from queue
        let queued = self.queue.write().pop()?;
        let job = queued.job;

        // Find best node for job
        let best_node = self.find_best_node(&job.spec, available_nodes)?;

        // Calculate price
        let price = self.calculate_price(&job.spec, &best_node);

        // Check if within budget
        if price > job.spec.max_price {
            // Re-queue job
            self.enqueue(job);
            return None;
        }

        // Create allocation
        let allocation = ResourceAllocation {
            job_id: job.id,
            node: best_node.address,
            gpu_memory_mb: job.spec.resources.gpu_memory_mb,
            gpu_compute_units: job.spec.resources.gpu_compute_units,
            cpu_cores: job.spec.resources.cpu_cores,
            ram_mb: job.spec.resources.ram_mb,
            storage_mb: job.spec.resources.storage_mb,
            price,
            allocated_at: current_timestamp(),
            expected_duration: job.spec.resources.max_duration_secs,
        };

        // Record allocation
        self.allocations.write().insert(job.id, allocation.clone());
        self.node_jobs.write()
            .entry(best_node.address)
            .or_insert_with(Vec::new)
            .push(job.id);

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.jobs_scheduled += 1;
            stats.total_allocated_value += price;
        }

        Some(allocation)
    }

    /// Find best node for job based on policy
    fn find_best_node(&self, spec: &JobSpec, nodes: &[ComputeNode]) -> Option<ComputeNode> {
        // Filter nodes that meet requirements
        let eligible: Vec<_> = nodes.iter()
            .filter(|n| {
                n.status == NodeStatus::Active &&
                n.is_online() &&
                n.reputation >= spec.min_reputation &&
                n.spec.meets_requirements(&spec.resources)
            })
            .collect();

        if eligible.is_empty() {
            return None;
        }

        // Score nodes based on policy
        let mut scored: Vec<NodeScore> = eligible.iter()
            .map(|n| {
                let score = self.score_node(n, spec);
                let price = self.calculate_price(spec, n);
                NodeScore {
                    node: (*n).clone(),
                    score,
                    price,
                }
            })
            .collect();

        // Sort by score (highest first)
        scored.sort_by(|a, b| b.score.cmp(&a.score));

        scored.first().map(|s| s.node.clone())
    }

    /// Score a node for a job
    fn score_node(&self, node: &ComputeNode, spec: &JobSpec) -> u64 {
        match self.policy {
            SchedulingPolicy::FCFS => 100, // All equal
            SchedulingPolicy::ShortestFirst => {
                // Prefer nodes with faster completion times
                1000 / (node.avg_completion_time.max(1))
            }
            SchedulingPolicy::PriorityBased => {
                // Just use reputation
                node.reputation as u64 * 10
            }
            SchedulingPolicy::LowestPrice => {
                // Inverse of price (lower is better)
                let price = self.calculate_price(spec, node);
                if price == 0 { 1000 } else { 1_000_000_000 / price }
            }
            SchedulingPolicy::BestReputation => {
                node.reputation as u64 * 10
            }
            SchedulingPolicy::Locality => {
                // Would need job's preferred region
                100
            }
            SchedulingPolicy::Balanced => {
                self.balanced_score(node, spec)
            }
        }
    }

    /// Calculate balanced score considering multiple factors
    fn balanced_score(&self, node: &ComputeNode, spec: &JobSpec) -> u64 {
        // Reputation score (0-100)
        let rep_score = node.reputation as u64;

        // Price score (inverse, normalized)
        let price = self.calculate_price(spec, node);
        let price_score = if price == 0 {
            100
        } else if price > spec.max_price {
            0
        } else {
            ((spec.max_price - price) * 100 / spec.max_price) as u64
        };

        // Success rate score
        let success_score = (node.success_rate() * 100.0) as u64;

        // Compute score (how well it matches requirements)
        let compute_score = {
            let gpu_ratio = node.spec.total_gpu_memory_mb * 100 / spec.resources.gpu_memory_mb.max(1);
            gpu_ratio.min(100)
        };

        // Weighted average
        (rep_score * self.reputation_weight as u64 +
         price_score * self.price_weight as u64 +
         success_score * 20 +
         compute_score * 10) / 100
    }

    /// Calculate price for job on node
    fn calculate_price(&self, spec: &JobSpec, node: &ComputeNode) -> u64 {
        // Base price from node's hourly rate
        let base_price = node.calculate_price(spec.resources.max_duration_secs);

        // Adjust for GPU requirements
        let gpu_factor = spec.resources.gpu_compute_units as u64;

        base_price * gpu_factor.max(1)
    }

    /// Get allocation for job
    pub fn get_allocation(&self, job_id: &H256) -> Option<ResourceAllocation> {
        self.allocations.read().get(job_id).cloned()
    }

    /// Release allocation (job completed or failed)
    pub fn release_allocation(&self, job_id: &H256) {
        if let Some(allocation) = self.allocations.write().remove(job_id) {
            // Remove from node's job list
            if let Some(jobs) = self.node_jobs.write().get_mut(&allocation.node) {
                jobs.retain(|id| id != job_id);
            }
        }
    }

    /// Get jobs assigned to node
    pub fn get_node_jobs(&self, node: &Address) -> Vec<H256> {
        self.node_jobs.read()
            .get(node)
            .cloned()
            .unwrap_or_default()
    }

    /// Get queue length
    pub fn queue_length(&self) -> usize {
        self.queue.read().len()
    }

    /// Get active allocations count
    pub fn active_allocations(&self) -> usize {
        self.allocations.read().len()
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> SchedulerStats {
        self.stats.read().clone()
    }

    /// Clear completed allocations older than threshold
    pub fn cleanup(&self, max_age_secs: u64) {
        let now = current_timestamp();
        let threshold = now.saturating_sub(max_age_secs);

        let to_remove: Vec<_> = self.allocations.read()
            .iter()
            .filter(|(_, a)| a.allocated_at < threshold)
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            self.release_allocation(&id);
        }
    }
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// Total jobs queued
    pub jobs_queued: u64,
    /// Total jobs scheduled
    pub jobs_scheduled: u64,
    /// Total allocated value (PYRAX)
    pub total_allocated_value: u64,
    /// Average wait time (seconds)
    pub avg_wait_time: u64,
    /// Average allocation time (seconds)
    pub avg_allocation_time: u64,
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
    use super::super::node::{NodeSpec, GpuInfo, NodeCapability};
    use super::super::job::{JobType, VerificationType};

    fn make_job_spec() -> JobSpec {
        JobSpec {
            job_type: JobType::Inference,
            model_id: Some(H256([1u8; 32])),
            input_hash: H256([2u8; 32]),
            input_size: 1024,
            resources: ResourceRequirements::default(),
            max_price: 1_000_000_000,
            priority: 5,
            params: None,
            min_reputation: 30,
            verification: VerificationType::Consensus,
        }
    }

    fn make_node() -> ComputeNode {
        let spec = NodeSpec {
            cpu_model: "AMD EPYC".to_string(),
            cpu_cores: 64,
            ram_mb: 256 * 1024,
            storage_mb: 2 * 1024 * 1024,
            storage_type: "NVMe".to_string(),
            gpus: vec![GpuInfo {
                model: "NVIDIA A100".to_string(),
                vendor: "NVIDIA".to_string(),
                vram_mb: 80 * 1024,
                compute_capability: Some("8.0".to_string()),
                driver_version: "535.0".to_string(),
                cuda_version: Some("12.2".to_string()),
                index: 0,
            }],
            total_gpu_memory_mb: 80 * 1024,
            bandwidth_mbps: 10000,
            region: "us-east".to_string(),
            capabilities: vec![NodeCapability::GpuCuda],
        };

        let mut node = ComputeNode::new(
            Address([1u8; 20]),
            "https://node.example.com".to_string(),
            spec,
            super::super::MIN_COMPUTE_STAKE,
        );
        node.activate().unwrap();
        node.price_per_hour = 100_000_000; // 1 PYRAX/hour
        node
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = JobScheduler::with_defaults();
        assert_eq!(scheduler.queue_length(), 0);
    }

    #[test]
    fn test_job_enqueue() {
        let scheduler = JobScheduler::with_defaults();
        let job = ComputeJob::new(Address([1u8; 20]), make_job_spec());

        scheduler.enqueue(job);
        assert_eq!(scheduler.queue_length(), 1);
    }

    #[test]
    fn test_schedule_next() {
        let scheduler = JobScheduler::with_defaults();
        let job = ComputeJob::new(Address([1u8; 20]), make_job_spec());
        let node = make_node();

        scheduler.enqueue(job);
        let allocation = scheduler.schedule_next(&[node]);

        assert!(allocation.is_some());
        assert_eq!(scheduler.active_allocations(), 1);
    }

    #[test]
    fn test_release_allocation() {
        let scheduler = JobScheduler::with_defaults();
        let job = ComputeJob::new(Address([1u8; 20]), make_job_spec());
        let job_id = job.id;
        let node = make_node();

        scheduler.enqueue(job);
        scheduler.schedule_next(&[node]);
        assert_eq!(scheduler.active_allocations(), 1);

        scheduler.release_allocation(&job_id);
        assert_eq!(scheduler.active_allocations(), 0);
    }

    #[test]
    fn test_scheduling_policies() {
        let policies = vec![
            SchedulingPolicy::FCFS,
            SchedulingPolicy::PriorityBased,
            SchedulingPolicy::BestReputation,
            SchedulingPolicy::Balanced,
        ];

        for policy in policies {
            let scheduler = JobScheduler::new(policy);
            let job = ComputeJob::new(Address([1u8; 20]), make_job_spec());
            let node = make_node();

            scheduler.enqueue(job);
            let allocation = scheduler.schedule_next(&[node]);
            assert!(allocation.is_some(), "Policy {:?} failed to schedule", policy);
        }
    }
}
