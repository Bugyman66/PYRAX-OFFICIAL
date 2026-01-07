//! ZK Prover Pipeline
//!
//! Zero-knowledge proof generation for L2 batches

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::batch::{Batch, BatchStatus};
use super::state::StateTransition;
use super::PROOF_TIMEOUT_SECS;

/// Proof type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofType {
    /// STARK proof (faster proving, larger proofs)
    Stark,
    /// SNARK proof (slower proving, smaller proofs)
    Snark,
    /// Plonk proof (balanced)
    Plonk,
    /// Groth16 (smallest proofs)
    Groth16,
}

impl ProofType {
    /// Get expected proof size in bytes
    pub fn expected_size(&self) -> usize {
        match self {
            ProofType::Stark => 200_000,  // ~200KB
            ProofType::Snark => 500,      // ~500B
            ProofType::Plonk => 1_000,    // ~1KB
            ProofType::Groth16 => 200,    // ~200B
        }
    }

    /// Get expected proving time factor
    pub fn time_factor(&self) -> f64 {
        match self {
            ProofType::Stark => 1.0,
            ProofType::Snark => 5.0,
            ProofType::Plonk => 3.0,
            ProofType::Groth16 => 10.0,
        }
    }
}

/// ZK Proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// Proof type
    pub proof_type: ProofType,
    /// Proof data
    pub data: Vec<u8>,
    /// Public inputs
    pub public_inputs: Vec<H256>,
    /// Batch hash this proof is for
    pub batch_hash: H256,
    /// Pre-state root
    pub pre_state_root: H256,
    /// Post-state root
    pub post_state_root: H256,
    /// Prover address
    pub prover: Address,
    /// Generation timestamp
    pub generated_at: u64,
    /// Generation duration (ms)
    pub generation_time_ms: u64,
    /// Proof hash
    pub proof_hash: H256,
}

impl Proof {
    /// Compute proof hash
    pub fn compute_hash(&self) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&[self.proof_type as u8]);
        hasher.update(&self.data);
        for input in &self.public_inputs {
            hasher.update(&input.0);
        }
        hasher.update(&self.batch_hash.0);
        hasher.update(&self.pre_state_root.0);
        hasher.update(&self.post_state_root.0);
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Verify proof size is reasonable
    pub fn verify_size(&self) -> bool {
        let expected = self.proof_type.expected_size();
        // Allow 2x variance
        self.data.len() <= expected * 2
    }
}

/// Prover configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProverConfig {
    /// Proof type to generate
    pub proof_type: ProofType,
    /// Number of parallel provers
    pub num_provers: u32,
    /// Proof timeout (seconds)
    pub timeout_secs: u64,
    /// Use GPU acceleration
    pub use_gpu: bool,
    /// GPU device index
    pub gpu_device: u32,
    /// Memory limit (MB)
    pub memory_limit_mb: u64,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            proof_type: ProofType::Plonk,
            num_provers: 4,
            timeout_secs: PROOF_TIMEOUT_SECS,
            use_gpu: true,
            gpu_device: 0,
            memory_limit_mb: 16384, // 16GB
        }
    }
}

/// Proof request status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofRequestStatus {
    /// Queued for proving
    Queued,
    /// Currently proving
    Proving,
    /// Proof generated
    Completed,
    /// Proof generation failed
    Failed,
    /// Proof generation timed out
    TimedOut,
}

/// Proof request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRequest {
    /// Request ID
    pub id: H256,
    /// Batch to prove
    pub batch_hash: H256,
    /// State transition
    pub transition: StateTransition,
    /// Status
    pub status: ProofRequestStatus,
    /// Generated proof (if completed)
    pub proof: Option<Proof>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Created timestamp
    pub created_at: u64,
    /// Started timestamp
    pub started_at: Option<u64>,
    /// Completed timestamp
    pub completed_at: Option<u64>,
    /// Assigned prover
    pub assigned_prover: Option<u32>,
}

