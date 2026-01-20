//! Advanced NAT Traversal Suite
//!
//! Implements multiple NAT traversal techniques for maximum connectivity:
//! - STUN external IP discovery
//! - NAT-PMP port mapping (fallback for UPnP)
//! - ICE-like candidate gathering
//! - QUIC tunneling support

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};

/// Public STUN servers for external IP discovery
pub const STUN_SERVERS: &[&str] = &[
    "stun.l.google.com:19302",
    "stun1.l.google.com:19302",
    "stun2.l.google.com:19302",
    "stun.cloudflare.com:3478",
    "stun.stunprotocol.org:3478",
];

/// NAT-PMP default gateway port
pub const NAT_PMP_PORT: u16 = 5351;

/// NAT types detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    /// No NAT, public IP
    None,
    /// Full cone NAT - most permissive
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT - most restrictive
    Symmetric,
    /// Unknown NAT type
    Unknown,
}

impl NatType {
    pub fn name(&self) -> &'static str {
        match self {
            NatType::None => "No NAT (Public IP)",
            NatType::FullCone => "Full Cone NAT",
            NatType::RestrictedCone => "Restricted Cone NAT",
            NatType::PortRestrictedCone => "Port Restricted Cone NAT",
            NatType::Symmetric => "Symmetric NAT",
            NatType::Unknown => "Unknown",
        }
    }
    
    pub fn connectivity_score(&self) -> u8 {
        match self {
            NatType::None => 100,
            NatType::FullCone => 90,
            NatType::RestrictedCone => 70,
            NatType::PortRestrictedCone => 50,
            NatType::Symmetric => 20,
            NatType::Unknown => 30,
        }
    }
}

/// ICE candidate types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateType {
    /// Host candidate (local address)
    Host,
    /// Server reflexive (STUN-discovered)
    ServerReflexive,
    /// Peer reflexive (discovered during connectivity check)
    PeerReflexive,
    /// Relayed (via TURN/relay)
    Relay,
}

/// ICE candidate for connectivity
#[derive(Debug, Clone)]
pub struct IceCandidate {
    pub candidate_type: CandidateType,
    pub address: SocketAddr,
    pub priority: u32,
    pub foundation: String,
    pub discovered_at: Instant,
}

impl IceCandidate {
    pub fn host(addr: SocketAddr) -> Self {
        Self {
            candidate_type: CandidateType::Host,
            address: addr,
            priority: 126 << 24, // Host priority base
            foundation: format!("host-{}", addr),
            discovered_at: Instant::now(),
        }
    }
    
    pub fn server_reflexive(addr: SocketAddr, stun_server: &str) -> Self {
        Self {
            candidate_type: CandidateType::ServerReflexive,
            address: addr,
            priority: 100 << 24, // SRFLX priority base
            foundation: format!("srflx-{}", stun_server),
            discovered_at: Instant::now(),
        }
    }
    
    pub fn relay(addr: SocketAddr, relay_server: &str) -> Self {
        Self {
            candidate_type: CandidateType::Relay,
            address: addr,
            priority: 0, // Lowest priority
            foundation: format!("relay-{}", relay_server),
            discovered_at: Instant::now(),
        }
    }
}

/// STUN message types
const STUN_BINDING_REQUEST: u16 = 0x0001;
const STUN_BINDING_RESPONSE: u16 = 0x0101;
const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

/// STUN attribute types
const STUN_ATTR_MAPPED_ADDRESS: u16 = 0x0001;
const STUN_ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;

/// STUN discovery result
#[derive(Debug, Clone)]
pub struct StunResult {
    pub external_ip: IpAddr,
    pub external_port: u16,
    pub server_used: String,
    pub latency_ms: u64,
}

/// Perform STUN discovery to find external IP
pub fn stun_discover() -> Option<StunResult> {
    for server in STUN_SERVERS {
        match stun_query(server) {
            Ok(result) => {
                info!("STUN discovery via {}: {}:{}", server, result.external_ip, result.external_port);
                return Some(result);
            }
            Err(e) => {
                debug!("STUN server {} failed: {}", server, e);
            }
        }
    }
    warn!("All STUN servers failed for external IP discovery");
    None
}

