use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, warn, debug};

use crate::types::H256;
use super::{Model, ModelFramework, ModelType, AIProvider, GpuSpec};

/// Model Registry - manages registered AI models
pub struct ModelRegistry {
    /// Models by ID
    models: Arc<RwLock<HashMap<H256, Model>>>,
    /// Models by owner
    models_by_owner: Arc<RwLock<HashMap<[u8; 20], Vec<H256>>>>,
    /// Providers by address
    providers: Arc<RwLock<HashMap<[u8; 20], AIProvider>>>,
    /// Active provider count
    active_provider_count: Arc<RwLock<u32>>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            models_by_owner: Arc::new(RwLock::new(HashMap::new())),
            providers: Arc::new(RwLock::new(HashMap::new())),
            active_provider_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Register a new model
    pub fn register_model(&self, model: Model) -> Result<H256, RegistryError> {
        let model_id = model.id;
        
        // Check if model already exists
        {
            let models = self.models.read();
            if models.contains_key(&model_id) {
                return Err(RegistryError::ModelAlreadyExists(model_id));
            }
        }

        // Validate model data
        self.validate_model(&model)?;

        // Store model
        {
            let mut models = self.models.write();
            models.insert(model_id, model.clone());
        }

        // Update owner index
        {
            let mut by_owner = self.models_by_owner.write();
            by_owner.entry(model.owner).or_insert_with(Vec::new).push(model_id);
        }

        info!(
            "Registered model: {} v{} ({})",
            model.name, model.version, hex::encode(&model_id.0[..8])
        );

        Ok(model_id)
    }

    /// Validate model data
    fn validate_model(&self, model: &Model) -> Result<(), RegistryError> {
        if model.name.is_empty() {
            return Err(RegistryError::InvalidModelData("name cannot be empty".into()));
        }
        if model.name.len() > 128 {
            return Err(RegistryError::InvalidModelData("name too long (max 128 chars)".into()));
        }
        if model.version.is_empty() {
            return Err(RegistryError::InvalidModelData("version cannot be empty".into()));
        }
        if model.ipfs_cid.is_empty() {
            return Err(RegistryError::InvalidModelData("IPFS CID required".into()));
        }
        if model.size_bytes == 0 {
            return Err(RegistryError::InvalidModelData("size must be > 0".into()));
        }
        Ok(())
    }

    /// Get model by ID
    pub fn get_model(&self, id: &H256) -> Option<Model> {
        self.models.read().get(id).cloned()
    }

