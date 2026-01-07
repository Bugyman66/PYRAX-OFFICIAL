use sha3::{Keccak256, Keccak512, Digest};

pub const EPOCH_LENGTH: u64 = 7500;
pub const CACHE_BYTES_INIT: usize = 16 * 1024 * 1024; // 16 MB
pub const CACHE_BYTES_GROWTH: usize = 128 * 1024; // 128 KB per epoch
pub const DAG_BYTES_INIT: usize = 1024 * 1024 * 1024; // 1 GB
pub const DAG_BYTES_GROWTH: usize = 8 * 1024 * 1024; // 8 MB per epoch

pub const PROGPOW_LANES: usize = 16;
pub const PROGPOW_REGS: usize = 32;
pub const PROGPOW_DAG_LOADS: usize = 4;
pub const PROGPOW_CNT_DAG: usize = 64;
pub const PROGPOW_CNT_MATH: usize = 18;
pub const PROGPOW_CNT_CACHE: usize = 11;

// DAG generation constants
pub const CACHE_ROUNDS: u32 = 3;
pub const DATASET_PARENTS: u32 = 256;

pub fn get_epoch(height: u64) -> u64 {
    height / EPOCH_LENGTH
}

pub fn get_cache_size(epoch: u64) -> usize {
    CACHE_BYTES_INIT + CACHE_BYTES_GROWTH * epoch as usize
}

pub fn get_dag_size(epoch: u64) -> usize {
    DAG_BYTES_INIT + DAG_BYTES_GROWTH * epoch as usize
}

pub fn compute_seed(epoch: u64) -> [u8; 32] {
    let mut seed = [0u8; 32];
    for _ in 0..epoch {
        let mut hasher = Keccak256::new();
        hasher.update(&seed);
        seed.copy_from_slice(&hasher.finalize());
    }
    seed
}

pub fn generate_cache(seed: &[u8; 32], cache_size: usize) -> Vec<[u8; 64]> {
    let num_items = cache_size / 64;
    let mut cache = Vec::with_capacity(num_items);

    // Initialize first item
    let mut hasher = Keccak512::new();
    hasher.update(seed);
    let mut item = [0u8; 64];
    item.copy_from_slice(&hasher.finalize());
    cache.push(item);

    // Generate remaining items
    for i in 1..num_items {
        let mut hasher = Keccak512::new();
        hasher.update(&cache[i - 1]);
        let mut item = [0u8; 64];
        item.copy_from_slice(&hasher.finalize());
        cache.push(item);
    }

    // RandMemoHash passes
    for _ in 0..3 {
        for i in 0..num_items {
            let v = u32::from_le_bytes(cache[i][0..4].try_into().unwrap()) as usize % num_items;
            let mut hasher = Keccak512::new();
            
            let prev_idx = if i == 0 { num_items - 1 } else { i - 1 };
            for j in 0..64 {
                hasher.update(&[cache[prev_idx][j] ^ cache[v][j]]);
            }
            
            cache[i].copy_from_slice(&hasher.finalize());
        }
    }

    cache
}

