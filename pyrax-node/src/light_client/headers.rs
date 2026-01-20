//! Light Header Chain for SPV clients

use std::collections::HashMap;

/// Lightweight block header (minimal fields for SPV)
#[derive(Debug, Clone)]
pub struct LightHeader {
    pub height: u64,
    pub hash: [u8; 32],
    pub parent_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u64,
    pub difficulty: u64,
    pub nonce: u64,
}

impl LightHeader {
    pub fn genesis() -> Self {
        Self {
            height: 0,
            hash: [0u8; 32],
            parent_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: 0,
            difficulty: 1,
            nonce: 0,
        }
    }
}

/// Header chain for light clients
pub struct HeaderChain {
    headers: Vec<LightHeader>,
    by_hash: HashMap<[u8; 32], usize>,
    max_headers: usize,
    total_difficulty: u128,
}

impl HeaderChain {
    pub fn new(max_headers: usize) -> Self {
        let genesis = LightHeader::genesis();
        let mut by_hash = HashMap::new();
        by_hash.insert(genesis.hash, 0);
        
        Self {
            headers: vec![genesis],
            by_hash,
            max_headers,
            total_difficulty: 1,
        }
    }
    
    /// Get current chain height
    pub fn height(&self) -> u64 {
        self.headers.last().map(|h| h.height).unwrap_or(0)
    }
    
    /// Get total accumulated difficulty
    pub fn total_difficulty(&self) -> u128 {
        self.total_difficulty
    }
    
    /// Add a new header to the chain
    pub fn add_header(&mut self, header: LightHeader) -> Result<(), String> {
        // Check if already have this header
        if self.by_hash.contains_key(&header.hash) {
            return Ok(());
        }
        
        // Verify parent exists
        if !self.by_hash.contains_key(&header.parent_hash) && header.height > 0 {
            return Err("Parent header not found".to_string());
        }
        
        // Verify height is sequential
        let expected_height = self.height() + 1;
        if header.height != expected_height && header.height > 0 {
            return Err(format!("Expected height {}, got {}", expected_height, header.height));
        }
        
        // Add header
        let idx = self.headers.len();
        self.by_hash.insert(header.hash, idx);
        self.total_difficulty += header.difficulty as u128;
        self.headers.push(header);
        
        // Prune old headers if needed
        self.prune();
        
        Ok(())
    }
    
    /// Get header by hash
    pub fn get_by_hash(&self, hash: &[u8; 32]) -> Option<&LightHeader> {
        self.by_hash.get(hash).map(|&idx| &self.headers[idx])
    }
    
    /// Get header at specific height
    pub fn get_at_height(&self, height: u64) -> Option<&LightHeader> {
        if height == 0 {
            return self.headers.first();
        }
        
        // Calculate offset from start (accounting for pruning)
        let start_height = self.headers.first().map(|h| h.height).unwrap_or(0);
        if height < start_height {
            return None;
        }
        
        let idx = (height - start_height) as usize;
        self.headers.get(idx)
    }
    
    /// Get the tip header
    pub fn tip(&self) -> Option<&LightHeader> {
        self.headers.last()
    }
    
    /// Get headers in range
    pub fn get_range(&self, start: u64, count: usize) -> Vec<&LightHeader> {
        let start_height = self.headers.first().map(|h| h.height).unwrap_or(0);
        if start < start_height {
            return Vec::new();
        }
        
        let start_idx = (start - start_height) as usize;
        self.headers.iter().skip(start_idx).take(count).collect()
    }
    
    /// Prune old headers to stay within limit
    fn prune(&mut self) {
        if self.headers.len() > self.max_headers {
            let prune_count = self.headers.len() - self.max_headers;
            
            // Remove from hash index
            for header in self.headers.drain(..prune_count) {
                self.by_hash.remove(&header.hash);
            }
            
            // Rebuild index with new positions
            self.by_hash.clear();
            for (idx, header) in self.headers.iter().enumerate() {
                self.by_hash.insert(header.hash, idx);
            }
        }
    }
    
    /// Verify header connects to chain
    pub fn verify_connection(&self, header: &LightHeader) -> bool {
        if header.height == 0 {
            return true;
        }
        self.by_hash.contains_key(&header.parent_hash)
    }
    
    /// Get locator hashes for sync
    pub fn get_locator(&self) -> Vec<[u8; 32]> {
        let mut locator = Vec::new();
        let mut step = 1usize;
        let mut height = self.height();
        
        while height > 0 {
            if let Some(header) = self.get_at_height(height) {
                locator.push(header.hash);
            }
            
            if locator.len() >= 10 {
                step *= 2;
            }
            
            if height < step as u64 {
                break;
            }
            height -= step as u64;
        }
        
        // Always include genesis
        if let Some(genesis) = self.headers.first() {
            if locator.last() != Some(&genesis.hash) {
                locator.push(genesis.hash);
            }
        }
        
        locator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_header_chain_creation() {
        let chain = HeaderChain::new(1000);
        assert_eq!(chain.height(), 0);
        assert!(chain.tip().is_some());
    }
    
    #[test]
    fn test_add_headers() {
        let mut chain = HeaderChain::new(1000);
        let genesis_hash = chain.tip().unwrap().hash;
        
        let header1 = LightHeader {
            height: 1,
            hash: [1u8; 32],
            parent_hash: genesis_hash,
            merkle_root: [0u8; 32],
            timestamp: 1000,
            difficulty: 1,
            nonce: 0,
        };
        
        assert!(chain.add_header(header1).is_ok());
        assert_eq!(chain.height(), 1);
    }
    
    #[test]
    fn test_header_pruning() {
        let mut chain = HeaderChain::new(5);
        let mut parent = chain.tip().unwrap().hash;
        
        for i in 1..=10 {
            let header = LightHeader {
                height: i,
                hash: [i as u8; 32],
                parent_hash: parent,
                merkle_root: [0u8; 32],
                timestamp: 1000 * i,
                difficulty: 1,
                nonce: 0,
            };
            parent = header.hash;
            chain.add_header(header).unwrap();
        }
        
        assert_eq!(chain.height(), 10);
        assert!(chain.headers.len() <= 5);
    }
    
    #[test]
    fn test_locator() {
        let mut chain = HeaderChain::new(100);
        let mut parent = chain.tip().unwrap().hash;
        
        for i in 1..=20 {
            let header = LightHeader {
                height: i,
                hash: [i as u8; 32],
                parent_hash: parent,
                merkle_root: [0u8; 32],
                timestamp: 1000 * i,
                difficulty: 1,
                nonce: 0,
            };
            parent = header.hash;
            chain.add_header(header).unwrap();
        }
        
        let locator = chain.get_locator();
        assert!(!locator.is_empty());
    }
}
