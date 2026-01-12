use crate::{Result, AiError};
use sha2::{Sha256, Digest};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// IPFS API response for add operation
#[derive(Debug, Deserialize)]
struct IpfsAddResponse {
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Size")]
    size: String,
}

/// IPFS API response for pin operations
#[derive(Debug, Deserialize)]
struct IpfsPinResponse {
    #[serde(rename = "Pins")]
    pins: Vec<String>,
}

/// Storage backend using IPFS for decentralized AI model and data storage
pub struct Storage {
    ipfs_endpoint: String,
    client: reqwest::Client,
    gateway_url: String,
}

impl Storage {
    pub fn new(ipfs_endpoint: &str) -> Self {
        Self {
            ipfs_endpoint: ipfs_endpoint.to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_default(),
            gateway_url: "https://ipfs.io/ipfs".to_string(),
        }
    }

    /// Create storage with custom gateway URL
    pub fn with_gateway(ipfs_endpoint: &str, gateway_url: &str) -> Self {
        Self {
            ipfs_endpoint: ipfs_endpoint.to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_default(),
            gateway_url: gateway_url.to_string(),
        }
    }

    /// Upload data to IPFS and return the CID
    pub async fn upload(&self, data: &[u8]) -> Result<String> {
        let url = format!("{}/api/v0/add?pin=true", self.ipfs_endpoint);
        
        // Create multipart form with the data
        let part = multipart::Part::bytes(data.to_vec())
            .file_name("data")
            .mime_str("application/octet-stream")
            .map_err(|e| AiError::StorageError(format!("Failed to create multipart: {}", e)))?;
        
        let form = multipart::Form::new().part("file", part);
        
        debug!("Uploading {} bytes to IPFS at {}", data.len(), self.ipfs_endpoint);
        
        let response = self.client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AiError::StorageError(format!("IPFS upload failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AiError::StorageError(format!(
                "IPFS upload failed with status {}: {}", status, body
            )));
        }
        
        let ipfs_response: IpfsAddResponse = response
            .json()
            .await
            .map_err(|e| AiError::StorageError(format!("Failed to parse IPFS response: {}", e)))?;
        
        debug!("Uploaded to IPFS: CID={}, Size={}", ipfs_response.hash, ipfs_response.size);
        
        Ok(format!("ipfs://{}", ipfs_response.hash))
    }

    /// Download data from IPFS using CID
    pub async fn download(&self, uri: &str) -> Result<Vec<u8>> {
        let cid = extract_cid(uri)?;
        
        // Try IPFS API first (for local node)
        let api_url = format!("{}/api/v0/cat?arg={}", self.ipfs_endpoint, cid);
        
        debug!("Downloading from IPFS: {}", cid);
        
        match self.client.post(&api_url).send().await {
            Ok(response) if response.status().is_success() => {
                let data = response
                    .bytes()
                    .await
                    .map_err(|e| AiError::StorageError(format!("Failed to read IPFS data: {}", e)))?;
                debug!("Downloaded {} bytes from IPFS API", data.len());
                return Ok(data.to_vec());
            }
            Ok(response) => {
                warn!("IPFS API returned {}, trying gateway", response.status());
            }
            Err(e) => {
                warn!("IPFS API failed: {}, trying gateway", e);
            }
        }
        
        // Fallback to public gateway
        let gateway_url = format!("{}/{}", self.gateway_url, cid);
        
        let response = self.client
            .get(&gateway_url)
            .send()
            .await
            .map_err(|e| AiError::StorageError(format!("Gateway download failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AiError::StorageError(format!(
                "Gateway download failed with status {}", response.status()
            )));
        }
        
        let data = response
            .bytes()
            .await
            .map_err(|e| AiError::StorageError(format!("Failed to read gateway data: {}", e)))?;
        
        debug!("Downloaded {} bytes from gateway", data.len());
        Ok(data.to_vec())
    }

    /// Pin content to ensure it stays available
    pub async fn pin(&self, cid: &str) -> Result<()> {
        let cid = extract_cid(cid)?;
        let url = format!("{}/api/v0/pin/add?arg={}", self.ipfs_endpoint, cid);
        
        debug!("Pinning CID: {}", cid);
        
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| AiError::StorageError(format!("IPFS pin failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AiError::StorageError(format!(
                "IPFS pin failed with status {}: {}", status, body
            )));
        }
        
        let pin_response: IpfsPinResponse = response
            .json()
            .await
            .map_err(|e| AiError::StorageError(format!("Failed to parse pin response: {}", e)))?;
        
        debug!("Pinned: {:?}", pin_response.pins);
        Ok(())
    }

    /// Unpin content to allow garbage collection
    pub async fn unpin(&self, cid: &str) -> Result<()> {
        let cid = extract_cid(cid)?;
        let url = format!("{}/api/v0/pin/rm?arg={}", self.ipfs_endpoint, cid);
        
        debug!("Unpinning CID: {}", cid);
        
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| AiError::StorageError(format!("IPFS unpin failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AiError::StorageError(format!(
                "IPFS unpin failed with status {}: {}", status, body
            )));
        }
        
        debug!("Unpinned: {}", cid);
        Ok(())
    }

    /// Check if content is pinned
    pub async fn is_pinned(&self, cid: &str) -> Result<bool> {
        let cid = extract_cid(cid)?;
        let url = format!("{}/api/v0/pin/ls?arg={}&type=all", self.ipfs_endpoint, cid);
        
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| AiError::StorageError(format!("IPFS pin check failed: {}", e)))?;
        
        Ok(response.status().is_success())
    }

    /// Get IPFS node stats
    pub async fn stats(&self) -> Result<IpfsStats> {
        let url = format!("{}/api/v0/stats/repo", self.ipfs_endpoint);
        
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| AiError::StorageError(format!("IPFS stats failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AiError::StorageError("Failed to get IPFS stats".to_string()));
        }
        
        response
            .json()
            .await
            .map_err(|e| AiError::StorageError(format!("Failed to parse stats: {}", e)))
    }

    /// Verify content hash matches expected
    pub fn verify_hash(&self, data: &[u8], expected_hash: &str) -> bool {
        let computed = content_hash(data);
        computed == expected_hash || computed == extract_cid(expected_hash).unwrap_or_default()
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new("http://127.0.0.1:5001")
    }
}

