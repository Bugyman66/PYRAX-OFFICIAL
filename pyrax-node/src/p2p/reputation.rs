//! Peer Reputation & Trust System
//!
//! Advanced multi-factor reputation scoring for P2P network peers.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Reputation score weights (must sum to 100)
pub const WEIGHT_BLOCK_RELAY: f64 = 25.0;
pub const WEIGHT_TX_VALIDITY: f64 = 20.0;
pub const WEIGHT_UPTIME: f64 = 20.0;
pub const WEIGHT_GEOGRAPHIC: f64 = 15.0;
pub const WEIGHT_PROTOCOL: f64 = 20.0;

pub const MAX_REPUTATION: f64 = 100.0;
pub const MIN_REPUTATION_THRESHOLD: f64 = 10.0;
pub const REPUTATION_DECAY_PER_HOUR: f64 = 2.0;

pub const EXCELLENT_RELAY_MS: u64 = 100;
pub const GOOD_RELAY_MS: u64 = 500;
pub const ACCEPTABLE_RELAY_MS: u64 = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeoRegion {
    NorthAmerica, SouthAmerica, Europe, Asia, Africa, Oceania, Unknown,
}

impl GeoRegion {
    pub fn from_ip(ip: &IpAddr) -> Self {
        match ip {
            IpAddr::V4(v4) => {
                match v4.octets()[0] {
                    1..=50 => GeoRegion::NorthAmerica,
                    51..=100 => GeoRegion::Europe,
                    101..=150 => GeoRegion::Asia,
                    151..=180 => GeoRegion::Oceania,
                    181..=200 => GeoRegion::SouthAmerica,
                    201..=220 => GeoRegion::Africa,
                    _ => GeoRegion::Unknown,
                }
            }
            IpAddr::V6(_) => GeoRegion::Unknown,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            GeoRegion::NorthAmerica => "North America",
            GeoRegion::SouthAmerica => "South America",
            GeoRegion::Europe => "Europe",
            GeoRegion::Asia => "Asia",
            GeoRegion::Africa => "Africa",
            GeoRegion::Oceania => "Oceania",
            GeoRegion::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    InvalidMessage, InvalidBlock, InvalidTransaction, DoubleSigning,
    SpamMessage, UnsolicitedData, VersionMismatch, TimeSkew,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationSeverity { Low, Medium, High, Critical }

impl ViolationSeverity {
    pub fn weight(&self) -> f64 {
        match self {
            Self::Low => 1.0, Self::Medium => 5.0, Self::High => 15.0, Self::Critical => 50.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolViolation {
    pub timestamp: Instant,
    pub violation_type: ViolationType,
    pub description: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Default)]
pub struct ProtocolCompliance {
    pub valid_messages: u64,
    pub invalid_messages: u64,
    pub violations: Vec<ProtocolViolation>,
}

impl ProtocolCompliance {
    pub fn record_valid(&mut self) { self.valid_messages += 1; }
    
    pub fn record_invalid(&mut self, reason: &str) {
        self.invalid_messages += 1;
        self.violations.push(ProtocolViolation {
            timestamp: Instant::now(),
            violation_type: ViolationType::InvalidMessage,
            description: reason.to_string(),
            severity: ViolationSeverity::Low,
        });
    }
    
    pub fn record_violation(&mut self, vtype: ViolationType, desc: &str, sev: ViolationSeverity) {
        self.violations.push(ProtocolViolation {
            timestamp: Instant::now(), violation_type: vtype,
            description: desc.to_string(), severity: sev,
        });
    }
    
    pub fn compliance_ratio(&self) -> f64 {
        let total = self.valid_messages + self.invalid_messages;
        if total == 0 { 1.0 } else { self.valid_messages as f64 / total as f64 }
    }
    
    pub fn violation_score(&self) -> f64 {
        let now = Instant::now();
        let one_hour = Duration::from_secs(3600);
        self.violations.iter()
            .filter(|v| now.duration_since(v.timestamp) < one_hour)
            .map(|v| v.severity.weight()).sum()
    }
    
    pub fn cleanup(&mut self) {
        let one_day = Duration::from_secs(86400);
        let now = Instant::now();
        self.violations.retain(|v| now.duration_since(v.timestamp) < one_day);
    }
}

/// Block relay performance tracking
#[derive(Debug, Clone, Default)]
pub struct BlockRelayStats {
    pub blocks_relayed: u64,
    pub first_seen_count: u64,
    pub avg_relay_latency_ms: Option<u64>,
    latency_samples: Vec<u64>,
}

impl BlockRelayStats {
    pub fn record_block(&mut self, was_first: bool, latency_ms: Option<u64>) {
        self.blocks_relayed += 1;
        if was_first { self.first_seen_count += 1; }
        if let Some(lat) = latency_ms {
            self.latency_samples.push(lat);
            if self.latency_samples.len() > 100 { self.latency_samples.remove(0); }
            let sum: u64 = self.latency_samples.iter().sum();
            self.avg_relay_latency_ms = Some(sum / self.latency_samples.len() as u64);
        }
    }
    
    pub fn relay_score(&self) -> f64 {
        if self.blocks_relayed == 0 { return 0.5; }
        let mut score = 0.0;
        if let Some(latency) = self.avg_relay_latency_ms {
            score += if latency <= EXCELLENT_RELAY_MS { 0.4 }
                else if latency <= GOOD_RELAY_MS { 0.32 }
                else if latency <= ACCEPTABLE_RELAY_MS { 0.2 }
                else { 0.08 };
        } else { score += 0.2; }
        score += (self.first_seen_count as f64 / self.blocks_relayed as f64) * 0.6;
        score
    }
}

/// Transaction validity tracking
#[derive(Debug, Clone, Default)]
pub struct TxValidityStats {
    pub valid_txs: u64,
    pub invalid_txs: u64,
    pub confirmed_txs: u64,
}

impl TxValidityStats {
    pub fn record_valid(&mut self) { self.valid_txs += 1; }
    pub fn record_invalid(&mut self) { self.invalid_txs += 1; }
    pub fn record_confirmed(&mut self) { self.confirmed_txs += 1; }
    
    pub fn validity_score(&self) -> f64 {
        let total = self.valid_txs + self.invalid_txs;
        if total == 0 { return 0.5; }
        let ratio = self.valid_txs as f64 / total as f64;
        let bonus = if self.valid_txs > 0 {
            (self.confirmed_txs as f64 / self.valid_txs as f64).min(0.2)
        } else { 0.0 };
        (ratio * 0.9 + bonus).min(1.0)
    }
}

/// Uptime tracking
#[derive(Debug, Clone)]
pub struct UptimeStats {
    pub first_seen: Instant,
    pub total_connected_secs: u64,
    pub session_count: u32,
    pub stable_sessions: u32,
    pub current_session_start: Option<Instant>,
}

impl Default for UptimeStats {
    fn default() -> Self {
        Self {
            first_seen: Instant::now(),
            total_connected_secs: 0,
            session_count: 0,
            stable_sessions: 0,
            current_session_start: None,
        }
    }
}

impl UptimeStats {
    pub fn session_start(&mut self) {
        self.session_count += 1;
        self.current_session_start = Some(Instant::now());
    }
    
    pub fn session_end(&mut self) {
        if let Some(start) = self.current_session_start.take() {
            let dur = start.elapsed();
            self.total_connected_secs += dur.as_secs();
            if dur > Duration::from_secs(600) { self.stable_sessions += 1; }
        }
    }
    
    pub fn uptime_score(&self) -> f64 {
        let current = self.current_session_start.map(|s| s.elapsed().as_secs()).unwrap_or(0);
        let total = self.total_connected_secs + current;
        let hours = total as f64 / 3600.0;
        let stability = if self.session_count > 0 {
            self.stable_sessions as f64 / self.session_count as f64
        } else { 0.0 };
        ((hours / 24.0).min(0.5) + stability * 0.5).min(1.0)
    }
}

/// Complete peer reputation data
#[derive(Debug, Clone)]
pub struct PeerReputation {
    pub peer_id: PeerId,
    pub region: GeoRegion,
    pub block_relay: BlockRelayStats,
    pub tx_validity: TxValidityStats,
    pub uptime: UptimeStats,
    pub compliance: ProtocolCompliance,
    pub last_updated: Instant,
    cached_score: Option<f64>,
}

impl PeerReputation {
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id, region: GeoRegion::Unknown,
            block_relay: BlockRelayStats::default(),
            tx_validity: TxValidityStats::default(),
            uptime: UptimeStats::default(),
            compliance: ProtocolCompliance::default(),
            last_updated: Instant::now(),
            cached_score: None,
        }
    }
    
    pub fn set_region_from_ip(&mut self, ip: &IpAddr) {
        self.region = GeoRegion::from_ip(ip);
    }
    
    pub fn calculate_score(&mut self, geo_diversity: f64) -> f64 {
        let block = self.block_relay.relay_score() * WEIGHT_BLOCK_RELAY;
        let tx = self.tx_validity.validity_score() * WEIGHT_TX_VALIDITY;
        let up = self.uptime.uptime_score() * WEIGHT_UPTIME;
        let geo = geo_diversity * WEIGHT_GEOGRAPHIC;
        let proto = (self.compliance.compliance_ratio() - 
            self.compliance.violation_score() / 100.0).max(0.0) * WEIGHT_PROTOCOL;
        let total = (block + tx + up + geo + proto).clamp(0.0, MAX_REPUTATION);
        self.cached_score = Some(total);
        self.last_updated = Instant::now();
        total
    }
    
    pub fn score(&self) -> f64 { self.cached_score.unwrap_or(50.0) }
}

/// Reputation manager for all peers
pub struct ReputationManager {
    peers: HashMap<PeerId, PeerReputation>,
    region_counts: HashMap<GeoRegion, usize>,
}

impl ReputationManager {
    pub fn new() -> Self {
        Self { peers: HashMap::new(), region_counts: HashMap::new() }
    }
    
    pub fn get_or_create(&mut self, peer_id: PeerId) -> &mut PeerReputation {
        self.peers.entry(peer_id).or_insert_with(|| PeerReputation::new(peer_id))
    }
    
    pub fn get(&self, peer_id: &PeerId) -> Option<&PeerReputation> { self.peers.get(peer_id) }
    
    pub fn update_region(&mut self, peer_id: &PeerId, ip: &IpAddr) {
        if let Some(rep) = self.peers.get_mut(peer_id) {
            let old = rep.region;
            rep.set_region_from_ip(ip);
            if old != rep.region {
                if old != GeoRegion::Unknown {
                    *self.region_counts.entry(old).or_insert(1) -= 1;
                }
                *self.region_counts.entry(rep.region).or_insert(0) += 1;
            }
        }
    }
    
    pub fn geo_diversity_score(&self, region: GeoRegion) -> f64 {
        let total: usize = self.region_counts.values().sum();
        if total == 0 { return 1.0; }
        let count = *self.region_counts.get(&region).unwrap_or(&0);
        (1.0 - count as f64 / total as f64).max(0.0)
    }
    
    pub fn update_all_scores(&mut self) {
        // First collect all regions
        let peer_regions: Vec<(PeerId, GeoRegion)> = self.peers.iter()
            .map(|(id, rep)| (*id, rep.region))
            .collect();
        
        // Calculate geo diversity scores for each region
        let geo_scores: std::collections::HashMap<GeoRegion, f64> = peer_regions.iter()
            .map(|(_, region)| *region)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(|region| (region, self.geo_diversity_score(region)))
            .collect();
        
        // Now update scores
        for (id, region) in peer_regions {
            if let Some(rep) = self.peers.get_mut(&id) {
                let geo = *geo_scores.get(&region).unwrap_or(&0.5);
                rep.calculate_score(geo);
            }
        }
    }
    
    pub fn peers_by_score(&self) -> Vec<(&PeerId, f64)> {
        let mut p: Vec<_> = self.peers.iter().map(|(id, r)| (id, r.score())).collect();
        p.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        p
    }
    
    pub fn low_reputation_peers(&self, threshold: f64) -> Vec<PeerId> {
        self.peers.iter().filter(|(_, r)| r.score() < threshold).map(|(id, _)| *id).collect()
    }
    
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        if let Some(rep) = self.peers.remove(peer_id) {
            if rep.region != GeoRegion::Unknown {
                if let Some(c) = self.region_counts.get_mut(&rep.region) { *c = c.saturating_sub(1); }
            }
        }
    }
    
    pub fn metrics(&self) -> ReputationMetrics {
        let scores: Vec<f64> = self.peers.values().map(|r| r.score()).collect();
        let avg = if scores.is_empty() { 0.0 } else { scores.iter().sum::<f64>() / scores.len() as f64 };
        ReputationMetrics {
            total_peers: self.peers.len(),
            avg_score: avg,
            low_rep_count: scores.iter().filter(|&&s| s < MIN_REPUTATION_THRESHOLD).count(),
            region_distribution: self.region_counts.clone(),
        }
    }
}

impl Default for ReputationManager { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone)]
pub struct ReputationMetrics {
    pub total_peers: usize,
    pub avg_score: f64,
    pub low_rep_count: usize,
    pub region_distribution: HashMap<GeoRegion, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reputation_scoring() {
        let peer_id = PeerId::random();
        let mut rep = PeerReputation::new(peer_id);
        rep.block_relay.record_block(true, Some(50));
        rep.tx_validity.record_valid();
        rep.compliance.record_valid();
        rep.uptime.session_start();
        let score = rep.calculate_score(0.8);
        assert!(score > 0.0 && score <= MAX_REPUTATION);
    }
    
    #[test]
    fn test_geo_region() {
        let ip: IpAddr = "8.8.8.8".parse().unwrap();
        assert_eq!(GeoRegion::from_ip(&ip), GeoRegion::NorthAmerica);
    }
}
