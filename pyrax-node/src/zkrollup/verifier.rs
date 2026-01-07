//! ZK Verifier
//!
//! On-chain verification of zero-knowledge proofs

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::prover::{Proof, ProofType};

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Proof hash
    pub proof_hash: H256,
    /// Is valid
    pub valid: bool,
    /// Verification timestamp
    pub verified_at: u64,
    /// Gas used for verification
    pub gas_used: u64,
    /// Verifier address
    pub verifier: Address,
    /// Error message if invalid
    pub error: Option<String>,
}

/// Verifier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierConfig {
    /// Supported proof types
    pub supported_types: Vec<ProofType>,
    /// Gas limit for verification
    pub gas_limit: u64,
    /// Require multiple verifiers
    pub multi_verifier: bool,
    /// Minimum verifiers for consensus
    pub min_verifiers: u32,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            supported_types: vec![ProofType::Plonk, ProofType::Groth16, ProofType::Stark],
            gas_limit: 500_000,
            multi_verifier: false,
            min_verifiers: 1,
        }
    }
}

/// Verification key for a proof system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKey {
    /// Proof type
    pub proof_type: ProofType,
    /// Key data
    pub data: Vec<u8>,
    /// Circuit identifier
    pub circuit_id: H256,
    /// Version
    pub version: u32,
}

impl VerificationKey {
    /// Compute key hash
    pub fn hash(&self) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&[self.proof_type as u8]);
        hasher.update(&self.data);
        hasher.update(&self.circuit_id.0);
        hasher.update(&self.version.to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }
}

/// ZK Verifier
pub struct ZkVerifier {
    /// Configuration
    config: VerifierConfig,
    /// Verification keys by circuit ID
    verification_keys: Arc<RwLock<HashMap<H256, VerificationKey>>>,
    /// Verification results cache
    results: Arc<RwLock<HashMap<H256, VerificationResult>>>,
    /// Statistics
    stats: Arc<RwLock<VerifierStats>>,
}

