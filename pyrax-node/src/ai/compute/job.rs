//! Compute Job Management
//!
//! Job schema, lifecycle, and execution tracking

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::{MAX_JOB_DURATION_SECS, RESULT_TIMEOUT_SECS};

/// Job status lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job created, awaiting assignment
    Pending,
    /// Job assigned to compute node
    Assigned,
    /// Job is being executed
    Running,
    /// Job completed, awaiting verification
    Completed,
    /// Job verified and finalized
    Verified,
    /// Job failed
    Failed,
    /// Job cancelled by submitter
    Cancelled,
    /// Job timed out
    TimedOut,
    /// Job disputed
    Disputed,
}

/// Job type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobType {
    /// Model inference
    Inference,
    /// Model training
    Training,
    /// Fine-tuning existing model
    FineTuning,
    /// Data preprocessing
    DataProcessing,
    /// Custom compute task
    Custom,
}

/// Resource requirements for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum GPU memory (MB)
    pub gpu_memory_mb: u64,
    /// Minimum GPU compute units
    pub gpu_compute_units: u32,
    /// Minimum CPU cores
    pub cpu_cores: u32,
    /// Minimum RAM (MB)
    pub ram_mb: u64,
    /// Minimum storage (MB)
    pub storage_mb: u64,
    /// Required GPU type (optional)
    pub gpu_type: Option<String>,
    /// Maximum execution time (seconds)
    pub max_duration_secs: u64,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            gpu_memory_mb: 4096,
            gpu_compute_units: 1,
            cpu_cores: 4,
            ram_mb: 8192,
            storage_mb: 10240,
            gpu_type: None,
            max_duration_secs: 3600,
        }
    }
}

/// Job specification submitted by user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    /// Job type
    pub job_type: JobType,
    /// Model ID (for inference/fine-tuning)
    pub model_id: Option<H256>,
    /// Input data hash (IPFS or on-chain)
    pub input_hash: H256,
    /// Input data size (bytes)
    pub input_size: u64,
    /// Resource requirements
    pub resources: ResourceRequirements,
    /// Maximum price willing to pay (in PYRAX smallest unit)
    pub max_price: u64,
    /// Priority level (1-10)
    pub priority: u8,
    /// Custom parameters (JSON)
    pub params: Option<String>,
    /// Required node reputation minimum
    pub min_reputation: u32,
    /// Verification method
    pub verification: VerificationType,
}

/// Verification type for job results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationType {
    /// No verification (trust node)
    None,
    /// Single node verification
    Single,
    /// Multi-node consensus (default)
    Consensus,
    /// Zero-knowledge proof
    ZKProof,
    /// Trusted execution environment
    TEE,
}

/// Compute job instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJob {
    /// Unique job ID
    pub id: H256,
    /// Job submitter address
    pub submitter: Address,
    /// Job specification
    pub spec: JobSpec,
    /// Current status
    pub status: JobStatus,
    /// Assigned compute node (if any)
    pub assigned_node: Option<Address>,
    /// Price agreed upon
    pub agreed_price: u64,
    /// Escrow amount locked
    pub escrow_amount: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Assignment timestamp
    pub assigned_at: Option<u64>,
    /// Start timestamp
    pub started_at: Option<u64>,
    /// Completion timestamp
    pub completed_at: Option<u64>,
    /// Result hash (if completed)
    pub result_hash: Option<H256>,
    /// Result size (bytes)
    pub result_size: Option<u64>,
    /// Gas used for verification
    pub verification_gas: u64,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Retry count
    pub retry_count: u32,
    /// Maximum retries
    pub max_retries: u32,
}

