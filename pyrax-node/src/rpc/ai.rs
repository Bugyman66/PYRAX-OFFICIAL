//! AI RPC Endpoints for PYRAX
//!
//! JSON-RPC methods for Foundry (model registry) and Crucible (AI jobs)

use std::sync::Arc;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::RpcError;
use crate::types::H256;
use crate::ai::{
    Model, ModelFramework, ModelType, AIJob, JobStatus, JobType,
    AIProvider, GpuSpec, TrainingConfig,
    registry::{ModelRegistry, RegistryStats},
    jobs::{JobManager, JobStats},
};

/// AI RPC API
#[rpc(server)]
pub trait AIRpc {
    // ===== Model Registry (Foundry) =====
    
    /// Register a new AI model
    #[method(name = "ai_registerModel")]
    async fn register_model(&self, params: RegisterModelParams) -> RpcResult<RpcModelId>;

    /// Get model by ID
    #[method(name = "ai_getModel")]
    async fn get_model(&self, model_id: String) -> RpcResult<Option<RpcModel>>;

    /// List all active models
    #[method(name = "ai_listModels")]
    async fn list_models(&self, limit: Option<usize>, offset: Option<usize>) -> RpcResult<Vec<RpcModel>>;

    /// Search models by type and framework
    #[method(name = "ai_searchModels")]
    async fn search_models(
        &self,
        model_type: Option<String>,
        framework: Option<String>,
        limit: Option<usize>,
    ) -> RpcResult<Vec<RpcModel>>;

    /// Get model registry statistics
    #[method(name = "ai_getRegistryStats")]
    async fn get_registry_stats(&self) -> RpcResult<RpcRegistryStats>;

    // ===== Provider Management =====

    /// Register as an AI provider
    #[method(name = "ai_registerProvider")]
    async fn register_provider(&self, params: RegisterProviderParams) -> RpcResult<RpcSuccess>;

    /// Get provider by address
    #[method(name = "ai_getProvider")]
    async fn get_provider(&self, address: String) -> RpcResult<Option<RpcProvider>>;

    /// List active providers
    #[method(name = "ai_listProviders")]
    async fn list_providers(&self, limit: Option<usize>, offset: Option<usize>) -> RpcResult<Vec<RpcProvider>>;

    /// Find providers for a specific model
    #[method(name = "ai_findProviders")]
    async fn find_providers(&self, model_id: String) -> RpcResult<Vec<RpcProvider>>;

    // ===== Job Management (Crucible) =====

    /// Submit an AI job
    #[method(name = "ai_submitJob")]
    async fn submit_job(&self, params: SubmitJobParams) -> RpcResult<RpcJobId>;

    /// Get job by ID
    #[method(name = "ai_getJob")]
    async fn get_job(&self, job_id: String) -> RpcResult<Option<RpcJob>>;

    /// Get jobs by requester
    #[method(name = "ai_getJobsByRequester")]
    async fn get_jobs_by_requester(&self, address: String) -> RpcResult<Vec<RpcJob>>;

    /// List pending jobs
    #[method(name = "ai_listPendingJobs")]
    async fn list_pending_jobs(&self, limit: Option<usize>) -> RpcResult<Vec<RpcJob>>;

    /// Cancel a job
    #[method(name = "ai_cancelJob")]
    async fn cancel_job(&self, job_id: String, requester: String) -> RpcResult<RpcCancelResult>;

    /// Get job statistics
    #[method(name = "ai_getJobStats")]
    async fn get_job_stats(&self) -> RpcResult<RpcJobStats>;
}

/// AI RPC Server Implementation
pub struct AIRpcImpl {
    registry: Arc<ModelRegistry>,
    jobs: Arc<JobManager>,
}

impl AIRpcImpl {
    pub fn new(registry: Arc<ModelRegistry>, jobs: Arc<JobManager>) -> Self {
        Self { registry, jobs }
    }
}

#[async_trait]
impl AIRpcServer for AIRpcImpl {
    async fn register_model(&self, params: RegisterModelParams) -> RpcResult<RpcModelId> {
        let owner = parse_address(&params.owner)?;
        
        let model_id = compute_model_id(&params.name, &params.version, &owner);
        
        let model = Model {
            id: model_id,
            name: params.name,
            version: params.version,
            owner,
            framework: ModelFramework::from_str(&params.framework)
                .ok_or_else(|| RpcError::InvalidParams("invalid framework".into()))?,
            model_type: ModelType::from_str(&params.model_type)
                .ok_or_else(|| RpcError::InvalidParams("invalid model_type".into()))?,
            size_bytes: params.size_bytes,
            ipfs_cid: params.ipfs_cid,
            input_schema: params.input_schema,
            output_schema: params.output_schema,
            min_gpu_memory_mb: params.min_gpu_memory_mb,
            price_per_1k: params.price_per_1k,
            registered_at: current_timestamp(),
            registered_height: 0, // TODO: get from chain
            is_active: true,
            total_inferences: 0,
            avg_latency_ms: 0,
        };

        self.registry.register_model(model)
            .map_err(|e| RpcError::InternalError(e.to_string()))?;

        Ok(RpcModelId {
            model_id: format!("0x{}", hex::encode(&model_id.0)),
        })
    }

