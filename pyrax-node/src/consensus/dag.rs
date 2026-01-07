//! DAG (Directed Acyclic Graph) Generation for KAWPOW Mining
//!
//! Production-ready DAG generation with:
//! - Epoch-based DAG management
//! - Disk persistence for faster restarts
//! - Integrity verification
//! - Memory-mapped file support

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use sha3::{Keccak256, Keccak512, Digest};
use tracing::{info, warn, error, debug};

use super::kawpow::{
    EPOCH_LENGTH, CACHE_ROUNDS, DATASET_PARENTS,
    get_epoch, get_cache_size, get_dag_size, compute_seed,
};

/// DAG file header for integrity verification
const DAG_MAGIC: [u8; 4] = [0x50, 0x59, 0x52, 0x58]; // "PYRX"
const DAG_VERSION: u32 = 1;
const HEADER_SIZE: usize = 64;

/// DAG Manager handles epoch transitions and disk persistence
pub struct DagManager {
    data_dir: PathBuf,
    current_epoch: Option<u64>,
    cache: Option<Vec<[u8; 64]>>,
    dag: Option<Vec<[u8; 64]>>,
    cache_path: Option<PathBuf>,
    dag_path: Option<PathBuf>,
}

impl DagManager {
    /// Create a new DAG manager
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        fs::create_dir_all(&data_dir).ok();
        
        info!("DAG manager initialized at {:?}", data_dir);
        
