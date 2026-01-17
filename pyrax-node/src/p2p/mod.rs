//! P2P Networking for PYRAX using libp2p
//!
//! DESIGN DOC - Mesh Networking Architecture
//! ==========================================
//!
//! State Machine:
//! START → DIAL_BOOTNODE → IDENTIFY → BOOTSTRAP_DISCOVERY → FILL_PEERS → MAINTAIN_PEERS (loop)
//!
//! Connection Parameters:
//! - TARGET_PEERS = 50 (steady-state goal)
//! - MIN_PEERS = 30 (dial aggressively below this)
//! - MAX_PEERS = 60 (prune above this)
//! - MAX_CONCURRENT_DIALS = 5 (avoid dial storms)
//! - DIAL_TIMEOUT = 10s
//! - KEEPALIVE_PING_INTERVAL = 15s
//! - PEER_REFRESH_INTERVAL = 30s
//! - PEER_REEVALUATE_INTERVAL = 60s
//!
//! Peer Scoring:
//! - RTT bonus: + (50 - min(RTT_ms, 200)) * 0.1
//! - Uptime bonus: + uptime_minutes * 0.05 (max 10 points)
//! - Disconnect penalty: - disconnects_last_hour * 2
//! - Failure penalty: - failures_last_hour * 1
//! - Subnet diversity penalty: -5 if same /24 has >= 3 peers
//! - Score clamped to [-50, +50]
//!
//! Production-ready P2P layer with:
//! - Kademlia DHT for peer discovery (primary)
//! - GossipSub for block/transaction propagation
//! - mDNS for local peer discovery (LAN)
//! - Ping for keep-alive and RTT measurement
//! - Identify for peer information exchange
//! - Connection Manager for mesh maintenance

mod registry;
mod peer_store;
mod connection_manager;

pub use registry::{PeerRegistry, ConnectedPeer, PeerDirection, parse_multiaddr};
pub use peer_store::{PeerStore, PeerStoreConfig, PeerData, PeerStoreMetrics};
pub use connection_manager::{ConnectionManager, ConnectionManagerConfig, ConnectionMetrics, NetworkState, ConnectionEvent};

use libp2p::{
    gossipsub, identify, kad, mdns, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, debug, error};
use futures::StreamExt;

use crate::types::{Block, Transaction, H256, NetworkId, BlockHeader};
use crate::storage::ChainDB;

/// P2P network configuration with mesh networking parameters
#[derive(Debug, Clone)]
pub struct P2PConfig {
    /// Listen address (multiaddr format)
    pub listen_addr: String,
    /// Bootstrap peer addresses
    pub bootstrap_peers: Vec<String>,
    /// Target number of peers (steady-state)
    pub target_peers: usize,
    /// Minimum peers (dial aggressively below this)
    pub min_peers: usize,
    /// Maximum peers (prune above this)  
    pub max_peers: usize,
    /// Maximum concurrent dial attempts
    pub max_concurrent_dials: usize,
    /// Dial timeout in seconds
    pub dial_timeout_secs: u64,
    /// Ping interval in seconds
    pub ping_interval_secs: u64,
    /// Peer refresh interval in seconds
    pub peer_refresh_interval_secs: u64,
    /// Peer reevaluation interval in seconds
    pub peer_reevaluate_interval_secs: u64,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/30303".to_string(),
            bootstrap_peers: vec![],
            target_peers: 50,
            min_peers: 30,
            max_peers: 60,
            max_concurrent_dials: 5,
            dial_timeout_secs: 10,
            ping_interval_secs: 15,
            peer_refresh_interval_secs: 30,
            peer_reevaluate_interval_secs: 60,
        }
    }
}

/// Messages broadcast over gossipsub
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GossipMessage {
    /// New block announcement
    NewBlock(Block),
    /// New transaction announcement
    NewTransaction(Transaction),
    /// Block header announcement (for initial sync)
    NewHeader(BlockHeader),
    /// Request blocks from a height range
    GetBlocks { start_height: u64, count: u64 },
    /// Response with blocks
    Blocks(Vec<Block>),
}

/// Request/Response protocol messages
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncRequest {
    /// Request block by hash
    GetBlock(H256),
    /// Request blocks by height range
    GetBlocks { start: u64, count: u32 },
    /// Request headers by height range
    GetHeaders { start: u64, count: u32 },
    /// Get peer's chain tip
    GetStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncResponse {
    Block(Option<Block>),
    Blocks(Vec<Block>),
    Headers(Vec<BlockHeader>),
    Status { height: u64, hash: H256, total_difficulty: u64 },
}

/// Combined network behaviour using libp2p macros
#[derive(NetworkBehaviour)]
pub struct PyraxBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub relay_server: relay::Behaviour,
    pub relay_client: relay::client::Behaviour,
}

