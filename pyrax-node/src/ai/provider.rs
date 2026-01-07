//! AI Provider Protocol
//!
//! P2P protocol for AI providers to claim and execute jobs

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{info, warn, debug, error};
use serde::{Deserialize, Serialize};

use crate::types::H256;
use super::{AIJob, JobStatus, JobType, AIProvider, GpuSpec};
use super::registry::ModelRegistry;
use super::jobs::JobManager;

/// AI Provider Node - runs on GPU provider machines
pub struct ProviderNode {
    /// Provider identity
    provider: AIProvider,
    /// Model registry reference
    registry: Arc<ModelRegistry>,
    /// Job manager reference
    jobs: Arc<JobManager>,
    /// Currently executing jobs
    active_jobs: Arc<RwLock<HashMap<H256, ActiveJob>>>,
    /// Job result sender
    result_tx: mpsc::Sender<JobResult>,
    /// Provider status
    status: Arc<RwLock<ProviderStatus>>,
    /// Configuration
    config: ProviderConfig,
}

/// Active job being executed
#[derive(Debug, Clone)]
pub struct ActiveJob {
    pub job_id: H256,
    pub model_id: H256,
    pub started_at: Instant,
    pub status: ExecutionStatus,
    pub progress: f32,
}

/// Execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Loading,
    Running,
    Completing,
    Done,
    Failed,
}

/// Provider status
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub is_online: bool,
    pub gpu_utilization: f32,
    pub memory_used_mb: u32,
    pub memory_total_mb: u32,
    pub active_job_count: u32,
    pub jobs_completed_session: u64,
    pub last_heartbeat: u64,
}

impl Default for ProviderStatus {
    fn default() -> Self {
        Self {
            is_online: true,
            gpu_utilization: 0.0,
            memory_used_mb: 0,
            memory_total_mb: 0,
            active_job_count: 0,
            jobs_completed_session: 0,
            last_heartbeat: current_timestamp(),
        }
    }
}

/// Provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Maximum concurrent jobs
    pub max_concurrent_jobs: u32,
    /// Job poll interval
    pub poll_interval_secs: u64,
    /// Heartbeat interval
    pub heartbeat_interval_secs: u64,
    /// Model cache size (MB)
    pub model_cache_mb: u32,
    /// Auto-accept jobs for supported models
    pub auto_accept: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 4,
            poll_interval_secs: 5,
            heartbeat_interval_secs: 30,
            model_cache_mb: 10240, // 10GB
            auto_accept: true,
        }
    }
}

/// Job execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: H256,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub proof: Option<String>,
}

impl ProviderNode {
    /// Create a new provider node
    pub fn new(
        provider: AIProvider,
        registry: Arc<ModelRegistry>,
        jobs: Arc<JobManager>,
        config: ProviderConfig,
    ) -> (Self, mpsc::Receiver<JobResult>) {
        let (result_tx, result_rx) = mpsc::channel(100);

        let node = Self {
            provider,
            registry,
            jobs,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            result_tx,
            status: Arc::new(RwLock::new(ProviderStatus::default())),
            config,
        };

        (node, result_rx)
    }

    /// Start the provider node
    pub async fn start(&self) {
        info!(
            "Starting AI provider node: {} with {} GPU(s)",
            self.provider.name,
            self.provider.gpu_specs.iter().map(|g| g.count as u32).sum::<u32>()
        );

        // Update status
        {
            let mut status = self.status.write();
            status.is_online = true;
            status.memory_total_mb = self.provider.total_gpu_memory_mb;
        }

        // Start job polling loop
        self.poll_jobs().await;
    }

    /// Poll for available jobs
    async fn poll_jobs(&self) {
        loop {
            // Check if we can accept more jobs
            let active_count = self.active_jobs.read().len() as u32;
            if active_count >= self.config.max_concurrent_jobs {
                tokio::time::sleep(Duration::from_secs(self.config.poll_interval_secs)).await;
                continue;
            }

            // Look for pending jobs for our supported models
            for model_id in &self.provider.supported_models {
                if let Some(job) = self.jobs.get_next_pending_job(model_id) {
                    if self.config.auto_accept {
                        if let Err(e) = self.claim_job(&job.id).await {
                            warn!("Failed to claim job {}: {}", hex::encode(&job.id.0[..8]), e);
                        }
                    }
                }
            }

            // Update heartbeat
            {
                let mut status = self.status.write();
                status.last_heartbeat = current_timestamp();
            }

            tokio::time::sleep(Duration::from_secs(self.config.poll_interval_secs)).await;
        }
    }

