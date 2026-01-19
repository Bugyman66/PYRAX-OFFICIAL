//! Peer Store - Manages peer information, scoring, RTT tracking, and dial backoff
//!
//! DESIGN DOC:
//! -----------
//! The PeerStore maintains comprehensive state for all known peers:
//! - Peer scoring based on RTT, stability, protocol behavior, subnet diversity
//! - Dial backoff with exponential retry (30s → 60s → 120s → max 15m)
//! - RTT tracking from ping responses
//! - Subnet tracking for diversity scoring
//! - Connection state management
//!
//! Score calculation:
//! - Base score: 0
//! - RTT bonus: + (50 - min(RTT_ms, 200)) * 0.1
//! - Uptime bonus: + uptime_minutes * 0.05 (capped at 10 points)
//! - Disconnect penalty: - disconnects_last_hour * 1.5
//! - Failure penalty: - failures_last_hour * 0.5
//! - Subnet penalty: - 5 if same /24 has >= 3 peers
//! - Bootnode bonus: +5 for bootnode connections
//! - Interaction bonus: + successful_interactions * 0.01 (max 5)
//! - Score clamped to [-100, +50]
//!
//! Banning:
//! - Auto-ban threshold: score <= -100
//! - Grace period: 5 minutes after discovery (no bans during grace)
//! - Ban duration: 1 hour

use libp2p::PeerId;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Peer store configuration
#[derive(Debug, Clone)]
pub struct PeerStoreConfig {
    /// Initial backoff duration after dial failure
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier for exponential increase
    pub backoff_multiplier: f64,
    /// Duration after which backoff is cleared on success
    pub backoff_clear_after: Duration,
    /// Maximum peers from same /24 subnet before penalty
    pub max_peers_per_subnet: usize,
    /// Peer timeout - consider dead if no activity
    pub peer_timeout: Duration,
}

impl Default for PeerStoreConfig {
    fn default() -> Self {
        Self {
            initial_backoff: Duration::from_secs(30),
            max_backoff: Duration::from_secs(900), // 15 minutes
            backoff_multiplier: 2.0,
            backoff_clear_after: Duration::from_secs(300), // 5 minutes
            max_peers_per_subnet: 3,
            peer_timeout: Duration::from_secs(120),
        }
    }
}

/// Connection direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionDirection {
    Inbound,
    Outbound,
}

/// Peer connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    /// Known but not connected
    Disconnected,
    /// Currently dialing
    Dialing,
    /// Connected and healthy
    Connected,
    /// Marked for disconnection
    Disconnecting,
    /// Banned (score too low or misbehavior)
    Banned,
}

/// Comprehensive peer information
#[derive(Debug, Clone)]
pub struct PeerData {
    /// Peer ID
    pub peer_id: PeerId,
    /// Known addresses for this peer
    pub addresses: Vec<String>,
    /// Extracted IP address (for subnet tracking)
    pub ip_addr: Option<IpAddr>,
    /// Current connection state
    pub state: PeerState,
    /// Connection direction (if connected)
    pub direction: Option<ConnectionDirection>,
    /// When we first learned about this peer
    pub discovered_at: Instant,
    /// When connection was established (if connected)
    pub connected_at: Option<Instant>,
    /// Last activity timestamp
    pub last_seen: Instant,
    /// Last successful ping time
    pub last_ping: Option<Instant>,
    /// Round-trip time in milliseconds (from ping)
    pub rtt_ms: Option<u64>,
    /// RTT history for averaging (last 10 measurements)
    pub rtt_history: Vec<u64>,
    /// Current score [-50, +50]
    pub score: i32,
    /// Number of disconnects in the last hour
    pub disconnects_last_hour: u32,
    /// Number of dial/protocol failures in the last hour
    pub failures_last_hour: u32,
    /// Successful protocol interactions
    pub successful_interactions: u64,
    /// Is this a bootnode?
    pub is_bootnode: bool,
    /// Client version string
    pub client_version: String,
    /// Best known block height from this peer
    pub best_height: u64,
    /// Timestamps of recent disconnects (for hourly tracking)
    disconnect_times: Vec<Instant>,
    /// Timestamps of recent failures (for hourly tracking)
    failure_times: Vec<Instant>,
    /// FIX: Track consecutive dial failures for pre-dial health check
    pub consecutive_failures: u32,
    /// FIX: Track last failure time for backoff calculation
    pub last_failure_time: Option<Instant>,
}