/// IPFS repository statistics
#[derive(Debug, Deserialize, Serialize)]
pub struct IpfsStats {
    #[serde(rename = "RepoSize")]
    pub repo_size: u64,
    #[serde(rename = "StorageMax")]
    pub storage_max: u64,
    #[serde(rename = "NumObjects")]
    pub num_objects: u64,
}

/// Extract CID from various URI formats
fn extract_cid(uri: &str) -> Result<String> {
    let cid = uri
        .strip_prefix("ipfs://")
        .or_else(|| uri.strip_prefix("/ipfs/"))
        .unwrap_or(uri);
    
    // Validate CID format (basic check)
    if cid.is_empty() {
        return Err(AiError::StorageError("Empty CID".to_string()));
    }
    
    // CIDv0 starts with Qm, CIDv1 starts with b or other base prefixes
    if !cid.starts_with("Qm") && !cid.starts_with("b") && !cid.starts_with("z") {
        warn!("Potentially invalid CID format: {}", cid);
    }
    
    Ok(cid.to_string())
}

/// Compute content hash using SHA-256
pub fn content_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cid() {
        assert_eq!(extract_cid("ipfs://QmTest").unwrap(), "QmTest");
        assert_eq!(extract_cid("/ipfs/QmTest").unwrap(), "QmTest");
        assert_eq!(extract_cid("QmTest").unwrap(), "QmTest");
    }

    #[test]
    fn test_content_hash() {
        let data = b"test data";
        let hash = content_hash(data);
        assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
    }
}
