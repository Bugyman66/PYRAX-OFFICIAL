//! Node management module for PYRAX Desktop
//!
//! Handles spawning, monitoring, and controlling the pyrax-node process.

use std::process::{Child, Command, Stdio};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{info, warn, error};

/// Node process manager
pub struct NodeManager {
    process: Option<Child>,
    data_dir: PathBuf,
    network: String,
    rpc_port: u16,
    p2p_port: u16,
}

impl NodeManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            process: None,
            data_dir,
            network: "mainnet".to_string(),
            rpc_port: 8545,
            p2p_port: 30303,
        }
    }

    pub fn set_network(&mut self, network: &str) {
        self.network = network.to_string();
    }

    pub fn set_rpc_port(&mut self, port: u16) {
        self.rpc_port = port;
    }

    pub fn set_p2p_port(&mut self, port: u16) {
        self.p2p_port = port;
    }

    /// Start the node process
    pub fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Err("Node is already running".to_string());
        }

        let node_path = self.find_node_binary()?;
        
        info!("Starting node: {:?}", node_path);
        info!("Network: {}, RPC: {}, P2P: {}", self.network, self.rpc_port, self.p2p_port);

        let child = Command::new(&node_path)
            .arg("--network")
            .arg(&self.network)
            .arg("--data-dir")
            .arg(&self.data_dir)
            .arg("--rpc-port")
            .arg(self.rpc_port.to_string())
            .arg("--p2p-port")
            .arg(self.p2p_port.to_string())
            .arg("--rpc")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start node: {}", e))?;

        info!("Node started with PID: {}", child.id());
        self.process = Some(child);
        Ok(())
    }

    /// Stop the node process
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            info!("Stopping node...");
            
            // Try graceful shutdown first
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                unsafe {
                    libc::kill(child.id() as i32, libc::SIGTERM);
                }
            }
            
            #[cfg(windows)]
            {
                let _ = child.kill();
            }

            // Wait for process to exit
            match child.wait() {
                Ok(status) => info!("Node exited with status: {}", status),
                Err(e) => warn!("Error waiting for node: {}", e),
            }
            
            Ok(())
        } else {
            Err("Node is not running".to_string())
        }
    }

    /// Check if node is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.process = None;
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Get node process ID
    pub fn pid(&self) -> Option<u32> {
        self.process.as_ref().map(|c| c.id())
    }

    fn find_node_binary(&self) -> Result<PathBuf, String> {
        // Check common locations
        let candidates = vec![
            self.data_dir.join("pyrax-node.exe"),
            self.data_dir.join("pyrax-node"),
            PathBuf::from("pyrax-node.exe"),
            PathBuf::from("pyrax-node"),
        ];

        for path in candidates {
            if path.exists() {
                return Ok(path);
            }
        }

        // Try to find in PATH
        if let Ok(path) = which::which("pyrax-node") {
            return Ok(path);
        }

        Err("pyrax-node binary not found".to_string())
    }
}

impl Drop for NodeManager {
    fn drop(&mut self) {
        if self.process.is_some() {
            let _ = self.stop();
        }
    }
}
