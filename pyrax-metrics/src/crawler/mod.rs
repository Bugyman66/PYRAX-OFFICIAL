//! Node Discovery Crawler
//!
//! Discovers nodes in the network by querying pyrax_getPeers from seed nodes.

use crate::config::CrawlerConfig;
use crate::observer::{RpcClient, PeerInfo};
use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use tokio::time::interval;
use tracing::{info, warn, debug};

/// Discovered node information
#[derive(Debug, Clone)]
pub struct DiscoveredNode {
    /// RPC endpoint URL
    pub endpoint: String,
    /// Peer ID
    pub peer_id: String,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Is reachable
    pub reachable: bool,
    /// Best block height
    pub best_height: u64,
    /// Node version
    pub version: Option<String>,
}

/// Shared discovered nodes list
pub type SharedDiscoveredNodes = Arc<RwLock<Vec<DiscoveredNode>>>;

/// Node Discovery Crawler
pub struct NodeCrawler {
    config: CrawlerConfig,
    seed_endpoints: Vec<String>,
    discovered: SharedDiscoveredNodes,
    seen_peers: Arc<RwLock<HashSet<String>>>,
}

impl NodeCrawler {
    /// Create a new node crawler
    pub fn new(config: CrawlerConfig, seed_endpoints: Vec<String>) -> Self {
        Self {
            config,
            seed_endpoints,
            discovered: Arc::new(RwLock::new(Vec::new())),
            seen_peers: Arc::new(RwLock::new(HashSet::new())),
        }
    }
    
    /// Get shared reference to discovered nodes
    pub fn discovered_nodes(&self) -> SharedDiscoveredNodes {
        self.discovered.clone()
    }
    
    /// Run the crawler loop
    pub async fn run(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Node crawler disabled, skipping");
            return Ok(());
        }
        
        let crawl_interval = Duration::from_millis(self.config.crawl_interval_ms);
        let mut interval = interval(crawl_interval);
        
        info!("Starting node crawler with {}ms interval", self.config.crawl_interval_ms);
        info!("Seed endpoints: {:?}", self.seed_endpoints);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.crawl_network().await {
                warn!("Crawl error: {}", e);
            }
            
            // All nodes from pyrax_getPeers are online (actively connected)
            // No need for RPC verification
            let discovered = self.discovered.read();
            let total = discovered.len();
            let online = discovered.iter().filter(|n| n.reachable).count();
            info!("{} nodes discovered, {} online", total, online);
        }
    }
    
    /// Verify discovered nodes are reachable via RPC
    async fn verify_nodes(&self) {
        let timeout = Duration::from_millis(self.config.probe_timeout_ms);
        let mut endpoints_to_check: Vec<(usize, String)> = Vec::new();
        
        // Collect endpoints to verify
        {
            let discovered = self.discovered.read();
            for (i, node) in discovered.iter().enumerate() {
                endpoints_to_check.push((i, node.endpoint.clone()));
            }
        }
        
        // Probe each endpoint
        for (idx, endpoint) in endpoints_to_check {
            let client = RpcClient::new(endpoint.clone(), timeout);
            
            // Try a simple RPC call to verify reachability
            let is_reachable = match client.get_status().await {
                Ok(status) => {
                    // Update block height while we're at it
                    let mut discovered = self.discovered.write();
                    if let Some(node) = discovered.get_mut(idx) {
                        node.best_height = status.block_height;
                        node.last_seen = chrono::Utc::now().timestamp() as u64;
                    }
                    status.reachable
                }
                Err(_) => false,
            };
            
            // Update reachable status
            let mut discovered = self.discovered.write();
            if let Some(node) = discovered.get_mut(idx) {
                node.reachable = is_reachable;
            }
        }
    }
    
    /// Perform one crawl cycle
    async fn crawl_network(&self) -> Result<()> {
        let timeout = Duration::from_millis(self.config.probe_timeout_ms);
        let timestamp = chrono::Utc::now().timestamp() as u64;
        
        // First, mark all discovered nodes as offline
        {
            let mut discovered = self.discovered.write();
            for node in discovered.iter_mut() {
                node.reachable = false;
            }
        }
        
        // Collect current peer IDs in this crawl cycle
        let mut current_peer_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        // Query seed endpoints for current peer list
        for endpoint in &self.seed_endpoints {
            let client = RpcClient::new(endpoint.clone(), timeout);
            
            match client.get_peers().await {
                Ok(peers) => {
                    debug!("Got {} peers from {}", peers.len(), endpoint);
                    
                    for peer in peers {
                        current_peer_ids.insert(peer.peer_id.clone());
                        
                        // Add or update node
                        if let Some(ip) = &peer.ip {
                            let node_endpoint = format!("http://{}:8545", ip);
                            
                            let mut discovered = self.discovered.write();
                            
                            // Check if already discovered
                            if let Some(existing) = discovered.iter_mut().find(|n| n.peer_id == peer.peer_id) {
                                // Update existing node - mark as online
                                existing.reachable = true;
                                existing.last_seen = timestamp;
                                existing.best_height = peer.block_height;
                            } else if discovered.len() < self.config.max_nodes {
                                // Add new node as online
                                let node = DiscoveredNode {
                                    endpoint: node_endpoint.clone(),
                                    peer_id: peer.peer_id.clone(),
                                    last_seen: timestamp,
                                    reachable: true, // Online - currently connected
                                    best_height: peer.block_height,
                                    version: peer.version.clone(),
                                };
                                info!("New node: {} (peer: {})", node_endpoint, &peer.peer_id[..16]);
                                discovered.push(node);
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to get peers from {}: {}", endpoint, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Process discovered peers
    async fn process_peers(&self, peers: Vec<PeerInfo>) {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        
        for peer in peers {
            // Skip if already seen
            if self.seen_peers.read().contains(&peer.peer_id) {
                continue;
            }
            
            // Mark as seen
            self.seen_peers.write().insert(peer.peer_id.clone());
            
            // Extract RPC endpoint from IP if available
            if let Some(ip) = &peer.ip {
                // Construct RPC URL from IP address
                // Assume RPC port is 8545 by default for discovered nodes
                let endpoint = format!("http://{}:8545", ip);
                
                let node = DiscoveredNode {
                    endpoint,
                    peer_id: peer.peer_id.clone(),
                    last_seen: timestamp,
                    reachable: true, // Will be verified on next probe
                    best_height: peer.block_height,
                    version: peer.version.clone(),
                };
                
                // Check if not already discovered
                let mut discovered = self.discovered.write();
                if !discovered.iter().any(|n| n.peer_id == peer.peer_id) {
                    if discovered.len() < self.config.max_nodes {
                        info!("Discovered new node: {} (height: {})", node.endpoint, node.best_height);
                        discovered.push(node);
                    }
                }
            }
        }
    }
    
    /// Convert P2P address to RPC endpoint
    fn addr_to_rpc_endpoint(&self, addr: &str) -> Option<String> {
        // Parse IP:PORT format
        let parts: Vec<&str> = addr.split(':').collect();
        if parts.len() >= 2 {
            let ip = parts[0];
            // Assume RPC port is 8545 by default for discovered nodes
            // In production, this should be configurable or discovered
            Some(format!("http://{}:8545", ip))
        } else {
            None
        }
    }
}

/// Create a dummy crawler that returns an empty list (for when disabled)
pub fn create_empty_discovered_nodes() -> SharedDiscoveredNodes {
    Arc::new(RwLock::new(Vec::new()))
}
