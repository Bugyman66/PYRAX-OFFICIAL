//! PYRAX Mining Module
//!
//! Stream A: BLAKE3 PoW - ASIC mining with Stratum pool support
//! Stream B: KAWPOW - GPU mining with Stratum pool support
//!
//! Includes:
//! - BLAKE3 stratum server for Stream A ASIC mining
//! - KAWPOW stratum server for Stream B GPU mining
//! - Stratum client for connecting to external pools
//! - GPU mining with CUDA/OpenCL support
//!
//! Production-ready for devnet, testnet, and mainnet.

pub mod stratum;
pub mod stratum_server;
pub mod blake3_stratum;
pub mod gpu;
pub mod desktop_integration;

use std::time::Instant;

pub use stratum::{StratumClient, StratumConfig, StratumVersion, MiningJob, ShareResult};
pub use stratum_server::{
    StratumServer, StratumServerConfig, BlockTemplate, ServerStats,
    Worker as StratumWorker, Share as StratumShare,
};
pub use blake3_stratum::{
    Blake3StratumServer, Blake3StratumConfig, Blake3BlockTemplate, Blake3ServerStats,
    Blake3Worker, Blake3Share, Blake3ShareResult, Blake3MiningJob,
    Blake3BlockSubmitFn, Blake3TemplateProviderFn,
};
pub use gpu::{GpuMiner, GpuMinerConfig, GpuError, MiningResult, detect_devices};
pub use desktop_integration::{
    DesktopMiner, DesktopMinerConfig, MinerStatus, DeviceStatus, MiningEvent,
    GpuDeviceInfo, BenchmarkResult, DesktopMiningSettings,
};

/// Run a mining test with specified difficulty and block count
pub fn run_mining_test(difficulty: u64, num_blocks: u32) {
    println!("═══════════════════════════════════════════════════════════════");
    println!("           PYRAX Stream A - CPU Mining Test");
    println!("           Algorithm: BLAKE3 PoW");
    println!("           Target Block Time: 10 seconds");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    // Benchmark BLAKE3 hashrate
    println!("Benchmarking BLAKE3 hashrate...");
    let hashrate = benchmark_hashrate(100_000);
    println!("  CPU Hashrate: {:.2} MH/s", hashrate / 1_000_000.0);
    println!();

    // Calculate target from difficulty
    let target = difficulty_to_target(difficulty);
    println!("Mining {} blocks at difficulty {}...", num_blocks, difficulty);
    println!();

    let mut parent_hash = [0u8; 32];
    let mut total_time = 0.0f64;
    let mut total_hashes = 0u64;

    for block_num in 1..=num_blocks {
        let (hash, nonce, hashes, duration) = mine_block(
            block_num as u64,
            &parent_hash,
            &target,
        );

        total_time += duration;
        total_hashes += hashes;

        println!(
            "  Block {} | Nonce: {:>12} | Hashes: {:>8} | Time: {:.3}s | Hash: {}...",
            block_num,
            nonce,
            hashes,
            duration,
            hex::encode(&hash[0..8])
        );

        parent_hash = hash;
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("Mining Complete!");
    println!("  Total Blocks: {}", num_blocks);
    println!("  Total Time: {:.2}s", total_time);
    println!("  Total Hashes: {}", total_hashes);
    println!("  Avg Block Time: {:.2}s", total_time / num_blocks as f64);
    println!("  Avg Hashrate: {:.2} KH/s", total_hashes as f64 / total_time / 1000.0);
    println!("═══════════════════════════════════════════════════════════════");
}

/// Mine a single block, returns (hash, nonce, hashes_computed, duration_secs)
fn mine_block(height: u64, parent_hash: &[u8; 32], target: &[u8; 32]) -> ([u8; 32], u64, u64, f64) {
    let start = Instant::now();
    
    // Build header
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut header = Vec::with_capacity(100);
    header.extend_from_slice(&1u32.to_le_bytes());      // version
    header.push(0);                                      // stream A
    header.extend_from_slice(parent_hash);               // parent hash
    header.extend_from_slice(&[0u8; 32]);                // merkle root (empty)
    header.extend_from_slice(&timestamp.to_le_bytes());  // timestamp
    header.extend_from_slice(&height.to_le_bytes());     // height

    // Mine
    let mut nonce: u64 = rand::random();
    let mut hashes = 0u64;

    loop {
        let hash = blake3_hash(&header, nonce);
        hashes += 1;

        if meets_target(&hash, target) {
            let duration = start.elapsed().as_secs_f64();
            return (hash, nonce, hashes, duration);
        }

        nonce = nonce.wrapping_add(1);

        // Safety limit for very high difficulty
        if hashes > 1_000_000_000 {
            let duration = start.elapsed().as_secs_f64();
            return ([0u8; 32], 0, hashes, duration);
        }
    }
}

/// Compute BLAKE3 PoW hash
fn blake3_hash(header: &[u8], nonce: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(header);
    hasher.update(&nonce.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Check if hash meets difficulty target
fn meets_target(hash: &[u8; 32], target: &[u8; 32]) -> bool {
    for i in 0..32 {
        if hash[i] < target[i] { return true; }
        if hash[i] > target[i] { return false; }
    }
    true
}

/// Convert difficulty to target bytes
fn difficulty_to_target(difficulty: u64) -> [u8; 32] {
    let mut target = [0xffu8; 32];
    
    if difficulty <= 1 {
        target[0] = 0x7f; // Easy but requires some work
        return target;
    }

    // Higher difficulty = lower target = harder
    let leading_zeros = (difficulty.ilog2() / 8) as usize;
    for i in 0..leading_zeros.min(32) {
        target[i] = 0;
    }
    if leading_zeros < 32 {
        target[leading_zeros] = (0xff >> (difficulty.ilog2() % 8)) as u8;
    }
    
    target
}

/// Benchmark BLAKE3 hashrate
fn benchmark_hashrate(iterations: u64) -> f64 {
    let test_header = [0u8; 80];
    let start = Instant::now();
    
    for nonce in 0..iterations {
        let _ = blake3_hash(&test_header, nonce);
    }
    
    iterations as f64 / start.elapsed().as_secs_f64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_deterministic() {
        let header = [1u8; 80];
        let h1 = blake3_hash(&header, 12345);
        let h2 = blake3_hash(&header, 12345);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_mine_easy_block() {
        let target = difficulty_to_target(1);
        let (hash, _, hashes, _) = mine_block(1, &[0u8; 32], &target);
        assert!(meets_target(&hash, &target));
        assert!(hashes < 1000, "Should find block quickly at difficulty 1");
    }
}