impl PeerData {
    pub fn new(peer_id: PeerId) -> Self {
        let now = Instant::now();
        Self {
            peer_id,
            addresses: Vec::new(),
            ip_addr: None,
            state: PeerState::Disconnected,
            direction: None,
            discovered_at: now,
            connected_at: None,
            last_seen: now,
            last_ping: None,
            rtt_ms: None,
            rtt_history: Vec::with_capacity(10),
            score: 0,
            disconnects_last_hour: 0,
            failures_last_hour: 0,
            successful_interactions: 0,
            is_bootnode: false,
            client_version: String::new(),
            best_height: 0,
            disconnect_times: Vec::new(),
            failure_times: Vec::new(),
            consecutive_failures: 0,
            last_failure_time: None,
        }
    }

    /// Update RTT from ping measurement
    pub fn update_rtt(&mut self, rtt: Duration) {
        let rtt_ms = rtt.as_millis() as u64;
        self.rtt_ms = Some(rtt_ms);
        self.last_ping = Some(Instant::now());
        self.last_seen = Instant::now();
        
        // Keep last 10 RTT measurements
        if self.rtt_history.len() >= 10 {
            self.rtt_history.remove(0);
        }
        self.rtt_history.push(rtt_ms);
    }

    /// Get average RTT from history
    pub fn average_rtt(&self) -> Option<u64> {
        if self.rtt_history.is_empty() {
            return self.rtt_ms;
        }
        let sum: u64 = self.rtt_history.iter().sum();
        Some(sum / self.rtt_history.len() as u64)
    }

    /// Record a disconnect event
    pub fn record_disconnect(&mut self) {
        self.disconnect_times.push(Instant::now());
        self.cleanup_old_events();
        self.disconnects_last_hour = self.disconnect_times.len() as u32;
    }

    /// Record a failure event (dial or protocol)
    pub fn record_failure(&mut self) {
        self.failure_times.push(Instant::now());
        self.cleanup_old_events();
        self.failures_last_hour = self.failure_times.len() as u32;
    }

    /// Record a successful interaction
    pub fn record_success(&mut self) {
        self.successful_interactions += 1;
        self.last_seen = Instant::now();
        // FIX: Reset consecutive failures on success
        self.consecutive_failures = 0;
    }

    /// Clean up events older than 1 hour
    fn cleanup_old_events(&mut self) {
        // Use checked_sub to prevent panic if system uptime < 1 hour
        // If subtraction would underflow, keep all events (system just started)
        let now = Instant::now();
        let one_hour = Duration::from_secs(3600);
        if let Some(one_hour_ago) = now.checked_sub(one_hour) {
            self.disconnect_times.retain(|t| *t > one_hour_ago);
            self.failure_times.retain(|t| *t > one_hour_ago);
        }
        // If checked_sub returns None, system uptime < 1 hour, keep all events
        self.disconnects_last_hour = self.disconnect_times.len() as u32;
        self.failures_last_hour = self.failure_times.len() as u32;
    }

    /// Get connection uptime in minutes
    pub fn uptime_minutes(&self) -> u64 {
        self.connected_at
            .map(|t| t.elapsed().as_secs() / 60)
            .unwrap_or(0)
    }

    /// Check if peer is stale (no recent activity)
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }

    /// Check if peer should be banned based on score
    /// Only ban if score is very low AND peer has been known for a while (grace period)
    /// FIX: Extended grace period and stricter ban threshold to prevent "scoring out" new users
    pub fn should_ban(&self) -> bool {
        // EXTENDED GRACE PERIOD: 15 minutes (was 5 minutes)
        // NAT traversal and relay connections can take time to stabilize
        // Many dial failures happen during initial connection attempts which is normal
        let grace_period = Duration::from_secs(900);
        if self.discovered_at.elapsed() < grace_period {
            return false;
        }
        // Stricter ban threshold: -150 (was -100)
        // Combined with reduced penalties, this makes banning much harder
        self.score <= -150
    }
}

/// Dial backoff tracking
#[derive(Debug, Clone)]
pub struct DialBackoff {
    /// Next allowed dial time
    pub next_dial_at: Instant,
    /// Current backoff duration
    pub current_backoff: Duration,
    /// Number of consecutive failures
    pub failure_count: u32,
}

impl DialBackoff {
    pub fn new(initial_backoff: Duration) -> Self {
        Self {
            next_dial_at: Instant::now(),
            current_backoff: initial_backoff,
            failure_count: 0,
        }
    }

