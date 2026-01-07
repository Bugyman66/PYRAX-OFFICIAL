//! Model Registry
//!
//! On-chain registry for AI/ML models with versioning

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};

/// Model type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    /// Large Language Model
    LLM,
    /// Image generation/classification
    Vision,
    /// Audio processing
    Audio,
    /// Multi-modal model
    MultiModal,
    /// Custom/other
    Custom,
}

/// Model license type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseType {
    /// Open source (MIT, Apache, etc.)
    OpenSource,
    /// Commercial use allowed
    Commercial,
    /// Research only
    ResearchOnly,
    /// Proprietary
    Proprietary,
    /// Custom license
    Custom,
}

/// Model version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Version number (semantic versioning)
    pub version: String,
    /// Model weights hash (IPFS/Arweave)
    pub weights_hash: H256,
    /// Model weights size (bytes)
    pub weights_size: u64,
    /// Config/architecture hash
    pub config_hash: H256,
    /// Tokenizer hash (if applicable)
    pub tokenizer_hash: Option<H256>,
    /// Release notes
    pub release_notes: Option<String>,
    /// Published timestamp
    pub published_at: u64,
    /// Publisher address
    pub publisher: Address,
    /// Is deprecated
    pub deprecated: bool,
    /// Deprecation reason
    pub deprecation_reason: Option<String>,
}

/// Model benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Benchmark name
    pub name: String,
    /// Score
    pub score: f64,
    /// Metric unit
    pub unit: String,
    /// Test date
    pub tested_at: u64,
    /// Test configuration
    pub config: Option<String>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique model ID
    pub id: H256,
    /// Model name
    pub name: String,
    /// Model description
    pub description: String,
    /// Model type
    pub model_type: ModelType,
    /// Owner address
    pub owner: Address,
    /// License type
    pub license: LicenseType,
    /// License URL (if custom)
    pub license_url: Option<String>,
    /// Model architecture (e.g., "transformer", "diffusion")
    pub architecture: String,
    /// Parameter count
    pub parameters: u64,
    /// Context length (for LLMs)
    pub context_length: Option<u32>,
    /// Input format description
    pub input_format: String,
    /// Output format description
    pub output_format: String,
    /// Supported frameworks (e.g., ["pytorch", "onnx"])
    pub frameworks: Vec<String>,
    /// Hardware requirements
    pub requirements: HardwareRequirements,
    /// All versions
    pub versions: Vec<ModelVersion>,
    /// Current/latest version
    pub current_version: String,
    /// Benchmark results
    pub benchmarks: Vec<BenchmarkResults>,
    /// Tags for discovery
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Total downloads/usages
    pub usage_count: u64,
    /// Average rating (0-100)
    pub rating: u32,
    /// Number of ratings
    pub rating_count: u32,
    /// Is verified by platform
    pub verified: bool,
    /// Is public
    pub is_public: bool,
    /// Usage price per inference (PYRAX)
    pub price_per_inference: u64,
    /// Training data hash (for transparency)
    pub training_data_hash: Option<H256>,
}

impl ModelInfo {
    /// Create new model info
    pub fn new(
        name: String,
        description: String,
        model_type: ModelType,
        owner: Address,
        architecture: String,
        parameters: u64,
    ) -> Self {
        let id = Self::generate_id(&name, &owner);
        
        Self {
            id,
            name,
            description,
            model_type,
            owner,
            license: LicenseType::OpenSource,
            license_url: None,
            architecture,
            parameters,
            context_length: None,
            input_format: String::new(),
            output_format: String::new(),
            frameworks: Vec::new(),
            requirements: HardwareRequirements::default(),
            versions: Vec::new(),
            current_version: String::new(),
            benchmarks: Vec::new(),
            tags: Vec::new(),
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
            usage_count: 0,
            rating: 0,
            rating_count: 0,
            verified: false,
            is_public: true,
            price_per_inference: 0,
            training_data_hash: None,
        }
    }

    /// Generate model ID
    fn generate_id(name: &str, owner: &Address) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(name.as_bytes());
        hasher.update(&owner.0);
        hasher.update(&current_timestamp().to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Add new version
    pub fn add_version(&mut self, version: ModelVersion) {
        self.current_version = version.version.clone();
        self.versions.push(version);
        self.updated_at = current_timestamp();
    }

    /// Get specific version
    pub fn get_version(&self, version: &str) -> Option<&ModelVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Get current version
    pub fn get_current_version(&self) -> Option<&ModelVersion> {
        self.versions.iter().find(|v| v.version == self.current_version)
    }

    /// Deprecate version
    pub fn deprecate_version(&mut self, version: &str, reason: String) -> bool {
        if let Some(v) = self.versions.iter_mut().find(|v| v.version == version) {
            v.deprecated = true;
            v.deprecation_reason = Some(reason);
            self.updated_at = current_timestamp();
            true
        } else {
            false
        }
    }

    /// Update rating
    pub fn add_rating(&mut self, rating: u32) {
        let total = self.rating as u64 * self.rating_count as u64 + rating as u64;
        self.rating_count += 1;
        self.rating = (total / self.rating_count as u64) as u32;
        self.updated_at = current_timestamp();
    }

    /// Increment usage count
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
    }