    /// Get all models by owner
    pub fn get_models_by_owner(&self, owner: &[u8; 20]) -> Vec<Model> {
        let by_owner = self.models_by_owner.read();
        let models = self.models.read();
        
        by_owner.get(owner)
            .map(|ids| ids.iter().filter_map(|id| models.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    /// List all active models
    pub fn list_models(&self, limit: usize, offset: usize) -> Vec<Model> {
        let models = self.models.read();
        models.values()
            .filter(|m| m.is_active)
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Search models by type and framework
    pub fn search_models(
        &self,
        model_type: Option<ModelType>,
        framework: Option<ModelFramework>,
        limit: usize,
    ) -> Vec<Model> {
        let models = self.models.read();
        models.values()
            .filter(|m| m.is_active)
            .filter(|m| model_type.map(|t| m.model_type == t).unwrap_or(true))
            .filter(|m| framework.map(|f| m.framework == f).unwrap_or(true))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Deactivate a model (only owner can do this)
    pub fn deactivate_model(&self, id: &H256, caller: &[u8; 20]) -> Result<(), RegistryError> {
        let mut models = self.models.write();
        let model = models.get_mut(id).ok_or(RegistryError::ModelNotFound(*id))?;
        
        if &model.owner != caller {
            return Err(RegistryError::NotAuthorized);
        }

        model.is_active = false;
        info!("Deactivated model: {}", hex::encode(&id.0[..8]));
        Ok(())
    }

    /// Update model statistics after inference
    pub fn record_inference(&self, id: &H256, latency_ms: u32) {
        let mut models = self.models.write();
        if let Some(model) = models.get_mut(id) {
            let total = model.total_inferences;
            let avg = model.avg_latency_ms as u64;
            
            // Rolling average
            model.avg_latency_ms = ((avg * total + latency_ms as u64) / (total + 1)) as u32;
            model.total_inferences += 1;
        }
    }

    /// Register a new AI provider
    pub fn register_provider(&self, provider: AIProvider) -> Result<(), RegistryError> {
        let addr = provider.address;

        // Validate provider data
        self.validate_provider(&provider)?;

        // Minimum stake requirement (1000 PYRAX = 1000 * 10^8 smallest units)
        const MIN_STAKE: u64 = 100_000_000_000; // 1000 PYRAX
        if provider.stake < MIN_STAKE {
            return Err(RegistryError::InsufficientStake(MIN_STAKE, provider.stake));
        }

        let is_new = {
            let providers = self.providers.read();
            !providers.contains_key(&addr)
        };

        {
            let mut providers = self.providers.write();
            providers.insert(addr, provider.clone());
        }

        if is_new && provider.is_active {
            let mut count = self.active_provider_count.write();
            *count += 1;
        }

        info!(
            "Registered provider: {} with {} GPU(s), stake: {} PYRAX",
            provider.name,
            provider.gpu_specs.iter().map(|g| g.count as u32).sum::<u32>(),
            provider.stake / 100_000_000
        );

        Ok(())
    }

    /// Validate provider data
    fn validate_provider(&self, provider: &AIProvider) -> Result<(), RegistryError> {
        if provider.name.is_empty() {
            return Err(RegistryError::InvalidProviderData("name cannot be empty".into()));
        }
        if provider.gpu_specs.is_empty() {
            return Err(RegistryError::InvalidProviderData("at least one GPU required".into()));
        }
        if provider.endpoint.is_empty() {
            return Err(RegistryError::InvalidProviderData("endpoint required".into()));
        }
        Ok(())
    }

    /// Get provider by address
    pub fn get_provider(&self, addr: &[u8; 20]) -> Option<AIProvider> {
        self.providers.read().get(addr).cloned()
    }

    /// List active providers
    pub fn list_providers(&self, limit: usize, offset: usize) -> Vec<AIProvider> {
        let providers = self.providers.read();
        providers.values()
            .filter(|p| p.is_active)
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Find providers that support a specific model
    pub fn find_providers_for_model(&self, model_id: &H256) -> Vec<AIProvider> {
        let providers = self.providers.read();
        providers.values()
            .filter(|p| p.is_active && p.supported_models.contains(model_id))
            .cloned()
            .collect()
    }

    /// Update provider statistics after job completion
    pub fn record_job_completion(&self, addr: &[u8; 20], success: bool, response_ms: u32) {
        let mut providers = self.providers.write();
        if let Some(provider) = providers.get_mut(addr) {
            if success {
                provider.jobs_completed += 1;
                // Update reputation (simple increase)
                provider.reputation = (provider.reputation + 1).min(1000);
            } else {
                provider.jobs_failed += 1;
                // Decrease reputation
                provider.reputation = provider.reputation.saturating_sub(10);
            }

            // Rolling average response time
            let total = provider.jobs_completed + provider.jobs_failed;
            let avg = provider.avg_response_ms as u64;
            provider.avg_response_ms = ((avg * (total - 1) + response_ms as u64) / total) as u32;
        }
    }

    /// Get total model count
    pub fn model_count(&self) -> usize {
        self.models.read().len()
    }

    /// Get active provider count
    pub fn active_provider_count(&self) -> u32 {
        *self.active_provider_count.read()
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> RegistryStats {
        let models = self.models.read();
        let providers = self.providers.read();
        
        let active_models = models.values().filter(|m| m.is_active).count();
        let total_inferences: u64 = models.values().map(|m| m.total_inferences).sum();
        let active_providers = providers.values().filter(|p| p.is_active).count();
        let total_gpu_memory: u64 = providers.values()
            .filter(|p| p.is_active)
            .map(|p| p.total_gpu_memory_mb as u64)
            .sum();

        RegistryStats {
            total_models: models.len(),
            active_models,
            total_providers: providers.len(),
            active_providers,
            total_inferences,
            total_gpu_memory_gb: total_gpu_memory / 1024,
        }
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistryStats {
    pub total_models: usize,
    pub active_models: usize,
    pub total_providers: usize,
    pub active_providers: usize,
    pub total_inferences: u64,
    pub total_gpu_memory_gb: u64,
}

/// Registry errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Model already exists: {0:?}")]
    ModelAlreadyExists(H256),
    
    #[error("Model not found: {0:?}")]
    ModelNotFound(H256),
    
    #[error("Invalid model data: {0}")]
    InvalidModelData(String),
    
    #[error("Invalid provider data: {0}")]
    InvalidProviderData(String),
    
    #[error("Not authorized")]
    NotAuthorized,
    
    #[error("Insufficient stake: required {0}, got {1}")]
    InsufficientStake(u64, u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> Model {
        Model {
            id: H256([1u8; 32]),
            name: "test-model".to_string(),
            version: "1.0.0".to_string(),
            owner: [0u8; 20],
            framework: ModelFramework::PyTorch,
            model_type: ModelType::LLM,
            size_bytes: 1024 * 1024 * 100, // 100MB
            ipfs_cid: "QmTest123".to_string(),
            input_schema: "{}".to_string(),
            output_schema: "{}".to_string(),
            min_gpu_memory_mb: 8192,
            price_per_1k: 1000,
            registered_at: 0,
            registered_height: 0,
            is_active: true,
            total_inferences: 0,
            avg_latency_ms: 0,
        }
    }

    fn create_test_provider() -> AIProvider {
        AIProvider {
            address: [1u8; 20],
            name: "test-provider".to_string(),
            stake: 100_000_000_000, // 1000 PYRAX
            gpu_specs: vec![GpuSpec {
                model: "RTX 4090".to_string(),
                vram_mb: 24576,
                count: 1,
            }],
            total_gpu_memory_mb: 24576,
            supported_models: vec![],
            is_active: true,
            registered_at: 0,
            jobs_completed: 0,
            jobs_failed: 0,
            avg_response_ms: 0,
            reputation: 500,
            endpoint: "/ip4/127.0.0.1/tcp/30400".to_string(),
        }
    }

    #[test]
    fn test_register_model() {
        let registry = ModelRegistry::new();
        let model = create_test_model();
        
        let result = registry.register_model(model.clone());
        assert!(result.is_ok());
        
        let fetched = registry.get_model(&model.id);
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "test-model");
    }

    #[test]
    fn test_duplicate_model() {
        let registry = ModelRegistry::new();
        let model = create_test_model();
        
        registry.register_model(model.clone()).unwrap();
        let result = registry.register_model(model);
        assert!(matches!(result, Err(RegistryError::ModelAlreadyExists(_))));
    }

    #[test]
    fn test_register_provider() {
        let registry = ModelRegistry::new();
        let provider = create_test_provider();
        
        let result = registry.register_provider(provider.clone());
        assert!(result.is_ok());
        
        let fetched = registry.get_provider(&provider.address);
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "test-provider");
    }

    #[test]
    fn test_insufficient_stake() {
        let registry = ModelRegistry::new();
        let mut provider = create_test_provider();
        provider.stake = 1000; // Too low
        
        let result = registry.register_provider(provider);
        assert!(matches!(result, Err(RegistryError::InsufficientStake(_, _))));
    }

    #[test]
    fn test_registry_stats() {
        let registry = ModelRegistry::new();
        
        registry.register_model(create_test_model()).unwrap();
        registry.register_provider(create_test_provider()).unwrap();
        
        let stats = registry.get_stats();
        assert_eq!(stats.total_models, 1);
        assert_eq!(stats.active_models, 1);
        assert_eq!(stats.active_providers, 1);
    }
}