    /// Check if dial is allowed now
    pub fn can_dial(&self) -> bool {
        Instant::now() >= self.next_dial_at
    }

    /// Record a dial failure and increase backoff
    pub fn record_failure(&mut self, max_backoff: Duration, multiplier: f64) {
        self.failure_count += 1;
        self.current_backoff = Duration::from_secs_f64(
            (self.current_backoff.as_secs_f64() * multiplier).min(max_backoff.as_secs_f64())
        );
        self.next_dial_at = Instant::now() + self.current_backoff;
    }

    /// Clear backoff on successful connection
    pub fn clear(&mut self, initial_backoff: Duration) {
        self.failure_count = 0;
        self.current_backoff = initial_backoff;
        self.next_dial_at = Instant::now();
    }
}

/// Subnet tracking for diversity scoring
#[derive(Debug, Default)]
struct SubnetTracker {
    /// Count of connected peers per /24 subnet
    subnet_counts: HashMap<String, usize>,
}

impl SubnetTracker {
    fn get_subnet_key(ip: &IpAddr) -> String {
        match ip {
            IpAddr::V4(v4) => {
                let octets = v4.octets();
                format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
            }
            IpAddr::V6(v6) => {
                // Use first 48 bits for IPv6
                let segments = v6.segments();
                format!("{:x}:{:x}:{:x}::/48", segments[0], segments[1], segments[2])
            }
        }
    }

    fn add_peer(&mut self, ip: &IpAddr) {
        let key = Self::get_subnet_key(ip);
        *self.subnet_counts.entry(key).or_insert(0) += 1;
    }

    fn remove_peer(&mut self, ip: &IpAddr) {
        let key = Self::get_subnet_key(ip);
        if let Some(count) = self.subnet_counts.get_mut(&key) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.subnet_counts.remove(&key);
            }
        }
    }

    fn peer_count_in_subnet(&self, ip: &IpAddr) -> usize {
        let key = Self::get_subnet_key(ip);
        *self.subnet_counts.get(&key).unwrap_or(&0)
    }
}

/// The main peer store
pub struct PeerStore {
    config: PeerStoreConfig,
    /// All known peers
    peers: HashMap<PeerId, PeerData>,
    /// Dial backoff tracking
    backoffs: HashMap<PeerId, DialBackoff>,
    /// Subnet tracking for connected peers
    subnet_tracker: SubnetTracker,
    /// Set of bootnode peer IDs
    bootnodes: HashSet<PeerId>,
    /// Banned peers (with ban expiry time)
    banned: HashMap<PeerId, Instant>,
}

impl PeerStore {
    pub fn new(config: PeerStoreConfig) -> Self {
        Self {
            config,
            peers: HashMap::new(),
            backoffs: HashMap::new(),
            subnet_tracker: SubnetTracker::default(),
            bootnodes: HashSet::new(),
            banned: HashMap::new(),
        }
    }