pub fn kawpow_hash(
    header_hash: &[u8; 32],
    nonce: u64,
    height: u64,
    cache: &[[u8; 64]],
) -> [u8; 32] {
    let seed = compute_mix_seed(header_hash, nonce);
    
    let mut mix = [[0u32; 32]; PROGPOW_LANES];
    for lane in 0..PROGPOW_LANES {
        mix[lane] = fill_mix(&seed, lane as u32);
    }

    let epoch = get_epoch(height);
    let dag_size = get_dag_size(epoch);
    let num_dag_items = dag_size / 256;

    // ProgPoW main loop
    for round in 0..PROGPOW_CNT_DAG {
        // Generate random program for this round
        let prog_seed = fnv1a(&seed, round as u32);
        
        // DAG accesses
        for lane in 0..PROGPOW_LANES {
            let dag_index = (mix[lane][0] as usize % num_dag_items) * 4;
            
            // Simulate DAG lookup using cache
            let cache_index = dag_index % cache.len();
            let dag_data = calculate_dag_item(cache, cache_index);
            
            for i in 0..32 {
                mix[lane][i] = fnv1a(&mix[lane][i].to_le_bytes(), 
                    u32::from_le_bytes(dag_data[i % 16].to_le_bytes()));
            }
        }

        // Math operations
        for _ in 0..PROGPOW_CNT_MATH {
            for lane in 0..PROGPOW_LANES {
                let src1 = mix[lane][prog_seed as usize % 32];
                let src2 = mix[lane][(prog_seed as usize + 1) % 32];
                let dst_idx = (prog_seed as usize + 2) % 32;
                
                mix[lane][dst_idx] = progpow_math(src1, src2, prog_seed);
            }
        }
    }

    // Final mix
    let mut digest = [0u32; 8];
    for lane in 0..PROGPOW_LANES {
        for i in 0..8 {
            digest[i] = fnv1a(&digest[i].to_le_bytes(), mix[lane][i]);
        }
    }

    // Convert to bytes
    let mut result = [0u8; 32];
    for (i, &val) in digest.iter().enumerate() {
        result[i * 4..(i + 1) * 4].copy_from_slice(&val.to_le_bytes());
    }

    // Final Keccak
    let mut hasher = Keccak256::new();
    hasher.update(header_hash);
    hasher.update(&nonce.to_le_bytes());
    hasher.update(&result);
    let mut final_hash = [0u8; 32];
    final_hash.copy_from_slice(&hasher.finalize());
    
    final_hash
}

fn compute_mix_seed(header_hash: &[u8; 32], nonce: u64) -> [u8; 64] {
    let mut hasher = Keccak512::new();
    hasher.update(header_hash);
    hasher.update(&nonce.to_le_bytes());
    let mut seed = [0u8; 64];
    seed.copy_from_slice(&hasher.finalize());
    seed
}

fn fill_mix(seed: &[u8; 64], lane: u32) -> [u32; 32] {
    let mut mix = [0u32; 32];
    for i in 0..32 {
        let idx = (i * 4) % 64;
        let base = u32::from_le_bytes(seed[idx..idx + 4].try_into().unwrap());
        mix[i] = fnv1a(&base.to_le_bytes(), lane ^ i as u32);
    }
    mix
}

fn calculate_dag_item(cache: &[[u8; 64]], index: usize) -> [u32; 16] {
    let mut item = [0u32; 16];
    let cache_item = &cache[index % cache.len()];
    
    for i in 0..16 {
        let idx = i * 4;
        item[i] = u32::from_le_bytes(cache_item[idx..idx + 4].try_into().unwrap());
    }
    
    item
}

fn fnv1a(data: &[u8], val: u32) -> u32 {
    const FNV_PRIME: u32 = 0x01000193;
    let mut hash = val;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn progpow_math(a: u32, b: u32, r: u32) -> u32 {
    match r % 11 {
        0 => a.wrapping_add(b),
        1 => a.wrapping_mul(b),
        2 => (a as u64 * b as u64 >> 32) as u32,
        3 => a.min(b),
        4 => a.rotate_left(b % 32),
        5 => a.rotate_right(b % 32),
        6 => a & b,
        7 => a | b,
        8 => a ^ b,
        9 => a.leading_zeros() + b.leading_zeros(),
        10 => a.count_ones() + b.count_ones(),
        _ => a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_calculation() {
        assert_eq!(get_epoch(0), 0);
        assert_eq!(get_epoch(7499), 0);
        assert_eq!(get_epoch(7500), 1);
        assert_eq!(get_epoch(15000), 2);
    }

    #[test]
    fn test_cache_generation() {
        let seed = compute_seed(0);
        let cache = generate_cache(&seed, 1024 * 64);
        assert_eq!(cache.len(), 1024);
    }
}
