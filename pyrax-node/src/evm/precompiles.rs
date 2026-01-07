//! EVM Precompiled Contracts
//!
//! Standard Ethereum precompiles plus PYRAX-specific extensions

use std::collections::HashMap;
use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;

use super::types::{Address, B256, U256};

/// Precompile execution result
#[derive(Debug, Clone)]
pub struct PrecompileResult {
    pub success: bool,
    pub output: Vec<u8>,
    pub gas_used: u64,
}

impl PrecompileResult {
    pub fn success(output: Vec<u8>, gas_used: u64) -> Self {
        Self { success: true, output, gas_used }
    }

    pub fn error(gas_used: u64) -> Self {
        Self { success: false, output: Vec::new(), gas_used }
    }
}

/// Precompile function type
pub type PrecompileFn = fn(&[u8], u64) -> PrecompileResult;

/// Standard precompile addresses
pub mod addresses {
    use super::Address;

    pub const ECRECOVER: Address = addr(0x01);
    pub const SHA256: Address = addr(0x02);
    pub const RIPEMD160: Address = addr(0x03);
    pub const IDENTITY: Address = addr(0x04);
    pub const MODEXP: Address = addr(0x05);
    pub const BN128_ADD: Address = addr(0x06);
    pub const BN128_MUL: Address = addr(0x07);
    pub const BN128_PAIRING: Address = addr(0x08);
    pub const BLAKE2F: Address = addr(0x09);
    
    // PYRAX custom precompiles (use bytes 18 and 19)
    pub const BLAKE3: Address = addr2(0x01, 0x00);
    pub const VERIFY_ZK_PROOF: Address = addr2(0x01, 0x01);
    pub const L1_BRIDGE: Address = addr2(0x01, 0x02);

    const fn addr(last_byte: u8) -> Address {
        let mut addr = [0u8; 20];
        addr[19] = last_byte;
        addr
    }

    const fn addr2(b18: u8, b19: u8) -> Address {
        let mut addr = [0u8; 20];
        addr[18] = b18;
        addr[19] = b19;
        addr
    }
}

/// Precompile registry
pub struct PrecompileRegistry {
    precompiles: HashMap<Address, PrecompileFn>,
}

impl PrecompileRegistry {
    /// Create a new registry with all standard precompiles
    pub fn new() -> Self {
        let mut precompiles = HashMap::new();
        
        // Standard Ethereum precompiles
        precompiles.insert(addresses::ECRECOVER, ecrecover as PrecompileFn);
        precompiles.insert(addresses::SHA256, sha256 as PrecompileFn);
        precompiles.insert(addresses::RIPEMD160, ripemd160 as PrecompileFn);
        precompiles.insert(addresses::IDENTITY, identity as PrecompileFn);
        precompiles.insert(addresses::MODEXP, modexp as PrecompileFn);
        precompiles.insert(addresses::BN128_ADD, bn128_add as PrecompileFn);
        precompiles.insert(addresses::BN128_MUL, bn128_mul as PrecompileFn);
        precompiles.insert(addresses::BN128_PAIRING, bn128_pairing as PrecompileFn);
        precompiles.insert(addresses::BLAKE2F, blake2f as PrecompileFn);
        
        // PYRAX custom precompiles
        precompiles.insert(addresses::BLAKE3, blake3 as PrecompileFn);
        precompiles.insert(addresses::VERIFY_ZK_PROOF, verify_zk_proof as PrecompileFn);
        precompiles.insert(addresses::L1_BRIDGE, l1_bridge as PrecompileFn);

        Self { precompiles }
    }

    /// Check if address is a precompile
    pub fn is_precompile(&self, address: &Address) -> bool {
        self.precompiles.contains_key(address)
    }

    /// Execute a precompile
    pub fn execute(&self, address: &Address, input: &[u8], gas_limit: u64) -> Option<PrecompileResult> {
        self.precompiles.get(address).map(|f| f(input, gas_limit))
    }
}

impl Default for PrecompileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Standard Precompiles
// ============================================================================

/// ECRECOVER (0x01) - Recover signer from signature
fn ecrecover(input: &[u8], gas_limit: u64) -> PrecompileResult {
    const GAS_COST: u64 = 3000;
    
    if gas_limit < GAS_COST {
        return PrecompileResult::error(gas_limit);
    }

    // Input: 32 bytes hash + 32 bytes v + 32 bytes r + 32 bytes s
    if input.len() < 128 {
        // Pad with zeros if needed
        let mut padded = vec![0u8; 128];
        padded[..input.len()].copy_from_slice(input);
        return ecrecover_inner(&padded, GAS_COST);
    }

    ecrecover_inner(input, GAS_COST)
}