    /// Claim a job for execution
    pub async fn claim_job(&self, job_id: &H256) -> Result<(), ProviderError> {
        // Assign job to us
        self.jobs.assign_job(job_id, self.provider.address)
            .map_err(|e| ProviderError::JobClaimFailed(e.to_string()))?;

        // Get job details
        let job = self.jobs.get_job(job_id)
            .ok_or(ProviderError::JobNotFound(*job_id))?;

        // Add to active jobs
        {
            let mut active = self.active_jobs.write();
            active.insert(*job_id, ActiveJob {
                job_id: *job_id,
                model_id: job.model_id,
                started_at: Instant::now(),
                status: ExecutionStatus::Loading,
                progress: 0.0,
            });
        }

        // Update status
        {
            let mut status = self.status.write();
            status.active_job_count += 1;
        }

        info!("Claimed job: {}", hex::encode(&job_id.0[..8]));

        // Start execution in background
        let job_id = *job_id;
        let jobs = self.jobs.clone();
        let active_jobs = self.active_jobs.clone();
        let result_tx = self.result_tx.clone();
        let provider_addr = self.provider.address;
        let status = self.status.clone();

        tokio::spawn(async move {
            let result = execute_job(&jobs, &job_id, &active_jobs).await;
            
            // Complete the job
            match &result {
                Ok(output) => {
                    let _ = jobs.complete_job(
                        &job_id,
                        &provider_addr,
                        output.clone(),
                        None, // TODO: Add ZK proof
                        0, // Price determined by job
                    );
                }
                Err(e) => {
                    let _ = jobs.fail_job(&job_id, e.to_string());
                }
            }

            // Remove from active jobs
            {
                let mut active = active_jobs.write();
                active.remove(&job_id);
            }

            // Update status
            {
                let mut s = status.write();
                s.active_job_count = s.active_job_count.saturating_sub(1);
                s.jobs_completed_session += 1;
            }

            // Send result
            let (success, output, error_msg) = match result {
                Ok(out) => (true, Some(out), None),
                Err(e) => (false, None, Some(e.to_string())),
            };
            let job_result = JobResult {
                job_id,
                success,
                output,
                error: error_msg,
                execution_time_ms: 0, // TODO: Track actual time
                proof: None,
            };
            let _ = result_tx.send(job_result).await;
        });

        Ok(())
    }

    /// Get provider status
    pub fn get_status(&self) -> ProviderStatus {
        self.status.read().clone()
    }

    /// Get active jobs
    pub fn get_active_jobs(&self) -> Vec<ActiveJob> {
        self.active_jobs.read().values().cloned().collect()
    }

    /// Stop the provider
    pub fn stop(&self) {
        let mut status = self.status.write();
        status.is_online = false;
        info!("Provider node stopped");
    }
}

/// Execute a job
async fn execute_job(
    jobs: &JobManager,
    job_id: &H256,
    active_jobs: &RwLock<HashMap<H256, ActiveJob>>,
) -> Result<String, ProviderError> {
    let job = jobs.get_job(job_id)
        .ok_or(ProviderError::JobNotFound(*job_id))?;

    // Mark as running
    jobs.start_job(job_id, &job.provider.unwrap())
        .map_err(|e| ProviderError::ExecutionFailed(e.to_string()))?;

    // Update active job status
    {
        let mut active = active_jobs.write();
        if let Some(aj) = active.get_mut(job_id) {
            aj.status = ExecutionStatus::Running;
        }
    }

    // Execute based on job type
    let output = match job.job_type {
        JobType::Inference => {
            execute_inference(&job).await?
        }
        JobType::BatchInference => {
            execute_batch_inference(&job).await?
        }
        JobType::Training => {
            execute_training(&job).await?
        }
        JobType::FineTuning => {
            execute_fine_tuning(&job).await?
        }
    };

    // Mark as completing
    {
        let mut active = active_jobs.write();
        if let Some(aj) = active.get_mut(job_id) {
            aj.status = ExecutionStatus::Completing;
            aj.progress = 1.0;
        }
    }

    Ok(output)
}