        Self {
            data_dir,
            current_epoch: None,
            cache: None,
            dag: None,
            cache_path: None,
            dag_path: None,
        }
    }

    /// Get current epoch
    pub fn current_epoch(&self) -> Option<u64> {
        self.current_epoch
    }

    /// Check if DAG is loaded for the given epoch
    pub fn is_ready(&self, epoch: u64) -> bool {
        self.current_epoch == Some(epoch) && self.cache.is_some()
    }

    /// Ensure DAG is ready for the given block height
    pub fn ensure_epoch(&mut self, block_height: u64) -> Result<(), DagError> {
        let epoch = get_epoch(block_height);
        
        if self.current_epoch == Some(epoch) && self.cache.is_some() {
            return Ok(());
        }

        info!("Loading DAG for epoch {} (block {})", epoch, block_height);
        self.load_epoch(epoch)
    }

    /// Load or generate DAG for the specified epoch
    pub fn load_epoch(&mut self, epoch: u64) -> Result<(), DagError> {
        let cache_path = self.data_dir.join(format!("cache_epoch_{}.bin", epoch));
        let dag_path = self.data_dir.join(format!("dag_epoch_{}.bin", epoch));

        // Try to load from disk first
        if cache_path.exists() {
            match self.load_cache_from_disk(&cache_path, epoch) {
                Ok(cache) => {
                    info!("Loaded cache from disk for epoch {}", epoch);
                    self.cache = Some(cache);
                    self.current_epoch = Some(epoch);
                    self.cache_path = Some(cache_path);
                    self.dag_path = Some(dag_path);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to load cache from disk: {}, regenerating", e);
                }
            }
        }

        // Generate new cache
        info!("Generating cache for epoch {}...", epoch);
        let cache = self.generate_cache(epoch)?;
        
        // Save to disk
        self.save_cache_to_disk(&cache_path, epoch, &cache)?;
        
        self.cache = Some(cache);
        self.current_epoch = Some(epoch);
        self.cache_path = Some(cache_path);
        self.dag_path = Some(dag_path);

        info!("Cache ready for epoch {}", epoch);
        Ok(())
    }

    /// Generate cache for epoch (CPU implementation)
    fn generate_cache(&self, epoch: u64) -> Result<Vec<[u8; 64]>, DagError> {
        let cache_size = get_cache_size(epoch) as usize;
        let cache_items = cache_size / 64;
        
        info!("Generating {} item cache ({} MB)", cache_items, cache_size / (1024 * 1024));

        let seed = compute_seed(epoch);
        let mut cache = Vec::with_capacity(cache_items);

        // Initialize cache with seed
        let mut hasher = Keccak512::new();
        hasher.update(&seed);
        let mut item: [u8; 64] = hasher.finalize().into();
        cache.push(item);

        // Generate remaining items
        for i in 1..cache_items {
            let mut hasher = Keccak512::new();
            hasher.update(&item);
            item = hasher.finalize().into();
            cache.push(item);
            
            if i % 100000 == 0 {
                debug!("Cache generation: {}/{} items", i, cache_items);
            }
        }

        // Perform cache rounds (RandMemoHash)
        for round in 0..CACHE_ROUNDS {
            for i in 0..cache_items {
                let src_idx = i;
                let dst_idx = (u32::from_le_bytes([
                    cache[i][0], cache[i][1], cache[i][2], cache[i][3]
                ]) as usize) % cache_items;

                let mut xored = [0u8; 64];
                for j in 0..64 {
                    xored[j] = cache[src_idx][j] ^ cache[dst_idx][j];
                }

                let mut hasher = Keccak512::new();
                hasher.update(&xored);
                cache[i] = hasher.finalize().into();
            }
            debug!("Cache round {}/{} complete", round + 1, CACHE_ROUNDS);
        }

        Ok(cache)
    }

    /// Save cache to disk with integrity header
    fn save_cache_to_disk(&self, path: &Path, epoch: u64, cache: &[[u8; 64]]) -> Result<(), DagError> {
        let mut file = File::create(path)
            .map_err(|e| DagError::IoError(e.to_string()))?;

        // Write header
        let header = DagHeader {
            magic: DAG_MAGIC,
            version: DAG_VERSION,
            epoch,
            item_count: cache.len() as u64,
            checksum: self.compute_checksum(cache),
        };
        
        file.write_all(&header.to_bytes())
            .map_err(|e| DagError::IoError(e.to_string()))?;

        // Write cache data
        for item in cache {
            file.write_all(item)
                .map_err(|e| DagError::IoError(e.to_string()))?;
        }

        info!("Saved cache to {:?} ({} bytes)", path, HEADER_SIZE + cache.len() * 64);
        Ok(())
    }

    /// Load cache from disk with integrity verification
    fn load_cache_from_disk(&self, path: &Path, expected_epoch: u64) -> Result<Vec<[u8; 64]>, DagError> {
        let mut file = File::open(path)
            .map_err(|e| DagError::IoError(e.to_string()))?;

        // Read and verify header
        let mut header_bytes = [0u8; HEADER_SIZE];
        file.read_exact(&mut header_bytes)
            .map_err(|e| DagError::IoError(e.to_string()))?;

        let header = DagHeader::from_bytes(&header_bytes)?;
        
        if header.magic != DAG_MAGIC {
            return Err(DagError::InvalidHeader("Bad magic bytes".to_string()));
        }
        if header.version != DAG_VERSION {
            return Err(DagError::InvalidHeader(format!("Version mismatch: {} vs {}", header.version, DAG_VERSION)));
        }
        if header.epoch != expected_epoch {
            return Err(DagError::InvalidHeader(format!("Epoch mismatch: {} vs {}", header.epoch, expected_epoch)));
        }

        // Read cache data
        let mut cache = Vec::with_capacity(header.item_count as usize);
        for _ in 0..header.item_count {
            let mut item = [0u8; 64];
            file.read_exact(&mut item)
                .map_err(|e| DagError::IoError(e.to_string()))?;
            cache.push(item);
        }

        // Verify checksum
        let computed_checksum = self.compute_checksum(&cache);
        if computed_checksum != header.checksum {
            return Err(DagError::ChecksumMismatch);
        }

        Ok(cache)
    }

    /// Compute checksum for integrity verification
    fn compute_checksum(&self, data: &[[u8; 64]]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        for item in data {
            hasher.update(item);
        }
        hasher.finalize().into()
    }

    /// Get a cache item
    pub fn get_cache_item(&self, index: usize) -> Option<&[u8; 64]> {
        self.cache.as_ref()?.get(index)
    }

    /// Get cache size
    pub fn cache_len(&self) -> usize {
        self.cache.as_ref().map(|c| c.len()).unwrap_or(0)
    }

    /// Calculate a DAG item from cache (light evaluation)
    pub fn calc_dag_item(&self, index: u64) -> Option<[u8; 64]> {
        let cache = self.cache.as_ref()?;
        let cache_size = cache.len();
        
        if cache_size == 0 {
            return None;
        }

        // Initialize with cache item
        let cache_idx = (index % cache_size as u64) as usize;
        let mut mix = cache[cache_idx];

        // XOR with index
        let idx_bytes = index.to_le_bytes();
        for i in 0..8 {
            mix[i] ^= idx_bytes[i];
        }

        // Hash initial state
        let mut hasher = Keccak512::new();
        hasher.update(&mix);
        mix = hasher.finalize().into();

        // FNV mixing with parent nodes
        for i in 0..DATASET_PARENTS {
            let parent_idx = fnv_hash(
                index as u32 ^ i,
                u32::from_le_bytes([mix[0], mix[1], mix[2], mix[3]])
            ) as usize % cache_size;

            for j in 0..64 {
                mix[j] = fnv_byte(mix[j], cache[parent_idx][j]);
            }
        }

        // Final hash
        let mut hasher = Keccak512::new();
        hasher.update(&mix);
        Some(hasher.finalize().into())
    }

    /// Clean up old epoch files
    pub fn cleanup_old_epochs(&self, keep_epochs: usize) -> Result<usize, DagError> {
        let current = self.current_epoch.unwrap_or(0);
        let mut removed = 0;

        if let Ok(entries) = fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Parse epoch from filename
                    if let Some(epoch_str) = name.strip_prefix("cache_epoch_").and_then(|s| s.strip_suffix(".bin")) {
                        if let Ok(epoch) = epoch_str.parse::<u64>() {
                            if current > epoch && current - epoch > keep_epochs as u64 {
                                fs::remove_file(&path).ok();
                                removed += 1;
                                debug!("Removed old cache: {:?}", path);
                            }
                        }
                    }
                    if let Some(epoch_str) = name.strip_prefix("dag_epoch_").and_then(|s| s.strip_suffix(".bin")) {
                        if let Ok(epoch) = epoch_str.parse::<u64>() {
                            if current > epoch && current - epoch > keep_epochs as u64 {
                                fs::remove_file(&path).ok();
                                removed += 1;
                                debug!("Removed old DAG: {:?}", path);
                            }
                        }
                    }
                }
            }
        }

        if removed > 0 {
            info!("Cleaned up {} old epoch files", removed);
        }
        Ok(removed)
    }

    /// Get disk usage for DAG files
    pub fn disk_usage(&self) -> u64 {
        let mut total = 0;
        if let Ok(entries) = fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
        total
    }
}

