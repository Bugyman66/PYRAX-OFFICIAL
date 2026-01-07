use std::collections::HashMap;
use crate::{Job, JobStatus, Result, AiError};

pub struct Marketplace {
    jobs: HashMap<String, Job>,
    pending_jobs: Vec<String>,
}

impl Marketplace {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            pending_jobs: Vec::new(),
        }
    }

    pub fn submit_job(&mut self, job: Job) -> Result<String> {
        let job_id = job.id.clone();
        self.pending_jobs.push(job_id.clone());
        self.jobs.insert(job_id.clone(), job);
        Ok(job_id)
    }

    pub fn get_job(&self, job_id: &str) -> Option<&Job> {
        self.jobs.get(job_id)
    }

    pub fn get_pending_jobs(&self) -> Vec<&Job> {
        self.pending_jobs.iter()
            .filter_map(|id| self.jobs.get(id))
            .filter(|j| j.status == JobStatus::Pending)
            .collect()
    }

    pub fn claim_job(&mut self, job_id: &str, worker: String) -> Result<()> {
        let job = self.jobs.get_mut(job_id)
            .ok_or_else(|| AiError::JobNotFound(job_id.to_string()))?;

        if !job.can_claim() {
            return Err(AiError::InvalidJobState {
                expected: "Pending".to_string(),
                actual: format!("{:?}", job.status),
            });
        }

        job.status = JobStatus::Matched;
        job.worker = Some(worker);
        job.started_at = Some(current_timestamp());

        // Remove from pending list
        self.pending_jobs.retain(|id| id != job_id);

        Ok(())
    }

    pub fn start_job(&mut self, job_id: &str) -> Result<()> {
        let job = self.jobs.get_mut(job_id)
            .ok_or_else(|| AiError::JobNotFound(job_id.to_string()))?;

        if job.status != JobStatus::Matched {
            return Err(AiError::InvalidJobState {
                expected: "Matched".to_string(),
                actual: format!("{:?}", job.status),
            });
        }

        job.status = JobStatus::Running;
        Ok(())
    }

    pub fn complete_job(&mut self, job_id: &str, output_hash: String) -> Result<()> {
        let job = self.jobs.get_mut(job_id)
            .ok_or_else(|| AiError::JobNotFound(job_id.to_string()))?;

        if !job.can_complete() {
            return Err(AiError::InvalidJobState {
                expected: "Running".to_string(),
                actual: format!("{:?}", job.status),
            });
        }

        job.status = JobStatus::Verifying;
        job.output_hash = Some(output_hash);
        job.completed_at = Some(current_timestamp());

        Ok(())
    }

    pub fn verify_job(&mut self, job_id: &str, success: bool) -> Result<()> {
        let job = self.jobs.get_mut(job_id)
            .ok_or_else(|| AiError::JobNotFound(job_id.to_string()))?;

        if job.status != JobStatus::Verifying {
            return Err(AiError::InvalidJobState {
                expected: "Verifying".to_string(),
                actual: format!("{:?}", job.status),
            });
        }

        job.status = if success {
            JobStatus::Completed
        } else {
            JobStatus::Disputed
        };

        Ok(())
    }

    pub fn cancel_job(&mut self, job_id: &str, requester: &str) -> Result<()> {
        let job = self.jobs.get_mut(job_id)
            .ok_or_else(|| AiError::JobNotFound(job_id.to_string()))?;

        if job.submitter != requester {
            return Err(AiError::ExecutionError("Only submitter can cancel".to_string()));
        }

        if job.status != JobStatus::Pending {
            return Err(AiError::InvalidJobState {
                expected: "Pending".to_string(),
                actual: format!("{:?}", job.status),
            });
        }

        job.status = JobStatus::Cancelled;
        self.pending_jobs.retain(|id| id != job_id);

        Ok(())
    }

    pub fn check_timeouts(&mut self) -> Vec<String> {
        let mut timed_out = Vec::new();

        for job in self.jobs.values_mut() {
            if job.status == JobStatus::Running && job.is_timed_out() {
                job.status = JobStatus::Failed;
                timed_out.push(job.id.clone());
            }
        }

        timed_out
    }
}

impl Default for Marketplace {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
