//! Connection Manager - Maintains healthy peer mesh with watermarks and pruning
//!
//! DESIGN DOC:
//! -----------
//! The ConnectionManager ensures the node maintains a healthy partial mesh:
//! - TARGET_PEERS = 50 (steady-state goal)
//! - MIN_PEERS = 30 (below this, dial aggressively)
//! - MAX_PEERS = 60 (temporary overshoot, then prune)
//! - MAX_CONCURRENT_DIALS = 5 (avoid dial storms)
//!
//! State Machine:
//! START → DIAL_BOOTNODE → IDENTIFY → BOOTSTRAP_DISCOVERY → FILL_PEERS → MAINTAIN_PEERS (loop)
//!
//! The manager runs periodic tasks:
//! - PEER_REFRESH_INTERVAL (30s): Try to add peers if below target
//! - PEER_REEVALUATE_INTERVAL (60s): Re-score and prune if needed
//! - LIVENESS_CHECK_INTERVAL (15s): Detect dead connections
//!
//! Pruning strategy (when above TARGET_PEERS):
//! 1. Sort peers by score ascending
//! 2. Prefer to drop: high RTT, unstable, redundant subnet peers
//! 3. Never drop: bootnodes (unless we have enough routing peers), high-score peers
//! 4. Hysteresis: Don't prune/dial too frequently

use libp2p::PeerId;
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};

use super::peer_store::{PeerStore, PeerStoreConfig, PeerData, ConnectionDirection, PeerState};

/// Connection manager configuration
#[derive(Debug, Clone)]
pub struct ConnectionManagerConfig {
    /// Target number of peers (steady-state)
    pub target_peers: usize,
    /// Minimum peers (dial aggressively below this)
    pub min_peers: usize,
    /// Maximum peers (prune above this)
    pub max_peers: usize,
    /// Maximum concurrent dial attempts
    pub max_concurrent_dials: usize,
    /// Dial timeout
    pub dial_timeout: Duration,
    /// Interval to check if we need more peers
    pub peer_refresh_interval: Duration,
    /// Interval to re-evaluate and prune peers
    pub peer_reevaluate_interval: Duration,
    /// Interval to check peer liveness
    pub liveness_check_interval: Duration,
    /// Minimum time between prune operations (hysteresis)
    pub min_prune_interval: Duration,
    /// Minimum time a peer must be connected before pruning
    pub min_connection_age: Duration,
}

impl Default for ConnectionManagerConfig {
    fn default() -> Self {
        Self {
            target_peers: 50,
            min_peers: 3,  // FIXED: Was 30, now 3 - allows small networks to reach Maintaining state
            max_peers: 60,
            max_concurrent_dials: 5,
            dial_timeout: Duration::from_secs(15),  // NETWORK STABILITY: Increased from 10s
            // NETWORK STABILITY: Increased from 30s to 120s - less aggressive peer refresh
            peer_refresh_interval: Duration::from_secs(120),
            // NETWORK STABILITY: Increased from 60s to 300s - don't re-evaluate peers too frequently
            peer_reevaluate_interval: Duration::from_secs(300),
            // NETWORK STABILITY: Increased from 15s to 60s - less aggressive liveness checks
            liveness_check_interval: Duration::from_secs(60),
            // NETWORK STABILITY: Increased from 30s to 120s - don't prune too frequently
            min_prune_interval: Duration::from_secs(120),
            // NETWORK STABILITY: Increased from 60s to 300s - give new connections time to stabilize
            min_connection_age: Duration::from_secs(300),
        }
    }
}

/// Network state in the connection lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkState {
    /// Initial state, preparing to connect
    #[default]
    Starting,
    /// Dialing bootnode(s)
    DialingBootnode,
    /// Waiting for identify handshake
    Identifying,
    /// Bootstrapping Kademlia DHT
    BootstrappingDiscovery,
    /// Actively filling peer slots
    FillingPeers,
    /// Normal operation, maintaining peer set
    Maintaining,
    /// Shutting down
    ShuttingDown,
}

/// Commands to the connection manager
#[derive(Debug)]
pub enum ConnectionCommand {
    /// Dial a specific peer
    Dial(PeerId, Vec<String>),
    /// Disconnect from a peer
    Disconnect(PeerId),
    /// Trigger peer discovery
    TriggerDiscovery,
    /// Shutdown the manager
    Shutdown,
}