    /// Register a bootnode
    pub fn add_bootnode(&mut self, peer_id: PeerId) {
        self.bootnodes.insert(peer_id);
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.is_bootnode = true;
        }
    }

    /// Check if peer is a bootnode
    pub fn is_bootnode(&self, peer_id: &PeerId) -> bool {
        self.bootnodes.contains(peer_id)
    }

    /// Add or update a peer
    pub fn upsert_peer(&mut self, peer_id: PeerId) -> &mut PeerData {
        let is_bootnode = self.bootnodes.contains(&peer_id);
        self.peers.entry(peer_id).or_insert_with(|| {
            let mut peer = PeerData::new(peer_id);
            peer.is_bootnode = is_bootnode;
            peer
        })
    }

    /// Get peer data
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerData> {
        self.peers.get(peer_id)
    }

    /// Get mutable peer data
    pub fn get_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut PeerData> {
        self.peers.get_mut(peer_id)
    }

    /// Add address for a peer
    pub fn add_address(&mut self, peer_id: &PeerId, address: String) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            if !peer.addresses.contains(&address) {
                // Try to extract IP address
                if peer.ip_addr.is_none() {
                    peer.ip_addr = Self::extract_ip(&address);
                }
                peer.addresses.push(address);
            }
        }
    }

    /// Extract IP address from multiaddr string
    fn extract_ip(addr: &str) -> Option<IpAddr> {
        let parts: Vec<&str> = addr.split('/').collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "ip4" {
                if let Some(ip_str) = parts.get(i + 1) {
                    return ip_str.parse().ok();
                }
            }
            if *part == "ip6" {
                if let Some(ip_str) = parts.get(i + 1) {
                    return ip_str.parse().ok();
                }
            }
        }
        None
    }

    /// Mark peer as connected
    pub fn peer_connected(&mut self, peer_id: &PeerId, direction: ConnectionDirection) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.state = PeerState::Connected;
            peer.direction = Some(direction);
            peer.connected_at = Some(Instant::now());
            peer.last_seen = Instant::now();
            
            // Track subnet
            if let Some(ip) = &peer.ip_addr {
                self.subnet_tracker.add_peer(ip);
            }
            
            // Clear backoff on successful connection
            if let Some(backoff) = self.backoffs.get_mut(peer_id) {
                backoff.clear(self.config.initial_backoff);
            }
            
            debug!("Peer {} connected ({:?})", peer_id, direction);
        }
    }

    /// Mark peer as disconnected
    pub fn peer_disconnected(&mut self, peer_id: &PeerId) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            // Remove from subnet tracking
            if let Some(ip) = &peer.ip_addr {
                self.subnet_tracker.remove_peer(ip);
            }
            
            peer.state = PeerState::Disconnected;
            peer.direction = None;
            peer.connected_at = None;
            peer.record_disconnect();
            
            debug!("Peer {} disconnected", peer_id);
        }
    }

    /// Record a dial failure
    pub fn record_dial_failure(&mut self, peer_id: &PeerId) {
        // Update peer data
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.record_failure();
            peer.state = PeerState::Disconnected;
            // FIX: Track consecutive failures for pre-dial health check
            peer.consecutive_failures += 1;
            peer.last_failure_time = Some(Instant::now());
        }
        
        // Update backoff
        let backoff = self.backoffs.entry(*peer_id).or_insert_with(|| {
            DialBackoff::new(self.config.initial_backoff)
        });
        backoff.record_failure(self.config.max_backoff, self.config.backoff_multiplier);
        
        debug!("Dial failure for {}, next attempt in {:?}", 
            peer_id, backoff.current_backoff);
    }

    /// Check if we can dial a peer (not in backoff)
    pub fn can_dial(&self, peer_id: &PeerId) -> bool {
        // Check if banned
        if self.is_banned(peer_id) {
            return false;
        }
        
        // Check backoff
        if let Some(backoff) = self.backoffs.get(peer_id) {
            return backoff.can_dial();
        }
        
        true
    }
    
    /// Clear backoff for a peer (used for critical reconnections like bootnodes)
    pub fn clear_backoff(&mut self, peer_id: &PeerId) {
        if let Some(backoff) = self.backoffs.get_mut(peer_id) {
            backoff.clear(self.config.initial_backoff);
        }
    }

    /// Check if peer is banned
    pub fn is_banned(&self, peer_id: &PeerId) -> bool {
        if let Some(expiry) = self.banned.get(peer_id) {
            if Instant::now() < *expiry {
                return true;
            }
            // Ban expired, will be cleaned up later
        }
        false
    }

    /// Ban a peer for a duration
    pub fn ban_peer(&mut self, peer_id: &PeerId, duration: Duration) {
        let expiry = Instant::now() + duration;
        self.banned.insert(*peer_id, expiry);
        
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.state = PeerState::Banned;
        }
        
        warn!("Banned peer {} for {:?}", peer_id, duration);
    }

    /// Calculate peer score
    pub fn calculate_score(&self, peer_id: &PeerId) -> i32 {
        let peer = match self.peers.get(peer_id) {
            Some(p) => p,
            None => return 0,
        };
        
        let mut score: f64 = 0.0;
        
        // RTT bonus: + (50 - min(RTT_ms, 200)) * 0.1
        if let Some(rtt) = peer.average_rtt() {
            let capped_rtt = rtt.min(200);
            score += (50.0 - capped_rtt as f64) * 0.1;
        }
        
        // Uptime bonus: + uptime_minutes * 0.05 (max 10 points)
        let uptime_bonus = (peer.uptime_minutes() as f64 * 0.05).min(10.0);
        score += uptime_bonus;
        
        // Disconnect penalty: - disconnects_last_hour * 0.5 (reduced from 1.5)
        // NAT traversal causes frequent reconnections which is normal behavior
        score -= peer.disconnects_last_hour as f64 * 0.5;
        
        // Failure penalty: - failures_last_hour * 0.2 (reduced from 0.5)
        // Many failures are due to NAT/relay issues, not misbehavior
        // New users behind NAT will have many dial failures during setup
        score -= peer.failures_last_hour as f64 * 0.2;
        
        // Subnet diversity penalty
        if let Some(ip) = &peer.ip_addr {
            let subnet_count = self.subnet_tracker.peer_count_in_subnet(ip);
            if subnet_count >= self.config.max_peers_per_subnet {
                score -= 5.0;
            }
        }
        
        // Bootnode bonus (prefer keeping bootnode connections)
        if peer.is_bootnode {
            score += 5.0;
        }
        
        // Successful interactions bonus
        let interaction_bonus = (peer.successful_interactions as f64 * 0.01).min(5.0);
        score += interaction_bonus;
        
        // Clamp to [-150, +50] - wider range for negative to allow gradual recovery
        // Score must go below -150 to trigger ban (with 15min grace period)
        score.clamp(-150.0, 50.0) as i32
    }

    /// Update all peer scores
    pub fn update_all_scores(&mut self) {
        let peer_ids: Vec<PeerId> = self.peers.keys().cloned().collect();
        for peer_id in peer_ids {
            let score = self.calculate_score(&peer_id);
            if let Some(peer) = self.peers.get_mut(&peer_id) {
                peer.score = score;
                
                // Auto-ban if score too low (only if not already banned to avoid log spam)
                if peer.should_ban() && !peer.is_bootnode && peer.state != PeerState::Banned {
                    self.banned.insert(peer_id, Instant::now() + Duration::from_secs(3600));
                    peer.state = PeerState::Banned;
                    warn!("Auto-banning peer {} due to low score: {} (banned for 1 hour)", peer_id, score);
                }
            }
        }
    }

    /// Get all connected peers
    pub fn connected_peers(&self) -> Vec<&PeerData> {
        self.peers.values()
            .filter(|p| p.state == PeerState::Connected)
            .collect()
    }

    /// Get connected peer count
    pub fn connected_count(&self) -> usize {
        self.peers.values()
            .filter(|p| p.state == PeerState::Connected)
            .count()
    }

    /// Get inbound peer count
    pub fn inbound_count(&self) -> usize {
        self.peers.values()
            .filter(|p| p.state == PeerState::Connected && p.direction == Some(ConnectionDirection::Inbound))
            .count()
    }

    /// Get outbound peer count
    pub fn outbound_count(&self) -> usize {
        self.peers.values()
            .filter(|p| p.state == PeerState::Connected && p.direction == Some(ConnectionDirection::Outbound))
            .count()
    }

    /// Get peers sorted by score (highest first)
    pub fn peers_by_score(&self) -> Vec<&PeerData> {
        let mut peers: Vec<_> = self.peers.values().collect();
        peers.sort_by(|a, b| b.score.cmp(&a.score));
        peers
    }

    /// Get lowest scored connected peers (candidates for pruning)
    pub fn lowest_scored_connected(&self, count: usize) -> Vec<PeerId> {
        let mut connected: Vec<_> = self.peers.values()
            .filter(|p| p.state == PeerState::Connected && !p.is_bootnode)
            .collect();
        
        connected.sort_by(|a, b| a.score.cmp(&b.score));
        
        connected.iter()
            .take(count)
            .map(|p| p.peer_id)
            .collect()
    }

    /// Get dialable candidates (not connected, not in backoff, not banned)
    pub fn dialable_candidates(&self) -> Vec<&PeerData> {
        self.peers.values()
            .filter(|p| {
                p.state == PeerState::Disconnected 
                    && !p.addresses.is_empty()
                    && self.can_dial(&p.peer_id)
            })
            .collect()
    }

    /// Get best candidates to dial, considering diversity
    pub fn best_dial_candidates(&self, count: usize) -> Vec<PeerId> {
        let mut candidates = self.dialable_candidates();
        
        // Sort by score descending, but also consider subnet diversity
        candidates.sort_by(|a, b| {
            // Prefer peers from underrepresented subnets
            let a_subnet_count = a.ip_addr.as_ref()
                .map(|ip| self.subnet_tracker.peer_count_in_subnet(ip))
                .unwrap_or(0);
            let b_subnet_count = b.ip_addr.as_ref()
                .map(|ip| self.subnet_tracker.peer_count_in_subnet(ip))
                .unwrap_or(0);
            
            // First compare by subnet diversity (fewer peers in subnet = better)
            match a_subnet_count.cmp(&b_subnet_count) {
                std::cmp::Ordering::Equal => {
                    // Then by score (higher = better)
                    b.score.cmp(&a.score)
                }
                other => other,
            }
        });
        
        candidates.iter()
            .take(count)
            .map(|p| p.peer_id)
            .collect()
    }

    /// Clean up expired bans and stale data
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        
        // Remove expired bans
        self.banned.retain(|_, expiry| now < *expiry);
        
        // Update peer states for expired bans
        for (peer_id, peer) in &mut self.peers {
            if peer.state == PeerState::Banned && !self.banned.contains_key(peer_id) {
                peer.state = PeerState::Disconnected;
            }
            // Cleanup old event timestamps
            peer.cleanup_old_events();
        }
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> PeerStoreMetrics {
        let connected = self.connected_peers();
        let total_rtt: u64 = connected.iter()
            .filter_map(|p| p.average_rtt())
            .sum();
        let rtt_count = connected.iter()
            .filter(|p| p.average_rtt().is_some())
            .count();
        
        let mut rtt_values: Vec<u64> = connected.iter()
            .filter_map(|p| p.average_rtt())
            .collect();
        rtt_values.sort();
        
        let p95_rtt = if rtt_values.len() >= 20 {
            let idx = (rtt_values.len() as f64 * 0.95) as usize;
            Some(rtt_values[idx.min(rtt_values.len() - 1)])
        } else {
            None
        };

        PeerStoreMetrics {
            total_known_peers: self.peers.len(),
            connected_peers: connected.len(),
            inbound_peers: self.inbound_count(),
            outbound_peers: self.outbound_count(),
            banned_peers: self.banned.len(),
            average_rtt_ms: if rtt_count > 0 { Some(total_rtt / rtt_count as u64) } else { None },
            p95_rtt_ms: p95_rtt,
            peers_in_backoff: self.backoffs.values().filter(|b| !b.can_dial()).count(),
        }
    }
}

