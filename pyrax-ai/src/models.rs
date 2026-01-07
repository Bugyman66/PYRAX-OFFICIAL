use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub version: String,
    pub owner: String,
    pub description: String,
    pub model_type: ModelType,
    pub hash: String,
    pub storage_uri: String,
    pub size_bytes: u64,
    pub license: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub registration_tx: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    LLM,
    ImageGeneration,
    ImageClassification,
    ObjectDetection,
    TextToSpeech,
    SpeechToText,
    Embedding,
    Custom,
}

pub struct ModelRegistry {
    models: HashMap<String, Model>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn register(&mut self, model: Model) -> crate::Result<()> {
        if self.models.contains_key(&model.id) {
            return Err(crate::AiError::ModelNotFound(
                format!("Model {} already exists", model.id)
            ));
        }
        self.models.insert(model.id.clone(), model);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Model> {
        self.models.get(id)
    }

    pub fn update(&mut self, model: Model) -> crate::Result<()> {
        if !self.models.contains_key(&model.id) {
            return Err(crate::AiError::ModelNotFound(model.id));
        }
        self.models.insert(model.id.clone(), model);
        Ok(())
    }

    pub fn list(&self) -> Vec<&Model> {
        self.models.values().collect()
    }

    pub fn list_by_owner(&self, owner: &str) -> Vec<&Model> {
        self.models.values()
            .filter(|m| m.owner == owner)
            .collect()
    }

    pub fn list_by_type(&self, model_type: ModelType) -> Vec<&Model> {
        self.models.values()
            .filter(|m| m.model_type == model_type)
            .collect()
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    pub fn new(
        name: String,
        version: String,
        owner: String,
        model_type: ModelType,
        hash: String,
        storage_uri: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: generate_model_id(&name, &version, &owner),
            name,
            version,
            owner,
            description: String::new(),
            model_type,
            hash,
            storage_uri,
            size_bytes: 0,
            license: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            registration_tx: None,
        }
    }
}

fn generate_model_id(name: &str, version: &str, owner: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    hasher.update(version.as_bytes());
    hasher.update(owner.as_bytes());
    hex::encode(&hasher.finalize()[..16])
}