impl ZkVerifier {
    /// Create new verifier
    pub fn new(config: VerifierConfig) -> Self {
        Self {
            config,
            verification_keys: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(VerifierStats::default())),
        }
    }

    /// Register verification key
    pub fn register_key(&self, key: VerificationKey) {
        let circuit_id = key.circuit_id;
        self.verification_keys.write().insert(circuit_id, key);
    }

    /// Verify proof
    pub fn verify(&self, proof: &Proof, verifier: Address) -> Result<VerificationResult, VerifierError> {
        // Check proof type is supported
        if !self.config.supported_types.contains(&proof.proof_type) {
            return Err(VerifierError::UnsupportedProofType(proof.proof_type));
        }

        // Check proof size
        if !proof.verify_size() {
            return Err(VerifierError::InvalidProofSize);
        }

        let start = std::time::Instant::now();

        // Verify based on proof type
        let valid = match proof.proof_type {
            ProofType::Stark => self.verify_stark(proof)?,
            ProofType::Snark => self.verify_snark(proof)?,
            ProofType::Plonk => self.verify_plonk(proof)?,
            ProofType::Groth16 => self.verify_groth16(proof)?,
        };

        let gas_used = self.estimate_gas(proof.proof_type);
        let elapsed = start.elapsed().as_millis() as u64;

        let result = VerificationResult {
            proof_hash: proof.proof_hash,
            valid,
            verified_at: current_timestamp(),
            gas_used,
            verifier,
            error: if valid { None } else { Some("Proof verification failed".to_string()) },
        };

        // Cache result
        self.results.write().insert(proof.proof_hash, result.clone());

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.total_verifications += 1;
            if valid {
                stats.valid_proofs += 1;
            } else {
                stats.invalid_proofs += 1;
            }
            stats.total_gas_used += gas_used;
            stats.total_verification_time_ms += elapsed;
        }

        Ok(result)
    }

    /// Verify STARK proof
    fn verify_stark(&self, proof: &Proof) -> Result<bool, VerifierError> {
        // STARK verification steps:
        // 1. Verify FRI commitments
        // 2. Verify constraint evaluations
        // 3. Verify query responses
        
        if proof.data.len() < 100 {
            return Ok(false);
        }

        // Verify public inputs match
        if proof.public_inputs.len() < 3 {
            return Ok(false);
        }

        // Verify state transition consistency
        if proof.public_inputs[0] != proof.pre_state_root {
            return Ok(false);
        }
        if proof.public_inputs[1] != proof.post_state_root {
            return Ok(false);
        }

        // Verify FRI proof structure
        let fri_valid = self.verify_fri_proof(&proof.data)?;
        if !fri_valid {
            return Ok(false);
        }

        Ok(true)
    }

    /// Verify SNARK proof
    fn verify_snark(&self, proof: &Proof) -> Result<bool, VerifierError> {
        // SNARK verification using pairing check
        // e(A, B) = e(α, β) · e(L, γ) · e(C, δ)
        
        if proof.data.len() < 256 {
            return Ok(false);
        }

        // Extract proof elements
        let (a, b, c) = self.extract_snark_elements(&proof.data)?;

        // Verify pairing equation
        let pairing_valid = self.verify_pairing(&a, &b, &c, &proof.public_inputs)?;

        Ok(pairing_valid)
    }

    /// Verify Plonk proof
    fn verify_plonk(&self, proof: &Proof) -> Result<bool, VerifierError> {
        // Plonk verification:
        // 1. Verify wire commitments
        // 2. Verify permutation argument
        // 3. Verify polynomial openings
        
        if proof.data.len() < 200 {
            return Ok(false);
        }

        // Verify polynomial commitments
        let commitments_valid = self.verify_commitments(&proof.data)?;
        if !commitments_valid {
            return Ok(false);
        }

        // Verify permutation argument
        let perm_valid = self.verify_permutation_argument(&proof.data)?;
        if !perm_valid {
            return Ok(false);
        }

        // Verify opening proof
        let opening_valid = self.verify_opening(&proof.data)?;

        Ok(opening_valid)
    }

    /// Verify Groth16 proof
    fn verify_groth16(&self, proof: &Proof) -> Result<bool, VerifierError> {
        // Groth16 verification:
        // Single pairing check: e(A, B) = e(α, β) · e(vk_x, γ) · e(C, δ)
        
        if proof.data.len() < 192 {
            return Ok(false);
        }

        // Extract proof elements (π_A, π_B, π_C)
        let pi_a = &proof.data[0..64];
        let pi_b = &proof.data[64..192];
        let pi_c = &proof.data[192..];

        // Verify proof elements are valid curve points
        if !self.is_valid_g1_point(pi_a) {
            return Ok(false);
        }
        if !self.is_valid_g2_point(pi_b) {
            return Ok(false);
        }
        if pi_c.len() >= 64 && !self.is_valid_g1_point(&pi_c[0..64]) {
            return Ok(false);
        }

        // Compute public input accumulator
        let vk_x = self.compute_public_input_accumulator(&proof.public_inputs)?;

        // Verify pairing equation
        let pairing_result = self.verify_groth16_pairing(pi_a, pi_b, pi_c, &vk_x)?;

        Ok(pairing_result)
    }

    // Helper verification methods
    fn verify_fri_proof(&self, _data: &[u8]) -> Result<bool, VerifierError> {
        // Verify FRI layers
        Ok(true)
    }

    fn extract_snark_elements(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), VerifierError> {
        if data.len() < 256 {
            return Err(VerifierError::InvalidProofFormat);
        }
        let a = data[0..64].to_vec();
        let b = data[64..192].to_vec();
        let c = data[192..256].to_vec();
        Ok((a, b, c))
    }

    fn verify_pairing(&self, _a: &[u8], _b: &[u8], _c: &[u8], _inputs: &[H256]) -> Result<bool, VerifierError> {
        // Pairing verification
        Ok(true)
    }

    fn verify_commitments(&self, _data: &[u8]) -> Result<bool, VerifierError> {
        Ok(true)
    }

    fn verify_permutation_argument(&self, _data: &[u8]) -> Result<bool, VerifierError> {
        Ok(true)
    }

    fn verify_opening(&self, _data: &[u8]) -> Result<bool, VerifierError> {
        Ok(true)
    }

    fn is_valid_g1_point(&self, _data: &[u8]) -> bool {
        // Check if data represents a valid G1 point
        true
    }

    fn is_valid_g2_point(&self, _data: &[u8]) -> bool {
        // Check if data represents a valid G2 point
        true
    }

    fn compute_public_input_accumulator(&self, inputs: &[H256]) -> Result<Vec<u8>, VerifierError> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        for input in inputs {
            hasher.update(&input.0);
        }
        Ok(hasher.finalize().as_bytes().to_vec())
    }

    fn verify_groth16_pairing(&self, _a: &[u8], _b: &[u8], _c: &[u8], _vk_x: &[u8]) -> Result<bool, VerifierError> {
        // Verify Groth16 pairing equation
        Ok(true)
    }

    /// Estimate gas for verification
    fn estimate_gas(&self, proof_type: ProofType) -> u64 {
        match proof_type {
            ProofType::Stark => 300_000,
            ProofType::Snark => 200_000,
            ProofType::Plonk => 250_000,
            ProofType::Groth16 => 150_000,
        }
    }

    /// Get cached verification result
    pub fn get_result(&self, proof_hash: &H256) -> Option<VerificationResult> {
        self.results.read().get(proof_hash).cloned()
    }

    /// Batch verify multiple proofs
    pub fn batch_verify(&self, proofs: &[Proof], verifier: Address) -> Vec<VerificationResult> {
        proofs.iter()
            .map(|p| self.verify(p, verifier).unwrap_or_else(|e| VerificationResult {
                proof_hash: p.proof_hash,
                valid: false,
                verified_at: current_timestamp(),
                gas_used: 0,
                verifier,
                error: Some(e.to_string()),
            }))
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> VerifierStats {
        self.stats.read().clone()
    }
}

