//! Data Availability Layer
//!
//! Ensures transaction data is available for verification

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::batch::Batch;

/// Data availability provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DaProvider {
    /// Data posted to L1 (calldata)
    L1Calldata,
    /// Data posted to L1 (blobs via EIP-4844)
    L1Blobs,
    /// Celestia DA layer
    Celestia,
    /// EigenDA
    EigenDA,
    /// Avail
    Avail,
    /// Custom DA solution
    Custom,
}

impl DaProvider {
    /// Get cost per byte (in wei equivalent)
    pub fn cost_per_byte(&self) -> u64 {
        match self {
            DaProvider::L1Calldata => 16,      // ~16 gas per byte
            DaProvider::L1Blobs => 1,          // ~1 gas per byte (much cheaper)
            DaProvider::Celestia => 1,
            DaProvider::EigenDA => 1,
            DaProvider::Avail => 1,
            DaProvider::Custom => 1,
        }
    }

    /// Get finality time (seconds)
    pub fn finality_time(&self) -> u64 {
        match self {
            DaProvider::L1Calldata => 900,    // ~15 minutes (safe L1 finality)
            DaProvider::L1Blobs => 900,
            DaProvider::Celestia => 12,       // ~12 seconds
            DaProvider::EigenDA => 12,
            DaProvider::Avail => 20,
            DaProvider::Custom => 60,
        }
    }
}

/// DA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaConfig {
    /// Primary DA provider
    pub provider: DaProvider,
    /// Fallback provider
    pub fallback: Option<DaProvider>,
    /// Maximum data size per submission (bytes)
    pub max_data_size: usize,
    /// Blob size (for EIP-4844)
    pub blob_size: usize,
    /// Retention period (seconds)
    pub retention_period: u64,
    /// Enable compression
    pub compression: bool,
    /// Commitment scheme
    pub commitment_scheme: CommitmentScheme,
}

impl Default for DaConfig {
    fn default() -> Self {
        Self {
            provider: DaProvider::L1Calldata,
            fallback: Some(DaProvider::Celestia),
            max_data_size: 128 * 1024, // 128 KB
            blob_size: 128 * 1024,     // 128 KB per blob
            retention_period: 30 * 24 * 60 * 60, // 30 days
            compression: true,
            commitment_scheme: CommitmentScheme::KZG,
        }
    }
}

/// Commitment scheme for data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitmentScheme {
    /// KZG polynomial commitment
    KZG,
    /// Merkle tree
    Merkle,
    /// Reed-Solomon erasure coding
    ReedSolomon,
}

/// Data availability commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaCommitment {
    /// Commitment hash
    pub commitment: H256,
    /// Data root
    pub data_root: H256,
    /// Provider used
    pub provider: DaProvider,
    /// Data size (bytes)
    pub data_size: usize,
    /// Blob index (for L1 blobs)
    pub blob_index: Option<u64>,
    /// External reference (e.g., Celestia height)
    pub external_ref: Option<String>,
    /// Timestamp
    pub timestamp: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Proof of availability
    pub availability_proof: Option<Vec<u8>>,
}

impl DaCommitment {
    /// Create new commitment
    pub fn new(
        data: &[u8],
        provider: DaProvider,
        config: &DaConfig,
    ) -> Self {
        let commitment = Self::compute_commitment(data, config.commitment_scheme);
        let data_root = Self::compute_data_root(data);
        let now = current_timestamp();

        Self {
            commitment,
            data_root,
            provider,
            data_size: data.len(),
            blob_index: None,
            external_ref: None,
            timestamp: now,
            expires_at: now + config.retention_period,
            availability_proof: None,
        }
    }

    /// Compute commitment based on scheme
    fn compute_commitment(data: &[u8], scheme: CommitmentScheme) -> H256 {
        match scheme {
            CommitmentScheme::KZG => Self::kzg_commitment(data),
            CommitmentScheme::Merkle => Self::merkle_commitment(data),
            CommitmentScheme::ReedSolomon => Self::rs_commitment(data),
        }
    }

