use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use tracing::{info, warn, debug, error};

use crate::types::H256;
use super::{AIJob, JobStatus, JobType, TrainingConfig};

/// AI Job Manager - handles job submission, assignment, and tracking
pub struct JobManager {
    /// All jobs by ID
    jobs: Arc<RwLock<HashMap<H256, AIJob>>>,
    /// Pending jobs queue (priority by max_price)
    pending_queue: Arc<RwLock<BinaryHeap<PendingJob>>>,
    /// Jobs by requester
    jobs_by_requester: Arc<RwLock<HashMap<[u8; 20], Vec<H256>>>>,
    /// Jobs by provider
    jobs_by_provider: Arc<RwLock<HashMap<[u8; 20], Vec<H256>>>>,
    /// Active job count
    active_jobs: Arc<RwLock<u32>>,
    /// Total completed jobs
    completed_jobs: Arc<RwLock<u64>>,
    /// Total PYRAX paid out
    total_paid: Arc<RwLock<u64>>,
}

/// Pending job wrapper for priority queue
#[derive(Clone, Eq, PartialEq)]
struct PendingJob {
    id: H256,
    max_price: u64,
    submitted_at: u64,
}

impl Ord for PendingJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher price = higher priority
        // Older submission = higher priority (FIFO within same price)
        other.max_price.cmp(&self.max_price)
            .then_with(|| self.submitted_at.cmp(&other.submitted_at))
    }
}

