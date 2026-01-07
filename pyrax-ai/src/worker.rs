use serde::{Deserialize, Serialize};
use crate::{Job, Result, AiError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub id: String,
    pub address: String,
    pub stake: String,
    pub reputation: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub capabilities: WorkerCapabilities,
    pub status: WorkerStatus,
    pub registered_at: u64,
    pub last_heartbeat: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub cuda_version: Option<String>,
    pub cuda_capability: Option<String>,
    pub vram_gb: Option<u32>,
    pub cpu_cores: u32,
    pub ram_gb: u32,
    pub supported_models: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Idle,
    Busy,
    Offline,
    Suspended,
}

impl Worker {
    pub fn new(address: String, stake: String, capabilities: WorkerCapabilities) -> Self {
        let now = current_timestamp();
        Self {
            id: generate_worker_id(&address),
            address,
            stake,
            reputation: 100,
            completed_jobs: 0,
            failed_jobs: 0,
            capabilities,
            status: WorkerStatus::Idle,
            registered_at: now,
            last_heartbeat: now,
        }
    }

    pub fn can_execute(&self, job: &Job) -> bool {
        if self.status != WorkerStatus::Idle {
            return false;
        }

        let reqs = &job.hardware_requirements;

        if let Some(min_vram) = reqs.min_vram_gb {
            if self.capabilities.vram_gb.unwrap_or(0) < min_vram {
                return false;
            }
        }

        if let Some(min_cores) = reqs.min_cpu_cores {
            if self.capabilities.cpu_cores < min_cores {
                return false;
            }
        }

        if let Some(min_ram) = reqs.min_ram_gb {
            if self.capabilities.ram_gb < min_ram {
                return false;
            }
        }

        true
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = current_timestamp();
    }

    pub fn is_stale(&self, timeout_seconds: u64) -> bool {
        let now = current_timestamp();
        now - self.last_heartbeat > timeout_seconds
    }

    pub fn record_completion(&mut self, success: bool) {
        if success {
            self.completed_jobs += 1;
            self.reputation = (self.reputation + 1).min(200);
        } else {
            self.failed_jobs += 1;
            self.reputation = self.reputation.saturating_sub(5);
        }
    }

    pub fn slash(&mut self, amount: &str) -> Result<()> {
        // TODO: Implement actual slashing logic
        self.reputation = self.reputation.saturating_sub(20);
        self.status = WorkerStatus::Suspended;
        Ok(())
    }
}

pub struct WorkerPool {
    workers: std::collections::HashMap<String, Worker>,
}

impl WorkerPool {
    pub fn new() -> Self {
        Self {
            workers: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, worker: Worker) -> Result<()> {
        self.workers.insert(worker.id.clone(), worker);
        Ok(())
    }

    pub fn unregister(&mut self, worker_id: &str) -> Result<()> {
        self.workers.remove(worker_id)
            .ok_or(AiError::WorkerNotRegistered)?;
        Ok(())
    }

    pub fn get(&self, worker_id: &str) -> Option<&Worker> {
        self.workers.get(worker_id)
    }

    pub fn get_mut(&mut self, worker_id: &str) -> Option<&mut Worker> {
        self.workers.get_mut(worker_id)
    }

    pub fn find_suitable(&self, job: &Job) -> Option<&Worker> {
        self.workers.values()
            .filter(|w| w.can_execute(job))
            .max_by_key(|w| w.reputation)
    }

    pub fn cleanup_stale(&mut self, timeout_seconds: u64) {
        for worker in self.workers.values_mut() {
            if worker.is_stale(timeout_seconds) {
                worker.status = WorkerStatus::Offline;
            }
        }
    }

    pub fn active_count(&self) -> usize {
        self.workers.values()
            .filter(|w| w.status == WorkerStatus::Idle || w.status == WorkerStatus::Busy)
            .count()
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_worker_id(address: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(address.as_bytes());
    hasher.update(current_timestamp().to_le_bytes());
    hex::encode(&hasher.finalize()[..16])
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