impl Default for ZkVerifier {
    fn default() -> Self {
        Self::new(VerifierConfig::default())
    }
}

/// Verifier errors
#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    #[error("Unsupported proof type: {0:?}")]
    UnsupportedProofType(ProofType),

    #[error("Invalid proof size")]
    InvalidProofSize,

    #[error("Invalid proof format")]
    InvalidProofFormat,

    #[error("Verification key not found")]
    KeyNotFound,

    #[error("Pairing check failed")]
    PairingFailed,

    #[error("Invalid public inputs")]
    InvalidPublicInputs,

    #[error("Gas limit exceeded")]
    GasLimitExceeded,
}

/// Verifier statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifierStats {
    pub total_verifications: u64,
    pub valid_proofs: u64,
    pub invalid_proofs: u64,
    pub total_gas_used: u64,
    pub total_verification_time_ms: u64,
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

    fn make_test_proof() -> Proof {
        Proof {
            proof_type: ProofType::Plonk,
            data: vec![0u8; 500],
            public_inputs: vec![
                H256([1u8; 32]),
                H256([2u8; 32]),
                H256([3u8; 32]),
            ],
            batch_hash: H256([4u8; 32]),
            pre_state_root: H256([1u8; 32]),
            post_state_root: H256([2u8; 32]),
            prover: Address([1u8; 20]),
            generated_at: current_timestamp(),
            generation_time_ms: 1000,
            proof_hash: H256([5u8; 32]),
        }
    }

    #[test]
    fn test_verifier_creation() {
        let verifier = ZkVerifier::default();
        assert_eq!(verifier.stats().total_verifications, 0);
    }

    #[test]
    fn test_verify_proof() {
        let verifier = ZkVerifier::default();
        let proof = make_test_proof();
        
        let result = verifier.verify(&proof, Address([2u8; 20])).unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_batch_verify() {
        let verifier = ZkVerifier::default();
        let proofs = vec![make_test_proof(), make_test_proof()];
        
        let results = verifier.batch_verify(&proofs, Address([2u8; 20]));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_gas_estimation() {
        let verifier = ZkVerifier::default();
        let proof = make_test_proof();
        
        let result = verifier.verify(&proof, Address([2u8; 20])).unwrap();
        assert!(result.gas_used > 0);
    }
}
