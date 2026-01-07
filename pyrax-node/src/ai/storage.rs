//! IPFS Storage Integration for AI Models
//!
//! Handles model upload, download, and verification using IPFS

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write, BufReader, BufWriter};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn, debug, error};
use blake3::Hasher;

use crate::types::H256;

/// IPFS Client for model storage
pub struct IpfsStorage {
    /// IPFS API endpoint
    api_url: String,
    /// Local cache directory
    cache_dir: PathBuf,
    /// HTTP client
    client: reqwest::Client,
    /// Gateway URL for downloads
    gateway_url: String,
}

impl IpfsStorage {
    /// Create new IPFS storage client
    pub fn new(api_url: &str, gateway_url: &str, cache_dir: PathBuf) -> Result<Self, StorageError> {
        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300)) // 5 min timeout for large models
            .build()
            .map_err(|e| StorageError::HttpError(e.to_string()))?;

        Ok(Self {
            api_url: api_url.to_string(),
            cache_dir,
            client,
            gateway_url: gateway_url.to_string(),
        })
    }

    /// Create with default local IPFS node
    pub fn local(cache_dir: PathBuf) -> Result<Self, StorageError> {
        Self::new(
            "http://127.0.0.1:5001",
            "http://127.0.0.1:8080",
            cache_dir,
        )
    }

    /// Create with public IPFS gateway (read-only)
    pub fn public_gateway(cache_dir: PathBuf) -> Result<Self, StorageError> {
        Self::new(
            "https://api.web3.storage", // For uploads (requires API key)
            "https://ipfs.io",          // Public gateway for downloads
            cache_dir,
        )
    }

    /// Upload a model file to IPFS
    pub async fn upload_model(&self, file_path: &Path) -> Result<UploadResult, StorageError> {
        // Read and hash the file
        let file = File::open(file_path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        let metadata = file.metadata()
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        let size = metadata.len();
        
        // Calculate content hash
        let mut reader = BufReader::new(file);
        let mut hasher = Hasher::new();
        let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
        
        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        let content_hash = H256(*hasher.finalize().as_bytes());

        // Upload to IPFS via API
        let file_bytes = fs::read(file_path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(file_bytes)
                .file_name(file_path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()));

        let response = self.client
            .post(format!("{}/api/v0/add", self.api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| StorageError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(StorageError::UploadFailed(format!("{}: {}", status, body)));
        }

        let add_response: IpfsAddResponse = response.json().await
            .map_err(|e| StorageError::HttpError(e.to_string()))?;

        info!(
            "Model uploaded to IPFS: {} ({} bytes)",
            add_response.hash, size
        );

        Ok(UploadResult {
            cid: add_response.hash,
            size,
            content_hash,
        })
    }

    /// Download a model from IPFS
    pub async fn download_model(&self, cid: &str, dest_path: &Path) -> Result<DownloadResult, StorageError> {
        // Check cache first
        let cache_path = self.cache_dir.join(cid);
        if cache_path.exists() {
            debug!("Model found in cache: {}", cid);
            fs::copy(&cache_path, dest_path)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
            
            let metadata = fs::metadata(dest_path)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
            
            return Ok(DownloadResult {
                size: metadata.len(),
                from_cache: true,
            });
        }

        // Download from IPFS gateway
        let url = format!("{}/ipfs/{}", self.gateway_url, cid);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| StorageError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(StorageError::DownloadFailed(
                format!("HTTP {}: {}", response.status(), cid)
            ));
        }

        let bytes = response.bytes().await
            .map_err(|e| StorageError::HttpError(e.to_string()))?;

        let size = bytes.len() as u64;

        // Write to destination
        let mut file = BufWriter::new(
            File::create(dest_path)
                .map_err(|e| StorageError::IoError(e.to_string()))?
        );
        file.write_all(&bytes)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Cache the file
        fs::copy(dest_path, &cache_path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        info!("Model downloaded from IPFS: {} ({} bytes)", cid, size);

        Ok(DownloadResult {
            size,
            from_cache: false,
        })
    }

    /// Pin a CID to ensure it's not garbage collected
    pub async fn pin(&self, cid: &str) -> Result<(), StorageError> {
        let response = self.client
            .post(format!("{}/api/v0/pin/add?arg={}", self.api_url, cid))
            .send()
            .await
            .map_err(|e| StorageError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(StorageError::PinFailed(cid.to_string()));
        }

        info!("Pinned IPFS content: {}", cid);
        Ok(())
    }

    /// Unpin a CID
    pub async fn unpin(&self, cid: &str) -> Result<(), StorageError> {
        let response = self.client
            .post(format!("{}/api/v0/pin/rm?arg={}", self.api_url, cid))
            .send()
            .await
            .map_err(|e| StorageError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            warn!("Failed to unpin: {}", cid);
        }

        Ok(())
    }

    /// Check if content exists on IPFS
    pub async fn exists(&self, cid: &str) -> bool {
        // Check cache first
        if self.cache_dir.join(cid).exists() {
            return true;
        }

        // Try to stat on IPFS
        let response = self.client
            .post(format!("{}/api/v0/files/stat?arg=/ipfs/{}", self.api_url, cid))
            .send()
            .await;

        match response {
            Ok(r) => r.status().is_success(),
            Err(_) => false,
        }
    }

    /// Get content info without downloading
    pub async fn get_info(&self, cid: &str) -> Result<ContentInfo, StorageError> {
        let response = self.client
            .post(format!("{}/api/v0/object/stat?arg={}", self.api_url, cid))
            .send()
            .await
            .map_err(|e| StorageError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(StorageError::NotFound(cid.to_string()));
        }

        let stat: IpfsStatResponse = response.json().await
            .map_err(|e| StorageError::HttpError(e.to_string()))?;

        Ok(ContentInfo {
            cid: cid.to_string(),
            size: stat.cumulative_size,
            num_links: stat.num_links,
        })
    }

    /// Verify content hash matches expected
    pub async fn verify(&self, cid: &str, expected_hash: &H256) -> Result<bool, StorageError> {
        let cache_path = self.cache_dir.join(cid);
        
        let path = if cache_path.exists() {
            cache_path
        } else {
            // Download to temp location
            let temp_path = self.cache_dir.join(format!("{}.tmp", cid));
            self.download_model(cid, &temp_path).await?;
            temp_path
        };

        // Calculate hash
        let file = File::open(&path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut hasher = Hasher::new();
        let mut buffer = vec![0u8; 1024 * 1024];

        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let actual_hash = H256(*hasher.finalize().as_bytes());
        Ok(&actual_hash == expected_hash)
    }

    /// Clear cache
    pub fn clear_cache(&self) -> Result<(), StorageError> {
        fs::remove_dir_all(&self.cache_dir)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        info!("IPFS cache cleared");
        Ok(())
    }

    /// Get cache size in bytes
    pub fn cache_size(&self) -> u64 {
        walkdir::WalkDir::new(&self.cache_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum()
    }
}

/// Upload result
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// IPFS CID
    pub cid: String,
    /// File size in bytes
    pub size: u64,
    /// Content hash (BLAKE3)
    pub content_hash: H256,
}

/// Download result
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Downloaded size in bytes
    pub size: u64,
    /// Whether served from cache
    pub from_cache: bool,
}

/// Content info
#[derive(Debug, Clone)]
pub struct ContentInfo {
    pub cid: String,
    pub size: u64,
    pub num_links: u32,
}

/// IPFS add response
#[derive(Debug, Deserialize)]
struct IpfsAddResponse {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Size")]
    size: String,
}

/// IPFS stat response
#[derive(Debug, Deserialize)]
struct IpfsStatResponse {
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "NumLinks")]
    num_links: u32,
    #[serde(rename = "BlockSize")]
    block_size: u64,
    #[serde(rename = "LinksSize")]
    links_size: u64,
    #[serde(rename = "DataSize")]
    data_size: u64,
    #[serde(rename = "CumulativeSize")]
    cumulative_size: u64,
}