    /// Transfer ownership
    pub fn transfer_ownership(&mut self, new_owner: Address) {
        self.owner = new_owner;
        self.updated_at = current_timestamp();
    }

    /// Set price
    pub fn set_price(&mut self, price: u64) {
        self.price_per_inference = price;
        self.updated_at = current_timestamp();
    }
}

/// Hardware requirements for model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRequirements {
    /// Minimum GPU memory (MB)
    pub min_gpu_memory_mb: u64,
    /// Recommended GPU memory (MB)
    pub recommended_gpu_memory_mb: u64,
    /// Minimum RAM (MB)
    pub min_ram_mb: u64,
    /// Minimum storage (MB)
    pub min_storage_mb: u64,
    /// Supported GPU types
    pub supported_gpus: Vec<String>,
    /// Requires specific GPU features
    pub gpu_features: Vec<String>,
}

impl Default for HardwareRequirements {
    fn default() -> Self {
        Self {
            min_gpu_memory_mb: 4096,
            recommended_gpu_memory_mb: 8192,
            min_ram_mb: 8192,
            min_storage_mb: 10240,
            supported_gpus: vec!["NVIDIA".to_string()],
            gpu_features: Vec::new(),
        }
    }
}

/// Model registry
pub struct ModelRegistry {
    /// Models by ID
    models: Arc<RwLock<HashMap<H256, ModelInfo>>>,
    /// Models by owner
    by_owner: Arc<RwLock<HashMap<Address, Vec<H256>>>>,
    /// Models by type
    by_type: Arc<RwLock<HashMap<ModelType, Vec<H256>>>>,
    /// Models by tag
    by_tag: Arc<RwLock<HashMap<String, Vec<H256>>>>,
    /// Verified models
    verified: Arc<RwLock<Vec<H256>>>,
}

