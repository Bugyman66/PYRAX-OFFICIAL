//! Shared Peer Registry for P2P network
//! 
//! Allows RPC and other components to access connected peer information

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Information about a connected peer
#[derive(Debug, Clone)]
pub struct ConnectedPeer {
    pub peer_id: String,
    pub address: String,
    pub ip: String,
    pub port: u16,
    pub direction: PeerDirection,
    pub connected_at: Instant,
    pub last_seen: Instant,
    pub client_version: String,
    pub best_height: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerDirection {
    Inbound,
    Outbound,
}

impl std::fmt::Display for PeerDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerDirection::Inbound => write!(f, "inbound"),
            PeerDirection::Outbound => write!(f, "outbound"),
        }
    }
}

/// Shared peer registry accessible by RPC and P2P
#[derive(Clone)]
pub struct PeerRegistry {
    inner: Arc<PeerRegistryInner>,
}

struct PeerRegistryInner {
    peers: RwLock<HashMap<String, ConnectedPeer>>,
    local_peer_id: RwLock<String>,
    listen_addresses: RwLock<Vec<String>>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(PeerRegistryInner {
                peers: RwLock::new(HashMap::new()),
                local_peer_id: RwLock::new(String::new()),
                listen_addresses: RwLock::new(Vec::new()),
            }),
        }
    }

    /// Set the local peer ID
    pub async fn set_local_peer_id(&self, id: String) {
        *self.inner.local_peer_id.write().await = id;
    }

    /// Get the local peer ID
    pub async fn local_peer_id(&self) -> String {
        self.inner.local_peer_id.read().await.clone()
    }

    /// Add a listen address
    pub async fn add_listen_address(&self, addr: String) {
        self.inner.listen_addresses.write().await.push(addr);
    }

    /// Get listen addresses
    pub async fn listen_addresses(&self) -> Vec<String> {
        self.inner.listen_addresses.read().await.clone()
    }

    /// Register a new peer connection
    pub async fn add_peer(&self, peer: ConnectedPeer) {
        self.inner.peers.write().await.insert(peer.peer_id.clone(), peer);
    }

    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: &str) {
        self.inner.peers.write().await.remove(peer_id);
    }

    /// Update peer's last seen time
    pub async fn update_peer_seen(&self, peer_id: &str) {
        if let Some(peer) = self.inner.peers.write().await.get_mut(peer_id) {
            peer.last_seen = Instant::now();
        }
    }

    /// Update peer's best height
    pub async fn update_peer_height(&self, peer_id: &str, height: u64) {
        if let Some(peer) = self.inner.peers.write().await.get_mut(peer_id) {
            peer.best_height = height;
            peer.last_seen = Instant::now();
        }
    }

    /// Update peer's client version
    pub async fn update_peer_version(&self, peer_id: &str, version: &str) {
        if let Some(peer) = self.inner.peers.write().await.get_mut(peer_id) {
            peer.client_version = version.to_string();
            peer.last_seen = Instant::now();
        }
    }

    /// Get all connected peers
    pub async fn get_peers(&self) -> Vec<ConnectedPeer> {
        self.inner.peers.read().await.values().cloned().collect()
    }

    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        self.inner.peers.read().await.len()
    }
}

impl Default for PeerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract IP and port from a multiaddr string
pub fn parse_multiaddr(addr: &str) -> (String, u16) {
    // Parse formats like:
    // /ip4/192.168.1.1/tcp/30303
    // /ip6/::1/tcp/30303
    let mut ip = String::new();
    let mut port = 0u16;

    let parts: Vec<&str> = addr.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "ip4" || *part == "ip6" {
            if let Some(addr_part) = parts.get(i + 1) {
                ip = addr_part.to_string();
            }
        }
        if *part == "tcp" || *part == "udp" {
            if let Some(port_part) = parts.get(i + 1) {
                port = port_part.parse().unwrap_or(0);
            }
        }
    }

    (ip, port)
}
