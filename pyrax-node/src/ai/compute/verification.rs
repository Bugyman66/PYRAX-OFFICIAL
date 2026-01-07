//! Result Verification
//!
//! Verification of compute job results for integrity and correctness

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::VERIFICATION_QUORUM;

/// Verification method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// No verification (trust primary node)
    None,
    /// Single verifier node
    Single,
    /// Multi-node consensus
    Consensus,
    /// Zero-knowledge proof
    ZKProof,
    /// Trusted execution environment attestation
    TEE,
    /// Deterministic replay
    Replay,
    /// Statistical sampling
    Sampling,
}

/// Verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Awaiting verification
    Pending,
    /// Verification in progress
    InProgress,
    /// Verified successfully
    Verified,
    /// Verification failed
    Failed,
    /// Disputed
    Disputed,
    /// Timeout
    TimedOut,
}

/// Verification result from a single verifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierResult {
    /// Verifier node address
    pub verifier: Address,
    /// Computed result hash
    pub result_hash: H256,
    /// Matches primary result
    pub matches: bool,
    /// Confidence score (0-100)
    pub confidence: u32,
    /// Verification timestamp
    pub verified_at: u64,
    /// Execution time (ms)
    pub execution_time_ms: u64,
    /// Verifier signature
    pub signature: Vec<u8>,
    /// Optional proof data
    pub proof: Option<Vec<u8>>,
}

/// Verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    /// Job ID
    pub job_id: H256,
    /// Primary result hash
    pub primary_hash: H256,
    /// Primary node address
    pub primary_node: Address,
    /// Input data hash
    pub input_hash: H256,
    /// Model ID (if applicable)
    pub model_id: Option<H256>,
    /// Verification method
    pub method: VerificationMethod,
    /// Required quorum
    pub quorum: u32,
    /// Creation timestamp
    pub created_at: u64,
    /// Deadline timestamp
    pub deadline: u64,
    /// Status
    pub status: VerificationStatus,
    /// Verifier results
    pub results: Vec<VerifierResult>,
    /// Final verdict
    pub verdict: Option<bool>,
    /// Dispute reason (if disputed)
    pub dispute_reason: Option<String>,
}

impl VerificationRequest {
    /// Create new verification request
    pub fn new(
        job_id: H256,
        primary_hash: H256,
        primary_node: Address,
        input_hash: H256,
        model_id: Option<H256>,
        method: VerificationMethod,
        deadline_secs: u64,
    ) -> Self {
        let quorum = match method {
            VerificationMethod::None => 0,
            VerificationMethod::Single => 1,
            VerificationMethod::Consensus => VERIFICATION_QUORUM,
            VerificationMethod::ZKProof => 1,
            VerificationMethod::TEE => 1,
            VerificationMethod::Replay => 2,
            VerificationMethod::Sampling => 3,
        };

        Self {
            job_id,
            primary_hash,
            primary_node,
            input_hash,
            model_id,
            method,
            quorum,
            created_at: current_timestamp(),
            deadline: current_timestamp() + deadline_secs,
            status: VerificationStatus::Pending,
            results: Vec::new(),
            verdict: None,
            dispute_reason: None,
        }
    }

    /// Add verifier result
    pub fn add_result(&mut self, result: VerifierResult) -> bool {
        // Don't accept results after deadline
        if current_timestamp() > self.deadline {
            self.status = VerificationStatus::TimedOut;
            return false;
        }

        // Don't accept duplicate verifiers
        if self.results.iter().any(|r| r.verifier == result.verifier) {
            return false;
        }

        self.results.push(result);
        self.status = VerificationStatus::InProgress;

        // Check if we have enough results
        if self.results.len() >= self.quorum as usize {
            self.finalize();
        }

        true
    }

    /// Finalize verification
    fn finalize(&mut self) {
        let matching = self.results.iter().filter(|r| r.matches).count();
        let total = self.results.len();

        // Consensus threshold: 2/3 must match
        let threshold = (total * 2) / 3;

        if matching >= threshold {
            self.verdict = Some(true);
            self.status = VerificationStatus::Verified;
        } else {
            self.verdict = Some(false);
            self.status = VerificationStatus::Failed;
        }
    }