    async fn get_model(&self, model_id: String) -> RpcResult<Option<RpcModel>> {
        let id = parse_hash(&model_id)?;
        Ok(self.registry.get_model(&id).map(RpcModel::from))
    }

    async fn list_models(&self, limit: Option<usize>, offset: Option<usize>) -> RpcResult<Vec<RpcModel>> {
        let models = self.registry.list_models(limit.unwrap_or(50), offset.unwrap_or(0));
        Ok(models.into_iter().map(RpcModel::from).collect())
    }

    async fn search_models(
        &self,
        model_type: Option<String>,
        framework: Option<String>,
        limit: Option<usize>,
    ) -> RpcResult<Vec<RpcModel>> {
        let mt = model_type.and_then(|s| ModelType::from_str(&s));
        let fw = framework.and_then(|s| ModelFramework::from_str(&s));
        
        let models = self.registry.search_models(mt, fw, limit.unwrap_or(50));
        Ok(models.into_iter().map(RpcModel::from).collect())
    }

    async fn get_registry_stats(&self) -> RpcResult<RpcRegistryStats> {
        let stats = self.registry.get_stats();
        Ok(RpcRegistryStats::from(stats))
    }

    async fn register_provider(&self, params: RegisterProviderParams) -> RpcResult<RpcSuccess> {
        let address = parse_address(&params.address)?;
        
        let provider = AIProvider {
            address,
            name: params.name,
            stake: params.stake,
            gpu_specs: params.gpu_specs.into_iter().map(|g| GpuSpec {
                model: g.model,
                vram_mb: g.vram_mb,
                count: g.count,
            }).collect(),
            total_gpu_memory_mb: params.total_gpu_memory_mb,
            supported_models: params.supported_models.iter()
                .filter_map(|s| parse_hash(s).ok())
                .collect(),
            is_active: true,
            registered_at: current_timestamp(),
            jobs_completed: 0,
            jobs_failed: 0,
            avg_response_ms: 0,
            reputation: 500, // Starting reputation
            endpoint: params.endpoint,
        };

        self.registry.register_provider(provider)
            .map_err(|e| RpcError::InternalError(e.to_string()))?;

        Ok(RpcSuccess { success: true })
    }

    async fn get_provider(&self, address: String) -> RpcResult<Option<RpcProvider>> {
        let addr = parse_address(&address)?;
        Ok(self.registry.get_provider(&addr).map(RpcProvider::from))
    }

    async fn list_providers(&self, limit: Option<usize>, offset: Option<usize>) -> RpcResult<Vec<RpcProvider>> {
        let providers = self.registry.list_providers(limit.unwrap_or(50), offset.unwrap_or(0));
        Ok(providers.into_iter().map(RpcProvider::from).collect())
    }

    async fn find_providers(&self, model_id: String) -> RpcResult<Vec<RpcProvider>> {
        let id = parse_hash(&model_id)?;
        let providers = self.registry.find_providers_for_model(&id);
        Ok(providers.into_iter().map(RpcProvider::from).collect())
    }

    async fn submit_job(&self, params: SubmitJobParams) -> RpcResult<RpcJobId> {
        let requester = parse_address(&params.requester)?;
        let model_id = parse_hash(&params.model_id)?;
        
        let job_id = compute_job_id(&requester, &model_id, current_timestamp());
        
        let job = AIJob {
            id: job_id,
            job_type: match params.job_type.as_str() {
                "inference" => JobType::Inference,
                "batch_inference" => JobType::BatchInference,
                "training" => JobType::Training,
                "fine_tuning" => JobType::FineTuning,
                _ => return Err(RpcError::InvalidParams("invalid job_type".into()).into()),
            },
            model_id,
            requester,
            provider: None,
            status: JobStatus::Pending,
            input: params.input,
            output: None,
            max_price: params.max_price,
            actual_price: None,
            collateral: params.collateral,
            submitted_at: current_timestamp(),
            submitted_height: 0, // TODO: get from chain
            assigned_at: None,
            completed_at: None,
            error: None,
            proof: None,
            timeout_secs: params.timeout_secs.unwrap_or(300),
        };

        self.jobs.submit_job(job)
            .map_err(|e| RpcError::InternalError(e.to_string()))?;

        Ok(RpcJobId {
            job_id: format!("0x{}", hex::encode(&job_id.0)),
        })
    }