impl ComputeJob {
    /// Create new compute job
    pub fn new(submitter: Address, spec: JobSpec) -> Self {
        let id = Self::generate_id(&submitter, &spec);
        let escrow = spec.max_price;
        
        Self {
            id,
            submitter,
            spec,
            status: JobStatus::Pending,
            assigned_node: None,
            agreed_price: 0,
            escrow_amount: escrow,
            created_at: current_timestamp(),
            assigned_at: None,
            started_at: None,
            completed_at: None,
            result_hash: None,
            result_size: None,
            verification_gas: 0,
            error: None,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// Generate job ID
    fn generate_id(submitter: &Address, spec: &JobSpec) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&submitter.0);
        hasher.update(&spec.input_hash.0);
        hasher.update(&current_timestamp().to_le_bytes());
        hasher.update(&spec.max_price.to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Assign job to compute node
    pub fn assign(&mut self, node: Address, price: u64) -> Result<(), JobError> {
        if self.status != JobStatus::Pending {
            return Err(JobError::InvalidState(self.status));
        }
        if price > self.spec.max_price {
            return Err(JobError::PriceTooHigh { offered: price, max: self.spec.max_price });
        }

        self.assigned_node = Some(node);
        self.agreed_price = price;
        self.status = JobStatus::Assigned;
        self.assigned_at = Some(current_timestamp());
        Ok(())
    }

    /// Start job execution
    pub fn start(&mut self) -> Result<(), JobError> {
        if self.status != JobStatus::Assigned {
            return Err(JobError::InvalidState(self.status));
        }

        self.status = JobStatus::Running;
        self.started_at = Some(current_timestamp());
        Ok(())
    }

    /// Complete job with result
    pub fn complete(&mut self, result_hash: H256, result_size: u64) -> Result<(), JobError> {
        if self.status != JobStatus::Running {
            return Err(JobError::InvalidState(self.status));
        }

        self.status = JobStatus::Completed;
        self.completed_at = Some(current_timestamp());
        self.result_hash = Some(result_hash);
        self.result_size = Some(result_size);
        Ok(())
    }

    /// Verify job result
    pub fn verify(&mut self) -> Result<(), JobError> {
        if self.status != JobStatus::Completed {
            return Err(JobError::InvalidState(self.status));
        }

        self.status = JobStatus::Verified;
        Ok(())
    }

    /// Fail job with error
    pub fn fail(&mut self, error: String) -> Result<(), JobError> {
        if self.status == JobStatus::Verified || self.status == JobStatus::Cancelled {
            return Err(JobError::InvalidState(self.status));
        }

        self.status = JobStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(current_timestamp());
        Ok(())
    }

    /// Cancel job
    pub fn cancel(&mut self) -> Result<(), JobError> {
        if self.status != JobStatus::Pending && self.status != JobStatus::Assigned {
            return Err(JobError::InvalidState(self.status));
        }

        self.status = JobStatus::Cancelled;
        self.completed_at = Some(current_timestamp());
        Ok(())
    }

    /// Check if job has timed out
    pub fn check_timeout(&mut self) -> bool {
        let now = current_timestamp();
        
        // Check assignment timeout (10 minutes)
        if self.status == JobStatus::Pending {
            if now - self.created_at > 600 {
                self.status = JobStatus::TimedOut;
                return true;
            }
        }
        
        // Check execution timeout
        if self.status == JobStatus::Running {
            if let Some(started) = self.started_at {
                if now - started > self.spec.resources.max_duration_secs {
                    self.status = JobStatus::TimedOut;
                    return true;
                }
            }
        }
        
        // Check result submission timeout
        if self.status == JobStatus::Completed {
            if let Some(completed) = self.completed_at {
                if now - completed > RESULT_TIMEOUT_SECS {
                    self.status = JobStatus::TimedOut;
                    return true;
                }
            }
        }

        false
    }

    /// Retry failed job
    pub fn retry(&mut self) -> Result<(), JobError> {
        if self.status != JobStatus::Failed && self.status != JobStatus::TimedOut {
            return Err(JobError::InvalidState(self.status));
        }
        if self.retry_count >= self.max_retries {
            return Err(JobError::MaxRetriesExceeded);
        }

        self.retry_count += 1;
        self.status = JobStatus::Pending;
        self.assigned_node = None;
        self.assigned_at = None;
        self.started_at = None;
        self.completed_at = None;
        self.result_hash = None;
        self.error = None;
        Ok(())
    }

    /// Get job duration (if completed)
    pub fn duration(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Check if job is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, JobStatus::Pending | JobStatus::Assigned | JobStatus::Running)
    }

    /// Check if job is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Verified | JobStatus::Failed | JobStatus::Cancelled | JobStatus::TimedOut
        )
    }
}

/// Job result submitted by compute node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Job ID
    pub job_id: H256,
    /// Compute node address
    pub node: Address,
    /// Result data hash
    pub result_hash: H256,
    /// Result size (bytes)
    pub result_size: u64,
    /// Execution metrics
    pub metrics: ExecutionMetrics,
    /// Node signature
    pub signature: Vec<u8>,
    /// Submission timestamp
    pub submitted_at: u64,
}

/// Execution metrics from compute node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// GPU time used (milliseconds)
    pub gpu_time_ms: u64,
    /// CPU time used (milliseconds)
    pub cpu_time_ms: u64,
    /// Peak memory usage (MB)
    pub peak_memory_mb: u64,
    /// Data transferred (bytes)
    pub data_transferred: u64,
    /// Number of iterations (for training)
    pub iterations: Option<u64>,
    /// Final loss value (for training)
    pub final_loss: Option<f64>,
}