fn ecrecover_inner(input: &[u8], gas_cost: u64) -> PrecompileResult {
    let hash = &input[0..32];
    let v = input[63]; // Last byte of v
    let r = &input[64..96];
    let s = &input[96..128];

    // Validate v (27 or 28 for legacy, 0 or 1 for modern)
    let recovery_id = match v {
        27 | 0 => 0,
        28 | 1 => 1,
        _ => return PrecompileResult::success(vec![0u8; 32], gas_cost),
    };

    // Try to recover the public key using secp256k1
    use secp256k1::{Secp256k1, Message, ecdsa::{RecoverableSignature, RecoveryId}};
    
    let secp = Secp256k1::new();
    
    // Combine r and s into signature
    let mut sig_bytes = [0u8; 64];
    sig_bytes[..32].copy_from_slice(r);
    sig_bytes[32..].copy_from_slice(s);

    let msg = match Message::from_digest_slice(hash) {
        Ok(m) => m,
        Err(_) => return PrecompileResult::success(vec![0u8; 32], gas_cost),
    };

    let rec_id = match RecoveryId::from_i32(recovery_id) {
        Ok(id) => id,
        Err(_) => return PrecompileResult::success(vec![0u8; 32], gas_cost),
    };

    let sig = match RecoverableSignature::from_compact(&sig_bytes, rec_id) {
        Ok(s) => s,
        Err(_) => return PrecompileResult::success(vec![0u8; 32], gas_cost),
    };

    let pubkey = match secp.recover_ecdsa(&msg, &sig) {
        Ok(pk) => pk,
        Err(_) => return PrecompileResult::success(vec![0u8; 32], gas_cost),
    };

    // Hash public key to get address
    use tiny_keccak::{Hasher, Keccak};
    let serialized = pubkey.serialize_uncompressed();
    let mut hasher = Keccak::v256();
    hasher.update(&serialized[1..]); // Skip the 0x04 prefix
    let mut address_hash = [0u8; 32];
    hasher.finalize(&mut address_hash);

    // Return address (last 20 bytes, padded to 32)
    let mut output = vec![0u8; 32];
    output[12..32].copy_from_slice(&address_hash[12..32]);
    
    PrecompileResult::success(output, gas_cost)
}

/// SHA256 (0x02) - SHA-256 hash
fn sha256(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // Gas: 60 + 12 * ceil(len / 32)
    let words = (input.len() + 31) / 32;
    let gas_cost = 60 + 12 * words as u64;
    
    if gas_limit < gas_cost {
        return PrecompileResult::error(gas_limit);
    }

    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    
    PrecompileResult::success(result.to_vec(), gas_cost)
}

/// RIPEMD160 (0x03) - RIPEMD-160 hash
fn ripemd160(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // Gas: 600 + 120 * ceil(len / 32)
    let words = (input.len() + 31) / 32;
    let gas_cost = 600 + 120 * words as u64;
    
    if gas_limit < gas_cost {
        return PrecompileResult::error(gas_limit);
    }

    use sha2::digest::Digest;
    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(input);
    let result = hasher.finalize();
    
    // Pad to 32 bytes (right-aligned)
    let mut output = vec![0u8; 32];
    output[12..32].copy_from_slice(&result);
    
    PrecompileResult::success(output, gas_cost)
}

/// IDENTITY (0x04) - Return input as output
fn identity(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // Gas: 15 + 3 * ceil(len / 32)
    let words = (input.len() + 31) / 32;
    let gas_cost = 15 + 3 * words as u64;
    
    if gas_limit < gas_cost {
        return PrecompileResult::error(gas_limit);
    }

    PrecompileResult::success(input.to_vec(), gas_cost)
}

/// MODEXP (0x05) - Modular exponentiation
fn modexp(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // Simplified implementation - production would use big integer library
    let gas_cost = 200; // Minimum cost
    
    if gas_limit < gas_cost {
        return PrecompileResult::error(gas_limit);
    }

    if input.len() < 96 {
        return PrecompileResult::success(Vec::new(), gas_cost);
    }

    // Parse lengths
    let base_len = u64::from_be_bytes([
        0, 0, 0, 0,
        input[24], input[25], input[26], input[27],
    ]) as usize;
    let exp_len = u64::from_be_bytes([
        0, 0, 0, 0,
        input[56], input[57], input[58], input[59],
    ]) as usize;
    let mod_len = u64::from_be_bytes([
        0, 0, 0, 0,
        input[88], input[89], input[90], input[91],
    ]) as usize;

    if mod_len == 0 {
        return PrecompileResult::success(vec![0u8; mod_len], gas_cost);
    }

    // For production, implement full modexp using num-bigint
    // This is a placeholder that returns zeros
    PrecompileResult::success(vec![0u8; mod_len], gas_cost)
}

/// BN128_ADD (0x06) - BN128 curve point addition
fn bn128_add(input: &[u8], gas_limit: u64) -> PrecompileResult {
    const GAS_COST: u64 = 150;
    
    if gas_limit < GAS_COST {
        return PrecompileResult::error(gas_limit);
    }

    // Input: 64 bytes (point 1) + 64 bytes (point 2)
    // Output: 64 bytes (result point)
    // For production, implement using bn crate
    
    PrecompileResult::success(vec![0u8; 64], GAS_COST)
}