    async fn get_job(&self, job_id: String) -> RpcResult<Option<RpcJob>> {
        let id = parse_hash(&job_id)?;
        Ok(self.jobs.get_job(&id).map(RpcJob::from))
    }

    async fn get_jobs_by_requester(&self, address: String) -> RpcResult<Vec<RpcJob>> {
        let addr = parse_address(&address)?;
        let jobs = self.jobs.get_jobs_by_requester(&addr);
        Ok(jobs.into_iter().map(RpcJob::from).collect())
    }

    async fn list_pending_jobs(&self, limit: Option<usize>) -> RpcResult<Vec<RpcJob>> {
        let jobs = self.jobs.list_pending_jobs(limit.unwrap_or(50));
        Ok(jobs.into_iter().map(RpcJob::from).collect())
    }

    async fn cancel_job(&self, job_id: String, requester: String) -> RpcResult<RpcCancelResult> {
        let id = parse_hash(&job_id)?;
        let addr = parse_address(&requester)?;
        
        let refund = self.jobs.cancel_job(&id, &addr)
            .map_err(|e| RpcError::InternalError(e.to_string()))?;
        
        Ok(RpcCancelResult {
            success: true,
            refund_amount: refund,
        })
    }

    async fn get_job_stats(&self) -> RpcResult<RpcJobStats> {
        let stats = self.jobs.get_stats();
        Ok(RpcJobStats::from(stats))
    }
}

// ===== RPC Request/Response Types =====

#[derive(Debug, Deserialize)]
pub struct RegisterModelParams {
    pub name: String,
    pub version: String,
    pub owner: String,
    pub framework: String,
    pub model_type: String,
    pub size_bytes: u64,
    pub ipfs_cid: String,
    pub input_schema: String,
    pub output_schema: String,
    pub min_gpu_memory_mb: u32,
    pub price_per_1k: u64,
}