    /// Check if verification is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            VerificationStatus::Verified | 
            VerificationStatus::Failed | 
            VerificationStatus::TimedOut |
            VerificationStatus::Disputed
        )
    }

    /// Check timeout
    pub fn check_timeout(&mut self) -> bool {
        if !self.is_complete() && current_timestamp() > self.deadline {
            self.status = VerificationStatus::TimedOut;
            true
        } else {
            false
        }
    }

    /// File dispute
    pub fn dispute(&mut self, reason: String) {
        if self.status == VerificationStatus::Verified || self.status == VerificationStatus::Failed {
            self.status = VerificationStatus::Disputed;
            self.dispute_reason = Some(reason);
        }
    }

    /// Get agreement percentage
    pub fn agreement_percentage(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let matching = self.results.iter().filter(|r| r.matches).count();
        (matching as f64 / self.results.len() as f64) * 100.0
    }

    /// Get average confidence
    pub fn average_confidence(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let total: u64 = self.results.iter().map(|r| r.confidence as u64).sum();
        total as f64 / self.results.len() as f64
    }
}

/// Verification result (final)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Job ID
    pub job_id: H256,
    /// Is verified
    pub verified: bool,
    /// Agreement percentage
    pub agreement: f64,
    /// Average confidence
    pub confidence: f64,
    /// Number of verifiers
    pub verifier_count: u32,
    /// Verification method used
    pub method: VerificationMethod,
    /// Time taken (seconds)
    pub duration_secs: u64,
    /// Timestamp
    pub timestamp: u64,
}

/// Result verifier
pub struct ResultVerifier {
    /// Pending verifications
    pending: Arc<RwLock<HashMap<H256, VerificationRequest>>>,
    /// Completed verifications
    completed: Arc<RwLock<HashMap<H256, VerificationResult>>>,
    /// Verifier pool (nodes that can verify)
    verifier_pool: Arc<RwLock<Vec<Address>>>,
    /// Assignments (job -> verifiers)
    assignments: Arc<RwLock<HashMap<H256, Vec<Address>>>>,
    /// Statistics
    stats: Arc<RwLock<VerifierStats>>,
    /// Default verification timeout (seconds)
    default_timeout: u64,
}