/// P2P Network manager with mesh networking support
pub struct Network {
    /// Local peer ID
    local_peer_id: PeerId,
    /// libp2p swarm
    swarm: Swarm<PyraxBehaviour>,
    /// Chain database
    db: Arc<ChainDB>,
    /// Network identifier
    network_id: NetworkId,
    /// Connection manager for mesh maintenance
    conn_manager: ConnectionManager,
    /// Receiver for connection manager events
    conn_event_rx: mpsc::Receiver<ConnectionEvent>,
    /// Legacy peer registry (for RPC compatibility)
    peer_registry: PeerRegistry,
    /// Bootstrap peer addresses
    bootstrap_peers: Vec<String>,
    /// Currently dialing peers (to avoid duplicate dials)
    dialing: HashSet<PeerId>,
    /// Peers pending disconnection
    pending_disconnect: HashSet<PeerId>,
    /// Configuration
    config: P2PConfig,
    /// Channels for received blocks/txs
    block_tx: mpsc::Sender<Block>,
    block_rx: Option<mpsc::Receiver<Block>>,
    tx_tx: mpsc::Sender<Transaction>,
    tx_rx: Option<mpsc::Receiver<Transaction>>,
    /// Metrics
    metrics: NetworkMetrics,
}

/// Network metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    pub connected_peers: usize,
    pub inbound_peers: usize,
    pub outbound_peers: usize,
    pub target_peers: usize,
    pub dial_attempts: u64,
    pub dial_successes: u64,
    pub dial_failures: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub blocks_received: u64,
    pub txs_received: u64,
    pub average_rtt_ms: Option<u64>,
    pub state: NetworkState,
}