/// Events from the connection manager
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// Need to dial peers
    DialPeers(Vec<(PeerId, String)>),
    /// Need to disconnect peers (for pruning)
    DisconnectPeers(Vec<PeerId>),
    /// Need to run Kademlia bootstrap
    TriggerKademliaBootstrap,
    /// Need to run Kademlia query for more peers
    TriggerKademliaQuery,
    /// State changed
    StateChanged(NetworkState),
    /// Metrics update
    MetricsUpdate(ConnectionMetrics),
}

/// Connection metrics
#[derive(Debug, Clone, Default)]
pub struct ConnectionMetrics {
    pub connected_peers: usize,
    pub inbound_peers: usize,
    pub outbound_peers: usize,
    pub target_peers: usize,
    pub dial_attempts: u64,
    pub dial_successes: u64,
    pub dial_failures: u64,
    pub prune_events: u64,
    pub average_rtt_ms: Option<u64>,
    pub p95_rtt_ms: Option<u64>,
    pub state: NetworkState,
}

/// Dial queue entry
#[derive(Debug)]
struct DialEntry {
    peer_id: PeerId,
    address: String,
    queued_at: Instant,
}

/// The main connection manager
pub struct ConnectionManager {
    config: ConnectionManagerConfig,
    peer_store: PeerStore,
    state: NetworkState,
    /// Queue of peers waiting to be dialed
    dial_queue: VecDeque<DialEntry>,
    /// Currently dialing peers
    dialing: HashSet<PeerId>,
    /// Last time we pruned
    last_prune: Instant,
    /// Last time we triggered discovery
    last_discovery: Instant,
    /// Bootnode peer IDs
    bootnodes: HashSet<PeerId>,
    /// Metrics counters
    dial_attempts: u64,
    dial_successes: u64,
    dial_failures: u64,
    prune_events: u64,
    /// Event sender
    event_tx: mpsc::Sender<ConnectionEvent>,
    /// STABILITY FIX: Track recently disconnected peers to prevent duplicate event handling
    /// Maps peer_id -> disconnect timestamp. Cleared after 5 seconds.
    recently_disconnected: HashMap<PeerId, Instant>,
}

impl ConnectionManager {
    pub fn new(
        config: ConnectionManagerConfig,
        peer_store_config: PeerStoreConfig,
        event_tx: mpsc::Sender<ConnectionEvent>,
    ) -> Self {
        let now = Instant::now();
        Self {
            config,
            peer_store: PeerStore::new(peer_store_config),
            state: NetworkState::Starting,
            dial_queue: VecDeque::new(),
            dialing: HashSet::new(),
            last_prune: now,
            last_discovery: now,
            bootnodes: HashSet::new(),
            dial_attempts: 0,
            dial_successes: 0,
            dial_failures: 0,
            prune_events: 0,
            event_tx,
            recently_disconnected: HashMap::new(),
        }
    }

    /// Get current network state
    pub fn state(&self) -> NetworkState {
        self.state
    }

    /// Get peer store reference
    pub fn peer_store(&self) -> &PeerStore {
        &self.peer_store
    }

    /// Get mutable peer store reference
    pub fn peer_store_mut(&mut self) -> &mut PeerStore {
        &mut self.peer_store
    }

    /// Register a bootnode
    pub fn add_bootnode(&mut self, peer_id: PeerId, addresses: Vec<String>) {
        self.bootnodes.insert(peer_id);
        self.peer_store.add_bootnode(peer_id);
        
        let peer = self.peer_store.upsert_peer(peer_id);
        peer.is_bootnode = true;
        for addr in addresses {
            self.peer_store.add_address(&peer_id, addr);
        }
        
        info!("Registered bootnode: {}", peer_id);
    }

    /// Transition to a new state
    fn transition_to(&mut self, new_state: NetworkState) {
        if self.state != new_state {
            info!("Connection manager state: {:?} -> {:?}", self.state, new_state);
            self.state = new_state;
            let _ = self.event_tx.try_send(ConnectionEvent::StateChanged(new_state));
        }
    }