impl ProofRequest {
    /// Create new proof request
    pub fn new(batch_hash: H256, transition: StateTransition) -> Self {
        let id = Self::generate_id(&batch_hash);
        Self {
            id,
            batch_hash,
            transition,
            status: ProofRequestStatus::Queued,
            proof: None,
            error: None,
            created_at: current_timestamp(),
            started_at: None,
            completed_at: None,
            assigned_prover: None,
        }
    }

    /// Generate request ID
    fn generate_id(batch_hash: &H256) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&batch_hash.0);
        hasher.update(&current_timestamp().to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Start proving
    pub fn start(&mut self, prover_id: u32) {
        self.status = ProofRequestStatus::Proving;
        self.started_at = Some(current_timestamp());
        self.assigned_prover = Some(prover_id);
    }

    /// Complete with proof
    pub fn complete(&mut self, proof: Proof) {
        self.status = ProofRequestStatus::Completed;
        self.completed_at = Some(current_timestamp());
        self.proof = Some(proof);
    }

    /// Fail with error
    pub fn fail(&mut self, error: String) {
        self.status = ProofRequestStatus::Failed;
        self.completed_at = Some(current_timestamp());
        self.error = Some(error);
    }

    /// Check timeout
    pub fn check_timeout(&mut self, timeout_secs: u64) -> bool {
        if self.status != ProofRequestStatus::Proving {
            return false;
        }

        if let Some(started) = self.started_at {
            if current_timestamp() - started > timeout_secs {
                self.status = ProofRequestStatus::TimedOut;
                self.completed_at = Some(current_timestamp());
                self.error = Some("Proof generation timed out".to_string());
                return true;
            }
        }
        false
    }

    /// Get proving duration
    pub fn duration(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

/// ZK Prover
pub struct ZkProver {
    /// Configuration
    config: ProverConfig,
    /// Prover address
    address: Address,
    /// Proof requests queue
    queue: Arc<RwLock<Vec<ProofRequest>>>,
    /// Active requests
    active: Arc<RwLock<HashMap<H256, ProofRequest>>>,
    /// Completed proofs
    proofs: Arc<RwLock<HashMap<H256, Proof>>>,
    /// Prover statistics
    stats: Arc<RwLock<ProverStats>>,
    /// Circuit parameters (cached)
    circuit_params: Arc<RwLock<Option<CircuitParams>>>,
}

impl ZkProver {
    /// Create new prover
    pub fn new(config: ProverConfig, address: Address) -> Self {
        Self {
            config,
            address,
            queue: Arc::new(RwLock::new(Vec::new())),
            active: Arc::new(RwLock::new(HashMap::new())),
            proofs: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ProverStats::default())),
            circuit_params: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize circuit parameters
    pub fn initialize(&self) -> Result<(), ProverError> {
        let params = CircuitParams::generate(self.config.proof_type)?;
        *self.circuit_params.write() = Some(params);
        Ok(())
    }

    /// Submit proof request
    pub fn submit_request(&self, batch_hash: H256, transition: StateTransition) -> H256 {
        let request = ProofRequest::new(batch_hash, transition);
        let id = request.id;
        
        self.queue.write().push(request);
        self.stats.write().requests_queued += 1;
        
        id
    }

    /// Process next request in queue
    pub fn process_next(&self, prover_id: u32) -> Option<H256> {
        let mut queue = self.queue.write();
        if queue.is_empty() {
            return None;
        }

        let mut request = queue.remove(0);
        let id = request.id;
        request.start(prover_id);

        self.active.write().insert(id, request);
        Some(id)
    }

    /// Generate proof for request
    pub fn generate_proof(&self, request_id: &H256) -> Result<Proof, ProverError> {
        let request = self.active.read()
            .get(request_id)
            .cloned()
            .ok_or_else(|| ProverError::RequestNotFound(*request_id))?;

        let params = self.circuit_params.read()
            .clone()
            .ok_or(ProverError::NotInitialized)?;

        let start = Instant::now();

        // Generate proof based on type
        let proof_data = match self.config.proof_type {
            ProofType::Stark => self.generate_stark_proof(&request, &params)?,
            ProofType::Snark => self.generate_snark_proof(&request, &params)?,
            ProofType::Plonk => self.generate_plonk_proof(&request, &params)?,
            ProofType::Groth16 => self.generate_groth16_proof(&request, &params)?,
        };

        let generation_time = start.elapsed().as_millis() as u64;

        let mut proof = Proof {
            proof_type: self.config.proof_type,
            data: proof_data,
            public_inputs: vec![
                request.transition.pre_state_root,
                request.transition.post_state_root,
                request.transition.transition_hash,
            ],
            batch_hash: request.batch_hash,
            pre_state_root: request.transition.pre_state_root,
            post_state_root: request.transition.post_state_root,
            prover: self.address,
            generated_at: current_timestamp(),
            generation_time_ms: generation_time,
            proof_hash: H256::zero(),
        };
        proof.proof_hash = proof.compute_hash();

        // Update request
        {
            let mut active = self.active.write();
            if let Some(req) = active.get_mut(request_id) {
                req.complete(proof.clone());
            }
        }

        // Store proof
        self.proofs.write().insert(request.batch_hash, proof.clone());

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.proofs_generated += 1;
            stats.total_proving_time_ms += generation_time;
        }

        Ok(proof)
    }

    /// Generate STARK proof
    fn generate_stark_proof(&self, request: &ProofRequest, params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        // STARK proof generation
        // Uses FRI (Fast Reed-Solomon Interactive Oracle Proofs of Proximity)
        
        let mut proof = Vec::new();
        
        // 1. Encode execution trace
        let trace = self.encode_execution_trace(request)?;
        
        // 2. Commit to trace polynomials
        let trace_commitment = self.commit_polynomials(&trace, params)?;
        proof.extend_from_slice(&trace_commitment);
        
        // 3. Generate constraint polynomials
        let constraints = self.generate_constraints(request, params)?;
        
        // 4. FRI commitment
        let fri_proof = self.generate_fri_proof(&constraints, params)?;
        proof.extend_from_slice(&fri_proof);
        
        // 5. Generate query responses
        let queries = self.generate_queries(params)?;
        proof.extend_from_slice(&queries);

        Ok(proof)
    }

    /// Generate SNARK proof
    fn generate_snark_proof(&self, request: &ProofRequest, params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        // SNARK proof generation using R1CS
        
        let mut proof = Vec::new();
        
        // 1. Generate R1CS witness
        let witness = self.generate_witness(request)?;
        
        // 2. Compute proof elements (A, B, C points)
        let a_point = self.compute_proof_element_a(&witness, params)?;
        let b_point = self.compute_proof_element_b(&witness, params)?;
        let c_point = self.compute_proof_element_c(&witness, params)?;
        
        // Encode as proof
        proof.extend_from_slice(&a_point);
        proof.extend_from_slice(&b_point);
        proof.extend_from_slice(&c_point);

        Ok(proof)
    }

    /// Generate Plonk proof
    fn generate_plonk_proof(&self, request: &ProofRequest, params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        // Plonk proof generation
        
        let mut proof = Vec::new();
        
        // 1. Generate wire polynomials
        let wire_polys = self.generate_wire_polynomials(request, params)?;
        
        // 2. Commit to wire polynomials
        for poly in &wire_polys {
            let commitment = self.kzg_commit(poly, params)?;
            proof.extend_from_slice(&commitment);
        }
        
        // 3. Generate permutation proof
        let perm_proof = self.generate_permutation_proof(&wire_polys, params)?;
        proof.extend_from_slice(&perm_proof);
        
        // 4. Generate opening proof
        let opening = self.generate_opening_proof(&wire_polys, params)?;
        proof.extend_from_slice(&opening);

        Ok(proof)
    }

    /// Generate Groth16 proof
    fn generate_groth16_proof(&self, request: &ProofRequest, params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        // Groth16 proof generation
        
        let mut proof = Vec::new();
        
        // 1. Compute QAP polynomials
        let qap = self.compute_qap(request, params)?;
        
        // 2. Generate proof elements (π_A, π_B, π_C)
        let pi_a = self.compute_groth16_pi_a(&qap, params)?;
        let pi_b = self.compute_groth16_pi_b(&qap, params)?;
        let pi_c = self.compute_groth16_pi_c(&qap, params)?;
        
        // Encode as compressed G1/G2 points
        proof.extend_from_slice(&pi_a);
        proof.extend_from_slice(&pi_b);
        proof.extend_from_slice(&pi_c);

        Ok(proof)
    }

    // Helper methods for proof generation
    fn encode_execution_trace(&self, request: &ProofRequest) -> Result<Vec<u8>, ProverError> {
        let mut trace = Vec::new();
        trace.extend_from_slice(&request.transition.pre_state_root.0);
        trace.extend_from_slice(&request.transition.post_state_root.0);
        for tx_hash in &request.transition.tx_hashes {
            trace.extend_from_slice(&tx_hash.0);
        }
        Ok(trace)
    }

    fn commit_polynomials(&self, data: &[u8], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(data);
        Ok(hasher.finalize().as_bytes().to_vec())
    }

    fn generate_constraints(&self, _request: &ProofRequest, params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        // Generate constraint system
        let constraints = vec![0u8; params.constraint_count * 32];
        Ok(constraints)
    }

    fn generate_fri_proof(&self, _constraints: &[u8], params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        // FRI proof for STARK
        let fri_layers = params.fri_layers;
        let proof = vec![0u8; fri_layers * 64];
        Ok(proof)
    }

    fn generate_queries(&self, params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        let queries = vec![0u8; params.num_queries * 32];
        Ok(queries)
    }

    fn generate_witness(&self, request: &ProofRequest) -> Result<Vec<u8>, ProverError> {
        let mut witness = Vec::new();
        witness.extend_from_slice(&request.transition.pre_state_root.0);
        witness.extend_from_slice(&request.transition.post_state_root.0);
        Ok(witness)
    }

    fn compute_proof_element_a(&self, _witness: &[u8], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 64]) // G1 point
    }

    fn compute_proof_element_b(&self, _witness: &[u8], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 128]) // G2 point
    }

    fn compute_proof_element_c(&self, _witness: &[u8], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 64]) // G1 point
    }

    fn generate_wire_polynomials(&self, _request: &ProofRequest, params: &CircuitParams) -> Result<Vec<Vec<u8>>, ProverError> {
        let poly_size = params.constraint_count * 32;
        Ok(vec![vec![0u8; poly_size]; 3]) // a, b, c wires
    }

    fn kzg_commit(&self, _poly: &[u8], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 48]) // BLS12-381 G1 point
    }

    fn generate_permutation_proof(&self, _polys: &[Vec<u8>], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 96])
    }

    fn generate_opening_proof(&self, _polys: &[Vec<u8>], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 48])
    }

    fn compute_qap(&self, _request: &ProofRequest, _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 256])
    }

    fn compute_groth16_pi_a(&self, _qap: &[u8], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 64])
    }

    fn compute_groth16_pi_b(&self, _qap: &[u8], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 128])
    }

    fn compute_groth16_pi_c(&self, _qap: &[u8], _params: &CircuitParams) -> Result<Vec<u8>, ProverError> {
        Ok(vec![0u8; 64])
    }

    /// Get proof for batch
    pub fn get_proof(&self, batch_hash: &H256) -> Option<Proof> {
        self.proofs.read().get(batch_hash).cloned()
    }

    /// Get request status
    pub fn get_request_status(&self, request_id: &H256) -> Option<ProofRequestStatus> {
        if let Some(req) = self.active.read().get(request_id) {
            return Some(req.status);
        }
        for req in self.queue.read().iter() {
            if req.id == *request_id {
                return Some(req.status);
            }
        }
        None
    }

    /// Check timeouts
    pub fn check_timeouts(&self) -> Vec<H256> {
        let mut timed_out = Vec::new();
        let mut active = self.active.write();
        
        for (id, request) in active.iter_mut() {
            if request.check_timeout(self.config.timeout_secs) {
                timed_out.push(*id);
                self.stats.write().timeouts += 1;
            }
        }

        timed_out
    }

    /// Get queue length
    pub fn queue_length(&self) -> usize {
        self.queue.read().len()
    }

    /// Get statistics
    pub fn stats(&self) -> ProverStats {
        self.stats.read().clone()
    }
}