impl PartialOrd for PendingJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            pending_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            jobs_by_requester: Arc::new(RwLock::new(HashMap::new())),
            jobs_by_provider: Arc::new(RwLock::new(HashMap::new())),
            active_jobs: Arc::new(RwLock::new(0)),
            completed_jobs: Arc::new(RwLock::new(0)),
            total_paid: Arc::new(RwLock::new(0)),
        }
    }

    /// Submit a new AI job
    pub fn submit_job(&self, job: AIJob) -> Result<H256, JobError> {
        let job_id = job.id;

        // Validate job
        self.validate_job(&job)?;

        // Check for duplicate
        {
            let jobs = self.jobs.read();
            if jobs.contains_key(&job_id) {
                return Err(JobError::JobAlreadyExists(job_id));
            }
        }

        // Store job
        {
            let mut jobs = self.jobs.write();
            jobs.insert(job_id, job.clone());
        }

        // Add to pending queue
        {
            let mut queue = self.pending_queue.write();
            queue.push(PendingJob {
                id: job_id,
                max_price: job.max_price,
                submitted_at: job.submitted_at,
            });
        }

        // Update requester index
        {
            let mut by_requester = self.jobs_by_requester.write();
            by_requester.entry(job.requester).or_insert_with(Vec::new).push(job_id);
        }

        info!(
            "Job submitted: {} (type: {:?}, max_price: {} PYRAX)",
            hex::encode(&job_id.0[..8]),
            job.job_type,
            job.max_price / 100_000_000
        );

        Ok(job_id)
    }

    /// Validate job data
    fn validate_job(&self, job: &AIJob) -> Result<(), JobError> {
        if job.input.is_empty() {
            return Err(JobError::InvalidJobData("input cannot be empty".into()));
        }
        if job.max_price == 0 {
            return Err(JobError::InvalidJobData("max_price must be > 0".into()));
        }
        if job.collateral == 0 {
            return Err(JobError::InvalidJobData("collateral required".into()));
        }
        if job.timeout_secs == 0 || job.timeout_secs > 86400 {
            return Err(JobError::InvalidJobData("timeout must be 1-86400 seconds".into()));
        }
        Ok(())
    }

    /// Get job by ID
    pub fn get_job(&self, id: &H256) -> Option<AIJob> {
        self.jobs.read().get(id).cloned()
    }

    /// Get jobs by requester
    pub fn get_jobs_by_requester(&self, requester: &[u8; 20]) -> Vec<AIJob> {
        let by_requester = self.jobs_by_requester.read();
        let jobs = self.jobs.read();

        by_requester.get(requester)
            .map(|ids| ids.iter().filter_map(|id| jobs.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    /// Get jobs by provider
    pub fn get_jobs_by_provider(&self, provider: &[u8; 20]) -> Vec<AIJob> {
        let by_provider = self.jobs_by_provider.read();
        let jobs = self.jobs.read();

        by_provider.get(provider)
            .map(|ids| ids.iter().filter_map(|id| jobs.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    /// Get next pending job for a provider
    pub fn get_next_pending_job(&self, model_id: &H256) -> Option<AIJob> {
        let queue = self.pending_queue.read();
        let jobs = self.jobs.read();

        // Find first pending job that matches the model
        for pending in queue.iter() {
            if let Some(job) = jobs.get(&pending.id) {
                if job.status == JobStatus::Pending && &job.model_id == model_id {
                    return Some(job.clone());
                }
            }
        }
        None
    }

    /// Assign a job to a provider
    pub fn assign_job(&self, job_id: &H256, provider: [u8; 20]) -> Result<(), JobError> {
        let now = current_timestamp();

        {
            let mut jobs = self.jobs.write();
            let job = jobs.get_mut(job_id).ok_or(JobError::JobNotFound(*job_id))?;

            if job.status != JobStatus::Pending {
                return Err(JobError::InvalidState(
                    format!("Job is {:?}, expected Pending", job.status)
                ));
            }

            job.status = JobStatus::Assigned;
            job.provider = Some(provider);
            job.assigned_at = Some(now);
        }

        // Update provider index
        {
            let mut by_provider = self.jobs_by_provider.write();
            by_provider.entry(provider).or_insert_with(Vec::new).push(*job_id);
        }

        // Update active count
        {
            let mut active = self.active_jobs.write();
            *active += 1;
        }

        info!(
            "Job assigned: {} to provider {}",
            hex::encode(&job_id.0[..8]),
            hex::encode(&provider[..8])
        );

        Ok(())
    }

    /// Mark job as running
    pub fn start_job(&self, job_id: &H256, provider: &[u8; 20]) -> Result<(), JobError> {
        let mut jobs = self.jobs.write();
        let job = jobs.get_mut(job_id).ok_or(JobError::JobNotFound(*job_id))?;

        // Verify provider
        if job.provider.as_ref() != Some(provider) {
            return Err(JobError::NotAuthorized);
        }

        if job.status != JobStatus::Assigned {
            return Err(JobError::InvalidState(
                format!("Job is {:?}, expected Assigned", job.status)
            ));
        }

        job.status = JobStatus::Running;
        debug!("Job started: {}", hex::encode(&job_id.0[..8]));

        Ok(())
    }

    /// Complete a job with result
    pub fn complete_job(
        &self,
        job_id: &H256,
        provider: &[u8; 20],
        output: String,
        proof: Option<String>,
        actual_price: u64,
    ) -> Result<(), JobError> {
        let now = current_timestamp();

        {
            let mut jobs = self.jobs.write();
            let job = jobs.get_mut(job_id).ok_or(JobError::JobNotFound(*job_id))?;

            // Verify provider
            if job.provider.as_ref() != Some(provider) {
                return Err(JobError::NotAuthorized);
            }

            if job.status != JobStatus::Running && job.status != JobStatus::Assigned {
                return Err(JobError::InvalidState(
                    format!("Job is {:?}, expected Running or Assigned", job.status)
                ));
            }

            // Verify price doesn't exceed max
            if actual_price > job.max_price {
                return Err(JobError::PriceExceeded(job.max_price, actual_price));
            }

            job.status = JobStatus::Completed;
            job.output = Some(output);
            job.proof = proof;
            job.actual_price = Some(actual_price);
            job.completed_at = Some(now);
        }

        // Update counters
        {
            let mut active = self.active_jobs.write();
            *active = active.saturating_sub(1);
        }
        {
            let mut completed = self.completed_jobs.write();
            *completed += 1;
        }
        {
            let mut paid = self.total_paid.write();
            *paid += actual_price;
        }

        info!(
            "Job completed: {} (price: {} PYRAX)",
            hex::encode(&job_id.0[..8]),
            actual_price / 100_000_000
        );

        Ok(())
    }

    /// Fail a job
    pub fn fail_job(&self, job_id: &H256, error: String) -> Result<(), JobError> {
        let now = current_timestamp();

        {
            let mut jobs = self.jobs.write();
            let job = jobs.get_mut(job_id).ok_or(JobError::JobNotFound(*job_id))?;

            job.status = JobStatus::Failed;
            job.error = Some(error.clone());
            job.completed_at = Some(now);
        }

        // Update counters
        {
            let mut active = self.active_jobs.write();
            *active = active.saturating_sub(1);
        }

        warn!("Job failed: {} - {}", hex::encode(&job_id.0[..8]), error);

        Ok(())
    }

    /// Cancel a job (only requester can cancel pending jobs)
    pub fn cancel_job(&self, job_id: &H256, requester: &[u8; 20]) -> Result<u64, JobError> {
        let collateral;

        {
            let mut jobs = self.jobs.write();
            let job = jobs.get_mut(job_id).ok_or(JobError::JobNotFound(*job_id))?;

            // Verify requester
            if &job.requester != requester {
                return Err(JobError::NotAuthorized);
            }

            // Can only cancel pending jobs
            if job.status != JobStatus::Pending {
                return Err(JobError::InvalidState(
                    format!("Cannot cancel job in {:?} state", job.status)
                ));
            }

            collateral = job.collateral;
            job.status = JobStatus::Cancelled;
            job.completed_at = Some(current_timestamp());
        }

        info!("Job cancelled: {}", hex::encode(&job_id.0[..8]));

        // Return collateral amount for refund
        Ok(collateral)
    }

    /// Check for expired jobs and mark them
    pub fn check_expired_jobs(&self) -> Vec<H256> {
        let now = current_timestamp();
        let mut expired = Vec::new();

        let mut jobs = self.jobs.write();
        for (id, job) in jobs.iter_mut() {
            if job.status == JobStatus::Running || job.status == JobStatus::Assigned {
                let start_time = job.assigned_at.unwrap_or(job.submitted_at);
                if now > start_time + job.timeout_secs as u64 {
                    job.status = JobStatus::Expired;
                    job.error = Some("Job timed out".to_string());
                    job.completed_at = Some(now);
                    expired.push(*id);
                }
            }
        }

        if !expired.is_empty() {
            let mut active = self.active_jobs.write();
            *active = active.saturating_sub(expired.len() as u32);
            warn!("Expired {} jobs", expired.len());
        }

        expired
    }

    /// List pending jobs
    pub fn list_pending_jobs(&self, limit: usize) -> Vec<AIJob> {
        let jobs = self.jobs.read();
        jobs.values()
            .filter(|j| j.status == JobStatus::Pending)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get job statistics
    pub fn get_stats(&self) -> JobStats {
        let jobs = self.jobs.read();
        
        let pending = jobs.values().filter(|j| j.status == JobStatus::Pending).count();
        let running = jobs.values().filter(|j| j.status == JobStatus::Running).count();
        let completed = *self.completed_jobs.read();
        let failed = jobs.values().filter(|j| j.status == JobStatus::Failed).count();

        JobStats {
            total_jobs: jobs.len(),
            pending_jobs: pending,
            running_jobs: running,
            completed_jobs: completed,
            failed_jobs: failed,
            total_paid_pyrax: *self.total_paid.read() / 100_000_000,
        }
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Job statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct JobStats {
    pub total_jobs: usize,
    pub pending_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: u64,
    pub failed_jobs: usize,
    pub total_paid_pyrax: u64,
}

/// Job errors
#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("Job already exists: {0:?}")]
    JobAlreadyExists(H256),

    #[error("Job not found: {0:?}")]
    JobNotFound(H256),

    #[error("Invalid job data: {0}")]
    InvalidJobData(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Not authorized")]
    NotAuthorized,

    #[error("Price exceeded: max {0}, actual {1}")]
    PriceExceeded(u64, u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_job() -> AIJob {
        AIJob {
            id: H256([1u8; 32]),
            job_type: JobType::Inference,
            model_id: H256([2u8; 32]),
            requester: [0u8; 20],
            provider: None,
            status: JobStatus::Pending,
            input: r#"{"prompt": "Hello"}"#.to_string(),
            output: None,
            max_price: 100_000_000, // 1 PYRAX
            actual_price: None,
            collateral: 10_000_000, // 0.1 PYRAX
            submitted_at: current_timestamp(),
            submitted_height: 100,
            assigned_at: None,
            completed_at: None,
            error: None,
            proof: None,
            timeout_secs: 300,
        }
    }

    #[test]
    fn test_submit_job() {
        let manager = JobManager::new();
        let job = create_test_job();

        let result = manager.submit_job(job.clone());
        assert!(result.is_ok());

        let fetched = manager.get_job(&job.id);
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().status, JobStatus::Pending);
    }

    #[test]
    fn test_assign_job() {
        let manager = JobManager::new();
        let job = create_test_job();
        let provider = [1u8; 20];

        manager.submit_job(job.clone()).unwrap();
        manager.assign_job(&job.id, provider).unwrap();

        let fetched = manager.get_job(&job.id).unwrap();
        assert_eq!(fetched.status, JobStatus::Assigned);
        assert_eq!(fetched.provider, Some(provider));
    }

    #[test]
    fn test_complete_job() {
        let manager = JobManager::new();
        let job = create_test_job();
        let provider = [1u8; 20];

        manager.submit_job(job.clone()).unwrap();
        manager.assign_job(&job.id, provider).unwrap();
        manager.complete_job(
            &job.id,
            &provider,
            "output result".to_string(),
            None,
            50_000_000,
        ).unwrap();

        let fetched = manager.get_job(&job.id).unwrap();
        assert_eq!(fetched.status, JobStatus::Completed);
        assert_eq!(fetched.actual_price, Some(50_000_000));
    }

    #[test]
    fn test_cancel_job() {
        let manager = JobManager::new();
        let job = create_test_job();

        manager.submit_job(job.clone()).unwrap();
        let refund = manager.cancel_job(&job.id, &job.requester).unwrap();

        assert_eq!(refund, job.collateral);
        let fetched = manager.get_job(&job.id).unwrap();
        assert_eq!(fetched.status, JobStatus::Cancelled);
    }

    #[test]
    fn test_job_stats() {
        let manager = JobManager::new();

        manager.submit_job(create_test_job()).unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.total_jobs, 1);
        assert_eq!(stats.pending_jobs, 1);
    }
}
