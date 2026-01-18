use anyhow::{Context, Result};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub struct PortManager;

impl PortManager {
    pub fn new() -> Self {
        Self
    }

    pub fn is_port_available(&self, port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    pub fn find_available_port(&self, start_port: u16) -> Result<u16> {
        for port in start_port..=65535 {
            if self.is_port_available(port) {
                return Ok(port);
            }
        }
        anyhow::bail!("No available ports found starting from {}", start_port)
    }

    pub fn find_available_port_range(&self, start_port: u16, count: u16) -> Result<Vec<u16>> {
        let mut ports = Vec::new();
        let mut current = start_port;

        while ports.len() < count as usize && current <= 65535 {
            if self.is_port_available(current) {
                ports.push(current);
            }
            current += 1;
        }

        if ports.len() < count as usize {
            anyhow::bail!(
                "Could not find {} available ports starting from {}",
                count,
                start_port
            );
        }

        Ok(ports)
    }

    pub fn is_port_responding(&self, host: &str, port: u16, timeout_ms: u64) -> bool {
        let addr = format!("{}:{}", host, port);
        TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_millis(timeout_ms),
        )
        .is_ok()
    }

    pub fn wait_for_port(&self, host: &str, port: u16, timeout_secs: u64) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            if self.is_port_responding(host, port, 500) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(500));
        }

        anyhow::bail!(
            "Port {} on {} did not become available within {} seconds",
            port,
            host,
            timeout_secs
        )
    }

    pub fn get_instance_ports(&self, instance_id: u32) -> (u16, u16) {
        let base_p2p = 30303 + ((instance_id - 1) * 10) as u16;
        let base_rpc = 28545 + ((instance_id - 1) * 10) as u16;
        (base_p2p, base_rpc)
    }

    pub fn resolve_port_conflicts(&self, instance_id: u32) -> Result<(u16, u16)> {
        let (base_p2p, base_rpc) = self.get_instance_ports(instance_id);

        let p2p_port = if self.is_port_available(base_p2p) {
            base_p2p
        } else {
            self.find_available_port(base_p2p + 1)?
        };

        let rpc_port = if self.is_port_available(base_rpc) {
            base_rpc
        } else {
            self.find_available_port(base_rpc + 1)?
        };

        Ok((p2p_port, rpc_port))
    }
}

impl Default for PortManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_availability() {
        let manager = PortManager::new();
        // Port 1 should not be available (privileged)
        // Higher ephemeral ports should be available
        let port = manager.find_available_port(49152).unwrap();
        assert!(port >= 49152);
    }

    #[test]
    fn test_instance_ports() {
        let manager = PortManager::new();
        assert_eq!(manager.get_instance_ports(1), (30303, 28545));
        assert_eq!(manager.get_instance_ports(2), (30313, 28555));
        assert_eq!(manager.get_instance_ports(3), (30323, 28565));
    }
}