/// Metrics snapshot from peer store
#[derive(Debug, Clone)]
pub struct PeerStoreMetrics {
    pub total_known_peers: usize,
    pub connected_peers: usize,
    pub inbound_peers: usize,
    pub outbound_peers: usize,
    pub banned_peers: usize,
    pub average_rtt_ms: Option<u64>,
    pub p95_rtt_ms: Option<u64>,
    pub peers_in_backoff: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_peer_scoring() {
        let config = PeerStoreConfig::default();
        let mut store = PeerStore::new(config);
        
        let peer_id = PeerId::random();
        store.upsert_peer(peer_id);
        store.add_address(&peer_id, "/ip4/192.168.1.1/tcp/30303".to_string());
        store.peer_connected(&peer_id, ConnectionDirection::Outbound);
        
        // Update RTT
        if let Some(peer) = store.get_peer_mut(&peer_id) {
            peer.update_rtt(Duration::from_millis(50));
        }
        
        let score = store.calculate_score(&peer_id);
        assert!(score >= 0, "Score should be positive for healthy peer");
    }
    
    #[test]
    fn test_dial_backoff() {
        let config = PeerStoreConfig::default();
        let mut store = PeerStore::new(config.clone());
        
        let peer_id = PeerId::random();
        store.upsert_peer(peer_id);
        store.add_address(&peer_id, "/ip4/192.168.1.1/tcp/30303".to_string());
        
        assert!(store.can_dial(&peer_id));
        
        store.record_dial_failure(&peer_id);
        assert!(!store.can_dial(&peer_id));
        
        let backoff = store.backoffs.get(&peer_id).unwrap();
        assert_eq!(backoff.current_backoff, config.initial_backoff);
    }
    
    #[test]
    fn test_subnet_diversity() {
        let config = PeerStoreConfig::default();
        let mut store = PeerStore::new(config);
        
        // Add 3 peers from same subnet
        for i in 0..3 {
            let peer_id = PeerId::random();
            store.upsert_peer(peer_id);
            store.add_address(&peer_id, format!("/ip4/192.168.1.{}/tcp/30303", i + 1));
            store.peer_connected(&peer_id, ConnectionDirection::Outbound);
        }
        
        // Fourth peer from same subnet should have penalty
        let peer4 = PeerId::random();
        store.upsert_peer(peer4);
        store.add_address(&peer4, "/ip4/192.168.1.100/tcp/30303".to_string());
        
        let score = store.calculate_score(&peer4);
        // Score should be negative due to subnet penalty
        assert!(score < 0, "Score should be negative due to subnet crowding");
    }
}