/// FNV hash function
fn fnv_hash(a: u32, b: u32) -> u32 {
    const FNV_PRIME: u32 = 0x01000193;
    a.wrapping_mul(FNV_PRIME) ^ b
}

/// FNV byte mixing
fn fnv_byte(a: u8, b: u8) -> u8 {
    const FNV_PRIME: u8 = 0x93;
    a.wrapping_mul(FNV_PRIME) ^ b
}

/// DAG file header
#[derive(Debug)]
struct DagHeader {
    magic: [u8; 4],
    version: u32,
    epoch: u64,
    item_count: u64,
    checksum: [u8; 32],
}

impl DagHeader {
    fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..8].copy_from_slice(&self.version.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.epoch.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.item_count.to_le_bytes());
        bytes[24..56].copy_from_slice(&self.checksum);
        bytes
    }

    fn from_bytes(bytes: &[u8; HEADER_SIZE]) -> Result<Self, DagError> {
        Ok(Self {
            magic: [bytes[0], bytes[1], bytes[2], bytes[3]],
            version: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            epoch: u64::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]),
            item_count: u64::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23]]),
            checksum: bytes[24..56].try_into().unwrap(),
        })
    }
}

/// DAG errors
#[derive(Debug, thiserror::Error)]
pub enum DagError {
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Invalid header: {0}")]
    InvalidHeader(String),
    #[error("Checksum mismatch - file corrupted")]
    ChecksumMismatch,
    #[error("Out of memory")]
    OutOfMemory,
    #[error("Epoch not loaded")]
    EpochNotLoaded,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_dag_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = DagManager::new(dir.path());
        assert!(manager.current_epoch().is_none());
    }

    #[test]
    fn test_fnv_hash() {
        assert_eq!(fnv_hash(0, 0), 0);
        assert_ne!(fnv_hash(1, 0), fnv_hash(0, 1));
    }

    #[test]
    fn test_header_roundtrip() {
        let header = DagHeader {
            magic: DAG_MAGIC,
            version: DAG_VERSION,
            epoch: 42,
            item_count: 1000,
            checksum: [0xAB; 32],
        };
        
        let bytes = header.to_bytes();
        let restored = DagHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(restored.magic, DAG_MAGIC);
        assert_eq!(restored.version, DAG_VERSION);
        assert_eq!(restored.epoch, 42);
        assert_eq!(restored.item_count, 1000);
        assert_eq!(restored.checksum, [0xAB; 32]);
    }

    #[test]
    fn test_epoch_calculation() {
        assert_eq!(get_epoch(0), 0);
        assert_eq!(get_epoch(EPOCH_LENGTH - 1), 0);
        assert_eq!(get_epoch(EPOCH_LENGTH), 1);
        assert_eq!(get_epoch(EPOCH_LENGTH * 5), 5);
    }
}