impl Network {
    /// Create a new P2P network with mesh networking and connection management
    pub async fn new(config: P2PConfig, db: Arc<ChainDB>, network_id: NetworkId, peer_registry: PeerRegistry) -> anyhow::Result<Self> {
        info!("╔═══════════════════════════════════════════════════════════════╗");
        info!("║     PYRAX P2P Network - Mesh Networking Initialized           ║");
        info!("║   Target: {} peers | Min: {} | Max: {}                    ║", 
            config.target_peers, config.min_peers, config.max_peers);
        info!("╚═══════════════════════════════════════════════════════════════╝");
        info!("Initializing P2P network for {}", network_id.name());

        // Generate keypair
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer ID: {}", local_peer_id);

        // Build swarm with tokio runtime and relay client for NAT traversal
        let ping_interval = Duration::from_secs(config.ping_interval_secs);
        let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            // Add relay client transport - enables nodes behind NAT to be reachable via relay circuits
            .with_relay_client(
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key, relay_client| {
                // GossipSub config - optimized for blockchain propagation
                // mesh_n_low=1 allows mesh to form even with just the bootnode
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .max_transmit_size(2 * 1024 * 1024) // 2MB for blocks
                    .mesh_n_low(1)      // Minimum peers in mesh (allows small networks to work)
                    .mesh_n(3)          // Target peers in mesh
                    .mesh_n_high(6)     // Maximum peers in mesh
                    .gossip_lazy(3)     // Peers to gossip to
                    .build()
                    .expect("Valid gossipsub config");

                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                ).expect("Valid gossipsub behaviour");

                // mDNS for local/LAN discovery
                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    local_peer_id,
                ).expect("Valid mDNS behaviour");

                // Identify protocol - learn peer info
                let identify = identify::Behaviour::new(
                    identify::Config::new(
                        format!("/pyrax/{}/1.0.0", network_id.name()),
                        key.public(),
                    )
                );

                // Ping for keep-alive and RTT measurement
                let ping = ping::Behaviour::new(
                    ping::Config::new()
                        .with_interval(ping_interval)
                        .with_timeout(Duration::from_secs(20))
                );

                // Kademlia DHT for peer discovery - primary discovery mechanism
                let store = kad::store::MemoryStore::new(local_peer_id);
                let mut kademlia_config = kad::Config::default();
                kademlia_config.set_protocol_names(vec![
                    libp2p::StreamProtocol::try_from_owned(format!("/pyrax/{}/kad/1.0.0", network_id.name())).unwrap()
                ]);
                kademlia_config.set_query_timeout(Duration::from_secs(60));
                kademlia_config.set_replication_factor(std::num::NonZeroUsize::new(20).unwrap());
                kademlia_config.set_parallelism(std::num::NonZeroUsize::new(5).unwrap());
                let mut kademlia = kad::Behaviour::with_config(local_peer_id, store, kademlia_config);
                
                // CRITICAL: Set Kademlia to Server mode so nodes can respond to DHT queries
                // Without this, nodes only act as DHT clients and won't serve routing info
                kademlia.set_mode(Some(kad::Mode::Server));

                // Relay SERVER behaviour - allows this node to act as a relay for others
                let relay_server = relay::Behaviour::new(local_peer_id, relay::Config::default());
                
                info!("P2P behaviours initialized with relay client for NAT traversal");

                PyraxBehaviour { gossipsub, mdns, identify, ping, kademlia, relay_server, relay_client }
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(300)))
            .build();

        // Create block/tx channels
        let (block_tx, block_rx) = mpsc::channel(100);
        let (tx_tx, tx_rx) = mpsc::channel(1000);

        // Create connection manager event channel
        let (conn_event_tx, conn_event_rx) = mpsc::channel(100);

        // Create connection manager with proper config
        let conn_manager_config = ConnectionManagerConfig {
            target_peers: config.target_peers,
            min_peers: config.min_peers,
            max_peers: config.max_peers,
            max_concurrent_dials: config.max_concurrent_dials,
            dial_timeout: Duration::from_secs(config.dial_timeout_secs),
            peer_refresh_interval: Duration::from_secs(config.peer_refresh_interval_secs),
            peer_reevaluate_interval: Duration::from_secs(config.peer_reevaluate_interval_secs),
            liveness_check_interval: Duration::from_secs(config.ping_interval_secs),
            min_prune_interval: Duration::from_secs(30),
            min_connection_age: Duration::from_secs(60),
        };

        let peer_store_config = PeerStoreConfig::default();
        let mut conn_manager = ConnectionManager::new(conn_manager_config, peer_store_config, conn_event_tx);

        // Register bootstrap peers in connection manager
        for peer_addr in &config.bootstrap_peers {
            if let Some(peer_id) = Self::extract_peer_id_from_str(peer_addr) {
                conn_manager.add_bootnode(peer_id, vec![peer_addr.clone()]);
                info!("Registered bootnode: {} at {}", peer_id, peer_addr);
            }
        }

        // Set local peer ID in registry
        peer_registry.set_local_peer_id(local_peer_id.to_string()).await;

        let metrics = NetworkMetrics {
            target_peers: config.target_peers,
            state: NetworkState::Starting,
            ..Default::default()
        };

        Ok(Self {
            local_peer_id,
            swarm,
            db,
            network_id,
            conn_manager,
            conn_event_rx,
            peer_registry,
            bootstrap_peers: config.bootstrap_peers.clone(),
            dialing: HashSet::new(),
            pending_disconnect: HashSet::new(),
            config,
            block_tx,
            block_rx: Some(block_rx),
            tx_tx,
            tx_rx: Some(tx_rx),
            metrics,
        })
    }

    /// Extract peer ID from multiaddr string
    fn extract_peer_id_from_str(addr: &str) -> Option<PeerId> {
        addr.parse::<Multiaddr>().ok().and_then(|ma| Self::extract_peer_id(&ma))
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// Start listening
    pub fn listen(&mut self, addr: &str) -> anyhow::Result<()> {
        let multiaddr: Multiaddr = addr.parse()?;
        self.swarm.listen_on(multiaddr)?;
        Ok(())
    }

    /// Connect to a peer
    pub fn dial(&mut self, addr: &str) -> anyhow::Result<()> {
        let multiaddr: Multiaddr = addr.parse()?;
        self.swarm.dial(multiaddr.clone())?;
        
        // Extract peer ID from multiaddr if present and add to Kademlia
        if let Some(peer_id) = Self::extract_peer_id(&multiaddr) {
            self.swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
            info!("Added bootstrap peer {} to Kademlia", peer_id);
        }
        Ok(())
    }

    /// Extract peer ID from a multiaddr containing /p2p/<peer_id>
    fn extract_peer_id(addr: &Multiaddr) -> Option<PeerId> {
        addr.iter().find_map(|p| {
            if let libp2p::multiaddr::Protocol::P2p(peer_id) = p {
                Some(peer_id)
            } else {
                None
            }
        })
    }

    /// Check if a multiaddr is publicly routable (not localhost or private network)
    /// This prevents address pollution where nodes advertise non-routable addresses
    fn is_routable_address(addr: &Multiaddr) -> bool {
        for protocol in addr.iter() {
            match protocol {
                libp2p::multiaddr::Protocol::Ip4(ip) => {
                    // Reject localhost
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
                }
                libp2p::multiaddr::Protocol::Ip6(ip) => {
                    // Reject localhost
                    if ip.is_loopback() {
                        return false;
                    }
                    // Reject unspecified (::)
                    if ip.is_unspecified() {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }

    /// Bootstrap Kademlia DHT for peer discovery
    pub fn bootstrap_kademlia(&mut self) {
        info!("Starting Kademlia DHT bootstrap for peer discovery...");
        if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
            warn!("Kademlia bootstrap failed: {:?}", e);
        }
    }

    /// Subscribe to gossipsub topics
    pub fn subscribe(&mut self) -> anyhow::Result<()> {
        let blocks_topic = gossipsub::IdentTopic::new(format!("pyrax/{}/blocks", self.network_id.name()));
        let txs_topic = gossipsub::IdentTopic::new(format!("pyrax/{}/txs", self.network_id.name()));

        self.swarm.behaviour_mut().gossipsub.subscribe(&blocks_topic)?;
        self.swarm.behaviour_mut().gossipsub.subscribe(&txs_topic)?;

        info!("Subscribed to gossipsub topics: blocks, txs");
        Ok(())
    }

    /// Broadcast a block
    pub fn broadcast_block(&mut self, block: &Block) -> anyhow::Result<()> {
        let topic = gossipsub::IdentTopic::new(format!("pyrax/{}/blocks", self.network_id.name()));
        let msg = GossipMessage::NewBlock(block.clone());
        let data = bincode::serialize(&msg)?;
        
        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
            warn!("Failed to publish block: {:?}", e);
        } else {
            debug!("Broadcast block {} at height {}", block.hash(), block.height());
        }
        Ok(())
    }

    /// Request blocks from peers starting at a given height
    pub fn request_blocks(&mut self, start_height: u64, count: u64) -> anyhow::Result<()> {
        let topic = gossipsub::IdentTopic::new(format!("pyrax/{}/blocks", self.network_id.name()));
        let msg = GossipMessage::GetBlocks { start_height, count };
        let data = bincode::serialize(&msg)?;
        
        info!("Sending GetBlocks request: start={}, count={}, topic={}", start_height, count, topic.hash());
        
        match self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
            Ok(_) => {
                info!("Successfully sent GetBlocks request for blocks {}-{}", start_height, start_height + count - 1);
            }
            Err(e) => {
                warn!("Failed to request blocks: {:?}", e);
            }
        }
        Ok(())
    }

    /// Broadcast a transaction
    pub fn broadcast_tx(&mut self, tx: &Transaction) -> anyhow::Result<()> {
        let topic = gossipsub::IdentTopic::new(format!("pyrax/{}/txs", self.network_id.name()));
        let msg = GossipMessage::NewTransaction(tx.clone());
        let data = bincode::serialize(&msg)?;
        
        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
            warn!("Failed to publish tx: {:?}", e);
        }
        Ok(())
    }

    /// Get connected peer count
    pub fn peer_count(&self) -> usize {
        self.conn_manager.peer_store().connected_count()
    }

    /// Get network metrics
    pub fn get_metrics(&self) -> NetworkMetrics {
        self.metrics.clone()
    }

    /// Get network state
    pub fn get_state(&self) -> NetworkState {
        self.conn_manager.state()
    }

    /// Take block receiver channel
    pub fn take_block_receiver(&mut self) -> Option<mpsc::Receiver<Block>> {
        self.block_rx.take()
    }

    /// Take tx receiver channel
    pub fn take_tx_receiver(&mut self) -> Option<mpsc::Receiver<Transaction>> {
        self.tx_rx.take()
    }

    /// Run the network event loop with block broadcast capability
    /// This is the main mesh networking loop with connection management
    pub async fn run_with_broadcast(mut self, mut mined_rx: tokio::sync::mpsc::Receiver<Block>) {
        info!("╔═══════════════════════════════════════════════════════════════╗");
        info!("║     PYRAX P2P Mesh Network Starting (with broadcast)          ║");
        info!("╚═══════════════════════════════════════════════════════════════╝");
        
        // Start connection manager - this will dial bootnodes
        self.conn_manager.start().await;
        
        let mut initial_sync_done = false;
        let mut last_requested_height: u64 = 0;
        
        // Timers for mesh maintenance
        let mut conn_manager_tick = tokio::time::interval(Duration::from_secs(1));
        conn_manager_tick.tick().await;
        
        let mut sync_timer = tokio::time::interval(Duration::from_secs(self.config.peer_refresh_interval_secs));
        sync_timer.tick().await;
        
        let mut metrics_timer = tokio::time::interval(Duration::from_secs(30));
        metrics_timer.tick().await;
        
        loop {
            tokio::select! {
                // Handle swarm events
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                }
                
                // Handle connection manager events
                Some(conn_event) = self.conn_event_rx.recv() => {
                    self.handle_conn_manager_event(conn_event).await;
                }
                
                // Broadcast mined blocks
                Some(block) = mined_rx.recv() => {
                    self.metrics.messages_sent += 1;
                    if let Err(e) = self.broadcast_block(&block) {
                        warn!("Failed to broadcast block {}: {}", block.hash(), e);
                    } else {
                        info!("Broadcast block {} (height {}) to {} peers", 
                            block.hash(), block.height(), self.peer_count());
                    }
                }
                
                // Connection manager tick - maintains mesh health
                _ = conn_manager_tick.tick() => {
                    self.conn_manager.tick().await;
                    
                    // Process any pending disconnects
                    self.process_pending_disconnects();
                }
                
                // Periodic sync check
                _ = sync_timer.tick() => {
                    let peer_count = self.peer_count();
                    if peer_count > 0 {
                        let our_height = self.db.get_tip().height;
                        if !initial_sync_done || our_height >= last_requested_height {
                            let start_height = our_height + 1;
                            debug!("Sync check: requesting blocks from height {} (peers: {})", start_height, peer_count);
                            let _ = self.request_blocks(start_height, 100);
                            last_requested_height = start_height;
                            initial_sync_done = true;
                        }
                    }
                }
                
                // Metrics logging
                _ = metrics_timer.tick() => {
                    self.log_metrics();
                }
            }
        }
    }

    /// Run the network event loop (without broadcast)
    /// This is the main mesh networking loop with connection management
    pub async fn run(mut self) {
        info!("╔═══════════════════════════════════════════════════════════════╗");
        info!("║     PYRAX P2P Mesh Network Starting                            ║");
        info!("╚═══════════════════════════════════════════════════════════════╝");
        
        // Start connection manager - this will dial bootnodes
        self.conn_manager.start().await;
        
        let mut initial_sync_done = false;
        let mut last_requested_height: u64 = 0;
        
        // Timers for mesh maintenance
        let mut conn_manager_tick = tokio::time::interval(Duration::from_secs(1));
        conn_manager_tick.tick().await;
        
        let mut sync_timer = tokio::time::interval(Duration::from_secs(self.config.peer_refresh_interval_secs));
        sync_timer.tick().await;
        
        let mut metrics_timer = tokio::time::interval(Duration::from_secs(30));
        metrics_timer.tick().await;
        
        loop {
            tokio::select! {
                // Handle swarm events
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                }
                
                // Handle connection manager events
                Some(conn_event) = self.conn_event_rx.recv() => {
                    self.handle_conn_manager_event(conn_event).await;
                }
                
                // Connection manager tick - maintains mesh health
                _ = conn_manager_tick.tick() => {
                    self.conn_manager.tick().await;
                    
                    // Process any pending disconnects
                    self.process_pending_disconnects();
                }
                
                // Periodic sync check
                _ = sync_timer.tick() => {
                    let peer_count = self.peer_count();
                    if peer_count > 0 {
                        let our_height = self.db.get_tip().height;
                        if !initial_sync_done || our_height >= last_requested_height {
                            let start_height = our_height + 1;
                            debug!("Sync check: requesting blocks from height {} (peers: {})", start_height, peer_count);
                            let _ = self.request_blocks(start_height, 100);
                            last_requested_height = start_height;
                            initial_sync_done = true;
                        }
                    }
                }
                
                // Metrics logging
                _ = metrics_timer.tick() => {
                    self.log_metrics();
                }
            }
        }
    }

    /// Handle swarm events and update connection manager
    async fn handle_swarm_event(&mut self, event: SwarmEvent<PyraxBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(behaviour_event) => {
                self.handle_behaviour_event(behaviour_event).await;
            }
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                let addr_str = endpoint.get_remote_address().to_string();
                let (ip, port) = parse_multiaddr(&addr_str);
                let is_inbound = !endpoint.is_dialer();
                let direction = if is_inbound { PeerDirection::Inbound } else { PeerDirection::Outbound };
                
                // Update connection manager
                self.conn_manager.on_connection_established(peer_id, addr_str.clone(), is_inbound).await;
                self.dialing.remove(&peer_id);
                
                // Update legacy registry for RPC compatibility
                self.peer_registry.add_peer(ConnectedPeer {
                    peer_id: peer_id.to_string(),
                    address: addr_str.clone(),
                    ip,
                    port,
                    direction,
                    connected_at: Instant::now(),
                    last_seen: Instant::now(),
                    client_version: format!("pyrax-node/{}", env!("CARGO_PKG_VERSION")),
                    best_height: 0,
                }).await;
                
                // Add to Kademlia for discovery
                if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
                
                // Add to gossipsub mesh
                self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                
                // Update metrics
                self.update_metrics();
                
                let peer_count = self.peer_count();
                info!("✓ Peer {} connected ({}) [{}/{}]", 
                    peer_id, direction, peer_count, self.config.target_peers);
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                // Update connection manager
                self.conn_manager.on_connection_closed(peer_id).await;
                
                // Update legacy registry
                self.peer_registry.remove_peer(&peer_id.to_string()).await;
                
                // Update metrics
                self.update_metrics();
                
                let peer_count = self.peer_count();
                info!("✗ Peer {} disconnected ({:?}) [{}/{}]", 
                    peer_id, cause, peer_count, self.config.target_peers);
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    self.conn_manager.on_dial_failure(peer_id).await;
                    self.dialing.remove(&peer_id);
                    self.metrics.dial_failures += 1;
                    debug!("Dial to {} failed: {:?}", peer_id, error);
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}/p2p/{}", address, self.local_peer_id);
                self.peer_registry.add_listen_address(format!("{}/p2p/{}", address, self.local_peer_id)).await;
            }
            SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                debug!("Incoming connection from {} to {}", send_back_addr, local_addr);
            }
            SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                debug!("Incoming connection error from {} to {}: {:?}", send_back_addr, local_addr, error);
            }
            _ => {}
        }
    }

    /// Handle connection manager events (dial requests, prune requests, etc.)
    async fn handle_conn_manager_event(&mut self, event: ConnectionEvent) {
        match event {
            ConnectionEvent::DialPeers(peers) => {
                for (peer_id, addr) in peers {
                    if self.dialing.contains(&peer_id) {
                        continue;
                    }
                    if let Ok(multiaddr) = addr.parse::<Multiaddr>() {
                        self.dialing.insert(peer_id);
                        self.metrics.dial_attempts += 1;
                        if let Err(e) = self.swarm.dial(multiaddr) {
                            debug!("Failed to dial {}: {:?}", peer_id, e);
                            self.dialing.remove(&peer_id);
                            self.conn_manager.on_dial_failure(peer_id).await;
                        }
                    }
                }
            }
            ConnectionEvent::DisconnectPeers(peers) => {
                for peer_id in peers {
                    self.pending_disconnect.insert(peer_id);
                }
            }
            ConnectionEvent::TriggerKademliaBootstrap => {
                info!("Triggering Kademlia DHT bootstrap...");
                if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
                    warn!("Kademlia bootstrap failed: {:?}", e);
                }
            }
            ConnectionEvent::TriggerKademliaQuery => {
                // Query for random peer IDs to discover more peers
                let random_peer_id = PeerId::random();
                self.swarm.behaviour_mut().kademlia.get_closest_peers(random_peer_id);
            }
            ConnectionEvent::StateChanged(new_state) => {
                info!("Network state: {:?}", new_state);
                self.metrics.state = new_state;
            }
            ConnectionEvent::MetricsUpdate(conn_metrics) => {
                self.metrics.connected_peers = conn_metrics.connected_peers;
                self.metrics.inbound_peers = conn_metrics.inbound_peers;
                self.metrics.outbound_peers = conn_metrics.outbound_peers;
                self.metrics.average_rtt_ms = conn_metrics.average_rtt_ms;
            }
        }
    }

    /// Process pending disconnect requests
    fn process_pending_disconnects(&mut self) {
        for peer_id in self.pending_disconnect.drain() {
            if let Err(e) = self.swarm.disconnect_peer_id(peer_id) {
                debug!("Failed to disconnect {}: {:?}", peer_id, e);
            } else {
                info!("Pruned peer {} (score-based)", peer_id);
            }
        }
    }

    /// Update metrics from connection manager
    fn update_metrics(&mut self) {
        let conn_metrics = self.conn_manager.metrics();
        self.metrics.connected_peers = conn_metrics.connected_peers;
        self.metrics.inbound_peers = conn_metrics.inbound_peers;
        self.metrics.outbound_peers = conn_metrics.outbound_peers;
        self.metrics.dial_attempts = conn_metrics.dial_attempts;
        self.metrics.dial_successes = conn_metrics.dial_successes;
        self.metrics.dial_failures = conn_metrics.dial_failures;
        self.metrics.average_rtt_ms = conn_metrics.average_rtt_ms;
        self.metrics.state = conn_metrics.state;
    }

    /// Log current network metrics
    fn log_metrics(&self) {
        let m = &self.metrics;
        info!("╔══════════════════════════════════════════════════════════════════╗");
        info!("║  P2P MESH STATUS                                                 ║");
        info!("╠══════════════════════════════════════════════════════════════════╣");
        info!("║  Peers: {}/{} (in: {}, out: {})                              ║", 
            m.connected_peers, m.target_peers, m.inbound_peers, m.outbound_peers);
        info!("║  State: {:?}                                              ║", m.state);
        info!("║  Dials: {} attempts, {} success, {} failed                   ║",
            m.dial_attempts, m.dial_successes, m.dial_failures);
        if let Some(rtt) = m.average_rtt_ms {
            info!("║  Avg RTT: {}ms                                              ║", rtt);
        }
        info!("║  Messages: {} sent, {} received                              ║", 
            m.messages_sent, m.messages_received);
        info!("╚══════════════════════════════════════════════════════════════════╝");
    }

    /// Handle behaviour events with connection manager integration
    async fn handle_behaviour_event(&mut self, event: PyraxBehaviourEvent) {
        match event {
            PyraxBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message_id: _,
                message,
            }) => {
                self.metrics.messages_received += 1;
                self.handle_gossip_message(propagation_source, &message.data).await;
                
                // Record successful interaction for scoring
                if let Some(peer) = self.conn_manager.peer_store_mut().get_peer_mut(&propagation_source) {
                    peer.record_success();
                }
            }
            PyraxBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed { peer_id, topic }) => {
                debug!("Peer {} subscribed to {}", peer_id, topic);
            }
            PyraxBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                // mDNS discovery - for LOCAL network peers only
                // Don't add to Kademlia since mDNS addresses are always private/local
                // and would pollute the DHT when propagated to external peers
                for (peer_id, addr) in peers {
                    if peer_id == self.local_peer_id {
                        continue;
                    }
                    info!("mDNS discovered (LAN): {} at {}", peer_id, addr);
                    
                    // Only notify connection manager for local mesh (don't propagate to DHT)
                    self.conn_manager.on_peer_discovered(peer_id, vec![addr.to_string()]);
                    
                    // NOTE: Don't add mDNS addresses to Kademlia - they are private IPs
                    // that would cause WrongPeerId errors when propagated to external peers
                }
            }
            PyraxBehaviourEvent::Mdns(mdns::Event::Expired(peers)) => {
                for (peer_id, _) in peers {
                    debug!("mDNS peer expired: {}", peer_id);
                }
            }
            PyraxBehaviourEvent::Identify(identify::Event::Received { peer_id, info }) => {
                info!("Identified peer {}: {} ({})", 
                    peer_id, info.protocol_version, info.agent_version);
                
                // Filter to only routable addresses (exclude localhost, private networks)
                let routable_addrs: Vec<&Multiaddr> = info.listen_addrs.iter()
                    .filter(|a| Self::is_routable_address(a))
                    .collect();
                
                // Update connection manager with peer info (only routable addresses)
                let listen_addrs: Vec<String> = routable_addrs.iter().map(|a| a.to_string()).collect();
                self.conn_manager.on_peer_identified(peer_id, info.agent_version.clone(), listen_addrs.clone());
                
                // Update legacy registry
                self.peer_registry.update_peer_version(&peer_id.to_string(), &info.agent_version).await;
                
                // Add only routable listen addresses to Kademlia
                // This prevents address pollution from localhost/private IPs
                for addr in routable_addrs {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                }
                
                if info.listen_addrs.len() != listen_addrs.len() {
                    debug!("Filtered {} non-routable addresses from peer {}", 
                        info.listen_addrs.len() - listen_addrs.len(), peer_id);
                }
                
                // Add to gossipsub mesh
                self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            }
            PyraxBehaviourEvent::Ping(ping::Event { peer, result, .. }) => {
                match result {
                    Ok(rtt) => {
                        debug!("Ping to {} successful: {:?}", peer, rtt);
                        
                        // Update RTT in connection manager for scoring
                        self.conn_manager.on_ping_result(peer, Some(rtt));
                        
                        // Update legacy registry
                        self.peer_registry.update_peer_seen(&peer.to_string()).await;
                    }
                    Err(e) => {
                        debug!("Ping to {} failed: {:?}", peer, e);
                        self.conn_manager.on_ping_result(peer, None);
                    }
                }
            }
            PyraxBehaviourEvent::Kademlia(event) => {
                match event {
                    kad::Event::RoutingUpdated { peer, addresses, .. } => {
                        // Filter to only routable addresses before notifying connection manager
                        let routable_addrs: Vec<String> = addresses.iter()
                            .filter(|a| Self::is_routable_address(a))
                            .map(|a| a.to_string())
                            .collect();
                        
                        let filtered_count = addresses.len() - routable_addrs.len();
                        if filtered_count > 0 {
                            debug!("Kademlia: Filtered {} non-routable addresses for peer {}", filtered_count, peer);
                        }
                        
                        info!("Kademlia: Routing updated for peer {} ({} routable addresses)", peer, routable_addrs.len());
                        
                        if !routable_addrs.is_empty() {
                            self.conn_manager.on_peer_discovered(peer, routable_addrs);
                        }
                    }
                    kad::Event::OutboundQueryProgressed { result, .. } => {
                        match result {
                            kad::QueryResult::GetClosestPeers(Ok(ok)) => {
                                info!("Kademlia: Found {} closest peers", ok.peers.len());
                                
                                // Notify connection manager of discovered peers
                                // The peers list contains peer IDs - we'll dial them directly
                                // and let the swarm use addresses from its routing table
                                for peer_id in &ok.peers {
                                    if *peer_id == self.local_peer_id {
                                        continue;
                                    }
                                    
                                    // Queue dial with empty address - swarm will resolve from routing table
                                    // We notify the connection manager which will queue the dial
                                    self.conn_manager.on_peer_discovered(*peer_id, vec![]);
                                    
                                    // Also try direct dial for immediate connectivity
                                    if !self.dialing.contains(peer_id) {
                                        self.dialing.insert(*peer_id);
                                        if let Err(e) = self.swarm.dial(*peer_id) {
                                            debug!("Kademlia: Failed to dial {}: {:?}", peer_id, e);
                                            self.dialing.remove(peer_id);
                                        }
                                    }
                                }
                            }
                            kad::QueryResult::Bootstrap(Ok(ok)) => {
                                info!("Kademlia: Bootstrap step completed ({} remaining)", ok.num_remaining);
                                if ok.num_remaining == 0 {
                                    self.conn_manager.on_kademlia_bootstrap_complete();
                                }
                            }
                            kad::QueryResult::Bootstrap(Err(e)) => {
                                warn!("Kademlia: Bootstrap failed: {:?}", e);
                            }
                            _ => {}
                        }
                    }
                    kad::Event::InboundRequest { request } => {
                        debug!("Kademlia: Inbound request: {:?}", request);
                    }
                    _ => {}
                }
            }
            PyraxBehaviourEvent::RelayServer(event) => {
                // Log relay server events (when we act as relay for others)
                debug!("Relay Server: {:?}", event);
            }
            PyraxBehaviourEvent::RelayClient(event) => {
                // Log relay client events (when we use relay for NAT traversal)
                match &event {
                    relay::client::Event::ReservationReqAccepted { relay_peer_id, renewal, limit } => {
                        info!("Relay reservation accepted by {} (renewal: {}, limit: {:?})", 
                            relay_peer_id, renewal, limit);
                    }
                    relay::client::Event::OutboundCircuitEstablished { relay_peer_id, limit } => {
                        info!("Outbound circuit established via relay {} (limit: {:?})", 
                            relay_peer_id, limit);
                    }
                    relay::client::Event::InboundCircuitEstablished { src_peer_id, limit } => {
                        info!("Inbound circuit established from {} (limit: {:?})", 
                            src_peer_id, limit);
                    }
                }
            }
            _ => {}
        }
    }

    /// Handle incoming gossip message
    async fn handle_gossip_message(&mut self, source: PeerId, data: &[u8]) {
        let msg: GossipMessage = match bincode::deserialize(data) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to deserialize gossip from {}: {}", source, e);
                return;
            }
        };

        match msg {
            GossipMessage::NewBlock(block) => {
                info!("Received block {} (height {}) from {}", 
                    block.hash(), block.height(), source);
                
                // Forward to block processor
                if let Err(e) = self.block_tx.send(block).await {
                    error!("Failed to forward block: {}", e);
                }
            }
            GossipMessage::NewTransaction(tx) => {
                debug!("Received tx {} from {}", tx.txid(), source);
                
                if let Err(e) = self.tx_tx.send(tx).await {
                    error!("Failed to forward tx: {}", e);
                }
            }
            GossipMessage::NewHeader(header) => {
                debug!("Received header {} from {}", header.hash(), source);
            }
            GossipMessage::GetBlocks { start_height, count } => {
                info!("Received GetBlocks request from {}: start={}, count={}", source, start_height, count);
                
                // Fetch blocks from our database and respond
                let mut blocks = Vec::new();
                let max_count = std::cmp::min(count, 100); // Limit to 100 blocks per request
                
                for height in start_height..(start_height + max_count) {
                    match self.db.get_block_by_height(height) {
                        Ok(Some(block)) => blocks.push(block),
                        Ok(None) => break, // No more blocks
                        Err(e) => {
                            warn!("Error fetching block at height {}: {}", height, e);
                            break;
                        }
                    }
                }
                
                if !blocks.is_empty() {
                    info!("Sending {} blocks (heights {}-{}) to peer", 
                        blocks.len(), start_height, start_height + blocks.len() as u64 - 1);
                    
                    // Broadcast the blocks response
                    let topic = gossipsub::IdentTopic::new(format!("pyrax/{}/blocks", self.network_id.name()));
                    let msg = GossipMessage::Blocks(blocks);
                    if let Ok(data) = bincode::serialize(&msg) {
                        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
                            warn!("Failed to send blocks response: {:?}", e);
                        }
                    }
                }
            }
            GossipMessage::Blocks(mut blocks) => {
                info!("Received {} blocks from {} for sync", blocks.len(), source);
                
                // Sort blocks by height and process in order
                blocks.sort_by_key(|b| b.height());
                
                let first_height = blocks.first().map(|b| b.height()).unwrap_or(0);
                let last_height = blocks.last().map(|b| b.height()).unwrap_or(0);
                
                for block in &blocks {
                    // Forward each block to processor
                    if let Err(e) = self.block_tx.send(block.clone()).await {
                        error!("Failed to forward synced block: {}", e);
                    }
                }
                
                // Always request next batch if we got a full batch (100 blocks)
                if blocks.len() >= 100 {
                    // Wait for blocks to be processed
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    let next_height = last_height + 1;
                    info!("Requesting next sync batch from height {}", next_height);
                    let _ = self.request_blocks(next_height, 100);
                } else if !blocks.is_empty() {
                    // Partial batch - check if we need more
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    let our_height = self.db.get_tip().height;
                    if our_height >= last_height {
                        info!("Sync complete! Our height: {}", our_height);
                    }
                }
            }
        }
    }
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub best_height: u64,
}