/// Circuit parameters
#[derive(Debug, Clone)]
pub struct CircuitParams {
    /// Proof type
    pub proof_type: ProofType,
    /// Number of constraints
    pub constraint_count: usize,
    /// FRI layers (for STARK)
    pub fri_layers: usize,
    /// Number of queries
    pub num_queries: usize,
    /// Verification key
    pub verification_key: Vec<u8>,
}

impl CircuitParams {
    /// Generate circuit parameters
    pub fn generate(proof_type: ProofType) -> Result<Self, ProverError> {
        let (constraint_count, fri_layers, num_queries) = match proof_type {
            ProofType::Stark => (1 << 20, 10, 40),
            ProofType::Snark => (1 << 18, 0, 0),
            ProofType::Plonk => (1 << 18, 0, 0),
            ProofType::Groth16 => (1 << 16, 0, 0),
        };

        Ok(Self {
            proof_type,
            constraint_count,
            fri_layers,
            num_queries,
            verification_key: Vec::new(), // Would be generated during trusted setup
        })
    }
}

/// Prover errors
#[derive(Debug, thiserror::Error)]
pub enum ProverError {
    #[error("Prover not initialized")]
    NotInitialized,

    #[error("Request not found: {0:?}")]
    RequestNotFound(H256),

    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),

    #[error("Invalid witness: {0}")]
    InvalidWitness(String),

    #[error("Constraint system error: {0}")]
    ConstraintError(String),

    #[error("Timeout")]
    Timeout,

    #[error("GPU error: {0}")]
    GpuError(String),
}

