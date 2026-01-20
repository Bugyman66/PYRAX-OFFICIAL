//! Persistent Peer Cache - Saves known peers to disk for faster reconnection
//!
//! DESIGN DOC:
//! -----------
//! The PeerCache persists discovered peers to disk so that:
//! - Nodes reconnect faster after restart (dial cached peers before bootnodes)
//! - Network is more resilient (less dependency on bootnodes for initial connections)
//! - Users experience faster sync times
//!
//! Cache format: JSON array of peer entries with addresses and metadata
//! Location: <data_dir>/peer_cache.json
//! Max entries: 200 (oldest entries pruned on save)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Maximum number of peers to cache
const MAX_CACHED_PEERS: usize = 200;

/// Minimum time a peer must be connected before being cached (5 minutes)
const MIN_CONNECTION_TIME_SECS: u64 = 300;

/// Maximum age of a cached peer entry before it's considered stale (7 days)
const MAX_CACHE_AGE_SECS: u64 = 7 * 24 * 60 * 60;

/// A cached peer entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPeer {
    /// Peer ID (libp2p format)
    pub peer_id: String,
    /// Known addresses for this peer
    pub addresses: Vec<String>,
    /// Last successful connection timestamp (Unix epoch seconds)
    pub last_connected: u64,
    /// Number of successful connections to this peer
    pub success_count: u32,
    /// Average RTT in milliseconds (if known)
    pub avg_rtt_ms: Option<u64>,
    /// Is this a bootnode?
    pub is_bootnode: bool,
}

impl CachedPeer {
    pub fn new(peer_id: String, addresses: Vec<String>) -> Self {
        Self {
            peer_id,
            addresses,
            last_connected: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            success_count: 1,
            avg_rtt_ms: None,
            is_bootnode: false,
        }
    }
    
    /// Check if this entry is stale (too old)
    pub fn is_stale(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now - self.last_connected > MAX_CACHE_AGE_SECS
    }
    
    /// Update the entry with a new successful connection
    pub fn record_success(&mut self, rtt_ms: Option<u64>) {
        self.last_connected = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.success_count += 1;
        
        // Update RTT with exponential moving average
        if let Some(new_rtt) = rtt_ms {
            self.avg_rtt_ms = Some(match self.avg_rtt_ms {
                Some(old_rtt) => (old_rtt * 7 + new_rtt * 3) / 10, // 70% old, 30% new
                None => new_rtt,
            });
        }
    }
}

/// Persistent peer cache
pub struct PeerCache {
    /// Path to the cache file
    cache_path: PathBuf,
    /// In-memory cache (peer_id -> CachedPeer)
    peers: HashMap<String, CachedPeer>,
    /// Whether the cache has been modified since last save
    dirty: bool,
}

impl PeerCache {
    /// Create a new peer cache
    pub fn new(data_dir: &PathBuf) -> Self {
        let cache_path = data_dir.join("peer_cache.json");
        Self {
            cache_path,
            peers: HashMap::new(),
            dirty: false,
        }
    }
    
    /// Load the cache from disk
    pub fn load(&mut self) -> Result<usize, std::io::Error> {
        if !self.cache_path.exists() {
            debug!("No peer cache file found at {:?}", self.cache_path);
            return Ok(0);
        }
        
        let content = std::fs::read_to_string(&self.cache_path)?;
        let cached_peers: Vec<CachedPeer> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        // Filter out stale entries
        let valid_peers: Vec<CachedPeer> = cached_peers
            .into_iter()
            .filter(|p| !p.is_stale())
            .collect();
        
        let count = valid_peers.len();
        
        for peer in valid_peers {
            self.peers.insert(peer.peer_id.clone(), peer);
        }
        
        info!("Loaded {} peers from cache at {:?}", count, self.cache_path);
        Ok(count)
    }
    
    /// Save the cache to disk
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if !self.dirty {
            return Ok(());
        }
        
        // Filter out stale entries and limit to MAX_CACHED_PEERS
        let mut peers: Vec<&CachedPeer> = self.peers.values()
            .filter(|p| !p.is_stale())
            .collect();
        