/// Storage errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Pin failed: {0}")]
    PinFailed(String),

    #[error("Content not found: {0}")]
    NotFound(String),

    #[error("Hash mismatch")]
    HashMismatch,
}

/// Model metadata for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub framework: String,
    pub model_type: String,
    pub size_bytes: u64,
    pub content_hash: String,
    pub input_schema: String,
    pub output_schema: String,
    pub min_gpu_memory_mb: u32,
    pub created_at: u64,
}

impl ModelMetadata {
    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Parse from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_model_metadata_serialization() {
        let metadata = ModelMetadata {
            name: "test-model".to_string(),
            version: "1.0.0".to_string(),
            framework: "pytorch".to_string(),
            model_type: "llm".to_string(),
            size_bytes: 1024 * 1024 * 100,
            content_hash: "0x1234".to_string(),
            input_schema: "{}".to_string(),
            output_schema: "{}".to_string(),
            min_gpu_memory_mb: 8192,
            created_at: 1704326400,
        };

        let json = metadata.to_json();
        let parsed = ModelMetadata::from_json(&json).unwrap();
        
        assert_eq!(parsed.name, "test-model");
        assert_eq!(parsed.version, "1.0.0");
        assert_eq!(parsed.framework, "pytorch");
    }

    #[test]
    fn test_storage_creation() {
        let temp = tempdir().unwrap();
        let storage = IpfsStorage::local(temp.path().to_path_buf());
        assert!(storage.is_ok());
    }
}
