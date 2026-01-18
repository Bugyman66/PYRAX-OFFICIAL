use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug, error};

/// Relay server status for health tracking
#[derive(Debug, Clone)]
pub struct RelayStatus {
    pub address: String,
    pub peer_id: String,
    pub last_success: Option<Instant>,
    pub last_failure: Option<Instant>,
    pub consecutive_failures: u32,
    pub is_primary: bool,
    pub latency_ms: Option<u64>,
}

impl RelayStatus {
    pub fn new(address: String, peer_id: String, is_primary: bool) -> Self {
        Self {
            address,
            peer_id,
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
            is_primary,
            latency_ms: None,
        }
    }
    
    /// Check if this relay is healthy (usable)
    pub fn is_healthy(&self) -> bool {
        // Unhealthy if more than 3 consecutive failures
        if self.consecutive_failures > 3 {
            return false;
        }
        
        // Unhealthy if failed recently and no success since
        if let Some(fail_time) = self.last_failure {
            if fail_time.elapsed() < Duration::from_secs(30) {
                if self.last_success.map(|s| s < fail_time).unwrap_or(true) {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Record a successful relay operation
    pub fn record_success(&mut self, latency_ms: Option<u64>) {
        self.last_success = Some(Instant::now());
        self.consecutive_failures = 0;
        self.latency_ms = latency_ms;
    }
    
    /// Record a failed relay operation
    pub fn record_failure(&mut self) {
        self.last_failure = Some(Instant::now());
        self.consecutive_failures += 1;
    }
}

/// TURN-like relay fallback manager
/// Ensures we always have a working relay for NAT traversal
pub struct RelayFallbackManager {
    /// All known relay servers
    relays: HashMap<String, RelayStatus>,
    /// Current active relay peer ID
    active_relay: Option<String>,
    /// Maximum failures before switching
    max_failures: u32,
    /// Time to wait before retrying a failed relay
    retry_cooldown: Duration,
}

impl RelayFallbackManager {
    pub fn new() -> Self {
        Self {
            relays: HashMap::new(),
            active_relay: None,
            max_failures: 3,
            retry_cooldown: Duration::from_secs(60),
        }
    }
    
    /// Add a relay server to the pool
    pub fn add_relay(&mut self, address: String, peer_id: String, is_primary: bool) {
        info!("Adding relay to fallback pool: {} (primary: {})", peer_id, is_primary);
        self.relays.insert(
            peer_id.clone(),
            RelayStatus::new(address, peer_id.clone(), is_primary)
        );
        
        // Set as active if it's primary and we don't have an active relay
        if is_primary && self.active_relay.is_none() {
            self.active_relay = Some(peer_id);
        }
    }
    
    /// Get the best available relay address for connection
    pub fn get_best_relay(&self) -> Option<&RelayStatus> {
        // First try the active relay if healthy
        if let Some(ref active_id) = self.active_relay {
            if let Some(relay) = self.relays.get(active_id) {
                if relay.is_healthy() {
                    return Some(relay);
                }
            }
        }
        
        // Find the best healthy relay
        // Priority: primary first, then by latency, then by success time
        let mut candidates: Vec<&RelayStatus> = self.relays.values()
            .filter(|r| r.is_healthy())
            .collect();
        
        if candidates.is_empty() {
            // No healthy relays - return the one with oldest failure (might have recovered)
            candidates = self.relays.values().collect();
            candidates.sort_by(|a, b| {
                let a_fail = a.last_failure.map(|t| t.elapsed()).unwrap_or(Duration::MAX);
                let b_fail = b.last_failure.map(|t| t.elapsed()).unwrap_or(Duration::MAX);
                b_fail.cmp(&a_fail) // Prefer older failures
            });
        } else {
            // Sort by: primary first, then latency
            candidates.sort_by(|a, b| {
                if a.is_primary != b.is_primary {
                    return b.is_primary.cmp(&a.is_primary);
                }
                let a_lat = a.latency_ms.unwrap_or(u64::MAX);
                let b_lat = b.latency_ms.unwrap_or(u64::MAX);
                a_lat.cmp(&b_lat)
            });
        }
        
        candidates.first().copied()
    }
    
    /// Record successful relay operation
    pub fn record_success(&mut self, peer_id: &str, latency_ms: Option<u64>) {
        if let Some(relay) = self.relays.get_mut(peer_id) {
            relay.record_success(latency_ms);
            debug!("Relay {} success (latency: {:?}ms)", peer_id, latency_ms);
        }
        
        // Update active relay if this one is better
        if self.active_relay.as_ref() != Some(&peer_id.to_string()) {
            if let Some(current) = self.active_relay.as_ref().and_then(|id| self.relays.get(id)) {
                if !current.is_healthy() {
                    info!("Switching active relay to {}", peer_id);
                    self.active_relay = Some(peer_id.to_string());
                }
            }
        }
    }
    
    /// Record failed relay operation
    pub fn record_failure(&mut self, peer_id: &str) {
        if let Some(relay) = self.relays.get_mut(peer_id) {
            relay.record_failure();
            warn!("Relay {} failed (consecutive: {})", peer_id, relay.consecutive_failures);
            
            // Switch to fallback if this was active relay and now unhealthy
            if self.active_relay.as_ref() == Some(&peer_id.to_string()) && !relay.is_healthy() {
                self.switch_to_fallback();
            }
        }
    }
    
    /// Switch to a fallback relay
    fn switch_to_fallback(&mut self) {
        if let Some(best) = self.get_best_relay() {
            let new_active = best.peer_id.clone();
            if self.active_relay.as_ref() != Some(&new_active) {
                info!("Switching to fallback relay: {}", new_active);
                self.active_relay = Some(new_active);
            }
        } else {
            warn!("No healthy relays available for fallback!");
            self.active_relay = None;
        }
    }
    
    /// Get current active relay
    pub fn get_active_relay(&self) -> Option<&RelayStatus> {
        self.active_relay.as_ref().and_then(|id| self.relays.get(id))
    }
    
    /// Get all relay addresses for circuit listening
    pub fn get_all_relay_addresses(&self) -> Vec<String> {
        self.relays.values()
            .filter(|r| r.is_healthy())
            .map(|r| r.address.clone())
            .collect()
    }
    
    /// Get relay circuit address for a specific relay
    pub fn get_circuit_address(&self, peer_id: &str) -> Option<String> {
        self.relays.get(peer_id).map(|r| {
            format!("{}/p2p-circuit", r.address)
        })
    }
    
    /// Check overall relay health - returns true if at least one relay is healthy
    pub fn has_healthy_relay(&self) -> bool {
        self.relays.values().any(|r| r.is_healthy())
    }
    
    /// Get health summary for logging
    pub fn health_summary(&self) -> String {
        let total = self.relays.len();
        let healthy = self.relays.values().filter(|r| r.is_healthy()).count();
        let active = self.active_relay.as_ref()
            .and_then(|id| self.relays.get(id))
            .map(|r| format!("{} ({}ms)", r.peer_id.chars().take(16).collect::<String>(), r.latency_ms.unwrap_or(0)))
            .unwrap_or_else(|| "none".to_string());
        
        format!("Relays: {}/{} healthy, active: {}", healthy, total, active)
    }
    
    /// Reset all relay states (useful after network changes)
    pub fn reset_all(&mut self) {
        for relay in self.relays.values_mut() {
            relay.consecutive_failures = 0;
            relay.last_failure = None;
        }
        info!("Reset all relay states");
    }
}

impl Default for RelayFallbackManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for TURN-like relay fallback
#[derive(Debug, Clone)]
pub struct RelayFallbackConfig {
    /// Primary relay servers (bootnodes)
    pub primary_relays: Vec<(String, String)>, // (address, peer_id)
    /// Backup relay servers
    pub backup_relays: Vec<(String, String)>,
    /// Enable automatic fallback
    pub auto_fallback: bool,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
}

impl Default for RelayFallbackConfig {
    fn default() -> Self {
        Self {
            primary_relays: Vec::new(),
            backup_relays: Vec::new(),
            auto_fallback: true,
            health_check_interval_secs: 30,
        }
    }
}