        // Sort by success_count * recency (prefer reliable, recent peers)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        peers.sort_by(|a, b| {
            let a_score = a.success_count as u64 * 1000 / (now - a.last_connected + 1);
            let b_score = b.success_count as u64 * 1000 / (now - b.last_connected + 1);
            b_score.cmp(&a_score)
        });
        
        // Take top MAX_CACHED_PEERS
        let peers_to_save: Vec<CachedPeer> = peers
            .into_iter()
            .take(MAX_CACHED_PEERS)
            .cloned()
            .collect();
        
        // Ensure parent directory exists
        if let Some(parent) = self.cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(&peers_to_save)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        std::fs::write(&self.cache_path, content)?;
        
        self.dirty = false;
        info!("Saved {} peers to cache at {:?}", peers_to_save.len(), self.cache_path);
        Ok(())
    }
    
    /// Add or update a peer in the cache
    pub fn upsert_peer(&mut self, peer_id: String, addresses: Vec<String>, rtt_ms: Option<u64>, is_bootnode: bool) {
        // Filter to only routable addresses
        let routable_addrs: Vec<String> = addresses
            .into_iter()
            .filter(|addr| Self::is_routable_address(addr))
            .collect();
        
        if routable_addrs.is_empty() && !is_bootnode {
            return; // Don't cache peers without routable addresses
        }
        
        if let Some(existing) = self.peers.get_mut(&peer_id) {
            // Update existing entry
            existing.record_success(rtt_ms);
            // Merge addresses (keep unique)
            for addr in routable_addrs {
                if !existing.addresses.contains(&addr) {
                    existing.addresses.push(addr);
                }
            }
            // Limit addresses per peer
            if existing.addresses.len() > 10 {
                existing.addresses.truncate(10);
            }
        } else {
            // Create new entry
            let mut peer = CachedPeer::new(peer_id.clone(), routable_addrs);
            peer.avg_rtt_ms = rtt_ms;
            peer.is_bootnode = is_bootnode;
            self.peers.insert(peer_id, peer);
        }
        
        self.dirty = true;
    }
    
    /// Get all cached peers (sorted by reliability score)
    pub fn get_peers(&self) -> Vec<&CachedPeer> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let mut peers: Vec<&CachedPeer> = self.peers.values()
            .filter(|p| !p.is_stale())
            .collect();
        
        // Sort by reliability score (success_count * recency)
        peers.sort_by(|a, b| {
            let a_score = a.success_count as u64 * 1000 / (now - a.last_connected + 1);
            let b_score = b.success_count as u64 * 1000 / (now - b.last_connected + 1);
            b_score.cmp(&a_score)
        });
        
        peers
    }
    
    /// Get cached peers to dial on startup (excludes bootnodes)
    pub fn get_startup_peers(&self, limit: usize) -> Vec<(String, Vec<String>)> {
        self.get_peers()
            .into_iter()
            .filter(|p| !p.is_bootnode)
            .take(limit)
            .map(|p| (p.peer_id.clone(), p.addresses.clone()))
            .collect()
    }
    
    /// Remove a peer from the cache (e.g., if it's been banned)
    pub fn remove_peer(&mut self, peer_id: &str) {
        if self.peers.remove(peer_id).is_some() {
            self.dirty = true;
        }
    }
    
    /// Check if an address is routable (not localhost/private)
    fn is_routable_address(address: &str) -> bool {
        // Reject bare /p2p/ addresses without IP
        if address.starts_with("/p2p/") && !address.contains("/ip4/") && !address.contains("/ip6/") {
            return false;
        }
        
        // Reject nested relay addresses
        if address.matches("/p2p-circuit").count() > 1 {
            return false;
        }
        
        // Allow single relay addresses (they're valid for NAT traversal)
        if address.contains("/p2p-circuit") {
            return true;
        }
        
        // Check for private IP ranges
        if address.contains("/ip4/127.") 
            || address.contains("/ip4/10.")
            || address.contains("/ip4/192.168.")
            || address.contains("/ip4/172.16.")
            || address.contains("/ip4/172.17.")
            || address.contains("/ip4/172.18.")
            || address.contains("/ip4/172.19.")
            || address.contains("/ip4/172.20.")
            || address.contains("/ip4/172.21.")
            || address.contains("/ip4/172.22.")
            || address.contains("/ip4/172.23.")
            || address.contains("/ip4/172.24.")
            || address.contains("/ip4/172.25.")
            || address.contains("/ip4/172.26.")
            || address.contains("/ip4/172.27.")
            || address.contains("/ip4/172.28.")
            || address.contains("/ip4/172.29.")
            || address.contains("/ip4/172.30.")
            || address.contains("/ip4/172.31.")
            || address.contains("/ip4/169.254.")
            || address.contains("/ip4/0.0.0.0")
        {
            return false;
        }
        
        true
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        let total = self.peers.len();
        let valid = self.peers.values().filter(|p| !p.is_stale()).count();
        (total, valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_peer_cache_basic() {
        let dir = tempdir().unwrap();
        let mut cache = PeerCache::new(&dir.path().to_path_buf());
        
        // Add a peer
        cache.upsert_peer(
            "12D3KooWTest123".to_string(),
            vec!["/ip4/1.2.3.4/tcp/30303".to_string()],
            Some(50),
            false,
        );
        
        assert_eq!(cache.peers.len(), 1);
        
        // Save and reload
        cache.save().unwrap();
        
        let mut cache2 = PeerCache::new(&dir.path().to_path_buf());
        let count = cache2.load().unwrap();
        assert_eq!(count, 1);
    }
    
    #[test]
    fn test_routable_address_filter() {
        assert!(PeerCache::is_routable_address("/ip4/1.2.3.4/tcp/30303"));
        assert!(!PeerCache::is_routable_address("/ip4/127.0.0.1/tcp/30303"));
        assert!(!PeerCache::is_routable_address("/ip4/192.168.1.1/tcp/30303"));
        assert!(!PeerCache::is_routable_address("/p2p/12D3KooW...")); // Bare peer ID
        assert!(PeerCache::is_routable_address("/ip4/1.2.3.4/tcp/30303/p2p-circuit/p2p/12D3KooW...")); // Single relay OK
    }
}