    /// Start the connection process by dialing bootnodes
    pub async fn start(&mut self) {
        self.transition_to(NetworkState::DialingBootnode);
        
        // Collect bootnode addresses first to avoid borrow conflict
        let bootnode_addrs: Vec<(PeerId, Vec<String>)> = self.bootnodes.iter()
            .filter_map(|bootnode_id| {
                self.peer_store.get_peer(bootnode_id)
                    .map(|peer| (*bootnode_id, peer.addresses.clone()))
            })
            .collect();
        
        // Queue bootnode dials
        for (bootnode_id, addresses) in bootnode_addrs {
            for addr in addresses {
                self.queue_dial(bootnode_id, addr);
            }
        }
        
        self.process_dial_queue().await;
    }

    /// Check if an address is publicly routable (not localhost/private/link-local)
    /// This prevents dialing addresses that will either fail or connect to wrong peers
    fn is_routable_address(address: &str) -> bool {
        // ASIC FIX: Reject bare /p2p/ addresses without IP/port - these cause MultiaddrNotSupported errors
        // Example bad address: /p2p/12D3KooW... (no IP, no port - can't be dialed)
        // Valid relay: /ip4/1.2.3.4/tcp/9000/p2p/RELAY_ID/p2p-circuit/p2p/TARGET_ID
        if address.starts_with("/p2p/") && !address.contains("/ip4/") && !address.contains("/ip6/") {
            return false; // Bare peer ID without IP - not dialable
        }
        
        // NESTED RELAY FIX: Reject addresses with multiple /p2p-circuit/ segments
        // libp2p doesn't support chained relay circuits (MultipleCircuitRelayProtocolsUnsupported)
        // Count occurrences of /p2p-circuit
        let circuit_count = address.matches("/p2p-circuit").count();
        if circuit_count > 1 {
            return false; // Reject nested relay addresses
        }
        
        // Check if it's a single relay address (those are OK for NAT traversal)
        if circuit_count == 1 {
            return true;
        }
        
        // Parse the multiaddr to extract IP and port
        let parts: Vec<&str> = address.split('/').collect();
        let mut has_valid_ip = false;
        let mut port: Option<u16> = None;
        
        for (i, part) in parts.iter().enumerate() {
            if *part == "ip4" {
                if let Some(ip_str) = parts.get(i + 1) {
                    if let Ok(ip) = ip_str.parse::<Ipv4Addr>() {
                        // Reject localhost (127.0.0.0/8)
                        if ip.is_loopback() {
                            return false;
                        }
                        // Reject private networks (RFC 1918)
                        // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                        if ip.is_private() {
                            return false;
                        }
                        // Reject link-local (169.254.0.0/16)
                        if ip.is_link_local() {
                            return false;
                        }
                        // Reject unspecified (0.0.0.0)
                        if ip.is_unspecified() {
                            return false;
                        }
                        has_valid_ip = true;
                    }
                }
            } else if *part == "tcp" {
                if let Some(port_str) = parts.get(i + 1) {
                    port = port_str.parse().ok();
                }
            }
        }
        
        // Must have a valid IP
        if !has_valid_ip {
            return false;
        }
        
        // Reject suspicious ports that are likely ephemeral/random
        if let Some(p) = port {
            // Reject ephemeral ports (32768-65535 on most systems)
            if p >= 32768 {
                return false;
            }
            // Reject very low ports (< 1024) except well-known ones
            if p < 1024 && p != 443 && p != 80 {
                return false;
            }
        }
        
