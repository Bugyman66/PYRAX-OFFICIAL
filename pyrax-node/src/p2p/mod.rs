//! P2P Networking for PYRAX using libp2p
//!
//! Production-ready P2P layer with:
//! - GossipSub for block/transaction propagation
//! - mDNS for local peer discovery
//! - Request/Response for block sync

mod registry;
pub use registry::{PeerRegistry, ConnectedPeer, PeerDirection, parse_multiaddr};

use libp2p::{
    gossipsub, identify, kad, mdns, noise, ping,
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

/// P2P network configuration
#[derive(Debug, Clone)]
pub struct P2PConfig {
    pub listen_addr: String,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: usize,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/30303".to_string(),
            bootstrap_peers: vec![],
            max_peers: 50,
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
}

/// P2P Network manager
pub struct Network {
    local_peer_id: PeerId,
    swarm: Swarm<PyraxBehaviour>,
    db: Arc<ChainDB>,
    network_id: NetworkId,
    connected_peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    peer_registry: PeerRegistry,
    bootstrap_peers: Vec<String>,
    // Channels for received blocks/txs
    block_tx: mpsc::Sender<Block>,
    block_rx: Option<mpsc::Receiver<Block>>,
    tx_tx: mpsc::Sender<Transaction>,
    tx_rx: Option<mpsc::Receiver<Transaction>>,
}

impl Network {
    /// Create a new P2P network with a shared peer registry
    pub async fn new(config: P2PConfig, db: Arc<ChainDB>, network_id: NetworkId, peer_registry: PeerRegistry) -> anyhow::Result<Self> {
        info!("Initializing P2P network for {}", network_id.name());

        // Generate keypair
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer ID: {}", local_peer_id);

        // Build swarm with tokio runtime
        let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                // GossipSub config
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .max_transmit_size(2 * 1024 * 1024) // 2MB for blocks
                    .build()
                    .expect("Valid gossipsub config");

                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                ).expect("Valid gossipsub behaviour");

                // mDNS for local discovery
                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    local_peer_id,
                ).expect("Valid mDNS behaviour");

                // Identify protocol
                let identify = identify::Behaviour::new(
                    identify::Config::new(
                        format!("/pyrax/{}/1.0.0", network_id.name()),
                        key.public(),
                    )
                );

                // Ping for keep-alive (every 15 seconds, timeout after 20 seconds)
                let ping = ping::Behaviour::new(
                    ping::Config::new()
                        .with_interval(Duration::from_secs(15))
                        .with_timeout(Duration::from_secs(20))
                );

                // Kademlia DHT for peer discovery
                let store = kad::store::MemoryStore::new(local_peer_id);
                let mut kademlia_config = kad::Config::default();
                kademlia_config.set_protocol_names(vec![
                    libp2p::StreamProtocol::try_from_owned(format!("/pyrax/{}/kad/1.0.0", network_id.name())).unwrap()
                ]);
                // Bootstrap more aggressively
                kademlia_config.set_query_timeout(Duration::from_secs(60));
                let kademlia = kad::Behaviour::with_config(local_peer_id, store, kademlia_config);

                PyraxBehaviour { gossipsub, mdns, identify, ping, kademlia }
            })?
            // Increase idle timeout to 5 minutes to prevent premature disconnections
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(300)))
            .build();

        // Create channels
        let (block_tx, block_rx) = mpsc::channel(100);
        let (tx_tx, tx_rx) = mpsc::channel(1000);

        // Set local peer ID in registry
        peer_registry.set_local_peer_id(local_peer_id.to_string()).await;

        Ok(Self {
            local_peer_id,
            swarm,
            db,
            network_id,
            connected_peers: Arc::new(RwLock::new(HashMap::new())),
            peer_registry,
            bootstrap_peers: config.bootstrap_peers,
            block_tx,
            block_rx: Some(block_rx),
            tx_tx,
            tx_rx: Some(tx_rx),
        })
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
    pub async fn peer_count(&self) -> usize {
        self.connected_peers.read().await.len()
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
    pub async fn run_with_broadcast(mut self, mut mined_rx: tokio::sync::mpsc::Receiver<Block>) {
        info!("P2P network running (with block broadcast)...");
        
        let mut initial_sync_done = false;
        let mut sync_delay_started = false;
        let mut initial_sync_timer = tokio::time::interval(tokio::time::Duration::from_secs(3));
        initial_sync_timer.tick().await;
        
        let mut periodic_sync_timer = tokio::time::interval(tokio::time::Duration::from_secs(30));
        periodic_sync_timer.tick().await;
        
        // Bootstrap reconnection timer - checks every 60 seconds if we need to reconnect
        let mut bootstrap_timer = tokio::time::interval(tokio::time::Duration::from_secs(60));
        bootstrap_timer.tick().await;
        
        let mut last_requested_height: u64 = 0;
        
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(event) => {
                            self.handle_behaviour_event(event).await;
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                            let addr_str = endpoint.get_remote_address().to_string();
                            let (ip, port) = parse_multiaddr(&addr_str);
                            let direction = if endpoint.is_dialer() { PeerDirection::Outbound } else { PeerDirection::Inbound };
                            
                            info!("Connected to peer: {} at {} ({})", peer_id, addr_str, direction);
                            
                            self.connected_peers.write().await.insert(peer_id, PeerInfo {
                                peer_id: peer_id.to_string(),
                                address: addr_str.clone(),
                                best_height: 0,
                            });
                            
                            self.peer_registry.add_peer(ConnectedPeer {
                                peer_id: peer_id.to_string(),
                                address: addr_str,
                                ip,
                                port,
                                direction,
                                connected_at: Instant::now(),
                                last_seen: Instant::now(),
                                client_version: format!("pyrax-node/{}", env!("CARGO_PKG_VERSION")),
                                best_height: 0,
                            }).await;
                            
                            if !sync_delay_started {
                                sync_delay_started = true;
                                info!("First peer connected, will request sync in 3 seconds");
                            }
                        }
                        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                            info!("Disconnected from peer: {} ({:?})", peer_id, cause);
                            self.connected_peers.write().await.remove(&peer_id);
                            self.peer_registry.remove_peer(&peer_id.to_string()).await;
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {}/p2p/{}", address, self.local_peer_id);
                            self.peer_registry.add_listen_address(format!("{}/p2p/{}", address, self.local_peer_id)).await;
                        }
                        SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                            debug!("Incoming connection from {} to {}", send_back_addr, local_addr);
                        }
                        _ => {}
                    }
                }
                Some(block) = mined_rx.recv() => {
                    // Broadcast mined block to peers
                    if let Err(e) = self.broadcast_block(&block) {
                        warn!("Failed to broadcast block {}: {}", block.hash(), e);
                    } else {
                        info!("Broadcast block {} (height {}) to peers", block.hash(), block.height());
                    }
                }
                // Initial sync - request blocks from our tip + 1
                _ = initial_sync_timer.tick(), if sync_delay_started && !initial_sync_done => {
                    let our_height = self.db.get_tip().height;
                    let peer_count = self.connected_peers.read().await.len();
                    
                    info!("Initial sync check (miner): our height={}, peers={}", our_height, peer_count);
                    
                    if peer_count > 0 {
                        let start_height = our_height + 1;
                        info!("Requesting blocks from height {} (we have {} blocks)", start_height, our_height);
                        let _ = self.request_blocks(start_height, 100);
                        last_requested_height = start_height;
                        initial_sync_done = true;
                    }
                }
                // Periodic sync - check every 30 seconds if peers have new blocks
                _ = periodic_sync_timer.tick(), if initial_sync_done => {
                    let our_height = self.db.get_tip().height;
                    let peer_count = self.connected_peers.read().await.len();
                    
                    if peer_count > 0 && our_height >= last_requested_height {
                        let start_height = our_height + 1;
                        debug!("Periodic sync check (miner): requesting blocks from height {}", start_height);
                        let _ = self.request_blocks(start_height, 100);
                        last_requested_height = start_height;
                    }
                }
                // Bootstrap reconnection - ensure we stay connected to bootstrap peers
                _ = bootstrap_timer.tick() => {
                    let peer_count = self.connected_peers.read().await.len();
                    if peer_count == 0 && !self.bootstrap_peers.is_empty() {
                        info!("No peers connected, attempting to reconnect to bootstrap peers...");
                        for peer_addr in &self.bootstrap_peers {
                            info!("Redialing bootstrap peer: {}", peer_addr);
                            if let Ok(addr) = peer_addr.parse::<Multiaddr>() {
                                if let Err(e) = self.swarm.dial(addr) {
                                    warn!("Failed to redial {}: {:?}", peer_addr, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Run the network event loop
    pub async fn run(mut self) {
        info!("P2P network running...");
        
        let mut initial_sync_done = false;
        let mut sync_delay_started = false;
        // Initial sync timer - fires 3 seconds after first peer connects
        let mut initial_sync_timer = tokio::time::interval(tokio::time::Duration::from_secs(3));
        initial_sync_timer.tick().await; // Skip first tick
        
        // Periodic sync timer - checks every 30 seconds for new blocks from peers
        let mut periodic_sync_timer = tokio::time::interval(tokio::time::Duration::from_secs(30));
        periodic_sync_timer.tick().await; // Skip first tick
        
        // Bootstrap reconnection timer - checks every 60 seconds if we need to reconnect
        let mut bootstrap_timer = tokio::time::interval(tokio::time::Duration::from_secs(60));
        bootstrap_timer.tick().await; // Skip first tick
        
        // Track last synced height to avoid duplicate requests
        let mut last_requested_height: u64 = 0;
        
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(event) => {
                            self.handle_behaviour_event(event).await;
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                            let addr_str = endpoint.get_remote_address().to_string();
                            let (ip, port) = parse_multiaddr(&addr_str);
                            let direction = if endpoint.is_dialer() { PeerDirection::Outbound } else { PeerDirection::Inbound };
                            
                            info!("Connected to peer: {} at {} ({})", peer_id, addr_str, direction);
                            
                            self.connected_peers.write().await.insert(peer_id, PeerInfo {
                                peer_id: peer_id.to_string(),
                                address: addr_str.clone(),
                                best_height: 0,
                            });
                            
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
                            
                            // Add peer to Kademlia DHT for discovery
                            if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                                self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                info!("Added peer {} to Kademlia DHT", peer_id);
                            }
                            
                            // Start sync timer and Kademlia bootstrap when first peer connects
                            if !sync_delay_started {
                                sync_delay_started = true;
                                info!("First peer connected, will request sync in 3 seconds");
                                // Start Kademlia bootstrap to discover more peers
                                info!("Starting Kademlia DHT bootstrap for peer discovery...");
                                if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
                                    warn!("Kademlia bootstrap failed: {:?}", e);
                                }
                            }
                        }
                        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                            info!("Disconnected from peer: {} ({:?})", peer_id, cause);
                            self.connected_peers.write().await.remove(&peer_id);
                            self.peer_registry.remove_peer(&peer_id.to_string()).await;
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {}/p2p/{}", address, self.local_peer_id);
                            self.peer_registry.add_listen_address(format!("{}/p2p/{}", address, self.local_peer_id)).await;
                        }
                        SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                            debug!("Incoming connection from {} to {}", send_back_addr, local_addr);
                        }
                        _ => {}
                    }
                }
                // Initial sync - request blocks from our tip + 1
                _ = initial_sync_timer.tick(), if sync_delay_started && !initial_sync_done => {
                    let our_height = self.db.get_tip().height;
                    let peer_count = self.connected_peers.read().await.len();
                    
                    info!("Initial sync check: our height={}, peers={}", our_height, peer_count);
                    
                    if peer_count > 0 {
                        // Always request blocks starting from our current height + 1
                        // This works for both fresh nodes (height 0) and restarted nodes
                        let start_height = our_height + 1;
                        info!("Requesting blocks from height {} (we have {} blocks)", start_height, our_height);
                        let _ = self.request_blocks(start_height, 100);
                        last_requested_height = start_height;
                        initial_sync_done = true;
                    }
                }
                // Periodic sync - check every 30 seconds if peers have new blocks
                _ = periodic_sync_timer.tick(), if initial_sync_done => {
                    let our_height = self.db.get_tip().height;
                    let peer_count = self.connected_peers.read().await.len();
                    
                    if peer_count > 0 && our_height >= last_requested_height {
                        // We've caught up to what we requested, check for more
                        let start_height = our_height + 1;
                        debug!("Periodic sync check: requesting blocks from height {}", start_height);
                        let _ = self.request_blocks(start_height, 100);
                        last_requested_height = start_height;
                    }
                }
                // Bootstrap reconnection - ensure we stay connected to bootstrap peers
                _ = bootstrap_timer.tick() => {
                    let peer_count = self.connected_peers.read().await.len();
                    if peer_count == 0 && !self.bootstrap_peers.is_empty() {
                        info!("No peers connected, attempting to reconnect to bootstrap peers...");
                        for peer_addr in &self.bootstrap_peers {
                            info!("Redialing bootstrap peer: {}", peer_addr);
                            if let Ok(addr) = peer_addr.parse::<Multiaddr>() {
                                if let Err(e) = self.swarm.dial(addr) {
                                    warn!("Failed to redial {}: {:?}", peer_addr, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Handle behaviour events
    async fn handle_behaviour_event(&mut self, event: PyraxBehaviourEvent) {
        match event {
            PyraxBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message_id: _,
                message,
            }) => {
                self.handle_gossip_message(propagation_source, &message.data).await;
            }
            PyraxBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed { peer_id, topic }) => {
                debug!("Peer {} subscribed to {}", peer_id, topic);
            }
            PyraxBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                for (peer_id, addr) in peers {
                    info!("mDNS discovered: {} at {}", peer_id, addr);
                    if let Err(e) = self.swarm.dial(addr.clone()) {
                        debug!("Failed to dial {}: {:?}", addr, e);
                    }
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
                
                // Update peer version in registry
                self.peer_registry.update_peer_version(&peer_id.to_string(), &info.agent_version).await;
                
                // Add peer's listen addresses
                for addr in info.listen_addrs {
                    self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                }
            }
            PyraxBehaviourEvent::Ping(ping::Event { peer, result, .. }) => {
                match result {
                    Ok(rtt) => {
                        debug!("Ping to {} successful: {:?}", peer, rtt);
                        // Update last seen time on successful ping
                        self.peer_registry.update_peer_seen(&peer.to_string()).await;
                    }
                    Err(e) => {
                        // Don't warn on unsupported - older clients may not have ping
                        debug!("Ping to {} failed: {:?}", peer, e);
                    }
                }
            }
            PyraxBehaviourEvent::Kademlia(event) => {
                match event {
                    kad::Event::RoutingUpdated { peer, addresses, .. } => {
                        info!("Kademlia: Routing updated for peer {} with {} addresses", peer, addresses.len());
                    }
                    kad::Event::OutboundQueryProgressed { result, .. } => {
                        match result {
                            kad::QueryResult::GetClosestPeers(Ok(ok)) => {
                                info!("Kademlia: Found {} closest peers", ok.peers.len());
                                for peer in ok.peers {
                                    debug!("  - Discovered peer: {}", peer);
                                }
                            }
                            kad::QueryResult::Bootstrap(Ok(ok)) => {
                                info!("Kademlia: Bootstrap step completed, {} remaining", ok.num_remaining);
                            }
                            kad::QueryResult::Bootstrap(Err(e)) => {
                                warn!("Kademlia: Bootstrap failed: {:?}", e);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
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
