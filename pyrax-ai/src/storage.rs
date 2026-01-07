use crate::{Result, AiError};
use sha2::{Sha256, Digest};

pub struct Storage {
    ipfs_endpoint: String,
}

impl Storage {
    pub fn new(ipfs_endpoint: &str) -> Self {
        Self {
            ipfs_endpoint: ipfs_endpoint.to_string(),
        }
    }

    pub async fn upload(&self, data: &[u8]) -> Result<String> {
        // TODO: Implement actual IPFS upload
        // For now, return content hash
        let hash = content_hash(data);
        Ok(format!("ipfs://{}", hash))
    }

    pub async fn download(&self, uri: &str) -> Result<Vec<u8>> {
        // TODO: Implement actual IPFS download
        Err(AiError::StorageError("IPFS download not implemented".to_string()))
    }

    pub async fn pin(&self, cid: &str) -> Result<()> {
        // TODO: Implement IPFS pinning
        Ok(())
    }

    pub async fn unpin(&self, cid: &str) -> Result<()> {
        // TODO: Implement IPFS unpinning
        Ok(())
    }

    pub fn verify_hash(&self, data: &[u8], expected_hash: &str) -> bool {
        let computed = content_hash(data);
        computed == expected_hash
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new("http://127.0.0.1:5001")
    }
}

pub fn content_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
