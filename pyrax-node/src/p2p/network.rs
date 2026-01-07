use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{info, warn, debug, error};

use super::{Result, P2PError, Message, PeerInfo, PeerId, PeerState};
use super::protocol::*;
use crate::config::P2PConfig;
use crate::types::{H256, BlockNumber};

#[derive(Clone)]
pub struct Network {
    inner: Arc<NetworkInner>,
}

struct NetworkInner {
    config: P2PConfig,
    local_id: PeerId,
    peers: RwLock<HashMap<PeerId, PeerInfo>>,
    genesis_hash: RwLock<H256>,
    best_height: RwLock<BlockNumber>,
    best_hash: RwLock<H256>,
}

impl Network {
    pub async fn new(config: &P2PConfig) -> Result<Self> {
        let local_id = PeerId::random();
        
        info!("Initializing P2P network with local ID: {}", local_id);

        Ok(Self {
            inner: Arc::new(NetworkInner {
                config: config.clone(),
                local_id,
                peers: RwLock::new(HashMap::new()),
                genesis_hash: RwLock::new(H256::zero()),
                best_height: RwLock::new(0),
                best_hash: RwLock::new(H256::zero()),
            }),
        })
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting P2P network event loop");

        // Main event loop
        loop {
            // Process peer connections
            self.maintain_peers().await?;
            
            // Small delay to prevent busy-looping
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    async fn maintain_peers(&self) -> Result<()> {
        let mut peers = self.inner.peers.write();
        
        // Remove stale peers
        let stale_timeout = std::time::Duration::from_secs(self.inner.config.connection_timeout * 2);
        let stale_ids: Vec<PeerId> = peers.iter()
            .filter(|(_, p)| p.is_stale(stale_timeout))
            .map(|(id, _)| *id)
            .collect();
        
        for id in stale_ids {
            warn!("Removing stale peer: {}", id);
            peers.remove(&id);
        }

        // Ban misbehaving peers
        let ban_ids: Vec<PeerId> = peers.iter()
            .filter(|(_, p)| p.should_ban())
            .map(|(id, _)| *id)
            .collect();
        
        for id in ban_ids {
            warn!("Banning misbehaving peer: {}", id);
            if let Some(peer) = peers.get_mut(&id) {
                peer.state = PeerState::Banned;
            }
        }

        Ok(())
    }

    pub fn add_peer(&self, peer: PeerInfo) {
        let mut peers = self.inner.peers.write();
        if peers.len() < self.inner.config.max_peers {
            info!("Adding peer: {} ({})", peer.id, peer.addr);
            peers.insert(peer.id, peer);
        }
    }

    pub fn remove_peer(&self, id: &PeerId) {
        let mut peers = self.inner.peers.write();
        if peers.remove(id).is_some() {
            info!("Removed peer: {}", id);
        }
    }

    pub fn get_peer(&self, id: &PeerId) -> Option<PeerInfo> {
        self.inner.peers.read().get(id).cloned()
    }

    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.inner.peers.read().values().cloned().collect()
    }

    pub fn peer_count(&self) -> usize {
        self.inner.peers.read().len()
    }

    pub fn connected_peer_count(&self) -> usize {
        self.inner.peers.read()
            .values()
            .filter(|p| p.state == PeerState::Connected || p.state == PeerState::Idle)
            .count()
    }

    pub fn update_chain_state(&self, height: BlockNumber, hash: H256) {
        *self.inner.best_height.write() = height;
        *self.inner.best_hash.write() = hash;
    }

    pub fn set_genesis_hash(&self, hash: H256) {
        *self.inner.genesis_hash.write() = hash;
    }

    pub fn local_id(&self) -> PeerId {
        self.inner.local_id
    }

    pub fn create_hello_message(&self) -> HelloMessage {
        HelloMessage {
            protocol_version: 1,
            network_id: 1,
            genesis_hash: *self.inner.genesis_hash.read(),
            best_height: *self.inner.best_height.read(),
            best_hash: *self.inner.best_hash.read(),
            node_id: self.inner.local_id.0,
            listen_port: 30303,
            client_version: format!("pyrax-node/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    pub fn create_status_message(&self) -> StatusMessage {
        StatusMessage {
            best_height: *self.inner.best_height.read(),
            best_hash: *self.inner.best_hash.read(),
            total_difficulty: ethereum_types::U256::zero(),
            genesis_hash: *self.inner.genesis_hash.read(),
        }
    }

    pub async fn broadcast_block(&self, block: &crate::types::Block) {
        let msg = Message::NewBlock(NewBlockMessage {
            block: block.clone(),
            total_difficulty: ethereum_types::U256::zero(),
        });

        let peers = self.get_peers();
        for peer in peers {
            if peer.state == PeerState::Connected || peer.state == PeerState::Idle {
                debug!("Broadcasting block {} to peer {}", block.height(), peer.id);
                // TODO: Actually send message over network
            }
        }
    }

    pub async fn broadcast_transaction(&self, tx: &crate::types::SignedTransaction) {
        let hash = tx.hash();
        let msg = Message::InvTx(InvTxMessage {
            hashes: vec![hash],
        });

        let peers = self.get_peers();
        for peer in peers {
            if peer.state == PeerState::Connected || peer.state == PeerState::Idle {
                debug!("Broadcasting tx {} to peer {}", hex::encode(hash.as_bytes()), peer.id);
                // TODO: Actually send message over network
            }
        }
    }
}