/// Job errors
#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("Invalid job state: {0:?}")]
    InvalidState(JobStatus),

    #[error("Price too high: offered {offered}, max {max}")]
    PriceTooHigh { offered: u64, max: u64 },

    #[error("Insufficient escrow: required {required}, available {available}")]
    InsufficientEscrow { required: u64, available: u64 },

    #[error("Job not found: {0:?}")]
    NotFound(H256),

    #[error("Job already exists: {0:?}")]
    AlreadyExists(H256),

    #[error("Max retries exceeded")]
    MaxRetriesExceeded,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid result: {0}")]
    InvalidResult(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Timeout")]
    Timeout,
}

/// Job manager for tracking all jobs
pub struct JobManager {
    /// Active jobs by ID
    jobs: Arc<RwLock<HashMap<H256, ComputeJob>>>,
    /// Jobs by submitter
    by_submitter: Arc<RwLock<HashMap<Address, Vec<H256>>>>,
    /// Jobs by assigned node
    by_node: Arc<RwLock<HashMap<Address, Vec<H256>>>>,
    /// Job results
    results: Arc<RwLock<HashMap<H256, JobResult>>>,
    /// Total jobs submitted
    total_jobs: Arc<RwLock<u64>>,
    /// Total compute value (PYRAX)
    total_value: Arc<RwLock<u64>>,
}

