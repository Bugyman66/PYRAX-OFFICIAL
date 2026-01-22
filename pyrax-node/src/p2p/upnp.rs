use std::net::{IpAddr, Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use tracing::{info, warn, debug};

/// UPnP port mapping result
#[derive(Debug, Clone)]
pub struct UPnPMapping {
    pub external_ip: Ipv4Addr,
    pub external_port: u16,
    pub internal_port: u16,
    pub protocol: &'static str,
}

/// UPnP manager for automatic NAT traversal via port mapping
pub struct UPnPManager {
    p2p_port: u16,
    lease_duration: u32,
    mappings: Vec<UPnPMapping>,
    /// Track consecutive renewal failures to trigger relay fallback
    consecutive_failures: u32,
}

impl UPnPManager {
    pub fn new(p2p_port: u16) -> Self {
        Self {
            p2p_port,
            lease_duration: 7200, // 2 hours - will be renewed periodically
            mappings: Vec::new(),
            consecutive_failures: 0,
        }
    }
    
    /// Create from listen address string (extracts port)
    pub fn from_listen_addr(listen_addr: &str) -> Self {
        // Extract port from multiaddr like "/ip4/0.0.0.0/tcp/30303"
        let port = listen_addr
            .split('/')
            .filter_map(|s| s.parse::<u16>().ok())
            .last()
            .unwrap_or(30303);
        Self::new(port)
    }
    
    /// Attempt to set up UPnP port mapping for P2P connectivity
    /// Returns the external IP and port if successful
    pub async fn setup_port_mapping(&mut self) -> Option<UPnPMapping> {
        info!("Attempting UPnP port mapping for P2P port {}...", self.p2p_port);
        
        // Run UPnP discovery in a blocking thread (igd-next is sync)
        let p2p_port = self.p2p_port;
        let lease_duration = self.lease_duration;
        
        let result = tokio::task::spawn_blocking(move || {
            Self::discover_and_map(p2p_port, lease_duration)
        }).await;
        
        match result {
            Ok(Some(mapping)) => {
                info!("✓ UPnP port mapping successful: {}:{} -> local:{}", 
                    mapping.external_ip, mapping.external_port, mapping.internal_port);
                self.mappings.push(mapping.clone());
                Some(mapping)
            }
            Ok(None) => {
                debug!("UPnP port mapping not available (router may not support it)");
                None
            }
            Err(e) => {
                warn!("UPnP task failed: {}", e);
                None
            }
        }
    }
    
    /// Internal synchronous discovery and mapping
    fn discover_and_map(p2p_port: u16, lease_duration: u32) -> Option<UPnPMapping> {
        use igd_next::PortMappingProtocol;
        
        // Try to discover UPnP gateway with timeout
        let gateway_result = std::thread::spawn(move || {
            // Use synchronous search with timeout
            let search_options = igd_next::SearchOptions {
                timeout: Some(Duration::from_secs(5)),
                ..Default::default()
            };
            igd_next::search_gateway(search_options)
        }).join();
        
        let gateway = match gateway_result {
            Ok(Ok(gw)) => {
                info!("Found UPnP gateway: {}", gw.addr);
                gw
            }
            Ok(Err(e)) => {
                debug!("No UPnP gateway found: {}", e);
                return None;
            }
            Err(_) => {
                debug!("UPnP gateway search thread panicked");
                return None;
            }
        };
        
        // Get external IP (returns IpAddr, we need Ipv4Addr)
        let external_ip = match gateway.get_external_ip() {
            Ok(ip) => {
                info!("UPnP external IP: {}", ip);
                match ip {
                    IpAddr::V4(v4) => v4,
                    IpAddr::V6(_) => {
                        warn!("UPnP returned IPv6 address, not supported");
                        return None;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get external IP via UPnP: {}", e);
                return None;
            }
        };
        
        // Get local IP for this gateway
        let local_ip = match get_local_ip_for_gateway(&gateway) {
            Some(ip) => ip,
            None => {
                warn!("Could not determine local IP for UPnP mapping");
                return None;
            }
        };
        
        let local_addr = SocketAddrV4::new(local_ip, p2p_port);
        
        // Try to add TCP port mapping
        let description = format!("PYRAX P2P Node (port {})", p2p_port);
        
        match gateway.add_port(
            PortMappingProtocol::TCP,
            p2p_port,
            local_addr.into(),
            lease_duration,
            &description,
        ) {
            Ok(()) => {
                info!("✓ UPnP TCP port {} mapped successfully", p2p_port);
            }
            Err(e) => {
                // Port might already be mapped - try to remove and re-add
                warn!("UPnP TCP mapping failed: {} - trying to remap...", e);
                let _ = gateway.remove_port(PortMappingProtocol::TCP, p2p_port);
                
                if let Err(e2) = gateway.add_port(
                    PortMappingProtocol::TCP,
                    p2p_port,
                    local_addr.into(),
                    lease_duration,
                    &description,
                ) {
                    warn!("UPnP TCP port mapping failed after retry: {}", e2);
                    return None;
                }
            }
        }
        
        // Also map UDP for hole-punching (best effort)
        match gateway.add_port(
            PortMappingProtocol::UDP,
            p2p_port,
            local_addr.into(),
            lease_duration,
            &format!("{} UDP", description),
        ) {
            Ok(()) => {
                debug!("UPnP UDP port {} mapped successfully", p2p_port);
            }
            Err(e) => {
                // UDP mapping is optional - log but don't fail
                debug!("UPnP UDP mapping failed (non-critical): {}", e);
            }
        }
        
        Some(UPnPMapping {
            external_ip,
            external_port: p2p_port,
            internal_port: p2p_port,
            protocol: "TCP",
        })
    }
    
    /// Renew all port mappings (call periodically, e.g., every hour)
    pub async fn renew_mappings(&mut self) {
        if self.mappings.is_empty() {
            return;
        }
        
        debug!("Renewing UPnP port mappings...");
        
        let p2p_port = self.p2p_port;
        let lease_duration = self.lease_duration;
        
        let result = tokio::task::spawn_blocking(move || {
            Self::discover_and_map(p2p_port, lease_duration)
        }).await;
        
        match result {
            Ok(Some(_)) => {
                debug!("UPnP mappings renewed successfully");
                self.consecutive_failures = 0;
            }
            Ok(None) | Err(_) => {
                self.consecutive_failures += 1;
                warn!("Failed to renew UPnP mappings (attempt {}) - NAT traversal may degrade", 
                    self.consecutive_failures);
                
                // FIX: After 3 consecutive failures, UPnP is likely broken
                // Caller should trigger relay fallback
                if self.consecutive_failures >= 3 {
                    warn!("UPnP appears permanently broken - recommend switching to relay mode");
                }
            }
        }
    }
    
    /// Check if UPnP has failed repeatedly and relay fallback should be used
    pub fn should_use_relay_fallback(&self) -> bool {
        self.consecutive_failures >= 3
    }
    
    /// Remove all port mappings (call on shutdown)
    pub fn cleanup(&self) {
        if self.mappings.is_empty() {
            return;
        }
        
        info!("Cleaning up UPnP port mappings...");
        
        let p2p_port = self.p2p_port;
        
        // Best effort cleanup - don't block on errors
        std::thread::spawn(move || {
            let search_options = igd_next::SearchOptions {
                timeout: Some(Duration::from_secs(3)),
                ..Default::default()
            };
            
            if let Ok(gateway) = igd_next::search_gateway(search_options) {
                let _ = gateway.remove_port(igd_next::PortMappingProtocol::TCP, p2p_port);
                let _ = gateway.remove_port(igd_next::PortMappingProtocol::UDP, p2p_port);
                debug!("UPnP port mappings removed");
            }
        });
    }
    
    /// Get the external address if UPnP mapping was successful
    pub fn get_external_address(&self) -> Option<String> {
        self.mappings.first().map(|m| {
            format!("/ip4/{}/tcp/{}", m.external_ip, m.external_port)
        })
    }
}

/// Helper to get local IP address for the gateway
fn get_local_ip_for_gateway(gateway: &igd_next::Gateway) -> Option<Ipv4Addr> {
    // Try to connect to gateway to determine local IP
    let gateway_ip = gateway.addr.ip();
    
    // Create a UDP socket and "connect" to gateway to find our local IP
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect((gateway_ip, 1)).ok()?;
    
    match socket.local_addr() {
        Ok(addr) => {
            if let std::net::IpAddr::V4(ip) = addr.ip() {
                Some(ip)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

impl Drop for UPnPManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
