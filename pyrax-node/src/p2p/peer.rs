use std::net::SocketAddr;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::types::{H256, BlockNumber};

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: PeerId,
    pub addr: SocketAddr,
    pub client_version: String,
    pub protocol_version: u32,
    pub best_height: BlockNumber,
    pub best_hash: H256,
    pub genesis_hash: H256,
    pub connected_at: Instant,
    pub last_seen: Instant,
    pub latency_ms: u64,
    pub state: PeerState,
    pub score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    pub fn random() -> Self {
        let mut id = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut id);
        PeerId(id)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 32 {
            return None;
        }
        let mut id = [0u8; 32];
        id.copy_from_slice(bytes);
        Some(PeerId(id))
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0[..8]))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerState {
    Connecting,
    Handshaking,
    Connected,
    Syncing,
    Idle,
    Disconnecting,
    Banned,
}

impl PeerInfo {
    pub fn new(id: PeerId, addr: SocketAddr) -> Self {
        let now = Instant::now();
        Self {
            id,
            addr,
            client_version: String::new(),
            protocol_version: 0,
            best_height: 0,
            best_hash: H256::zero(),
            genesis_hash: H256::zero(),
            connected_at: now,
            last_seen: now,
            latency_ms: 0,
            state: PeerState::Connecting,
            score: 100,
        }
    }

    pub fn update_seen(&mut self) {
        self.last_seen = Instant::now();
    }

    pub fn update_best(&mut self, height: BlockNumber, hash: H256) {
        self.best_height = height;
        self.best_hash = hash;
        self.update_seen();
    }

    pub fn increase_score(&mut self, delta: i32) {
        self.score = (self.score + delta).min(200);
    }

    pub fn decrease_score(&mut self, delta: i32) {
        self.score = (self.score - delta).max(-100);
    }

    pub fn should_ban(&self) -> bool {
        self.score <= -100
    }

    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }

    pub fn connection_duration(&self) -> Duration {
        self.connected_at.elapsed()
    }
}

#[derive(Debug, Default)]
pub struct PeerStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub blocks_received: u64,
    pub txs_received: u64,
    pub invalid_messages: u64,
}

impl PeerStats {
    pub fn record_sent(&mut self, bytes: usize) {
        self.messages_sent += 1;
        self.bytes_sent += bytes as u64;
    }

    pub fn record_received(&mut self, bytes: usize) {
        self.messages_received += 1;
        self.bytes_received += bytes as u64;
    }
}