impl ResultVerifier {
    /// Create new verifier
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            completed: Arc::new(RwLock::new(HashMap::new())),
            verifier_pool: Arc::new(RwLock::new(Vec::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(VerifierStats::default())),
            default_timeout: 300, // 5 minutes
        }
    }

    /// Register verifier node
    pub fn register_verifier(&self, address: Address) {
        let mut pool = self.verifier_pool.write();
        if !pool.contains(&address) {
            pool.push(address);
        }
    }

    /// Unregister verifier node
    pub fn unregister_verifier(&self, address: &Address) {
        self.verifier_pool.write().retain(|a| a != address);
    }

    /// Create verification request
    pub fn request_verification(
        &self,
        job_id: H256,
        primary_hash: H256,
        primary_node: Address,
        input_hash: H256,
        model_id: Option<H256>,
        method: VerificationMethod,
    ) -> Result<Vec<Address>, VerificationError> {
        // Check if already pending
        if self.pending.read().contains_key(&job_id) {
            return Err(VerificationError::AlreadyPending(job_id));
        }

        // For methods that don't need verification
        if method == VerificationMethod::None {
            let result = VerificationResult {
                job_id,
                verified: true,
                agreement: 100.0,
                confidence: 100.0,
                verifier_count: 0,
                method,
                duration_secs: 0,
                timestamp: current_timestamp(),
            };
            self.completed.write().insert(job_id, result);
            return Ok(Vec::new());
        }

        // Create request
        let request = VerificationRequest::new(
            job_id,
            primary_hash,
            primary_node,
            input_hash,
            model_id,
            method,
            self.default_timeout,
        );

        // Select verifiers
        let verifiers = self.select_verifiers(&request)?;

        // Store request
        self.pending.write().insert(job_id, request);
        self.assignments.write().insert(job_id, verifiers.clone());
        self.stats.write().requests_created += 1;

        Ok(verifiers)
    }

    /// Select verifiers for a request
    fn select_verifiers(&self, request: &VerificationRequest) -> Result<Vec<Address>, VerificationError> {
        let pool = self.verifier_pool.read();
        
        if pool.len() < request.quorum as usize {
            return Err(VerificationError::InsufficientVerifiers {
                required: request.quorum as usize,
                available: pool.len(),
            });
        }

        // Filter out primary node
        let eligible: Vec<_> = pool.iter()
            .filter(|a| **a != request.primary_node)
            .cloned()
            .collect();

        if eligible.len() < request.quorum as usize {
            return Err(VerificationError::InsufficientVerifiers {
                required: request.quorum as usize,
                available: eligible.len(),
            });
        }

        // Select random verifiers (in production, would use VRF or similar)
        let selected: Vec<_> = eligible.into_iter()
            .take(request.quorum as usize)
            .collect();

        Ok(selected)
    }

    /// Submit verification result
    pub fn submit_result(
        &self,
        job_id: H256,
        result: VerifierResult,
    ) -> Result<bool, VerificationError> {
        let mut pending = self.pending.write();
        let request = pending.get_mut(&job_id)
            .ok_or_else(|| VerificationError::NotFound(job_id))?;

        // Verify this node is assigned
        let assignments = self.assignments.read();
        if let Some(assigned) = assignments.get(&job_id) {
            if !assigned.contains(&result.verifier) {
                return Err(VerificationError::NotAssigned(result.verifier));
            }
        }

        // Add result
        if !request.add_result(result) {
            return Ok(false);
        }

        // Check if complete
        if request.is_complete() {
            let verified = request.verdict.unwrap_or(false);
            let final_result = VerificationResult {
                job_id,
                verified,
                agreement: request.agreement_percentage(),
                confidence: request.average_confidence(),
                verifier_count: request.results.len() as u32,
                method: request.method,
                duration_secs: current_timestamp() - request.created_at,
                timestamp: current_timestamp(),
            };

            // Move to completed
            drop(pending);
            self.pending.write().remove(&job_id);
            self.completed.write().insert(job_id, final_result);
            self.assignments.write().remove(&job_id);

            // Update stats
            let mut stats = self.stats.write();
            stats.requests_completed += 1;
            if verified {
                stats.verified_count += 1;
            } else {
                stats.failed_count += 1;
            }
        }

        Ok(true)
    }

    /// Get verification request
    pub fn get_request(&self, job_id: &H256) -> Option<VerificationRequest> {
        self.pending.read().get(job_id).cloned()
    }

    /// Get verification result
    pub fn get_result(&self, job_id: &H256) -> Option<VerificationResult> {
        self.completed.read().get(job_id).cloned()
    }

    /// File dispute
    pub fn file_dispute(&self, job_id: &H256, reason: String) -> Result<(), VerificationError> {
        // Check completed first
        if self.completed.read().contains_key(job_id) {
            // Would need to reopen for dispute resolution
            return Err(VerificationError::AlreadyCompleted(*job_id));
        }

        let mut pending = self.pending.write();
        let request = pending.get_mut(job_id)
            .ok_or_else(|| VerificationError::NotFound(*job_id))?;

        request.dispute(reason);
        self.stats.write().disputes_filed += 1;
        Ok(())
    }

    /// Check and update timed out requests
    pub fn check_timeouts(&self) -> Vec<H256> {
        let mut timed_out = Vec::new();
        let mut pending = self.pending.write();

        for (id, request) in pending.iter_mut() {
            if request.check_timeout() {
                timed_out.push(*id);
            }
        }

        // Move timed out to completed
        for id in &timed_out {
            if let Some(request) = pending.remove(id) {
                let result = VerificationResult {
                    job_id: *id,
                    verified: false,
                    agreement: request.agreement_percentage(),
                    confidence: request.average_confidence(),
                    verifier_count: request.results.len() as u32,
                    method: request.method,
                    duration_secs: current_timestamp() - request.created_at,
                    timestamp: current_timestamp(),
                };
                self.completed.write().insert(*id, result);
            }
        }

        self.stats.write().timeouts += timed_out.len() as u64;
        timed_out
    }

    /// Get pending count
    pub fn pending_count(&self) -> usize {
        self.pending.read().len()
    }

    /// Get verifier pool size
    pub fn verifier_count(&self) -> usize {
        self.verifier_pool.read().len()
    }

    /// Get statistics
    pub fn stats(&self) -> VerifierStats {
        self.stats.read().clone()
    }
}