    /// KZG polynomial commitment
    fn kzg_commitment(data: &[u8]) -> H256 {
        // Simplified KZG commitment (production would use actual BLS12-381)
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(b"KZG_COMMITMENT");
        hasher.update(data);
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Merkle tree commitment
    fn merkle_commitment(data: &[u8]) -> H256 {
        // Split data into chunks and build merkle tree
        let chunk_size = 256;
        let chunks: Vec<_> = data.chunks(chunk_size).collect();
        
        if chunks.is_empty() {
            return H256::zero();
        }

        let mut hashes: Vec<H256> = chunks.iter()
            .map(|chunk| {
                use blake3::Hasher;
                let mut hasher = Hasher::new();
                hasher.update(chunk);
                H256::from_slice(hasher.finalize().as_bytes())
            })
            .collect();

        // Build tree
        while hashes.len() > 1 {
            let mut next = Vec::new();
            for pair in hashes.chunks(2) {
                let left = pair[0];
                let right = pair.get(1).copied().unwrap_or(H256::zero());
                
                use blake3::Hasher;
                let mut hasher = Hasher::new();
                hasher.update(&left.0);
                hasher.update(&right.0);
                next.push(H256::from_slice(hasher.finalize().as_bytes()));
            }
            hashes = next;
        }

        hashes[0]
    }

    /// Reed-Solomon commitment
    fn rs_commitment(data: &[u8]) -> H256 {
        // Simplified RS commitment
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(b"RS_COMMITMENT");
        hasher.update(data);
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Compute data root
    fn compute_data_root(data: &[u8]) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(data);
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Set blob index
    pub fn with_blob_index(mut self, index: u64) -> Self {
        self.blob_index = Some(index);
        self
    }

    /// Set external reference
    pub fn with_external_ref(mut self, ref_: String) -> Self {
        self.external_ref = Some(ref_);
        self
    }

    /// Set availability proof
    pub fn with_proof(mut self, proof: Vec<u8>) -> Self {
        self.availability_proof = Some(proof);
        self
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// Verify commitment against data
    pub fn verify(&self, data: &[u8], scheme: CommitmentScheme) -> bool {
        let computed = Self::compute_commitment(data, scheme);
        computed == self.commitment
    }
}

/// Data submission result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaSubmission {
    /// Submission ID
    pub id: H256,
    /// Batch hash
    pub batch_hash: H256,
    /// Commitment
    pub commitment: DaCommitment,
    /// L1 transaction hash (if applicable)
    pub l1_tx_hash: Option<H256>,
    /// Submission cost
    pub cost: u64,
    /// Submitted at
    pub submitted_at: u64,
    /// Confirmed at
    pub confirmed_at: Option<u64>,
    /// Is confirmed
    pub confirmed: bool,
}

/// Data Availability Manager
pub struct DataAvailability {
    /// Configuration
    config: DaConfig,
    /// Pending submissions
    pending: Arc<RwLock<HashMap<H256, DaSubmission>>>,
    /// Confirmed submissions
    confirmed: Arc<RwLock<HashMap<H256, DaSubmission>>>,
    /// Commitments by batch
    by_batch: Arc<RwLock<HashMap<H256, H256>>>,
    /// Data cache
    data_cache: Arc<RwLock<HashMap<H256, Vec<u8>>>>,
    /// Statistics
    stats: Arc<RwLock<DaStats>>,
}

impl DataAvailability {
    /// Create new DA manager
    pub fn new(config: DaConfig) -> Self {
        Self {
            config,
            pending: Arc::new(RwLock::new(HashMap::new())),
            confirmed: Arc::new(RwLock::new(HashMap::new())),
            by_batch: Arc::new(RwLock::new(HashMap::new())),
            data_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(DaStats::default())),
        }
    }

    /// Submit batch data
    pub fn submit_batch(&self, batch: &Batch) -> Result<DaSubmission, DaError> {
        // Serialize batch data
        let data = self.serialize_batch(batch)?;

        // Check size limit
        if data.len() > self.config.max_data_size {
            return Err(DaError::DataTooLarge {
                size: data.len(),
                max: self.config.max_data_size,
            });
        }

        // Compress if enabled
        let final_data = if self.config.compression {
            self.compress(&data)?
        } else {
            data.clone()
        };

        // Create commitment
        let commitment = DaCommitment::new(&final_data, self.config.provider, &self.config);

        // Calculate cost
        let cost = self.calculate_cost(final_data.len());

        // Create submission
        let submission = DaSubmission {
            id: commitment.commitment,
            batch_hash: batch.hash(),
            commitment: commitment.clone(),
            l1_tx_hash: None,
            cost,
            submitted_at: current_timestamp(),
            confirmed_at: None,
            confirmed: false,
        };

        // Store
        self.pending.write().insert(submission.id, submission.clone());
        self.by_batch.write().insert(batch.hash(), submission.id);
        self.data_cache.write().insert(commitment.data_root, final_data);

        self.stats.write().submissions += 1;
        self.stats.write().total_data_size += data.len() as u64;

        Ok(submission)
    }

    /// Serialize batch for DA
    fn serialize_batch(&self, batch: &Batch) -> Result<Vec<u8>, DaError> {
        // Compact encoding of batch
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(&batch.header.batch_number.to_le_bytes());
        data.extend_from_slice(&batch.header.pre_state_root.0);
        data.extend_from_slice(&batch.header.post_state_root.0);
        data.extend_from_slice(&batch.header.tx_root.0);
        data.extend_from_slice(&batch.header.tx_count.to_le_bytes());

        // Transactions
        for tx in &batch.transactions {
            data.extend_from_slice(&tx.from.0);
            if let Some(to) = &tx.to {
                data.push(1);
                data.extend_from_slice(&to.0);
            } else {
                data.push(0);
            }
            data.extend_from_slice(&tx.value.to_le_bytes());
            data.extend_from_slice(&tx.gas_limit.to_le_bytes());
            data.extend_from_slice(&tx.nonce.to_le_bytes());
            data.extend_from_slice(&(tx.data.len() as u32).to_le_bytes());
            data.extend_from_slice(&tx.data);
        }

        Ok(data)
    }

    /// Compress data
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, DaError> {
        // Simple RLE-like compression for repeated bytes
        // Production would use zstd or similar
        let mut compressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let byte = data[i];
            let mut count = 1u8;

            while (i + count as usize) < data.len() 
                && data[i + count as usize] == byte 
                && count < 255 
            {
                count += 1;
            }

            if count > 3 {
                // RLE encode
                compressed.push(0xFF); // Escape byte
                compressed.push(count);
                compressed.push(byte);
                i += count as usize;
            } else {
                // Literal
                if byte == 0xFF {
                    compressed.push(0xFF);
                    compressed.push(1);
                    compressed.push(0xFF);
                } else {
                    compressed.push(byte);
                }
                i += 1;
            }
        }

        Ok(compressed)
    }