impl JobManager {
    /// Create new job manager
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            by_submitter: Arc::new(RwLock::new(HashMap::new())),
            by_node: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            total_jobs: Arc::new(RwLock::new(0)),
            total_value: Arc::new(RwLock::new(0)),
        }
    }

    /// Submit new job
    pub fn submit_job(&self, submitter: Address, spec: JobSpec) -> Result<ComputeJob, JobError> {
        let job = ComputeJob::new(submitter, spec);
        let id = job.id;

        {
            let mut jobs = self.jobs.write();
            if jobs.contains_key(&id) {
                return Err(JobError::AlreadyExists(id));
            }
            jobs.insert(id, job.clone());
        }

        self.by_submitter.write()
            .entry(submitter)
            .or_insert_with(Vec::new)
            .push(id);

        *self.total_jobs.write() += 1;
        *self.total_value.write() += job.escrow_amount;

        Ok(job)
    }

    /// Get job by ID
    pub fn get_job(&self, id: &H256) -> Option<ComputeJob> {
        self.jobs.read().get(id).cloned()
    }

    /// Assign job to node
    pub fn assign_job(&self, job_id: &H256, node: Address, price: u64) -> Result<(), JobError> {
        let mut jobs = self.jobs.write();
        let job = jobs.get_mut(job_id)
            .ok_or_else(|| JobError::NotFound(*job_id))?;

        job.assign(node, price)?;

        drop(jobs);

        self.by_node.write()
            .entry(node)
            .or_insert_with(Vec::new)
            .push(*job_id);

        Ok(())
    }

    /// Start job execution
    pub fn start_job(&self, job_id: &H256, node: &Address) -> Result<(), JobError> {
        let mut jobs = self.jobs.write();
        let job = jobs.get_mut(job_id)
            .ok_or_else(|| JobError::NotFound(*job_id))?;

        if job.assigned_node != Some(*node) {
            return Err(JobError::Unauthorized("Not assigned to this node".to_string()));
        }

        job.start()
    }

    /// Submit job result
    pub fn submit_result(&self, result: JobResult) -> Result<(), JobError> {
        let mut jobs = self.jobs.write();
        let job = jobs.get_mut(&result.job_id)
            .ok_or_else(|| JobError::NotFound(result.job_id))?;

        if job.assigned_node != Some(result.node) {
            return Err(JobError::Unauthorized("Not assigned to this node".to_string()));
        }

        job.complete(result.result_hash, result.result_size)?;

        drop(jobs);

        self.results.write().insert(result.job_id, result);
        Ok(())
    }

    /// Verify job result
    pub fn verify_job(&self, job_id: &H256) -> Result<(), JobError> {
        let mut jobs = self.jobs.write();
        let job = jobs.get_mut(job_id)
            .ok_or_else(|| JobError::NotFound(*job_id))?;

        job.verify()
    }

    /// Fail job
    pub fn fail_job(&self, job_id: &H256, error: String) -> Result<(), JobError> {
        let mut jobs = self.jobs.write();
        let job = jobs.get_mut(job_id)
            .ok_or_else(|| JobError::NotFound(*job_id))?;

        job.fail(error)
    }

    /// Get jobs by submitter
    pub fn get_by_submitter(&self, submitter: &Address) -> Vec<ComputeJob> {
        let ids = self.by_submitter.read()
            .get(submitter)
            .cloned()
            .unwrap_or_default();

        let jobs = self.jobs.read();
        ids.iter()
            .filter_map(|id| jobs.get(id).cloned())
            .collect()
    }

    /// Get jobs by node
    pub fn get_by_node(&self, node: &Address) -> Vec<ComputeJob> {
        let ids = self.by_node.read()
            .get(node)
            .cloned()
            .unwrap_or_default();

        let jobs = self.jobs.read();
        ids.iter()
            .filter_map(|id| jobs.get(id).cloned())
            .collect()
    }

    /// Get pending jobs
    pub fn get_pending_jobs(&self) -> Vec<ComputeJob> {
        self.jobs.read()
            .values()
            .filter(|j| j.status == JobStatus::Pending)
            .cloned()
            .collect()
    }

    /// Check and update timed out jobs
    pub fn check_timeouts(&self) -> Vec<H256> {
        let mut timed_out = Vec::new();
        let mut jobs = self.jobs.write();

        for (id, job) in jobs.iter_mut() {
            if job.check_timeout() {
                timed_out.push(*id);
            }
        }

        timed_out
    }

    /// Get manager statistics
    pub fn stats(&self) -> JobManagerStats {
        let jobs = self.jobs.read();
        
        let mut by_status = HashMap::new();
        for job in jobs.values() {
            *by_status.entry(job.status).or_insert(0u64) += 1;
        }

        JobManagerStats {
            total_jobs: *self.total_jobs.read(),
            total_value: *self.total_value.read(),
            active_jobs: jobs.values().filter(|j| j.is_active()).count() as u64,
            completed_jobs: by_status.get(&JobStatus::Verified).copied().unwrap_or(0),
            failed_jobs: by_status.get(&JobStatus::Failed).copied().unwrap_or(0),
            by_status,
        }
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Job manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobManagerStats {
    pub total_jobs: u64,
    pub total_value: u64,
    pub active_jobs: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub by_status: HashMap<JobStatus, u64>,
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
            min_reputation: 50,
            verification: VerificationType::Consensus,
        }
    }

    #[test]
    fn test_job_creation() {
        let submitter = Address([1u8; 20]);
        let spec = make_job_spec();
        let job = ComputeJob::new(submitter, spec);

        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.submitter, submitter);
        assert!(job.is_active());
    }

    #[test]
    fn test_job_lifecycle() {
        let submitter = Address([1u8; 20]);
        let node = Address([2u8; 20]);
        let spec = make_job_spec();
        let mut job = ComputeJob::new(submitter, spec);

        // Assign
        assert!(job.assign(node, 500_000_000).is_ok());
        assert_eq!(job.status, JobStatus::Assigned);

        // Start
        assert!(job.start().is_ok());
        assert_eq!(job.status, JobStatus::Running);

        // Complete
        let result_hash = H256([3u8; 32]);
        assert!(job.complete(result_hash, 2048).is_ok());
        assert_eq!(job.status, JobStatus::Completed);

        // Verify
        assert!(job.verify().is_ok());
        assert_eq!(job.status, JobStatus::Verified);
        assert!(job.is_terminal());
    }

    #[test]
    fn test_job_manager() {
        let manager = JobManager::new();
        let submitter = Address([1u8; 20]);
        let spec = make_job_spec();

        let job = manager.submit_job(submitter, spec).unwrap();
        let job_id = job.id;

        assert!(manager.get_job(&job_id).is_some());
        assert_eq!(manager.get_by_submitter(&submitter).len(), 1);

        // Assign
        let node = Address([2u8; 20]);
        assert!(manager.assign_job(&job_id, node, 500_000_000).is_ok());

        // Start
        assert!(manager.start_job(&job_id, &node).is_ok());

        let stats = manager.stats();
        assert_eq!(stats.active_jobs, 1);
    }

    #[test]
    fn test_job_cancel() {
        let submitter = Address([1u8; 20]);
        let spec = make_job_spec();
        let mut job = ComputeJob::new(submitter, spec);

        assert!(job.cancel().is_ok());
        assert_eq!(job.status, JobStatus::Cancelled);
        assert!(job.is_terminal());
    }

    #[test]
    fn test_job_retry() {
        let submitter = Address([1u8; 20]);
        let spec = make_job_spec();
        let mut job = ComputeJob::new(submitter, spec);

        job.fail("Test error".to_string()).unwrap();
        assert_eq!(job.status, JobStatus::Failed);

        job.retry().unwrap();
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.retry_count, 1);
    }
}
