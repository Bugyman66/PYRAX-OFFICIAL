//! BLAKE3 Proof-of-Work for Stream A
//!
//! Stream A uses BLAKE3 as its PoW algorithm because:
//! - Extremely fast on modern CPUs (for testing without ASICs)
//! - Parallelizable and ASIC-friendly in production
//! - Cryptographically secure (based on ChaCha)
//! - Simple implementation with no DAG/cache like KAWPOW
//!
//! The difficulty target is a 256-bit number. A valid block hash must be
//! less than or equal to the target.

use crate::types::{H256, BlockNumber};

/// BLAKE3 PoW hasher for Stream A blocks
pub struct Blake3Pow;

impl Blake3Pow {
    /// Compute the PoW hash for a block header
    /// 
    /// The hash is computed as: BLAKE3(header_bytes || nonce || extra_nonce)
    pub fn hash(header_bytes: &[u8], nonce: u64, extra_nonce: u64) -> H256 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(header_bytes);
        hasher.update(&nonce.to_le_bytes());
        hasher.update(&extra_nonce.to_le_bytes());
        let result = hasher.finalize();
        H256::from_slice(result.as_bytes())
    }
    
    /// Check if a hash meets the difficulty target
    pub fn meets_target(hash: &H256, target: &H256) -> bool {
        // Compare as big-endian 256-bit numbers
        hash.as_bytes() <= target.as_bytes()
    }
    
    /// Convert difficulty to target
    /// 
    /// Target = MAX_TARGET / difficulty
    /// where MAX_TARGET = 2^256 - 1
    pub fn difficulty_to_target(difficulty: u64) -> H256 {
        if difficulty == 0 {
            return H256::from_slice(&[0xff; 32]);
        }
        
        // MAX_TARGET as U256
        let max_target = ethereum_types::U256::MAX;
        let target = max_target / ethereum_types::U256::from(difficulty);
        
        let mut bytes = [0u8; 32];
        target.to_big_endian(&mut bytes);
        H256::from_slice(&bytes)
    }
    
    /// Convert target back to difficulty
    pub fn target_to_difficulty(target: &H256) -> u64 {
        let target_u256 = ethereum_types::U256::from_big_endian(target.as_bytes());
        if target_u256.is_zero() {
            return u64::MAX;
        }
        
        let max_target = ethereum_types::U256::MAX;
        let difficulty = max_target / target_u256;
        
        // Clamp to u64
        if difficulty > ethereum_types::U256::from(u64::MAX) {
            u64::MAX
        } else {
            difficulty.as_u64()
        }
    }
    
    /// Mine a block (CPU reference implementation for testing)
    /// 
    /// This is a simple single-threaded miner for development/testing.
    /// Production ASICs will be much faster.
    /// 
    /// Returns (nonce, extra_nonce, hash) if found within max_iterations
    pub fn mine(
        header_bytes: &[u8],
        target: &H256,
        start_nonce: u64,
        max_iterations: u64,
    ) -> Option<(u64, u64, H256)> {
        let mut nonce = start_nonce;
        let mut extra_nonce = 0u64;
        
        for _ in 0..max_iterations {
            let hash = Self::hash(header_bytes, nonce, extra_nonce);
            
            if Self::meets_target(&hash, target) {
                return Some((nonce, extra_nonce, hash));
            }
            
            // Increment nonce, rollover to extra_nonce
            nonce = nonce.wrapping_add(1);
            if nonce == 0 {
                extra_nonce = extra_nonce.wrapping_add(1);
            }
        }
        
        None
    }
    
    /// Mine with a callback for progress reporting
    pub fn mine_with_callback<F>(
        header_bytes: &[u8],
        target: &H256,
        start_nonce: u64,
        max_iterations: u64,
        mut callback: F,
    ) -> Option<(u64, u64, H256)>
    where
        F: FnMut(u64) -> bool, // Returns false to stop mining
    {
        let mut nonce = start_nonce;
        let mut extra_nonce = 0u64;
        let report_interval = 100_000u64;
        
        for iteration in 0..max_iterations {
            let hash = Self::hash(header_bytes, nonce, extra_nonce);
            
            if Self::meets_target(&hash, target) {
                return Some((nonce, extra_nonce, hash));
            }
            
            // Report progress periodically
            if iteration % report_interval == 0 && iteration > 0 {
                if !callback(iteration) {
                    return None; // Callback requested stop
                }
            }
            
            nonce = nonce.wrapping_add(1);
            if nonce == 0 {
                extra_nonce = extra_nonce.wrapping_add(1);
            }
        }
        
        None
    }
    
    /// Estimate hashrate based on time to find N hashes
    pub fn benchmark(iterations: u64) -> f64 {
        use std::time::Instant;
        
        let test_header = [0u8; 100];
        let start = Instant::now();
        
        for nonce in 0..iterations {
            let _ = Self::hash(&test_header, nonce, 0);
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        iterations as f64 / elapsed
    }
}

