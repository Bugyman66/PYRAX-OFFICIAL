#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use crate::types::H256;

pub mod registry;
pub mod jobs;
pub mod storage;
pub mod provider;
pub mod compute;

/// AI Model metadata stored on-chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Unique model identifier (content hash)
    pub id: H256,
    /// Human-readable name
    pub name: String,
    /// Model version (semver)
    pub version: String,
    /// Owner address
    pub owner: [u8; 20],
    /// Model framework (pytorch, tensorflow, onnx, etc.)
    pub framework: ModelFramework,
    /// Model type/architecture
    pub model_type: ModelType,
    /// Size in bytes
    pub size_bytes: u64,
    /// IPFS CID for model weights
    pub ipfs_cid: String,
    /// Input schema (JSON)
    pub input_schema: String,
    /// Output schema (JSON)
    pub output_schema: String,
    /// Minimum GPU memory required (MB)
    pub min_gpu_memory_mb: u32,
    /// Price per 1000 tokens/inferences (in smallest PYRAX unit)
    pub price_per_1k: u64,
    /// Registration timestamp
    pub registered_at: u64,
    /// Registration block height
    pub registered_height: u64,
    /// Is model active and available
    pub is_active: bool,
    /// Total inference count
    pub total_inferences: u64,
    /// Average latency in ms
    pub avg_latency_ms: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelFramework {
    PyTorch,
    TensorFlow,
    ONNX,
    JAX,
    HuggingFace,
    Custom,
}

impl ModelFramework {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelFramework::PyTorch => "pytorch",
            ModelFramework::TensorFlow => "tensorflow",
            ModelFramework::ONNX => "onnx",
            ModelFramework::JAX => "jax",
            ModelFramework::HuggingFace => "huggingface",
            ModelFramework::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pytorch" | "torch" => Some(ModelFramework::PyTorch),
            "tensorflow" | "tf" => Some(ModelFramework::TensorFlow),
            "onnx" => Some(ModelFramework::ONNX),
            "jax" => Some(ModelFramework::JAX),
            "huggingface" | "hf" => Some(ModelFramework::HuggingFace),
            "custom" => Some(ModelFramework::Custom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelType {
    /// Large Language Model
    LLM,
    /// Image generation
    ImageGeneration,
    /// Image classification
    ImageClassification,
    /// Object detection
    ObjectDetection,
    /// Speech to text
    SpeechToText,
    /// Text to speech
    TextToSpeech,
    /// Embedding model
    Embedding,
    /// General purpose
    General,
}

impl ModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelType::LLM => "llm",
            ModelType::ImageGeneration => "image_generation",
            ModelType::ImageClassification => "image_classification",
            ModelType::ObjectDetection => "object_detection",
            ModelType::SpeechToText => "speech_to_text",
            ModelType::TextToSpeech => "text_to_speech",
            ModelType::Embedding => "embedding",
            ModelType::General => "general",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "llm" | "language" => Some(ModelType::LLM),
            "image_generation" | "imagegen" => Some(ModelType::ImageGeneration),
            "image_classification" | "imageclassify" => Some(ModelType::ImageClassification),
            "object_detection" | "detection" => Some(ModelType::ObjectDetection),
            "speech_to_text" | "stt" | "asr" => Some(ModelType::SpeechToText),
            "text_to_speech" | "tts" => Some(ModelType::TextToSpeech),
            "embedding" | "embed" => Some(ModelType::Embedding),
            "general" | "other" => Some(ModelType::General),
            _ => None,
        }
    }
}

/// AI Job status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    /// Job submitted, waiting for provider
    Pending,
    /// Job assigned to provider
    Assigned,
    /// Job currently executing
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job cancelled by requester
    Cancelled,
    /// Job expired (timeout)
    Expired,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Assigned => "assigned",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
            JobStatus::Expired => "expired",
        }
    }
}

/// AI Job type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobType {
    /// Single inference request
    Inference,
    /// Batch inference
    BatchInference,
    /// Model training
    Training,
    /// Model fine-tuning
    FineTuning,
}

/// AI Job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIJob {
    /// Unique job identifier
    pub id: H256,
    /// Job type
    pub job_type: JobType,
    /// Model ID to use
    pub model_id: H256,
    /// Requester address
    pub requester: [u8; 20],
    /// Assigned provider address (if assigned)
    pub provider: Option<[u8; 20]>,
    /// Job status
    pub status: JobStatus,
    /// Input data (JSON or IPFS CID for large data)
    pub input: String,
    /// Output data (set when completed)
    pub output: Option<String>,
    /// Maximum price willing to pay (smallest PYRAX unit)
    pub max_price: u64,
    /// Actual price paid
    pub actual_price: Option<u64>,
    /// Collateral deposited
    pub collateral: u64,
    /// Submission timestamp
    pub submitted_at: u64,
    /// Submission block height
    pub submitted_height: u64,
    /// Assigned timestamp
    pub assigned_at: Option<u64>,
    /// Completion timestamp
    pub completed_at: Option<u64>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Proof of computation (ZK proof or signature)
    pub proof: Option<String>,
    /// Time limit in seconds
    pub timeout_secs: u32,
}

/// AI Provider registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProvider {
    /// Provider address
    pub address: [u8; 20],
    /// Provider name
    pub name: String,
    /// Staked amount (smallest PYRAX unit)
    pub stake: u64,
    /// GPU specifications
    pub gpu_specs: Vec<GpuSpec>,
    /// Total GPU memory available (MB)
    pub total_gpu_memory_mb: u32,
    /// Supported model IDs
    pub supported_models: Vec<H256>,
    /// Is provider currently active
    pub is_active: bool,
    /// Registration timestamp
    pub registered_at: u64,
    /// Total jobs completed
    pub jobs_completed: u64,
    /// Total jobs failed
    pub jobs_failed: u64,
    /// Average response time in ms
    pub avg_response_ms: u32,
    /// Reputation score (0-1000)
    pub reputation: u32,
    /// P2P endpoint for direct communication
    pub endpoint: String,
}

/// GPU specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSpec {
    /// GPU model name (e.g., "RTX 4090", "A100")
    pub model: String,
    /// VRAM in MB
    pub vram_mb: u32,
    /// Count of this GPU type
    pub count: u8,
}

/// Training job configuration (for Crucible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Base model ID (for fine-tuning) or architecture
    pub base_model: Option<H256>,
    /// Dataset IPFS CID
    pub dataset_cid: String,
    /// Training parameters (JSON)
    pub hyperparameters: String,
    /// Maximum epochs
    pub max_epochs: u32,
    /// Batch size
    pub batch_size: u32,
    /// Learning rate
    pub learning_rate: f64,
    /// Checkpoint interval (epochs)
    pub checkpoint_interval: u32,
    /// Validation split ratio
    pub validation_split: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_framework_conversion() {
        assert_eq!(ModelFramework::from_str("pytorch"), Some(ModelFramework::PyTorch));
        assert_eq!(ModelFramework::from_str("TENSORFLOW"), Some(ModelFramework::TensorFlow));
        assert_eq!(ModelFramework::PyTorch.as_str(), "pytorch");
    }

    #[test]
    fn test_model_type_conversion() {
        assert_eq!(ModelType::from_str("llm"), Some(ModelType::LLM));
        assert_eq!(ModelType::from_str("embedding"), Some(ModelType::Embedding));
        assert_eq!(ModelType::LLM.as_str(), "llm");
    }

    #[test]
    fn test_job_status() {
        assert_eq!(JobStatus::Pending.as_str(), "pending");
        assert_eq!(JobStatus::Completed.as_str(), "completed");
    }
}