    /// Decompress data
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, DaError> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] == 0xFF && i + 2 < data.len() {
                let count = data[i + 1] as usize;
                let byte = data[i + 2];
                for _ in 0..count {
                    decompressed.push(byte);
                }
                i += 3;
            } else {
                decompressed.push(data[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }

    /// Calculate submission cost
    fn calculate_cost(&self, size: usize) -> u64 {
        let cost_per_byte = self.config.provider.cost_per_byte();
        size as u64 * cost_per_byte
    }

    /// Confirm submission
    pub fn confirm_submission(
        &self,
        submission_id: &H256,
        l1_tx_hash: Option<H256>,
    ) -> Result<(), DaError> {
        let mut submission = self.pending.write()
            .remove(submission_id)
            .ok_or_else(|| DaError::SubmissionNotFound(*submission_id))?;

        submission.l1_tx_hash = l1_tx_hash;
        submission.confirmed_at = Some(current_timestamp());
        submission.confirmed = true;

        self.confirmed.write().insert(*submission_id, submission);
        self.stats.write().confirmations += 1;

        Ok(())
    }

    /// Get data by commitment
    pub fn get_data(&self, data_root: &H256) -> Option<Vec<u8>> {
        let data = self.data_cache.read().get(data_root).cloned()?;
        
        if self.config.compression {
            self.decompress(&data).ok()
        } else {
            Some(data)
        }
    }

    /// Verify data availability
    pub fn verify_availability(&self, commitment: &DaCommitment) -> Result<bool, DaError> {
        // Check if we have the data
        if let Some(data) = self.data_cache.read().get(&commitment.data_root) {
            return Ok(commitment.verify(data, self.config.commitment_scheme));
        }

        // Check if expired
        if commitment.is_expired() {
            return Err(DaError::DataExpired);
        }

        // Would need to fetch from DA layer
        Ok(false)
    }

    /// Generate availability proof
    pub fn generate_proof(&self, commitment: &DaCommitment) -> Result<Vec<u8>, DaError> {
        let data = self.data_cache.read()
            .get(&commitment.data_root)
            .cloned()
            .ok_or(DaError::DataNotFound)?;

        // Generate proof based on commitment scheme
        match self.config.commitment_scheme {
            CommitmentScheme::KZG => self.generate_kzg_proof(&data),
            CommitmentScheme::Merkle => self.generate_merkle_proof(&data),
            CommitmentScheme::ReedSolomon => self.generate_rs_proof(&data),
        }
    }

    /// Generate KZG proof
    fn generate_kzg_proof(&self, data: &[u8]) -> Result<Vec<u8>, DaError> {
        // Simplified KZG proof
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(b"KZG_PROOF");
        hasher.update(data);
        Ok(hasher.finalize().as_bytes().to_vec())
    }

    /// Generate Merkle proof
    fn generate_merkle_proof(&self, data: &[u8]) -> Result<Vec<u8>, DaError> {
        // Generate merkle proof for random samples
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(b"MERKLE_PROOF");
        hasher.update(data);
        Ok(hasher.finalize().as_bytes().to_vec())
    }

    /// Generate Reed-Solomon proof
    fn generate_rs_proof(&self, data: &[u8]) -> Result<Vec<u8>, DaError> {
        // Generate RS proof
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(b"RS_PROOF");
        hasher.update(data);
        Ok(hasher.finalize().as_bytes().to_vec())
    }

    /// Get submission by batch
    pub fn get_by_batch(&self, batch_hash: &H256) -> Option<DaSubmission> {
        let submission_id = self.by_batch.read().get(batch_hash).copied()?;
        self.confirmed.read().get(&submission_id).cloned()
            .or_else(|| self.pending.read().get(&submission_id).cloned())
    }

    /// Clean up expired data
    pub fn cleanup_expired(&self) {
        let now = current_timestamp();
        
        // Remove expired from cache
        self.data_cache.write().retain(|_, _| true); // Would check expiry

        // Update stats
        self.stats.write().cleanups += 1;
    }

    /// Get statistics
    pub fn stats(&self) -> DaStats {
        self.stats.read().clone()
    }
}

impl Default for DataAvailability {
    fn default() -> Self {
        Self::new(DaConfig::default())
    }
}

/// DA errors
#[derive(Debug, thiserror::Error)]
pub enum DaError {
    #[error("Data too large: {size} bytes, max {max}")]
    DataTooLarge { size: usize, max: usize },

    #[error("Submission not found: {0:?}")]
    SubmissionNotFound(H256),

    #[error("Data not found")]
    DataNotFound,

    #[error("Data expired")]
    DataExpired,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Provider error: {0}")]
    ProviderError(String),
}

/// DA statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DaStats {
    pub submissions: u64,
    pub confirmations: u64,
    pub total_data_size: u64,
    pub total_cost: u64,
    pub cleanups: u64,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::batch::{Batch, L2Transaction};

    fn make_test_batch() -> Batch {
        let mut batch = Batch::new(
            1,
            H256::zero(),
            H256([1u8; 32]),
            Address([1u8; 20]),
            100,
        );
        
        let tx = L2Transaction::transfer(
            Address([1u8; 20]),
            Address([2u8; 20]),
            1000,
            0,
        );
        batch.add_transaction(tx).unwrap();
        batch.seal(H256([2u8; 32])).unwrap();
        batch
    }

    #[test]
    fn test_da_config() {
        let config = DaConfig::default();
        assert_eq!(config.provider, DaProvider::L1Calldata);
        assert!(config.compression);
    }

    #[test]
    fn test_da_commitment() {
        let data = vec![1, 2, 3, 4, 5];
        let config = DaConfig::default();
        let commitment = DaCommitment::new(&data, DaProvider::L1Calldata, &config);

        assert!(commitment.verify(&data, CommitmentScheme::KZG));
    }

    #[test]
    fn test_submit_batch() {
        let da = DataAvailability::default();
        let batch = make_test_batch();

        let submission = da.submit_batch(&batch).unwrap();
        assert!(!submission.confirmed);
        assert!(submission.cost > 0);
    }

    #[test]
    fn test_confirm_submission() {
        let da = DataAvailability::default();
        let batch = make_test_batch();

        let submission = da.submit_batch(&batch).unwrap();
        da.confirm_submission(&submission.id, Some(H256([5u8; 32]))).unwrap();

        let confirmed = da.get_by_batch(&batch.hash()).unwrap();
        assert!(confirmed.confirmed);
    }

    #[test]
    fn test_compression() {
        let da = DataAvailability::default();
        
        // Data with repeated bytes
        let data = vec![0xAB; 100];
        let compressed = da.compress(&data).unwrap();
        let decompressed = da.decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len());
    }
}
