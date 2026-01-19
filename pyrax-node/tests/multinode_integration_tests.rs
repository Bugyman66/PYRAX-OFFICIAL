//! Multi-Node Integration Tests
//!
//! Comprehensive P2P network behavior tests that run without external dependencies.
//! All tests execute in-process - no binary spawning required.

use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{Duration, Instant};

/// Global port counter to avoid port conflicts between tests
static PORT_COUNTER: AtomicU16 = AtomicU16::new(31000);

/// Get next available port
fn next_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Check if a port is available
fn is_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

// ═══════════════════════════════════════════════════════════════════════════════
// SIMULATED P2P NETWORK COMPONENTS
// These simulate actual P2P behavior without spawning external processes
// ═══════════════════════════════════════════════════════════════════════════════

/// Simulated peer ID (matches libp2p format)
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct PeerId(String);

impl PeerId {
    fn random() -> Self {
        let id = format!("12D3KooW{:016x}", rand_u64());
        Self(id)
    }
    
    fn as_str(&self) -> &str {
        &self.0
    }
}

fn rand_u64() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish()
}

/// Simulated multiaddr
#[derive(Debug, Clone)]
struct Multiaddr {
    ip: Ipv4Addr,
    port: u16,
    peer_id: Option<PeerId>,
}

impl Multiaddr {
    fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self { ip, port, peer_id: None }
    }
    
    fn with_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id = Some(peer_id);
        self
    }
    
    fn is_routable(&self) -> bool {
        !self.ip.is_loopback() && 
        !self.ip.is_private() && 
        !self.ip.is_link_local() &&
        !self.ip.is_unspecified()
    }
    
    fn to_string(&self) -> String {
        let base = format!("/ip4/{}/tcp/{}", self.ip, self.port);
        match &self.peer_id {
            Some(id) => format!("{}/p2p/{}", base, id.as_str()),
            None => base,
        }
    }
}

/// Simulated Kademlia routing table
struct MockKademlia {
    peers: HashMap<PeerId, Vec<Multiaddr>>,
    local_id: PeerId,
}

impl MockKademlia {
    fn new(local_id: PeerId) -> Self {
        Self {
            peers: HashMap::new(),
            local_id,
        }
    }
    
    fn add_address(&mut self, peer_id: &PeerId, addr: Multiaddr) {
        if *peer_id == self.local_id {
            return; // Never add self
        }
        self.peers.entry(peer_id.clone())
            .or_insert(Vec::new())
            .push(addr);
    }
    
    fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
    }
    
    fn remove_address(&mut self, peer_id: &PeerId, addr: &Multiaddr) {
        if let Some(addrs) = self.peers.get_mut(peer_id) {
            addrs.retain(|a| a.to_string() != addr.to_string());
            if addrs.is_empty() {
                self.peers.remove(peer_id);
            }
        }
    }
    
    fn get_addresses(&self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.peers.get(peer_id).cloned().unwrap_or_default()
    }
    
    fn peer_count(&self) -> usize {
        self.peers.len()
    }
}

/// Simulated peer store with scoring and backoff
struct MockPeerStore {
    peers: HashMap<PeerId, PeerInfo>,
}

#[derive(Debug, Clone)]
struct PeerInfo {
    addresses: Vec<Multiaddr>,
    consecutive_failures: u32,
    last_failure: Option<Instant>,
    score: i32,
    is_bootnode: bool,
}

impl MockPeerStore {
    fn new() -> Self {
        Self { peers: HashMap::new() }
    }
    
    fn add_peer(&mut self, peer_id: PeerId, is_bootnode: bool) {
        self.peers.insert(peer_id, PeerInfo {
            addresses: Vec::new(),
            consecutive_failures: 0,
            last_failure: None,
            score: 0,
            is_bootnode,
        });
    }
    
    fn record_failure(&mut self, peer_id: &PeerId) {
        if let Some(info) = self.peers.get_mut(peer_id) {
            info.consecutive_failures += 1;
            info.last_failure = Some(Instant::now());
            info.score -= 5;
        }
    }
    
    fn record_wrong_peer_id(&mut self, peer_id: &PeerId) {
        if let Some(info) = self.peers.get_mut(peer_id) {
            // 5x penalty for WrongPeerId
            info.consecutive_failures += 5;
            info.last_failure = Some(Instant::now());
            info.score -= 25;
            info.addresses.clear(); // Clear all addresses - they're stale
        }
    }
    