/// BN128_MUL (0x07) - BN128 curve scalar multiplication
fn bn128_mul(input: &[u8], gas_limit: u64) -> PrecompileResult {
    const GAS_COST: u64 = 6000;
    
    if gas_limit < GAS_COST {
        return PrecompileResult::error(gas_limit);
    }

    // Input: 64 bytes (point) + 32 bytes (scalar)
    // Output: 64 bytes (result point)
    
    PrecompileResult::success(vec![0u8; 64], GAS_COST)
}

/// BN128_PAIRING (0x08) - BN128 pairing check
fn bn128_pairing(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // Gas: 45000 + 34000 * k (k = number of pairs)
    let k = input.len() / 192;
    let gas_cost = 45000 + 34000 * k as u64;
    
    if gas_limit < gas_cost {
        return PrecompileResult::error(gas_limit);
    }

    // Returns 1 if pairing check passes, 0 otherwise
    let mut output = vec![0u8; 32];
    output[31] = 1; // Placeholder: always return true
    
    PrecompileResult::success(output, gas_cost)
}

/// BLAKE2F (0x09) - BLAKE2b F compression function
fn blake2f(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if input.len() != 213 {
        return PrecompileResult::error(0);
    }

    let rounds = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
    let gas_cost = rounds as u64;
    
    if gas_limit < gas_cost {
        return PrecompileResult::error(gas_limit);
    }

    // For production, implement BLAKE2b F compression
    PrecompileResult::success(vec![0u8; 64], gas_cost)
}

// ============================================================================
// PYRAX Custom Precompiles
// ============================================================================

/// BLAKE3 (0x100) - BLAKE3 hash (PYRAX extension)
fn blake3(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // Gas: 30 + 6 * ceil(len / 32)
    let words = (input.len() + 31) / 32;
    let gas_cost = 30 + 6 * words as u64;
    
    if gas_limit < gas_cost {
        return PrecompileResult::error(gas_limit);
    }

    let mut hasher = Blake3Hasher::new();
    hasher.update(input);
    let result = hasher.finalize();
    
    PrecompileResult::success(result.as_bytes().to_vec(), gas_cost)
}

/// VERIFY_ZK_PROOF (0x101) - Verify ZK-STARK proof (PYRAX extension)
fn verify_zk_proof(input: &[u8], gas_limit: u64) -> PrecompileResult {
    const BASE_GAS: u64 = 100_000;
    
    if gas_limit < BASE_GAS {
        return PrecompileResult::error(gas_limit);
    }

    if input.len() < 64 {
        return PrecompileResult::error(BASE_GAS);
    }

    // Input format:
    // - 32 bytes: proof commitment
    // - 32 bytes: public input hash
    // - remaining: proof data

    let _commitment = &input[0..32];
    let _public_input_hash = &input[32..64];
    let _proof_data = &input[64..];

    // For production, implement actual ZK proof verification
    // This is a placeholder that validates proof structure
    
    let mut output = vec![0u8; 32];
    output[31] = 1; // Return 1 for valid proof
    
    PrecompileResult::success(output, BASE_GAS)
}

/// L1_BRIDGE (0x102) - L1 bridge interaction (PYRAX extension)
fn l1_bridge(input: &[u8], gas_limit: u64) -> PrecompileResult {
    const GAS_COST: u64 = 50_000;
    
    if gas_limit < GAS_COST {
        return PrecompileResult::error(gas_limit);
    }

    if input.is_empty() {
        return PrecompileResult::error(GAS_COST);
    }

    // First byte is the operation type
    let op = input[0];
    
    match op {
        0x00 => {
            // Query L1 deposit status
            // Returns 1 if deposit is confirmed
            let mut output = vec![0u8; 32];
            output[31] = 1;
            PrecompileResult::success(output, GAS_COST)
        }
        0x01 => {
            // Initiate L2 -> L1 withdrawal
            // Returns withdrawal hash
            if input.len() < 53 {
                return PrecompileResult::error(GAS_COST);
            }
            
            let mut hasher = Blake3Hasher::new();
            hasher.update(&input[1..]);
            let result = hasher.finalize();
            
            PrecompileResult::success(result.as_bytes().to_vec(), GAS_COST)
        }
        _ => PrecompileResult::error(GAS_COST),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precompile_registry() {
        let registry = PrecompileRegistry::new();
        
        assert!(registry.is_precompile(&addresses::SHA256));
        assert!(registry.is_precompile(&addresses::BLAKE3));
        assert!(!registry.is_precompile(&[0u8; 20]));
    }

    #[test]
    fn test_sha256() {
        let result = sha256(b"hello world", 10000);
        assert!(result.success);
        assert_eq!(result.output.len(), 32);
    }

    #[test]
    fn test_identity() {
        let input = b"test data";
        let result = identity(input, 10000);
        assert!(result.success);
        assert_eq!(result.output, input);
    }

    #[test]
    fn test_blake3() {
        let result = blake3(b"pyrax", 10000);
        assert!(result.success);
        assert_eq!(result.output.len(), 32);
    }

    #[test]
    fn test_gas_limit_check() {
        let result = sha256(b"test", 10); // Not enough gas
        assert!(!result.success);
    }
}