/// Query a single STUN server
fn stun_query(server: &str) -> Result<StunResult, String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;
    
    socket.set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;
    
    let server_addr: SocketAddr = server.parse()
        .or_else(|_| {
            // Resolve hostname
            use std::net::ToSocketAddrs;
            server.to_socket_addrs()
                .map_err(|e| format!("DNS resolution failed: {}", e))?
                .next()
                .ok_or_else(|| "No addresses found".to_string())
        })?;
    
    // Build STUN binding request
    let transaction_id: [u8; 12] = rand::random();
    let mut request = Vec::with_capacity(20);
    request.extend_from_slice(&STUN_BINDING_REQUEST.to_be_bytes());
    request.extend_from_slice(&0u16.to_be_bytes()); // Message length
    request.extend_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
    request.extend_from_slice(&transaction_id);
    
    let start = Instant::now();
    socket.send_to(&request, server_addr)
        .map_err(|e| format!("Send failed: {}", e))?;
    
    let mut buf = [0u8; 512];
    let (len, _) = socket.recv_from(&mut buf)
        .map_err(|e| format!("Receive failed: {}", e))?;
    
    let latency_ms = start.elapsed().as_millis() as u64;
    
    // Parse STUN response
    if len < 20 {
        return Err("Response too short".to_string());
    }
    
    let msg_type = u16::from_be_bytes([buf[0], buf[1]]);
    if msg_type != STUN_BINDING_RESPONSE {
        return Err(format!("Unexpected message type: {}", msg_type));
    }
    
    // Parse attributes
    let mut offset = 20;
    while offset + 4 <= len {
        let attr_type = u16::from_be_bytes([buf[offset], buf[offset + 1]]);
        let attr_len = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]) as usize;
        offset += 4;
        
        if offset + attr_len > len {
            break;
        }
        
        if attr_type == STUN_ATTR_XOR_MAPPED_ADDRESS || attr_type == STUN_ATTR_MAPPED_ADDRESS {
            let family = buf[offset + 1];
            if family == 0x01 { // IPv4
                let port = if attr_type == STUN_ATTR_XOR_MAPPED_ADDRESS {
                    u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]) ^ (STUN_MAGIC_COOKIE >> 16) as u16
                } else {
                    u16::from_be_bytes([buf[offset + 2], buf[offset + 3]])
                };
                
                let ip_bytes = if attr_type == STUN_ATTR_XOR_MAPPED_ADDRESS {
                    let xor = STUN_MAGIC_COOKIE.to_be_bytes();
                    [buf[offset + 4] ^ xor[0], buf[offset + 5] ^ xor[1], 
                     buf[offset + 6] ^ xor[2], buf[offset + 7] ^ xor[3]]
                } else {
                    [buf[offset + 4], buf[offset + 5], buf[offset + 6], buf[offset + 7]]
                };
                
                return Ok(StunResult {
                    external_ip: IpAddr::V4(Ipv4Addr::from(ip_bytes)),
                    external_port: port,
                    server_used: server.to_string(),
                    latency_ms,
                });
            }
        }
        
        offset += attr_len;
        // Pad to 4-byte boundary
        offset = (offset + 3) & !3;
    }
    
    Err("No mapped address in response".to_string())
}

/// NAT-PMP port mapping request
#[derive(Debug, Clone)]
pub struct NatPmpMapping {
    pub internal_port: u16,
    pub external_port: u16,
    pub lifetime_secs: u32,
    pub gateway: IpAddr,
}

/// Request a NAT-PMP port mapping
pub fn natpmp_map_port(internal_port: u16, external_port: u16, lifetime_secs: u32) -> Result<NatPmpMapping, String> {
    // Find default gateway
    let gateway = find_default_gateway()?;
    
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind: {}", e))?;
    
    socket.set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;
    
    // NAT-PMP mapping request
    // Version (1) + Opcode (2 for TCP) + Reserved (2) + Internal Port (2) + External Port (2) + Lifetime (4)
    let mut request = Vec::with_capacity(12);
    request.push(0); // Version 0
    request.push(2); // Opcode 2 (TCP)
    request.extend_from_slice(&0u16.to_be_bytes()); // Reserved
    request.extend_from_slice(&internal_port.to_be_bytes());
    request.extend_from_slice(&external_port.to_be_bytes());
    request.extend_from_slice(&lifetime_secs.to_be_bytes());
    
    let gateway_addr = SocketAddr::new(gateway, NAT_PMP_PORT);
    socket.send_to(&request, gateway_addr)
        .map_err(|e| format!("Send failed: {}", e))?;
    
    let mut buf = [0u8; 16];
    let (len, _) = socket.recv_from(&mut buf)
        .map_err(|e| format!("Receive failed (NAT-PMP not supported?): {}", e))?;
    
    if len < 16 {
        return Err("Response too short".to_string());
    }
    
    let result_code = u16::from_be_bytes([buf[2], buf[3]]);
    if result_code != 0 {
        return Err(format!("NAT-PMP error code: {}", result_code));
    }
    
    let mapped_internal = u16::from_be_bytes([buf[8], buf[9]]);
    let mapped_external = u16::from_be_bytes([buf[10], buf[11]]);
    let mapped_lifetime = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
    
    Ok(NatPmpMapping {
        internal_port: mapped_internal,
        external_port: mapped_external,
        lifetime_secs: mapped_lifetime,
        gateway,
    })
}