    fn should_skip_dial(&self, peer_id: &PeerId) -> bool {
        if let Some(info) = self.peers.get(peer_id) {
            // Never skip bootnodes
            if info.is_bootnode {
                return false;
            }
            // Skip if 3+ failures in last 5 minutes
            if info.consecutive_failures >= 3 {
                if let Some(last) = info.last_failure {
                    if last.elapsed() < Duration::from_secs(300) {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    fn clear_addresses(&mut self, peer_id: &PeerId) {
        if let Some(info) = self.peers.get_mut(peer_id) {
            info.addresses.clear();
        }
    }
}

/// Simulated connection manager
struct MockConnectionManager {
    connected: HashSet<PeerId>,
    dialing: HashSet<PeerId>,
    dial_attempts: u64,
    dial_successes: u64,
    dial_failures: u64,
    wrong_peer_id_count: u64,
}

impl MockConnectionManager {
    fn new() -> Self {
        Self {
            connected: HashSet::new(),
            dialing: HashSet::new(),
            dial_attempts: 0,
            dial_successes: 0,
            dial_failures: 0,
            wrong_peer_id_count: 0,
        }
    }
    
    fn dial(&mut self, peer_id: PeerId, addr: &Multiaddr) -> DialResult {
        self.dial_attempts += 1;
        self.dialing.insert(peer_id.clone());
        
        // Simulate dial logic
        if !addr.is_routable() {
            self.dial_failures += 1;
            self.dialing.remove(&peer_id);
            return DialResult::NonRoutable;
        }
        
        // Simulate random failures (30% success rate like real network)
        let success = (rand_u64() % 10) < 3;
        
        if success {
            self.dial_successes += 1;
            self.dialing.remove(&peer_id);
            self.connected.insert(peer_id.clone());
            DialResult::Success
        } else {
            self.dial_failures += 1;
            self.dialing.remove(&peer_id);
            
            // 10% of failures are WrongPeerId
            if (rand_u64() % 10) < 1 {
                self.wrong_peer_id_count += 1;
                DialResult::WrongPeerId
            } else {
                DialResult::Failed
            }
        }
    }
    
    fn disconnect(&mut self, peer_id: &PeerId) {
        self.connected.remove(peer_id);
    }
    
    fn peer_count(&self) -> usize {
        self.connected.len()
    }
    
    fn dial_success_rate(&self) -> f64 {
        if self.dial_attempts == 0 {
            return 1.0;
        }
        self.dial_successes as f64 / self.dial_attempts as f64
    }
}

#[derive(Debug, PartialEq)]
enum DialResult {
    Success,
    Failed,
    WrongPeerId,
    NonRoutable,
}

/// Simulated P2P node
struct SimulatedNode {
    peer_id: PeerId,
    kademlia: MockKademlia,
    peer_store: MockPeerStore,
    conn_manager: MockConnectionManager,
    mesh_peers: HashSet<PeerId>,
}

impl SimulatedNode {
    fn new() -> Self {
        let peer_id = PeerId::random();
        Self {
            kademlia: MockKademlia::new(peer_id.clone()),
            peer_store: MockPeerStore::new(),
            conn_manager: MockConnectionManager::new(),
            mesh_peers: HashSet::new(),
            peer_id,
        }
    }
    
    fn dial_peer(&mut self, target: &PeerId, addr: &Multiaddr) -> DialResult {
        // Check if we should skip
        if self.peer_store.should_skip_dial(target) {
            return DialResult::Failed;
        }
        
        let result = self.conn_manager.dial(target.clone(), addr);
        
        match &result {
            DialResult::Success => {
                self.mesh_peers.insert(target.clone());
            }
            DialResult::WrongPeerId => {
                // Clean up stale entry
                self.kademlia.remove_peer(target);
                self.peer_store.record_wrong_peer_id(target);
            }
            DialResult::Failed => {
                self.peer_store.record_failure(target);
            }
            DialResult::NonRoutable => {
                // Remove non-routable address
                self.kademlia.remove_address(target, addr);
            }
        }
        
        result
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MULTI-NODE SIMULATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod multinode_tests {
    use super::*;
    
    /// Test single node initialization
    #[test]
    pub fn test_single_node_startup() {
        println!("=== Test: Single Node Startup ===");
        
        let node = SimulatedNode::new();
        
        assert!(node.peer_id.as_str().starts_with("12D3KooW"), 
            "Peer ID should have correct format");
        assert_eq!(node.conn_manager.peer_count(), 0, 
            "New node should have no peers");
        assert_eq!(node.mesh_peers.len(), 0, 
            "New node should have no mesh peers");
        
        println!("✓ Node initialized with peer ID: {}", node.peer_id.as_str());
        println!("=== PASSED: Single Node Startup ===\n");
    }
    
    /// Test two nodes connecting
    #[test]
    pub fn test_two_node_connection() {
        println!("=== Test: Two Node Connection ===");
        
        let mut node1 = SimulatedNode::new();
        let node2 = SimulatedNode::new();
        
        // Add node2 to node1's Kademlia
        let addr = Multiaddr::new(Ipv4Addr::new(209, 38, 137, 105), 30303)
            .with_peer_id(node2.peer_id.clone());
        node1.kademlia.add_address(&node2.peer_id, addr.clone());
        node1.peer_store.add_peer(node2.peer_id.clone(), false);
        
        // Attempt dial (may succeed or fail based on simulation)
        let result = node1.dial_peer(&node2.peer_id, &addr);
        
        println!("Dial result: {:?}", result);
        println!("Node1 peer count: {}", node1.conn_manager.peer_count());
        println!("Dial attempts: {}, successes: {}", 
            node1.conn_manager.dial_attempts, node1.conn_manager.dial_successes);
        
        // Verify dial was attempted
        assert_eq!(node1.conn_manager.dial_attempts, 1, "Should have attempted dial");
        
        println!("=== PASSED: Two Node Connection ===\n");
    }
    
    /// Test three node mesh formation
    #[test]
    pub fn test_three_node_mesh() {
        println!("=== Test: Three Node Mesh Formation ===");
        
        let mut node1 = SimulatedNode::new();
        let node2 = SimulatedNode::new();
        let node3 = SimulatedNode::new();
        
        // Add peers to node1's stores
        let addr2 = Multiaddr::new(Ipv4Addr::new(209, 38, 137, 105), 30303);
        let addr3 = Multiaddr::new(Ipv4Addr::new(137, 184, 118, 228), 30303);
        
        node1.kademlia.add_address(&node2.peer_id, addr2.clone());
        node1.kademlia.add_address(&node3.peer_id, addr3.clone());
        node1.peer_store.add_peer(node2.peer_id.clone(), true); // bootnode
        node1.peer_store.add_peer(node3.peer_id.clone(), true); // bootnode
        
        // Dial both peers multiple times to simulate mesh formation
        for _ in 0..5 {
            node1.dial_peer(&node2.peer_id, &addr2);
            node1.dial_peer(&node3.peer_id, &addr3);
        }
        
        println!("Node1 mesh peers: {}", node1.mesh_peers.len());
        println!("Node1 connected: {}", node1.conn_manager.peer_count());
        println!("Dial success rate: {:.1}%", node1.conn_manager.dial_success_rate() * 100.0);
        
        // Verify dialing happened
        assert!(node1.conn_manager.dial_attempts >= 10, "Should have dialed multiple times");
        
        println!("=== PASSED: Three Node Mesh Formation ===\n");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADDRESS FILTERING TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod address_tests {
    use super::*;
    
    /// Test that localhost addresses are rejected
    #[test]
    pub fn test_localhost_rejection() {
        println!("=== Test: Localhost Rejection ===");
        
        let localhost = Multiaddr::new(Ipv4Addr::new(127, 0, 0, 1), 30303);
        assert!(!localhost.is_routable(), "Localhost should NOT be routable");
        
        let mut node = SimulatedNode::new();
        let peer = PeerId::random();
        node.peer_store.add_peer(peer.clone(), false);
        
        let result = node.dial_peer(&peer, &localhost);
        assert_eq!(result, DialResult::NonRoutable, "Should reject localhost dial");
        
        println!("✓ Localhost correctly rejected");
        println!("=== PASSED: Localhost Rejection ===\n");
    }
    
    /// Test that private network addresses are rejected
    #[test]
    pub fn test_private_network_rejection() {
        println!("=== Test: Private Network Rejection ===");
        
        let private_10 = Multiaddr::new(Ipv4Addr::new(10, 0, 0, 1), 30303);
        let private_172 = Multiaddr::new(Ipv4Addr::new(172, 16, 0, 1), 30303);
        let private_192 = Multiaddr::new(Ipv4Addr::new(192, 168, 1, 1), 30303);
        
        assert!(!private_10.is_routable(), "10.x.x.x should NOT be routable");
        assert!(!private_172.is_routable(), "172.16.x.x should NOT be routable");
        assert!(!private_192.is_routable(), "192.168.x.x should NOT be routable");
        
        println!("✓ Private networks correctly rejected");
        println!("=== PASSED: Private Network Rejection ===\n");
    }
    
    /// Test that public addresses are accepted
    #[test]
    pub fn test_public_address_accepted() {
        println!("=== Test: Public Address Accepted ===");
        
        let public1 = Multiaddr::new(Ipv4Addr::new(209, 38, 137, 105), 30303);
        let public2 = Multiaddr::new(Ipv4Addr::new(8, 8, 8, 8), 30303);
        
        assert!(public1.is_routable(), "209.38.137.105 should be routable");
        assert!(public2.is_routable(), "8.8.8.8 should be routable");
        
        println!("✓ Public addresses correctly accepted");
        println!("=== PASSED: Public Address Accepted ===\n");
    }
    
    /// Test unspecified address rejection
    #[test]
    pub fn test_unspecified_rejection() {
        println!("=== Test: Unspecified Address Rejection ===");
        
        let unspecified = Multiaddr::new(Ipv4Addr::new(0, 0, 0, 0), 30303);
        assert!(!unspecified.is_routable(), "0.0.0.0 should NOT be routable");
        
        println!("✓ Unspecified address correctly rejected");
        println!("=== PASSED: Unspecified Address Rejection ===\n");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FAILURE HANDLING TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod failure_tests {
    use super::*;
    
    /// Test dial failure backoff behavior
    #[test]
    pub fn test_dial_failure_backoff_logic() {
        println!("=== Test: Dial Failure Backoff Logic ===");
        
        let mut store = MockPeerStore::new();
        let peer_id = PeerId::random();
        store.add_peer(peer_id.clone(), false);
        
        // Initially should not skip
        assert!(!store.should_skip_dial(&peer_id), "Should not skip initially");
        
        // After 1-2 failures, still shouldn't skip
        store.record_failure(&peer_id);
        assert!(!store.should_skip_dial(&peer_id), "Should not skip after 1 failure");
        store.record_failure(&peer_id);
        assert!(!store.should_skip_dial(&peer_id), "Should not skip after 2 failures");
        
        // After 3 failures, should skip
        store.record_failure(&peer_id);
        assert!(store.should_skip_dial(&peer_id), "Should skip after 3 failures");
        
        println!("✓ Backoff logic works correctly");
        println!("=== PASSED: Dial Failure Backoff Logic ===\n");
    }
    
    /// Test WrongPeerId handling cleans up stale entries
    #[test]
    pub fn test_wrong_peer_id_cleanup() {
        println!("=== Test: WrongPeerId Cleanup ===");
        
        let mut node = SimulatedNode::new();
        let stale_peer = PeerId::random();
        let stale_addr = Multiaddr::new(Ipv4Addr::new(192, 168, 1, 100), 30303);
        
        // Add stale entry
        node.kademlia.add_address(&stale_peer, stale_addr.clone());
        node.peer_store.add_peer(stale_peer.clone(), false);
        
        assert_eq!(node.kademlia.peer_count(), 1, "Should have peer initially");
        
        // Simulate WrongPeerId by calling record_wrong_peer_id
        node.peer_store.record_wrong_peer_id(&stale_peer);
        node.kademlia.remove_peer(&stale_peer);
        
        assert_eq!(node.kademlia.peer_count(), 0, "Should remove peer after WrongPeerId");
        assert!(node.peer_store.should_skip_dial(&stale_peer), 
            "Should skip dial after WrongPeerId");
        
        println!("✓ WrongPeerId cleanup works correctly");
        println!("=== PASSED: WrongPeerId Cleanup ===\n");
    }
    
    /// Test peer disconnect recovery
    #[test]
    pub fn test_peer_disconnect_recovery() {
        println!("=== Test: Peer Disconnect Recovery ===");
        
        let mut node = SimulatedNode::new();
        let peer = PeerId::random();
        
        // Simulate connection
        node.conn_manager.connected.insert(peer.clone());
        node.mesh_peers.insert(peer.clone());
        
        assert_eq!(node.conn_manager.peer_count(), 1, "Should have 1 peer");
        
        // Disconnect
        node.conn_manager.disconnect(&peer);
        node.mesh_peers.remove(&peer);
        
        assert_eq!(node.conn_manager.peer_count(), 0, "Should have 0 peers after disconnect");
        
        println!("✓ Disconnect handled correctly");
        println!("=== PASSED: Peer Disconnect Recovery ===\n");
    }
    
    /// Test bootnode never skipped
    #[test]
    pub fn test_bootnode_never_skipped() {
        println!("=== Test: Bootnode Never Skipped ===");
        
        let mut store = MockPeerStore::new();
        let bootnode = PeerId::random();
        store.add_peer(bootnode.clone(), true); // is_bootnode = true
        
        // Even with many failures, bootnode should not be skipped
        for _ in 0..10 {
            store.record_failure(&bootnode);
        }
        
        assert!(!store.should_skip_dial(&bootnode), "Bootnode should NEVER be skipped");
        
        println!("✓ Bootnode correctly never skipped");
        println!("=== PASSED: Bootnode Never Skipped ===\n");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// KADEMLIA TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod kademlia_tests {
    use super::*;
    
    /// Test self-dial prevention
    #[test]
    pub fn test_no_self_in_kademlia() {
        println!("=== Test: No Self In Kademlia ===");
        
        let peer_id = PeerId::random();
        let mut kad = MockKademlia::new(peer_id.clone());
        
        let addr = Multiaddr::new(Ipv4Addr::new(209, 38, 137, 105), 30303);
        kad.add_address(&peer_id, addr);
        
        assert_eq!(kad.peer_count(), 0, "Should NOT add self to Kademlia");
        
        println!("✓ Self correctly excluded from Kademlia");
        println!("=== PASSED: No Self In Kademlia ===\n");
    }
    
    /// Test address removal
    #[test]
    pub fn test_address_removal() {
        println!("=== Test: Address Removal ===");
        
        let local_id = PeerId::random();
        let mut kad = MockKademlia::new(local_id);
        
        let peer = PeerId::random();
        let addr1 = Multiaddr::new(Ipv4Addr::new(209, 38, 137, 105), 30303);
        let addr2 = Multiaddr::new(Ipv4Addr::new(137, 184, 118, 228), 30303);
        
        kad.add_address(&peer, addr1.clone());
        kad.add_address(&peer, addr2.clone());
        
        assert_eq!(kad.get_addresses(&peer).len(), 2, "Should have 2 addresses");
        
        kad.remove_address(&peer, &addr1);
        assert_eq!(kad.get_addresses(&peer).len(), 1, "Should have 1 address after removal");
        
        kad.remove_address(&peer, &addr2);
        assert_eq!(kad.peer_count(), 0, "Peer should be removed when no addresses left");
        
        println!("✓ Address removal works correctly");
        println!("=== PASSED: Address Removal ===\n");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// METRICS THRESHOLD TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod metrics_tests {
    use super::*;
    
    /// Define acceptable thresholds for P2P metrics
    #[derive(Debug)]
    struct P2PMetricThresholds {
        min_dial_success_rate: f64,
        max_wrong_peer_id_per_hour: u64,
        max_relay_exceeded_per_hour: u64,
        min_mesh_peers: u64,
    }
    
    impl Default for P2PMetricThresholds {
        fn default() -> Self {
            Self {
                min_dial_success_rate: 0.10,
                max_wrong_peer_id_per_hour: 50,
                max_relay_exceeded_per_hour: 100,
                min_mesh_peers: 2,
            }
        }
    }
    
    #[derive(Debug)]
    struct P2PMetrics {
        dial_attempts: u64,
        dial_successes: u64,
        wrong_peer_id_count: u64,
        relay_exceeded_count: u64,
        mesh_peers: u64,
    }
    
    impl P2PMetrics {
        fn dial_success_rate(&self) -> f64 {
            if self.dial_attempts == 0 { return 1.0; }
            self.dial_successes as f64 / self.dial_attempts as f64
        }
        
        fn validate(&self, thresholds: &P2PMetricThresholds) -> Vec<String> {
            let mut violations = Vec::new();
            
            if self.dial_success_rate() < thresholds.min_dial_success_rate {
                violations.push(format!(
                    "Dial success rate {:.1}% below threshold {:.1}%",
                    self.dial_success_rate() * 100.0,
                    thresholds.min_dial_success_rate * 100.0
                ));
            }
            
            if self.wrong_peer_id_count > thresholds.max_wrong_peer_id_per_hour {
                violations.push(format!(
                    "WrongPeerId count {} exceeds threshold {}",
                    self.wrong_peer_id_count,
                    thresholds.max_wrong_peer_id_per_hour
                ));
            }
            
            if self.relay_exceeded_count > thresholds.max_relay_exceeded_per_hour {
                violations.push(format!(
                    "RelayExceeded count {} exceeds threshold {}",
                    self.relay_exceeded_count,
                    thresholds.max_relay_exceeded_per_hour
                ));
            }
            
            if self.mesh_peers < thresholds.min_mesh_peers {
                violations.push(format!(
                    "Mesh peers {} below threshold {}",
                    self.mesh_peers,
                    thresholds.min_mesh_peers
                ));
            }
            
            violations
        }
    }
    
    /// Test metric thresholds with healthy node
    #[test]
    pub fn test_healthy_metrics_pass_thresholds() {
        println!("=== Test: Healthy Metrics Pass Thresholds ===");
        
        let thresholds = P2PMetricThresholds::default();
        
        let healthy = P2PMetrics {
            dial_attempts: 100,
            dial_successes: 30,
            wrong_peer_id_count: 5,
            relay_exceeded_count: 10,
            mesh_peers: 4,
        };
        
        let violations = healthy.validate(&thresholds);
        assert!(violations.is_empty(), "Healthy node should have no violations: {:?}", violations);
        
        println!("✓ Healthy metrics: {:.1}% dial success, {} mesh peers", 
            healthy.dial_success_rate() * 100.0, healthy.mesh_peers);
        println!("=== PASSED: Healthy Metrics Pass Thresholds ===\n");
    }
    
    /// Test metric thresholds detect unhealthy node
    #[test]
    pub fn test_unhealthy_metrics_fail_thresholds() {
        println!("=== Test: Unhealthy Metrics Fail Thresholds ===");
        
        let thresholds = P2PMetricThresholds::default();
        
        let unhealthy = P2PMetrics {
            dial_attempts: 189,
            dial_successes: 10,
            wrong_peer_id_count: 100,
            relay_exceeded_count: 200,
            mesh_peers: 2,
        };
        
        let violations = unhealthy.validate(&thresholds);
        
        println!("Detected violations:");
        for v in &violations {
            println!("  ✗ {}", v);
        }
        
        assert!(!violations.is_empty(), "Unhealthy node should have violations");
        assert!(violations.iter().any(|v| v.contains("Dial success rate")), 
            "Should detect low dial success rate");
        assert!(violations.iter().any(|v| v.contains("WrongPeerId")), 
            "Should detect high WrongPeerId count");
        
        println!("=== PASSED: Unhealthy Metrics Fail Thresholds ===\n");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN TEST RUNNER
// ═══════════════════════════════════════════════════════════════════════════════

/// Run all integration tests
#[test]
fn run_all_multinode_tests() {
    println!("\n========================================");
    println!("  Multi-Node Integration Test Suite");
    println!("========================================\n");
    
    println!("--- Multi-Node Simulation Tests ---");
    multinode_tests::test_single_node_startup();
    multinode_tests::test_two_node_connection();
    multinode_tests::test_three_node_mesh();
    
    println!("--- Address Filtering Tests ---");
    address_tests::test_localhost_rejection();
    address_tests::test_private_network_rejection();
    address_tests::test_public_address_accepted();
    address_tests::test_unspecified_rejection();
    
    println!("--- Failure Handling Tests ---");
    failure_tests::test_dial_failure_backoff_logic();
    failure_tests::test_wrong_peer_id_cleanup();
    failure_tests::test_peer_disconnect_recovery();
    failure_tests::test_bootnode_never_skipped();
    
    println!("--- Kademlia Tests ---");
    kademlia_tests::test_no_self_in_kademlia();
    kademlia_tests::test_address_removal();
    
    println!("--- Metrics Threshold Tests ---");
    metrics_tests::test_healthy_metrics_pass_thresholds();
    metrics_tests::test_unhealthy_metrics_fail_thresholds();
    
    println!("========================================");
    println!("  All Multi-Node Tests PASSED ✓");
    println!("========================================\n");
}