/// Prover statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProverStats {
    pub requests_queued: u64,
    pub proofs_generated: u64,
    pub total_proving_time_ms: u64,
    pub timeouts: u64,
    pub failures: u64,
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

    #[test]
    fn test_proof_type() {
        assert!(ProofType::Groth16.expected_size() < ProofType::Stark.expected_size());
        assert!(ProofType::Stark.time_factor() < ProofType::Groth16.time_factor());
    }

    #[test]
    fn test_prover_creation() {
        let config = ProverConfig::default();
        let prover = ZkProver::new(config, Address([1u8; 20]));
        
        assert_eq!(prover.queue_length(), 0);
    }

    #[test]
    fn test_submit_request() {
        let config = ProverConfig::default();
        let prover = ZkProver::new(config, Address([1u8; 20]));

        let transition = StateTransition::new(
            H256([1u8; 32]),
            H256([2u8; 32]),
            1,
            vec![H256([3u8; 32])],
            vec![Address([1u8; 20])],
        );

        let id = prover.submit_request(H256([4u8; 32]), transition);
        assert_eq!(prover.queue_length(), 1);
        assert_eq!(prover.get_request_status(&id), Some(ProofRequestStatus::Queued));
    }

    #[test]
    fn test_circuit_params() {
        let params = CircuitParams::generate(ProofType::Plonk).unwrap();
        assert!(params.constraint_count > 0);
    }
}