impl Default for ResultVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification errors
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("Verification not found: {0:?}")]
    NotFound(H256),

    #[error("Verification already pending: {0:?}")]
    AlreadyPending(H256),

    #[error("Verification already completed: {0:?}")]
    AlreadyCompleted(H256),

    #[error("Verifier not assigned: {0:?}")]
    NotAssigned(Address),

    #[error("Insufficient verifiers: required {required}, available {available}")]
    InsufficientVerifiers { required: usize, available: usize },

    #[error("Verification timeout")]
    Timeout,

    #[error("Invalid proof: {0}")]
    InvalidProof(String),
}

/// Verifier statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifierStats {
    pub requests_created: u64,
    pub requests_completed: u64,
    pub verified_count: u64,
    pub failed_count: u64,
    pub disputes_filed: u64,
    pub timeouts: u64,
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
    fn test_verification_request() {
        let request = VerificationRequest::new(
            H256([1u8; 32]),
            H256([2u8; 32]),
            Address([1u8; 20]),
            H256([3u8; 32]),
            None,
            VerificationMethod::Consensus,
            300,
        );

        assert_eq!(request.status, VerificationStatus::Pending);
        assert_eq!(request.quorum, VERIFICATION_QUORUM);
    }

    #[test]
    fn test_add_verifier_result() {
        let mut request = VerificationRequest::new(
            H256([1u8; 32]),
            H256([2u8; 32]),
            Address([1u8; 20]),
            H256([3u8; 32]),
            None,
            VerificationMethod::Consensus,
            300,
        );

        for i in 0..VERIFICATION_QUORUM {
            let result = VerifierResult {
                verifier: Address([i as u8 + 10; 20]),
                result_hash: H256([2u8; 32]),
                matches: true,
                confidence: 95,
                verified_at: current_timestamp(),
                execution_time_ms: 100,
                signature: vec![0u8; 64],
                proof: None,
            };
            request.add_result(result);
        }

        assert!(request.is_complete());
        assert_eq!(request.verdict, Some(true));
    }

    #[test]
    fn test_result_verifier() {
        let verifier = ResultVerifier::new();

        // Register verifiers
        for i in 0..5 {
            verifier.register_verifier(Address([i + 10; 20]));
        }

        assert_eq!(verifier.verifier_count(), 5);

        // Request verification
        let result = verifier.request_verification(
            H256([1u8; 32]),
            H256([2u8; 32]),
            Address([1u8; 20]),
            H256([3u8; 32]),
            None,
            VerificationMethod::Consensus,
        );

        assert!(result.is_ok());
        assert_eq!(verifier.pending_count(), 1);
    }

    #[test]
    fn test_no_verification() {
        let verifier = ResultVerifier::new();

        let result = verifier.request_verification(
            H256([1u8; 32]),
            H256([2u8; 32]),
            Address([1u8; 20]),
            H256([3u8; 32]),
            None,
            VerificationMethod::None,
        );

        assert!(result.is_ok());
        assert_eq!(verifier.pending_count(), 0);
        
        let final_result = verifier.get_result(&H256([1u8; 32]));
        assert!(final_result.is_some());
        assert!(final_result.unwrap().verified);
    }
}