/// Stream A difficulty adjustment parameters
pub struct DifficultyAdjustment;

impl DifficultyAdjustment {
    /// Target block time in seconds
    pub const TARGET_BLOCK_TIME: u64 = 10;
    
    /// Number of blocks in the averaging window
    pub const AVERAGING_WINDOW: u64 = 720; // ~2 hours at 10s blocks
    
    /// Maximum difficulty change per adjustment (25%)
    pub const MAX_ADJUSTMENT_FACTOR: f64 = 1.25;
    
    /// Minimum difficulty (for devnet/testing)
    pub const MIN_DIFFICULTY: u64 = 1;
    
    /// Calculate next difficulty based on recent block times
    pub fn calculate_next(
        current_difficulty: u64,
        actual_time_secs: u64,  // Time taken for AVERAGING_WINDOW blocks
        expected_time_secs: u64, // Should be AVERAGING_WINDOW * TARGET_BLOCK_TIME
    ) -> u64 {
        if actual_time_secs == 0 {
            return current_difficulty;
        }
        
        // Calculate adjustment ratio
        let ratio = expected_time_secs as f64 / actual_time_secs as f64;
        
        // Clamp to maximum adjustment
        let clamped_ratio = ratio
            .max(1.0 / Self::MAX_ADJUSTMENT_FACTOR)
            .min(Self::MAX_ADJUSTMENT_FACTOR);
        
        // Apply adjustment
        let new_difficulty = (current_difficulty as f64 * clamped_ratio) as u64;
        
        // Ensure minimum difficulty
        new_difficulty.max(Self::MIN_DIFFICULTY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blake3_hash_deterministic() {
        let header = b"test header data";
        let hash1 = Blake3Pow::hash(header, 12345, 0);
        let hash2 = Blake3Pow::hash(header, 12345, 0);
        assert_eq!(hash1, hash2);
        
        // Different nonce = different hash
        let hash3 = Blake3Pow::hash(header, 12346, 0);
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_difficulty_to_target_roundtrip() {
        for difficulty in [1, 100, 1000, 1_000_000, u64::MAX / 2] {
            let target = Blake3Pow::difficulty_to_target(difficulty);
            let recovered = Blake3Pow::target_to_difficulty(&target);
            // Allow for rounding errors
            let diff = if recovered > difficulty {
                recovered - difficulty
            } else {
                difficulty - recovered
            };
            assert!(diff <= 1, "Difficulty {} -> {} (diff {})", difficulty, recovered, diff);
        }
    }
    
    #[test]
    fn test_mine_low_difficulty() {
        // Very low difficulty - should find solution quickly
        let header = b"test block header for mining";
        let target = Blake3Pow::difficulty_to_target(1); // Easiest possible
        
        let result = Blake3Pow::mine(header, &target, 0, 100);
        assert!(result.is_some(), "Should find solution at difficulty 1");
        
        let (nonce, extra_nonce, hash) = result.unwrap();
        assert!(Blake3Pow::meets_target(&hash, &target));
        
        // Verify the hash
        let verify_hash = Blake3Pow::hash(header, nonce, extra_nonce);
        assert_eq!(hash, verify_hash);
    }
    
    #[test]
    fn test_difficulty_adjustment() {
        let current = 1000;
        
        // Blocks too fast (5s instead of 10s) -> increase difficulty
        let expected = 720 * 10; // 7200 seconds
        let actual = 720 * 5;    // 3600 seconds (too fast)
        let new_diff = DifficultyAdjustment::calculate_next(current, actual, expected);
        assert!(new_diff > current, "Difficulty should increase when blocks are fast");
        
        // Blocks too slow (20s instead of 10s) -> decrease difficulty
        let actual_slow = 720 * 20; // 14400 seconds (too slow)
        let new_diff_slow = DifficultyAdjustment::calculate_next(current, actual_slow, expected);
        assert!(new_diff_slow < current, "Difficulty should decrease when blocks are slow");
    }
    
    #[test]
    fn test_benchmark() {
        // Quick benchmark to ensure it runs
        let hashrate = Blake3Pow::benchmark(10_000);
        assert!(hashrate > 0.0, "Hashrate should be positive");
        println!("BLAKE3 hashrate: {:.2} H/s", hashrate);
    }
}