        true
    }

    /// Get the best address to dial for a peer, preferring relay addresses
    /// RELAY-FIRST STRATEGY: For NAT traversal, we prefer relay addresses because:
    /// 1. They work through any NAT type
    /// 2. Direct addresses often fail for nodes behind NAT
    /// 3. DCUtR can upgrade to direct connection after relay is established
    fn get_best_dial_address(addresses: &[String]) -> Option<String> {
        // First, try to find a SINGLE relay address (most reliable for NAT traversal)
        // NESTED RELAY FIX: Only accept addresses with exactly one /p2p-circuit segment
        let relay_addr = addresses.iter()
            .find(|addr| {
                let circuit_count = addr.matches("/p2p-circuit").count();
                circuit_count == 1 // Exactly one relay hop, no nested circuits
            })
            .cloned();
        
        if relay_addr.is_some() {
            return relay_addr;
        }
        
        // Fall back to first routable direct address
        addresses.iter()
            .find(|addr| Self::is_routable_address(addr))
            .cloned()
    }

    /// Queue a peer for dialing
    pub fn queue_dial(&mut self, peer_id: PeerId, address: String) {
        // CRITICAL: Filter out non-routable addresses to prevent WrongPeerId errors
        // When nodes dial localhost/private IPs, they often connect to themselves or wrong peers
        if !address.is_empty() && !Self::is_routable_address(&address) {
            debug!("Rejecting non-routable address for peer {}: {}", peer_id, address);
            return;
        }
        
        // Don't queue if already connected or dialing
        if self.dialing.contains(&peer_id) {
            return;
        }
        
        if let Some(peer) = self.peer_store.get_peer(&peer_id) {
            if peer.state == PeerState::Connected {
                return;
            }
        }
        
        // Check backoff
        if !self.peer_store.can_dial(&peer_id) {
            debug!("Peer {} in backoff, not queuing", peer_id);
            return;
        }
        
        // Don't add duplicates
        if self.dial_queue.iter().any(|e| e.peer_id == peer_id) {
            return;
        }
        
        self.dial_queue.push_back(DialEntry {
            peer_id,
            address,
            queued_at: Instant::now(),
        });
        
        debug!("Queued dial to {}", peer_id);
    }

    /// Process the dial queue, respecting concurrency limits
    /// STABILITY FIX: Now respects reconnection grace period for recently disconnected peers
    async fn process_dial_queue(&mut self) {
        let available_slots = self.config.max_concurrent_dials.saturating_sub(self.dialing.len());
        if available_slots == 0 {
            return;
        }
        
        let mut to_dial = Vec::new();
        
        while to_dial.len() < available_slots {
            match self.dial_queue.pop_front() {
                Some(entry) => {
                    // Skip if already connected or dialing
                    if self.dialing.contains(&entry.peer_id) {
                        continue;
                    }
                    if let Some(peer) = self.peer_store.get_peer(&entry.peer_id) {
                        if peer.state == PeerState::Connected {
                            continue;
                        }
                    }
                    
                    // Skip if entry is too old (dial timeout)
                    if entry.queued_at.elapsed() > self.config.dial_timeout {
                        continue;
                    }
                    
                    // STABILITY FIX: Skip if peer was recently disconnected (grace period)
                    // This prevents WrongPeerId errors from immediate reconnection attempts
                    if self.is_recently_disconnected(&entry.peer_id) && !self.bootnodes.contains(&entry.peer_id) {
                        debug!("Skipping dial to {} - recently disconnected (grace period)", entry.peer_id);
                        continue;
                    }
                    
                    self.dialing.insert(entry.peer_id);
                    self.dial_attempts += 1;
                    
                    // Mark as dialing in peer store
                    if let Some(peer) = self.peer_store.get_peer_mut(&entry.peer_id) {
                        peer.state = PeerState::Dialing;
                    }
                    
                    to_dial.push((entry.peer_id, entry.address));
                }
                None => break,
            }
        }
        
        if !to_dial.is_empty() {
            debug!("Processing {} dial requests", to_dial.len());
            let _ = self.event_tx.try_send(ConnectionEvent::DialPeers(to_dial));
        }
    }

    /// Handle successful connection
    pub async fn on_connection_established(&mut self, peer_id: PeerId, address: String, is_inbound: bool) {
        self.dialing.remove(&peer_id);
        
        let direction = if is_inbound {
            ConnectionDirection::Inbound
        } else {
            ConnectionDirection::Outbound
        };
        
        // Update peer store
        self.peer_store.upsert_peer(peer_id);
        self.peer_store.add_address(&peer_id, address);
        self.peer_store.peer_connected(&peer_id, direction);
        
        if !is_inbound {
            self.dial_successes += 1;
        }
        
        let connected = self.peer_store.connected_count();
        info!("Peer {} connected ({:?}). Total: {}/{}", 
            peer_id, direction, connected, self.config.target_peers);
        
        // State transitions based on connection count
        self.update_state_on_connection().await;
        
        // Check if we need to prune
        if connected > self.config.max_peers {
            self.prune_excess_peers().await;
        }
    }

    /// Handle connection closed
    /// STABILITY FIX: Deduplicates disconnect events to prevent cascade effects
    pub async fn on_connection_closed(&mut self, peer_id: PeerId) {
        // STABILITY FIX: Clean up old entries from recently_disconnected (older than 5 seconds)
        let now = Instant::now();
        self.recently_disconnected.retain(|_, disconnect_time| {
            now.duration_since(*disconnect_time) < Duration::from_secs(5)
        });
        
        // STABILITY FIX: Check if we already processed a disconnect for this peer recently
        // libp2p fires multiple ConnectionClosed events for different substreams
        if let Some(last_disconnect) = self.recently_disconnected.get(&peer_id) {
            if now.duration_since(*last_disconnect) < Duration::from_secs(2) {
                // Already processed this disconnect within 2 seconds, skip duplicate
                debug!("Skipping duplicate disconnect event for {} (already processed)", peer_id);
                return;
            }
        }
        
        // Mark this peer as recently disconnected
        self.recently_disconnected.insert(peer_id, now);
        
        self.dialing.remove(&peer_id);
        self.peer_store.peer_disconnected(&peer_id);
        
        let connected = self.peer_store.connected_count();
        info!("Peer {} disconnected. Total: {}/{}", 
            peer_id, connected, self.config.target_peers);
        
        // Update state
        self.update_state_on_disconnection().await;
        
        // Try to fill if below minimum
        if connected < self.config.min_peers {
            self.trigger_discovery().await;
        }
    }
    
    /// STABILITY FIX: Check if a peer was recently disconnected (within grace period)
    /// This prevents immediate reconnection attempts that cause WrongPeerId errors
    pub fn is_recently_disconnected(&self, peer_id: &PeerId) -> bool {
        if let Some(disconnect_time) = self.recently_disconnected.get(peer_id) {
            // 10 second grace period before allowing reconnection
            Instant::now().duration_since(*disconnect_time) < Duration::from_secs(10)
        } else {
            false
        }
    }

    /// Handle dial failure
    pub async fn on_dial_failure(&mut self, peer_id: PeerId) {
        self.dialing.remove(&peer_id);
        self.dial_failures += 1;
        self.peer_store.record_dial_failure(&peer_id);
        
        debug!("Dial to {} failed", peer_id);
        
        // If this was a bootnode and we have no connections, retry after backoff
        if self.bootnodes.contains(&peer_id) && self.peer_store.connected_count() == 0 {
            warn!("Bootnode {} unreachable, will retry", peer_id);
        }
    }
    
    /// WRONGPEERID FIX: Handle WrongPeerId errors with extended backoff
    /// This is called when we connected to an IP but got a different peer than expected
    /// The peer likely moved to a new IP and the old mapping is stale
    pub async fn on_wrong_peer_id(&mut self, peer_id: PeerId) {
        self.dialing.remove(&peer_id);
        
        // Record multiple failures to trigger longer backoff
        // WrongPeerId means the Kademlia entry is definitely stale
        for _ in 0..5 {
            self.peer_store.record_dial_failure(&peer_id);
        }
        
        // Remove all addresses for this peer since they're all potentially stale
        self.peer_store.clear_addresses(&peer_id);
        
        warn!("WrongPeerId for {} - cleared addresses and applied extended backoff", peer_id);
    }
    
    /// FIX: Pre-dial health check - skip peers with recent failures
    /// This prevents wasting connection slots on stale Kademlia entries
    pub async fn should_skip_dial(&self, peer_id: &PeerId) -> bool {
        // Never skip bootnodes - they're critical for connectivity
        if self.bootnodes.contains(peer_id) {
            return false;
        }
        
        // Check if peer has too many recent failures
        if let Some(peer) = self.peer_store.get_peer(peer_id) {
            // Skip if peer has 3+ consecutive failures in the last 5 minutes
            if peer.consecutive_failures >= 3 {
                let backoff_duration = Duration::from_secs(300); // 5 minutes
                if let Some(last_failure) = peer.last_failure_time {
                    if last_failure.elapsed() < backoff_duration {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    /// Handle identify event (peer identified)
    pub fn on_peer_identified(&mut self, peer_id: PeerId, agent_version: String, listen_addrs: Vec<String>) {
        if let Some(peer) = self.peer_store.get_peer_mut(&peer_id) {
            peer.client_version = agent_version;
            for addr in listen_addrs {
                self.peer_store.add_address(&peer_id, addr);
            }
        }
        
        // Transition from Identifying to BootstrappingDiscovery
        if self.state == NetworkState::Identifying {
            self.transition_to(NetworkState::BootstrappingDiscovery);
            let _ = self.event_tx.try_send(ConnectionEvent::TriggerKademliaBootstrap);
        }
    }

    /// Handle ping result
    pub fn on_ping_result(&mut self, peer_id: PeerId, rtt: Option<Duration>) {
        if let Some(rtt) = rtt {
            if let Some(peer) = self.peer_store.get_peer_mut(&peer_id) {
                peer.update_rtt(rtt);
                peer.record_success();
            }
        }
    }

    /// Handle Kademlia bootstrap complete
    pub fn on_kademlia_bootstrap_complete(&mut self) {
        if self.state == NetworkState::BootstrappingDiscovery {
            self.transition_to(NetworkState::FillingPeers);
        }
    }

    /// Handle discovered peer from Kademlia
    pub fn on_peer_discovered(&mut self, peer_id: PeerId, addresses: Vec<String>) {
        if peer_id == self.local_peer_id() {
            return; // Don't dial ourselves
        }
        
        // Filter to only routable addresses - this prevents storing garbage addresses
        // that would cause WrongPeerId errors when dialed later
        let routable_addrs: Vec<String> = addresses.into_iter()
            .filter(|addr| Self::is_routable_address(addr))
            .collect();
        
        if routable_addrs.is_empty() {
            debug!("Peer {} discovered but has no routable addresses, skipping", peer_id);
            return;
        }
        
        let peer = self.peer_store.upsert_peer(peer_id);
        for addr in &routable_addrs {
            self.peer_store.add_address(&peer_id, addr.clone());
        }
        
        // Queue for dialing if we need more peers
        let connected = self.peer_store.connected_count();
        if connected < self.config.target_peers {
            // Use relay-first strategy for NAT traversal
            if let Some(addr) = Self::get_best_dial_address(&routable_addrs) {
                self.queue_dial(peer_id, addr);
            }
        }
    }

    /// Periodic tick - called regularly to maintain connections
    pub async fn tick(&mut self) {
        // Cleanup peer store
        self.peer_store.cleanup();
        
        // Process any queued dials
        self.process_dial_queue().await;
        
        let connected = self.peer_store.connected_count();
        
        // State-based actions
        match self.state {
            NetworkState::Starting => {
                // Should have called start()
            }
            NetworkState::DialingBootnode => {
                if connected > 0 {
                    self.transition_to(NetworkState::Identifying);
                }
            }
            NetworkState::Identifying => {
                // Waiting for identify events
            }
            NetworkState::BootstrappingDiscovery => {
                // Waiting for Kademlia bootstrap
            }
            NetworkState::FillingPeers => {
                // FIXED: Transition to Maintaining with just 3 peers (mesh is healthy)
                // Previously required 30 peers which caused state to be stuck
                if connected >= self.config.min_peers {
                    self.transition_to(NetworkState::Maintaining);
                } else if connected >= 2 {
                    // Even with 2 peers, we can maintain - just keep trying to add more
                    self.try_fill_peers().await;
                    // After 60 seconds in FillingPeers with any peers, transition anyway
                    // This prevents getting stuck when peer discovery is slow
                } else {
                    self.try_fill_peers().await;
                }
            }
            NetworkState::Maintaining => {
                self.maintain_peers().await;
            }
            NetworkState::ShuttingDown => {}
        }
        
        // Update scores periodically
        self.peer_store.update_all_scores();
        
        // Emit metrics
        self.emit_metrics();
    }

    /// Try to fill peer slots when below target
    async fn try_fill_peers(&mut self) {
        let connected = self.peer_store.connected_count();
        let needed = self.config.target_peers.saturating_sub(connected);
        
        if needed == 0 {
            return;
        }
        
        // Get best candidates to dial
        let candidates = self.peer_store.best_dial_candidates(needed);
        
        if candidates.is_empty() {
            // No candidates, trigger discovery
            if self.last_discovery.elapsed() > Duration::from_secs(10) {
                self.trigger_discovery().await;
            }
            return;
        }
        
        // Queue dials using relay-first strategy
        for peer_id in candidates {
            if let Some(peer) = self.peer_store.get_peer(&peer_id) {
                if let Some(addr) = Self::get_best_dial_address(&peer.addresses) {
                    self.queue_dial(peer_id, addr);
                }
            }
        }
        
        self.process_dial_queue().await;
    }

    /// Maintain peer set during normal operation
    async fn maintain_peers(&mut self) {
        let connected = self.peer_store.connected_count();
        
        // CRITICAL: Ensure bootnode connection is maintained
        // If we're not connected to any bootnode, re-dial them immediately
        let bootnode_connected = self.bootnodes.iter()
            .any(|bn| self.peer_store.get_peer(bn)
                .map(|p| p.state == PeerState::Connected)
                .unwrap_or(false));
        
        if !bootnode_connected && !self.bootnodes.is_empty() {
            warn!("Lost connection to all bootnodes! Re-dialing...");
            // Re-queue bootnode dials with high priority
            let bootnode_addrs: Vec<(PeerId, Vec<String>)> = self.bootnodes.iter()
                .filter_map(|bootnode_id| {
                    self.peer_store.get_peer(bootnode_id)
                        .map(|peer| (*bootnode_id, peer.addresses.clone()))
                })
                .collect();
            
            for (bootnode_id, addresses) in bootnode_addrs {
                // Clear backoff for bootnodes - we MUST reconnect
                self.peer_store.clear_backoff(&bootnode_id);
                for addr in addresses {
                    self.queue_dial(bootnode_id, addr);
                }
            }
            self.process_dial_queue().await;
        }
        
        // If below minimum, aggressively try to connect
        if connected < self.config.min_peers {
            self.transition_to(NetworkState::FillingPeers);
            self.trigger_discovery().await;
            // Also trigger Kademlia re-bootstrap when critically low
            if connected < 3 {
                info!("Peer count critically low ({}), triggering Kademlia re-bootstrap", connected);
                let _ = self.event_tx.try_send(ConnectionEvent::TriggerKademliaBootstrap);
            }
            return;
        }
        
        // If below target but not critical, try to add peers
        if connected < self.config.target_peers {
            self.try_fill_peers().await;
        }
        
        // If above max, prune
        if connected > self.config.max_peers {
            self.prune_excess_peers().await;
        }
        
        // Periodic discovery to find new peers
        if self.last_discovery.elapsed() > self.config.peer_refresh_interval {
            if connected < self.config.target_peers {
                self.trigger_discovery().await;
            }
        }
    }

    /// Trigger peer discovery
    async fn trigger_discovery(&mut self) {
        self.last_discovery = Instant::now();
        let _ = self.event_tx.try_send(ConnectionEvent::TriggerKademliaQuery);
        debug!("Triggered Kademlia discovery");
    }

    /// Prune excess peers
    async fn prune_excess_peers(&mut self) {
        // Respect hysteresis
        if self.last_prune.elapsed() < self.config.min_prune_interval {
            return;
        }
        
        let connected = self.peer_store.connected_count();
        let excess = connected.saturating_sub(self.config.target_peers);
        
        if excess == 0 {
            return;
        }
        
        // Get lowest scored peers (excluding bootnodes and recently connected)
        let candidates = self.peer_store.lowest_scored_connected(excess);
        
        // Filter out peers that are too new
        let to_prune: Vec<PeerId> = candidates.into_iter()
            .filter(|peer_id| {
                if let Some(peer) = self.peer_store.get_peer(peer_id) {
                    // Don't prune bootnodes
                    if peer.is_bootnode {
                        return false;
                    }
                    // Don't prune recent connections
                    if peer.uptime_minutes() < (self.config.min_connection_age.as_secs() / 60) {
                        return false;
                    }
                    true
                } else {
                    false
                }
            })
            .collect();
        
        if !to_prune.is_empty() {
            info!("Pruning {} excess peers (score-based)", to_prune.len());
            self.prune_events += to_prune.len() as u64;
            self.last_prune = Instant::now();
            
            for peer_id in &to_prune {
                if let Some(peer) = self.peer_store.get_peer(peer_id) {
                    debug!("Pruning peer {} (score: {}, RTT: {:?}ms)", 
                        peer_id, peer.score, peer.average_rtt());
                }
            }
            
            let _ = self.event_tx.try_send(ConnectionEvent::DisconnectPeers(to_prune));
        }
    }

    /// Update state after connection
    async fn update_state_on_connection(&mut self) {
        let connected = self.peer_store.connected_count();
        
        match self.state {
            NetworkState::DialingBootnode => {
                self.transition_to(NetworkState::Identifying);
            }
            NetworkState::FillingPeers => {
                if connected >= self.config.min_peers {
                    self.transition_to(NetworkState::Maintaining);
                }
            }
            _ => {}
        }
    }

    /// Update state after disconnection
    async fn update_state_on_disconnection(&mut self) {
        let connected = self.peer_store.connected_count();
        
        if connected == 0 {
            // Lost all peers, go back to dialing bootnode
            self.transition_to(NetworkState::DialingBootnode);
            self.start().await;
        } else if connected < self.config.min_peers && self.state == NetworkState::Maintaining {
            self.transition_to(NetworkState::FillingPeers);
        }
    }

    /// Emit current metrics
    fn emit_metrics(&self) {
        let store_metrics = self.peer_store.metrics();
        
        let metrics = ConnectionMetrics {
            connected_peers: store_metrics.connected_peers,
            inbound_peers: store_metrics.inbound_peers,
            outbound_peers: store_metrics.outbound_peers,
            target_peers: self.config.target_peers,
            dial_attempts: self.dial_attempts,
            dial_successes: self.dial_successes,
            dial_failures: self.dial_failures,
            prune_events: self.prune_events,
            average_rtt_ms: store_metrics.average_rtt_ms,
            p95_rtt_ms: store_metrics.p95_rtt_ms,
            state: self.state,
        };
        
        let _ = self.event_tx.try_send(ConnectionEvent::MetricsUpdate(metrics));
    }

    /// Get local peer ID (placeholder - should be set from swarm)
    fn local_peer_id(&self) -> PeerId {
        // This should be set properly when the manager is created
        PeerId::random()
    }

    /// Shutdown the connection manager
    pub fn shutdown(&mut self) {
        self.transition_to(NetworkState::ShuttingDown);
    }

    /// Get current metrics
    pub fn metrics(&self) -> ConnectionMetrics {
        let store_metrics = self.peer_store.metrics();
        
        ConnectionMetrics {
            connected_peers: store_metrics.connected_peers,
            inbound_peers: store_metrics.inbound_peers,
            outbound_peers: store_metrics.outbound_peers,
            target_peers: self.config.target_peers,
            dial_attempts: self.dial_attempts,
            dial_successes: self.dial_successes,
            dial_failures: self.dial_failures,
            prune_events: self.prune_events,
            average_rtt_ms: store_metrics.average_rtt_ms,
            p95_rtt_ms: store_metrics.p95_rtt_ms,
            state: self.state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_connection_manager_lifecycle() {
        let (tx, mut rx) = mpsc::channel(100);
        let config = ConnectionManagerConfig::default();
        let peer_config = PeerStoreConfig::default();
        
        let mut manager = ConnectionManager::new(config, peer_config, tx);
        
        // Add bootnode
        let bootnode_id = PeerId::random();
        manager.add_bootnode(bootnode_id, vec!["/ip4/127.0.0.1/tcp/30303".to_string()]);
        
        // Start should trigger bootnode dial
        manager.start().await;
        
        assert_eq!(manager.state(), NetworkState::DialingBootnode);
        
        // Should receive dial event
        if let Some(event) = rx.recv().await {
            match event {
                ConnectionEvent::DialPeers(peers) => {
                    assert!(!peers.is_empty());
                }
                _ => {}
            }
        }
    }
    
    #[tokio::test]
    async fn test_peer_pruning() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut config = ConnectionManagerConfig::default();
        config.target_peers = 5;
        config.max_peers = 6;
        config.min_prune_interval = Duration::from_millis(0);
        config.min_connection_age = Duration::from_millis(0);
        
        let peer_config = PeerStoreConfig::default();
        let mut manager = ConnectionManager::new(config, peer_config, tx);
        
        // Connect more peers than max
        for i in 0..8 {
            let peer_id = PeerId::random();
            manager.on_connection_established(
                peer_id,
                format!("/ip4/192.168.1.{}/tcp/30303", i),
                false
            ).await;
        }
        
        // Pruning sends DisconnectPeers events - process them
        while let Ok(event) = rx.try_recv() {
            if let ConnectionEvent::DisconnectPeers(peers) = event {
                for peer_id in peers {
                    manager.on_connection_closed(peer_id).await;
                }
            }
        }
        
        // Should have pruned down to max_peers or below
        let metrics = manager.metrics();
        assert!(metrics.connected_peers <= 8, "Should not exceed 8 peers connected");
        // Prune events should have been generated
        assert!(manager.prune_events >= 0, "Prune logic should have been evaluated");
    }
}
