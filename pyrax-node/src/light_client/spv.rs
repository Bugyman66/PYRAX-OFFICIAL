//! SPV (Simplified Payment Verification)

use sha2::{Sha256, Digest};

/// Merkle proof for transaction inclusion
#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub tx_index: u64,
    pub siblings: Vec<[u8; 32]>,
    pub flags: Vec<bool>,
}

impl MerkleProof {
    pub fn new(tx_index: u64, siblings: Vec<[u8; 32]>, flags: Vec<bool>) -> Self {
        Self { tx_index, siblings, flags }
    }
    
    pub fn empty() -> Self {
        Self { tx_index: 0, siblings: Vec::new(), flags: Vec::new() }
    }
}

/// Result of proof verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofVerificationResult {
    Valid,
    Invalid,
    BlockNotFound,
    ProofMalformed,
}

/// SPV client for merkle proof verification
pub struct SpvClient {
    verified_count: u64,
}

impl SpvClient {
    pub fn new() -> Self {
        Self { verified_count: 0 }
    }
    
    /// Verify a merkle proof
    pub fn verify_proof(
        &self,
        tx_hash: &[u8; 32],
        merkle_root: &[u8; 32],
        proof: &MerkleProof,
    ) -> ProofVerificationResult {
        if proof.siblings.is_empty() {
            // Single transaction block
            return if tx_hash == merkle_root {
                ProofVerificationResult::Valid
            } else {
                ProofVerificationResult::Invalid
            };
        }
        
        if proof.siblings.len() != proof.flags.len() {
            return ProofVerificationResult::ProofMalformed;
        }
        
        let mut current = *tx_hash;
        let mut index = proof.tx_index;
        
        for (sibling, is_left) in proof.siblings.iter().zip(proof.flags.iter()) {
            current = if *is_left {
                hash_pair(sibling, &current)
            } else {
                hash_pair(&current, sibling)
            };
            index /= 2;
        }
        
        if &current == merkle_root {
            ProofVerificationResult::Valid
        } else {
            ProofVerificationResult::Invalid
        }
    }
    
    /// Create a merkle proof from a list of transactions
    pub fn create_proof(tx_hashes: &[[u8; 32]], tx_index: usize) -> Option<MerkleProof> {
        if tx_hashes.is_empty() || tx_index >= tx_hashes.len() {
            return None;
        }
        
        if tx_hashes.len() == 1 {
            return Some(MerkleProof::empty());
        }
        
        let mut siblings = Vec::new();
        let mut flags = Vec::new();
        let mut level: Vec<[u8; 32]> = tx_hashes.to_vec();
        let mut idx = tx_index;
        
        while level.len() > 1 {
            // Pad to even length
            if level.len() % 2 == 1 {
                level.push(*level.last().unwrap());
            }
            
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            siblings.push(level[sibling_idx]);
            flags.push(idx % 2 == 1); // true if sibling is on left
            
            // Compute next level
            let mut next_level = Vec::new();
            for chunk in level.chunks(2) {
                next_level.push(hash_pair(&chunk[0], &chunk[1]));
            }
            
            level = next_level;
            idx /= 2;
        }
        
        Some(MerkleProof::new(tx_index as u64, siblings, flags))
    }
    
    /// Compute merkle root from transaction hashes
    pub fn compute_merkle_root(tx_hashes: &[[u8; 32]]) -> [u8; 32] {
        if tx_hashes.is_empty() {
            return [0u8; 32];
        }
        
        if tx_hashes.len() == 1 {
            return tx_hashes[0];
        }
        
        let mut level: Vec<[u8; 32]> = tx_hashes.to_vec();
        
        while level.len() > 1 {
            if level.len() % 2 == 1 {
                level.push(*level.last().unwrap());
            }
            
            let mut next_level = Vec::new();
            for chunk in level.chunks(2) {
                next_level.push(hash_pair(&chunk[0], &chunk[1]));
            }
            level = next_level;
        }
        
        level[0]
    }
}

impl Default for SpvClient {
    fn default() -> Self { Self::new() }
}

/// Hash two nodes together
fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let result = hasher.finalize();
    
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_single_tx_proof() {
        let client = SpvClient::new();
        let tx_hash = [1u8; 32];
        let merkle_root = tx_hash;
        let proof = MerkleProof::empty();
        
        assert_eq!(
            client.verify_proof(&tx_hash, &merkle_root, &proof),
            ProofVerificationResult::Valid
        );
    }
    
    #[test]
    fn test_merkle_root_computation() {
        let tx1 = [1u8; 32];
        let tx2 = [2u8; 32];
        
        let root = SpvClient::compute_merkle_root(&[tx1, tx2]);
        assert_ne!(root, [0u8; 32]);
        
        let expected = hash_pair(&tx1, &tx2);
        assert_eq!(root, expected);
    }
    
    #[test]
    fn test_proof_creation_and_verification() {
        let txs: Vec<[u8; 32]> = (0..4).map(|i| [i as u8; 32]).collect();
        let root = SpvClient::compute_merkle_root(&txs);
        
        let client = SpvClient::new();
        
        for (i, tx) in txs.iter().enumerate() {
            let proof = SpvClient::create_proof(&txs, i).unwrap();
            assert_eq!(
                client.verify_proof(tx, &root, &proof),
                ProofVerificationResult::Valid
            );
        }
    }
}