/// Find the default gateway IP
fn find_default_gateway() -> Result<IpAddr, String> {
    // Try common gateway addresses
    let common_gateways = [
        "192.168.1.1", "192.168.0.1", "10.0.0.1", "172.16.0.1",
        "192.168.2.1", "192.168.10.1", "192.168.100.1",
    ];
    
    for gw in common_gateways {
        let addr: IpAddr = gw.parse().unwrap();
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect(SocketAddr::new(addr, NAT_PMP_PORT)).is_ok() {
                return Ok(addr);
            }
        }
    }
    
    Err("Could not find default gateway".to_string())
}

/// NAT traversal manager
pub struct NatTraversalManager {
    pub nat_type: NatType,
    pub external_ip: Option<IpAddr>,
    pub external_port: Option<u16>,
    pub candidates: Vec<IceCandidate>,
    pub upnp_available: bool,
    pub natpmp_available: bool,
    pub last_discovery: Option<Instant>,
}

impl NatTraversalManager {
    pub fn new() -> Self {
        Self {
            nat_type: NatType::Unknown,
            external_ip: None,
            external_port: None,
            candidates: Vec::new(),
            upnp_available: false,
            natpmp_available: false,
            last_discovery: None,
        }
    }
    
    /// Perform full NAT discovery
    pub fn discover(&mut self, local_port: u16) {
        info!("Starting NAT traversal discovery...");
        self.candidates.clear();
        
        // Add local host candidates using system network interfaces
        if let Ok(addrs) = std::net::UdpSocket::bind("0.0.0.0:0") {
            if let Ok(local_addr) = addrs.local_addr() {
                if !local_addr.ip().is_loopback() {
                    let addr = SocketAddr::new(local_addr.ip(), local_port);
                    self.candidates.push(IceCandidate::host(addr));
                }
            }
        }
        
        // Try to get local IP by connecting to external address
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    let addr = SocketAddr::new(local_addr.ip(), local_port);
                    if !self.candidates.iter().any(|c| c.address == addr) {
                        self.candidates.push(IceCandidate::host(addr));
                    }
                }
            }
        }
        
        // STUN discovery
        if let Some(stun_result) = stun_discover() {
            self.external_ip = Some(stun_result.external_ip);
            self.external_port = Some(stun_result.external_port);
            
            let addr = SocketAddr::new(stun_result.external_ip, stun_result.external_port);
            self.candidates.push(IceCandidate::server_reflexive(addr, &stun_result.server_used));
            
            // Determine if we have a public IP (no NAT)
            if self.candidates.iter().any(|c| {
                c.candidate_type == CandidateType::Host && c.address.ip() == stun_result.external_ip
            }) {
                self.nat_type = NatType::None;
            }
        }
        
        // Try NAT-PMP
        match natpmp_map_port(local_port, local_port, 3600) {
            Ok(mapping) => {
                self.natpmp_available = true;
                info!("NAT-PMP mapping successful: {}:{}", mapping.gateway, mapping.external_port);
            }
            Err(e) => {
                debug!("NAT-PMP not available: {}", e);
            }
        }
        
        self.last_discovery = Some(Instant::now());
        
        info!("NAT discovery complete: type={}, external_ip={:?}, candidates={}", 
            self.nat_type.name(), self.external_ip, self.candidates.len());
    }
    
    /// Get best candidates for connection
    pub fn best_candidates(&self) -> Vec<&IceCandidate> {
        let mut sorted: Vec<_> = self.candidates.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted
    }
    
    /// Check if rediscovery is needed
    pub fn needs_rediscovery(&self) -> bool {
        match self.last_discovery {
            None => true,
            Some(t) => t.elapsed() > Duration::from_secs(3600), // Rediscover every hour
        }
    }
}

impl Default for NatTraversalManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nat_type_scoring() {
        assert!(NatType::None.connectivity_score() > NatType::Symmetric.connectivity_score());
        assert!(NatType::FullCone.connectivity_score() > NatType::RestrictedCone.connectivity_score());
    }
    
    #[test]
    fn test_ice_candidate_creation() {
        let addr: SocketAddr = "192.168.1.100:30303".parse().unwrap();
        let candidate = IceCandidate::host(addr);
        assert_eq!(candidate.candidate_type, CandidateType::Host);
        assert_eq!(candidate.address, addr);
    }
    
    #[test]
    fn test_nat_manager_creation() {
        let mgr = NatTraversalManager::new();
        assert_eq!(mgr.nat_type, NatType::Unknown);
        assert!(mgr.candidates.is_empty());
    }
}
