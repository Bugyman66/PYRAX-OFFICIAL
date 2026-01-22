//! Bloom Filter for Address Filtering

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Bloom filter for efficient set membership testing
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<bool>,
    size: usize,
    hash_count: usize,
}

impl BloomFilter {
    /// Create a new bloom filter with given size and hash count
    pub fn new(size: usize, hash_count: usize) -> Self {
        Self {
            bits: vec![false; size],
            size,
            hash_count,
        }
    }
    
    /// Create bloom filter optimized for expected items and false positive rate
    pub fn with_fp_rate(expected_items: usize, fp_rate: f64) -> Self {
        let size = optimal_size(expected_items, fp_rate);
        let hash_count = optimal_hash_count(size, expected_items);
        Self::new(size, hash_count)
    }
    
    /// Add an item to the filter
    pub fn add<T: Hash>(&mut self, item: &T) {
        for i in 0..self.hash_count {
            let idx = self.hash_index(item, i);
            self.bits[idx] = true;
        }
    }
    
    /// Check if item might be in the set
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        for i in 0..self.hash_count {
            let idx = self.hash_index(item, i);
            if !self.bits[idx] {
                return false;
            }
        }
        true
    }
    
    /// Clear the filter
    pub fn clear(&mut self) {
        self.bits.fill(false);
    }
    
    /// Get filter as bytes for network transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((self.size + 7) / 8);
        for chunk in self.bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << i;
                }
            }
            bytes.push(byte);
        }
        bytes
    }
    
    /// Create filter from bytes
    pub fn from_bytes(bytes: &[u8], hash_count: usize) -> Self {
        let mut bits = Vec::with_capacity(bytes.len() * 8);
        for byte in bytes {
            for i in 0..8 {
                bits.push((byte >> i) & 1 == 1);
            }
        }
        Self {
            size: bits.len(),
            bits,
            hash_count,
        }
    }
    
    fn hash_index<T: Hash>(&self, item: &T, seed: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        seed.hash(&mut hasher);
        (hasher.finish() as usize) % self.size
    }
}

/// Calculate optimal bloom filter size
fn optimal_size(items: usize, fp_rate: f64) -> usize {
    let ln2_sq = std::f64::consts::LN_2.powi(2);
    let size = -(items as f64 * fp_rate.ln()) / ln2_sq;
    size.ceil() as usize
}

/// Calculate optimal number of hash functions
fn optimal_hash_count(size: usize, items: usize) -> usize {
    let count = (size as f64 / items as f64) * std::f64::consts::LN_2;
    count.ceil() as usize
}

/// Address filter for watching specific addresses
#[derive(Debug, Clone)]
pub struct AddressFilter {
    filter: BloomFilter,
    addresses: Vec<[u8; 20]>,
}

impl AddressFilter {
    pub fn new(fp_rate: f64) -> Self {
        Self {
            filter: BloomFilter::with_fp_rate(100, fp_rate),
            addresses: Vec::new(),
        }
    }
    
    pub fn add_address(&mut self, address: &[u8; 20]) {
        self.filter.add(address);
        self.addresses.push(*address);
    }
    
    pub fn might_contain(&self, address: &[u8; 20]) -> bool {
        self.filter.contains(address)
    }
    
    pub fn definitely_contains(&self, address: &[u8; 20]) -> bool {
        self.addresses.iter().any(|a| a == address)
    }
    
    pub fn get_filter_bytes(&self) -> Vec<u8> {
        self.filter.to_bytes()
    }
    
    pub fn watched_count(&self) -> usize {
        self.addresses.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bloom_filter_basic() {
        let mut filter = BloomFilter::new(1000, 3);
        
        filter.add(&"hello");
        filter.add(&"world");
        
        assert!(filter.contains(&"hello"));
        assert!(filter.contains(&"world"));
        // Might have false positives but very unlikely for this test
    }
    
    #[test]
    fn test_bloom_filter_serialization() {
        let mut filter = BloomFilter::new(64, 3);
        filter.add(&42u64);
        
        let bytes = filter.to_bytes();
        let restored = BloomFilter::from_bytes(&bytes, 3);
        
        assert!(restored.contains(&42u64));
    }
    
    #[test]
    fn test_address_filter() {
        let mut filter = AddressFilter::new(0.001);
        let addr = [1u8; 20];
        
        filter.add_address(&addr);
        
        assert!(filter.might_contain(&addr));
        assert!(filter.definitely_contains(&addr));
        assert!(!filter.definitely_contains(&[2u8; 20]));
    }
}