impl ModelRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            by_owner: Arc::new(RwLock::new(HashMap::new())),
            by_type: Arc::new(RwLock::new(HashMap::new())),
            by_tag: Arc::new(RwLock::new(HashMap::new())),
            verified: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register new model
    pub fn register(&self, model: ModelInfo) -> Result<H256, RegistryError> {
        let id = model.id;
        
        {
            let mut models = self.models.write();
            if models.contains_key(&id) {
                return Err(RegistryError::AlreadyExists(id));
            }
            
            // Index by owner
            self.by_owner.write()
                .entry(model.owner)
                .or_insert_with(Vec::new)
                .push(id);
            
            // Index by type
            self.by_type.write()
                .entry(model.model_type)
                .or_insert_with(Vec::new)
                .push(id);
            
            // Index by tags
            for tag in &model.tags {
                self.by_tag.write()
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(id);
            }
            
            models.insert(id, model);
        }

        Ok(id)
    }

    /// Get model by ID
    pub fn get(&self, id: &H256) -> Option<ModelInfo> {
        self.models.read().get(id).cloned()
    }

    /// Update model
    pub fn update(&self, id: &H256, f: impl FnOnce(&mut ModelInfo)) -> Result<(), RegistryError> {
        let mut models = self.models.write();
        let model = models.get_mut(id)
            .ok_or_else(|| RegistryError::NotFound(*id))?;
        f(model);
        Ok(())
    }

    /// Add version to model
    pub fn add_version(&self, model_id: &H256, version: ModelVersion) -> Result<(), RegistryError> {
        self.update(model_id, |m| m.add_version(version))
    }

    /// Get models by owner
    pub fn get_by_owner(&self, owner: &Address) -> Vec<ModelInfo> {
        let ids = self.by_owner.read()
            .get(owner)
            .cloned()
            .unwrap_or_default();

        let models = self.models.read();
        ids.iter()
            .filter_map(|id| models.get(id).cloned())
            .collect()
    }

    /// Get models by type
    pub fn get_by_type(&self, model_type: ModelType) -> Vec<ModelInfo> {
        let ids = self.by_type.read()
            .get(&model_type)
            .cloned()
            .unwrap_or_default();

        let models = self.models.read();
        ids.iter()
            .filter_map(|id| models.get(id).cloned())
            .collect()
    }

    /// Get models by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<ModelInfo> {
        let ids = self.by_tag.read()
            .get(tag)
            .cloned()
            .unwrap_or_default();

        let models = self.models.read();
        ids.iter()
            .filter_map(|id| models.get(id).cloned())
            .collect()
    }

    /// Search models
    pub fn search(&self, query: &str) -> Vec<ModelInfo> {
        let query_lower = query.to_lowercase();
        self.models.read()
            .values()
            .filter(|m| {
                m.is_public && (
                    m.name.to_lowercase().contains(&query_lower) ||
                    m.description.to_lowercase().contains(&query_lower) ||
                    m.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                )
            })
            .cloned()
            .collect()
    }

    /// Verify model (admin only)
    pub fn verify_model(&self, id: &H256) -> Result<(), RegistryError> {
        self.update(id, |m| m.verified = true)?;
        self.verified.write().push(*id);
        Ok(())
    }

    /// Get verified models
    pub fn get_verified(&self) -> Vec<ModelInfo> {
        let ids = self.verified.read().clone();
        let models = self.models.read();
        ids.iter()
            .filter_map(|id| models.get(id).cloned())
            .collect()
    }

    /// Get top models by usage
    pub fn get_top_by_usage(&self, limit: usize) -> Vec<ModelInfo> {
        let mut models: Vec<_> = self.models.read()
            .values()
            .filter(|m| m.is_public)
            .cloned()
            .collect();
        
        models.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        models.truncate(limit);
        models
    }

    /// Get top models by rating
    pub fn get_top_by_rating(&self, limit: usize) -> Vec<ModelInfo> {
        let mut models: Vec<_> = self.models.read()
            .values()
            .filter(|m| m.is_public && m.rating_count >= 5)
            .cloned()
            .collect();
        
        models.sort_by(|a, b| b.rating.cmp(&a.rating));
        models.truncate(limit);
        models
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let models = self.models.read();
        
        let mut by_type = HashMap::new();
        for model in models.values() {
            *by_type.entry(model.model_type).or_insert(0u64) += 1;
        }

        RegistryStats {
            total_models: models.len() as u64,
            public_models: models.values().filter(|m| m.is_public).count() as u64,
            verified_models: self.verified.read().len() as u64,
            total_usage: models.values().map(|m| m.usage_count).sum(),
            by_type,
        }
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Model not found: {0:?}")]
    NotFound(H256),

    #[error("Model already exists: {0:?}")]
    AlreadyExists(H256),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid model: {0}")]
    Invalid(String),
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_models: u64,
    pub public_models: u64,
    pub verified_models: u64,
    pub total_usage: u64,
    pub by_type: HashMap<ModelType, u64>,
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
    fn test_model_creation() {
        let owner = Address([1u8; 20]);
        let model = ModelInfo::new(
            "TestModel".to_string(),
            "A test model".to_string(),
            ModelType::LLM,
            owner,
            "transformer".to_string(),
            7_000_000_000,
        );

        assert_eq!(model.name, "TestModel");
        assert_eq!(model.model_type, ModelType::LLM);
        assert!(model.is_public);
    }

    #[test]
    fn test_model_versioning() {
        let owner = Address([1u8; 20]);
        let mut model = ModelInfo::new(
            "TestModel".to_string(),
            "A test model".to_string(),
            ModelType::LLM,
            owner,
            "transformer".to_string(),
            7_000_000_000,
        );

        let version = ModelVersion {
            version: "1.0.0".to_string(),
            weights_hash: H256([1u8; 32]),
            weights_size: 14_000_000_000,
            config_hash: H256([2u8; 32]),
            tokenizer_hash: Some(H256([3u8; 32])),
            release_notes: Some("Initial release".to_string()),
            published_at: current_timestamp(),
            publisher: owner,
            deprecated: false,
            deprecation_reason: None,
        };

        model.add_version(version);

        assert_eq!(model.current_version, "1.0.0");
        assert_eq!(model.versions.len(), 1);
    }

    #[test]
    fn test_registry() {
        let registry = ModelRegistry::new();
        let owner = Address([1u8; 20]);

        let mut model = ModelInfo::new(
            "TestModel".to_string(),
            "A test model".to_string(),
            ModelType::LLM,
            owner,
            "transformer".to_string(),
            7_000_000_000,
        );
        model.tags = vec!["llm".to_string(), "chat".to_string()];

        let id = registry.register(model).unwrap();

        assert!(registry.get(&id).is_some());
        assert_eq!(registry.get_by_owner(&owner).len(), 1);
        assert_eq!(registry.get_by_type(ModelType::LLM).len(), 1);
        assert_eq!(registry.get_by_tag("llm").len(), 1);
    }

    #[test]
    fn test_model_rating() {
        let owner = Address([1u8; 20]);
        let mut model = ModelInfo::new(
            "TestModel".to_string(),
            "A test model".to_string(),
            ModelType::LLM,
            owner,
            "transformer".to_string(),
            7_000_000_000,
        );

        model.add_rating(80);
        model.add_rating(90);
        model.add_rating(100);

        assert_eq!(model.rating_count, 3);
        assert_eq!(model.rating, 90); // Average
    }
}