#[derive(Debug, Deserialize)]
pub struct RegisterProviderParams {
    pub address: String,
    pub name: String,
    pub stake: u64,
    pub gpu_specs: Vec<RpcGpuSpec>,
    pub total_gpu_memory_mb: u32,
    pub supported_models: Vec<String>,
    pub endpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct RpcGpuSpec {
    pub model: String,
    pub vram_mb: u32,
    pub count: u8,
}

#[derive(Debug, Deserialize)]
pub struct SubmitJobParams {
    pub requester: String,
    pub model_id: String,
    pub job_type: String,
    pub input: String,
    pub max_price: u64,
    pub collateral: u64,
    pub timeout_secs: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcModelId {
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcJobId {
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcSuccess {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcCancelResult {
    pub success: bool,
    pub refund_amount: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcModel {
    pub id: String,
    pub name: String,
    pub version: String,
    pub owner: String,
    pub framework: String,
    pub model_type: String,
    pub size_bytes: u64,
    pub ipfs_cid: String,
    pub input_schema: String,
    pub output_schema: String,
    pub min_gpu_memory_mb: u32,
    pub price_per_1k: u64,
    pub registered_at: u64,
    pub is_active: bool,
    pub total_inferences: u64,
    pub avg_latency_ms: u32,
}

impl From<Model> for RpcModel {
    fn from(m: Model) -> Self {
        Self {
            id: format!("0x{}", hex::encode(&m.id.0)),
            name: m.name,
            version: m.version,
            owner: format!("0x{}", hex::encode(&m.owner)),
            framework: m.framework.as_str().to_string(),
            model_type: m.model_type.as_str().to_string(),
            size_bytes: m.size_bytes,
            ipfs_cid: m.ipfs_cid,
            input_schema: m.input_schema,
            output_schema: m.output_schema,
            min_gpu_memory_mb: m.min_gpu_memory_mb,
            price_per_1k: m.price_per_1k,
            registered_at: m.registered_at,
            is_active: m.is_active,
            total_inferences: m.total_inferences,
            avg_latency_ms: m.avg_latency_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcProvider {
    pub address: String,
    pub name: String,
    pub stake: u64,
    pub gpu_specs: Vec<RpcGpuSpecOut>,
    pub total_gpu_memory_mb: u32,
    pub supported_models: Vec<String>,
    pub is_active: bool,
    pub jobs_completed: u64,
    pub jobs_failed: u64,
    pub avg_response_ms: u32,
    pub reputation: u32,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcGpuSpecOut {
    pub model: String,
    pub vram_mb: u32,
    pub count: u8,
}

impl From<AIProvider> for RpcProvider {
    fn from(p: AIProvider) -> Self {
        Self {
            address: format!("0x{}", hex::encode(&p.address)),
            name: p.name,
            stake: p.stake,
            gpu_specs: p.gpu_specs.into_iter().map(|g| RpcGpuSpecOut {
                model: g.model,
                vram_mb: g.vram_mb,
                count: g.count,
            }).collect(),
            total_gpu_memory_mb: p.total_gpu_memory_mb,
            supported_models: p.supported_models.iter()
                .map(|id| format!("0x{}", hex::encode(&id.0)))
                .collect(),
            is_active: p.is_active,
            jobs_completed: p.jobs_completed,
            jobs_failed: p.jobs_failed,
            avg_response_ms: p.avg_response_ms,
            reputation: p.reputation,
            endpoint: p.endpoint,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcJob {
    pub id: String,
    pub job_type: String,
    pub model_id: String,
    pub requester: String,
    pub provider: Option<String>,
    pub status: String,
    pub input: String,
    pub output: Option<String>,
    pub max_price: u64,
    pub actual_price: Option<u64>,
    pub collateral: u64,
    pub submitted_at: u64,
    pub assigned_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub error: Option<String>,
    pub timeout_secs: u32,
}

impl From<AIJob> for RpcJob {
    fn from(j: AIJob) -> Self {
        Self {
            id: format!("0x{}", hex::encode(&j.id.0)),
            job_type: match j.job_type {
                JobType::Inference => "inference",
                JobType::BatchInference => "batch_inference",
                JobType::Training => "training",
                JobType::FineTuning => "fine_tuning",
            }.to_string(),
            model_id: format!("0x{}", hex::encode(&j.model_id.0)),
            requester: format!("0x{}", hex::encode(&j.requester)),
            provider: j.provider.map(|p| format!("0x{}", hex::encode(&p))),
            status: j.status.as_str().to_string(),
            input: j.input,
            output: j.output,
            max_price: j.max_price,
            actual_price: j.actual_price,
            collateral: j.collateral,
            submitted_at: j.submitted_at,
            assigned_at: j.assigned_at,
            completed_at: j.completed_at,
            error: j.error,
            timeout_secs: j.timeout_secs,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcRegistryStats {
    pub total_models: usize,
    pub active_models: usize,
    pub total_providers: usize,
    pub active_providers: usize,
    pub total_inferences: u64,
    pub total_gpu_memory_gb: u64,
}

impl From<RegistryStats> for RpcRegistryStats {
    fn from(s: RegistryStats) -> Self {
        Self {
            total_models: s.total_models,
            active_models: s.active_models,
            total_providers: s.total_providers,
            active_providers: s.active_providers,
            total_inferences: s.total_inferences,
            total_gpu_memory_gb: s.total_gpu_memory_gb,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcJobStats {
    pub total_jobs: usize,
    pub pending_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: u64,
    pub failed_jobs: usize,
    pub total_paid_pyrax: u64,
}

impl From<JobStats> for RpcJobStats {
    fn from(s: JobStats) -> Self {
        Self {
            total_jobs: s.total_jobs,
            pending_jobs: s.pending_jobs,
            running_jobs: s.running_jobs,
            completed_jobs: s.completed_jobs,
            failed_jobs: s.failed_jobs,
            total_paid_pyrax: s.total_paid_pyrax,
        }
    }
}

// ===== Helper Functions =====

fn parse_hash(s: &str) -> RpcResult<H256> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)
        .map_err(|_| RpcError::InvalidParams("invalid hash".into()))?;
    if bytes.len() != 32 {
        return Err(RpcError::InvalidParams("hash must be 32 bytes".into()).into());
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(H256(arr))
}

fn parse_address(s: &str) -> RpcResult<[u8; 20]> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)
        .map_err(|_| RpcError::InvalidParams("invalid address".into()))?;
    if bytes.len() != 20 {
        return Err(RpcError::InvalidParams("address must be 20 bytes".into()).into());
    }
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

fn compute_model_id(name: &str, version: &str, owner: &[u8; 20]) -> H256 {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(name.as_bytes());
    hasher.update(version.as_bytes());
    hasher.update(owner);
    let hash = hasher.finalize();
    H256(*hash.as_bytes())
}

fn compute_job_id(requester: &[u8; 20], model_id: &H256, timestamp: u64) -> H256 {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(requester);
    hasher.update(&model_id.0);
    hasher.update(&timestamp.to_le_bytes());
    let hash = hasher.finalize();
    H256(*hash.as_bytes())
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
