//! PYRAX Stream A CPU Mining Test
//!
//! A standalone binary to test BLAKE3 PoW mining on CPU.
//! This proves the consensus works without needing ASICs.

use std::time::Instant;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           PYRAX Stream A - CPU Mining Test                    ║");
    println!("║           Algorithm: BLAKE3 PoW                               ║");
    println!("║           Block Time Target: 10 seconds                       ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // Benchmark BLAKE3 hashrate
    println!("Benchmarking BLAKE3 hashrate...");
    let iterations = 1_000_000u64;
    let test_data = [0u8; 100];
    
    let start = Instant::now();
    for nonce in 0..iterations {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&test_data);
        hasher.update(&nonce.to_le_bytes());
        let _ = hasher.finalize();
    }
    let elapsed = start.elapsed().as_secs_f64();
    let hashrate = iterations as f64 / elapsed;
    
    println!("  Iterations: {}", iterations);
    println!("  Time: {:.2}s", elapsed);
    println!("  Hashrate: {:.2} MH/s", hashrate / 1_000_000.0);
    println!();

    // Mine some test blocks
    println!("Mining test blocks at difficulty 1 (very easy)...");
    let difficulty = 1u64;
    let target = difficulty_to_target(difficulty);
    
    let mut parent_hash = [0u8; 32];
    let num_blocks = 5;
    
    for block_num in 1..=num_blocks {
        let block_start = Instant::now();
        
        // Create header data
        let mut header = Vec::new();
        header.extend_from_slice(&1u32.to_le_bytes()); // version
        header.extend_from_slice(&parent_hash);        // parent
        header.extend_from_slice(&[0u8; 32]);          // merkle root
        header.extend_from_slice(&std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_le_bytes());                           // timestamp
        header.extend_from_slice(&difficulty.to_le_bytes());
        header.extend_from_slice(&(block_num as u64).to_le_bytes()); // height
        
        // Mine
        let mut nonce = rand::random::<u64>();
        let mut hashes = 0u64;
        
        loop {
            let hash = blake3_pow_hash(&header, nonce, 0);
            hashes += 1;
            
            if meets_target(&hash, &target) {
                let duration = block_start.elapsed();
                println!(
                    "  Block {} mined! Nonce: {}, Hashes: {}, Time: {:.3}s",
                    block_num, nonce, hashes, duration.as_secs_f64()
                );
                println!("    Hash: {}", hex::encode(&hash[0..16]));
                parent_hash = hash;
                break;
            }
            
            nonce = nonce.wrapping_add(1);
            
            // Safety limit
            if hashes > 100_000_000 {
                println!("  Block {} - gave up after {} hashes", block_num, hashes);
                break;
            }
        }
    }
    
    println!();
    println!("Mining test complete!");
    println!();
    println!("Stream A Parameters:");
    println!("  Algorithm: BLAKE3");
    println!("  Block Time: 10 seconds");
    println!("  Block Reward: 50 PYRAX");
    println!("  Hardware: ASIC-friendly (CPU for testing)");
}

fn blake3_pow_hash(header: &[u8], nonce: u64, extra_nonce: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(header);
    hasher.update(&nonce.to_le_bytes());
    hasher.update(&extra_nonce.to_le_bytes());
    *hasher.finalize().as_bytes()
}

fn difficulty_to_target(difficulty: u64) -> [u8; 32] {
    if difficulty == 0 {
        return [0xff; 32];
    }
    
    // Simple target calculation: lower difficulty = easier (higher target)
    // At difficulty 1, target is very high (easy)
    let mut target = [0u8; 32];
    let leading_zeros = (difficulty.leading_zeros() / 8) as usize;
    
    // For difficulty 1, make it very easy
    if difficulty == 1 {
        target = [0xff; 32];
        target[0] = 0x7f; // Still need some work
    } else {
        // More complex difficulty calculation for higher difficulties
        let shift = 256 - (64 - difficulty.leading_zeros()) as usize;
        if shift < 256 {
            let byte_idx = shift / 8;
            if byte_idx < 32 {
                target[byte_idx] = 0xff;
                for i in (byte_idx + 1)..32 {
                    target[i] = 0xff;
                }
            }
        }
    }
    
    target
}

fn meets_target(hash: &[u8; 32], target: &[u8; 32]) -> bool {
    // Compare as big-endian numbers
    for i in 0..32 {
        if hash[i] < target[i] {
            return true;
        }
        if hash[i] > target[i] {
            return false;
        }
    }
    true
}