/// Execute inference job
async fn execute_inference(job: &AIJob) -> Result<String, ProviderError> {
    // Parse input
    let input: serde_json::Value = serde_json::from_str(&job.input)
        .map_err(|e| ProviderError::InvalidInput(e.to_string()))?;

    // In production, this would:
    // 1. Load model from IPFS if not cached
    // 2. Run inference on GPU
    // 3. Return output

    // For now, we simulate inference processing
    debug!("Executing inference for model: {}", hex::encode(&job.model_id.0[..8]));

    // Simulate processing time based on input size
    let delay_ms = 100 + (job.input.len() / 10) as u64;
    tokio::time::sleep(Duration::from_millis(delay_ms.min(5000))).await;

    // Return structured output
    let output = serde_json::json!({
        "model_id": hex::encode(&job.model_id.0),
        "status": "completed",
        "result": {
            "type": "inference",
            "tokens_processed": job.input.len() / 4,
            "latency_ms": delay_ms,
        }
    });

    Ok(output.to_string())
}

/// Execute batch inference
async fn execute_batch_inference(job: &AIJob) -> Result<String, ProviderError> {
    let input: serde_json::Value = serde_json::from_str(&job.input)
        .map_err(|e| ProviderError::InvalidInput(e.to_string()))?;

    let batch_size = input.as_array().map(|a| a.len()).unwrap_or(1);
    debug!("Executing batch inference: {} items", batch_size);

    // Simulate batch processing
    let delay_ms = (batch_size * 50) as u64;
    tokio::time::sleep(Duration::from_millis(delay_ms.min(30000))).await;

    let output = serde_json::json!({
        "model_id": hex::encode(&job.model_id.0),
        "status": "completed",
        "result": {
            "type": "batch_inference",
            "batch_size": batch_size,
            "latency_ms": delay_ms,
        }
    });

    Ok(output.to_string())
}

/// Execute training job
async fn execute_training(job: &AIJob) -> Result<String, ProviderError> {
    let input: serde_json::Value = serde_json::from_str(&job.input)
        .map_err(|e| ProviderError::InvalidInput(e.to_string()))?;

    let epochs = input.get("epochs").and_then(|v| v.as_u64()).unwrap_or(1);
    debug!("Executing training: {} epochs", epochs);

    // Training is long-running - simulate with shorter delay for demo
    let delay_ms = (epochs * 1000) as u64;
    tokio::time::sleep(Duration::from_millis(delay_ms.min(60000))).await;

    let output = serde_json::json!({
        "model_id": hex::encode(&job.model_id.0),
        "status": "completed",
        "result": {
            "type": "training",
            "epochs_completed": epochs,
            "final_loss": 0.05,
            "checkpoint_cid": "QmTrainingCheckpoint123",
        }
    });

    Ok(output.to_string())
}

/// Execute fine-tuning job
async fn execute_fine_tuning(job: &AIJob) -> Result<String, ProviderError> {
    let input: serde_json::Value = serde_json::from_str(&job.input)
        .map_err(|e| ProviderError::InvalidInput(e.to_string()))?;

    let steps = input.get("steps").and_then(|v| v.as_u64()).unwrap_or(100);
    debug!("Executing fine-tuning: {} steps", steps);

    let delay_ms = (steps * 100) as u64;
    tokio::time::sleep(Duration::from_millis(delay_ms.min(30000))).await;

    let output = serde_json::json!({
        "model_id": hex::encode(&job.model_id.0),
        "status": "completed",
        "result": {
            "type": "fine_tuning",
            "steps_completed": steps,
            "final_loss": 0.02,
            "output_model_cid": "QmFineTunedModel456",
        }
    });

    Ok(output.to_string())
}

/// Provider errors
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Job not found: {0:?}")]
    JobNotFound(H256),

    #[error("Job claim failed: {0}")]
    JobClaimFailed(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("GPU error: {0}")]
    GpuError(String),
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
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.max_concurrent_jobs, 4);
        assert!(config.auto_accept);
    }

    #[test]
    fn test_provider_status_default() {
        let status = ProviderStatus::default();
        assert!(status.is_online);
        assert_eq!(status.active_job_count, 0);
    }

    #[test]
    fn test_job_result_serialization() {
        let result = JobResult {
            job_id: H256([1u8; 32]),
            success: true,
            output: Some("test output".to_string()),
            error: None,
            execution_time_ms: 100,
            proof: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test output"));
    }
}
