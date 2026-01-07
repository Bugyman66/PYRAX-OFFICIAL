use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobType {
    Inference,
    Training,
    FineTuning,
    Embedding,
    ImageGeneration,
    TextToSpeech,
    SpeechToText,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Matched,
    Running,
    Verifying,
    Completed,
    Failed,
    Disputed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub submitter: String,
    pub worker: Option<String>,
    pub model_id: Option<String>,
    pub input_hash: String,
    pub output_hash: Option<String>,
    pub max_cost: String,
    pub escrow_tx: Option<String>,
    pub verification_type: VerificationType,
    pub hardware_requirements: HardwareRequirements,
    pub timeout_seconds: u64,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationType {
    HashCheck,
    Redundant { n: u32, m: u32 },
    ZkProof,
    TeeAttestation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRequirements {
    pub min_vram_gb: Option<u32>,
    pub cuda_capability: Option<String>,
    pub min_cpu_cores: Option<u32>,
    pub min_ram_gb: Option<u32>,
}

impl Default for HardwareRequirements {
    fn default() -> Self {
        Self {
            min_vram_gb: None,
            cuda_capability: None,
            min_cpu_cores: None,
            min_ram_gb: None,
        }
    }
}

impl Job {
    pub fn new(
        job_type: JobType,
        submitter: String,
        input_hash: String,
        max_cost: String,
    ) -> Self {
        Self {
            id: generate_job_id(),
            job_type,
            status: JobStatus::Pending,
            submitter,
            worker: None,
            model_id: None,
            input_hash,
            output_hash: None,
            max_cost,
            escrow_tx: None,
            verification_type: VerificationType::HashCheck,
            hardware_requirements: HardwareRequirements::default(),
            timeout_seconds: 3600,
            created_at: current_timestamp(),
            started_at: None,
            completed_at: None,
        }
    }

    pub fn can_claim(&self) -> bool {
        self.status == JobStatus::Pending
    }

    pub fn can_complete(&self) -> bool {
        self.status == JobStatus::Running
    }

    pub fn is_timed_out(&self) -> bool {
        if let Some(started_at) = self.started_at {
            let now = current_timestamp();
            now - started_at > self.timeout_seconds
        } else {
            false
        }
    }
}

fn generate_job_id() -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(current_timestamp().to_le_bytes());
    hasher.update(rand::random::<[u8; 32]>());
    hex::encode(&hasher.finalize()[..16])
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
