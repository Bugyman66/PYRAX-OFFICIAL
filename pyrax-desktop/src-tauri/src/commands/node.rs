use crate::state::AppState;
use crate::rpc::RpcClient;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::process::{Command, Stdio};
use std::fs;
use std::io::{BufRead, BufReader};
use tauri::{State, Manager, AppHandle};
use tracing::{info, error, warn};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Remote server log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteLogEntry {
    pub timestamp: String,
    pub level: String,
    pub category: String,
    pub message: String,
}

/// Remote server status including logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteServerStatus {
    pub online: bool,
    pub streams: StreamStatus,
    pub peer_count: u32,
    pub block_height: u64,
    pub logs: Vec<RemoteLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStatus {
    pub stream_a_rpc: bool,
    pub stream_b_stratum: bool,
    pub stream_c_staking: bool,
}

#[derive(Clone, Serialize)]
struct LogPayload {
    level: String,
    category: String,
    message: String,
}

fn emit_log(app: &AppHandle, level: &str, category: &str, message: &str) {
    let _ = app.emit_all("node-log", LogPayload {
        level: level.to_string(),
        category: category.to_string(),
        message: message.to_string(),
    });
}

/// Check if a log line should be filtered out (noise reduction)
fn should_filter_log(line: &str) -> bool {
    // Filter out repetitive dial failure messages for NAT'd peers
    // These create noise when peers behind NAT are discovered but unreachable
    if line.contains("Connection attempt to peer failed") && line.contains("ConnectionRefused") {
        return true;
    }
    if line.contains("Dial to") && line.contains("failed") && !line.contains("bootnode") {
        return true;
    }
    if line.contains("Dial failure for") && !line.contains("bootnode") {
        return true;
    }
    // Filter out noisy swarm polling messages
    if line.contains("dialing address") && !line.contains("209.38.137.105") {
        return true;
    }
    // Filter out ANSI escape codes spam
    if line.contains("[1mSwarm::poll[0m") && (line.contains("ConnectionRefused") || line.contains("Timeout")) {
        return true;
    }
    false
}

/// Parse pyrax-node tracing log format
/// Example: "2026-01-16T05:12:43.406091Z  INFO Connected to peer: 12D3KooW..."
/// Returns (level, category, message) - returns empty strings if should be filtered
fn parse_node_log(line: &str) -> (String, String, String) {
    // Tracing format: TIMESTAMP LEVEL [target] message
    // or: TIMESTAMP LEVEL message
    let line = line.trim();
    
    // Filter out noisy logs
    if should_filter_log(line) {
        return (String::new(), String::new(), String::new());
    }
    
    // Skip timestamp (ISO 8601 format)
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return ("info".to_string(), "node".to_string(), line.to_string());
    }
    
    // Extract level (INFO, WARN, ERROR, DEBUG, TRACE)
    let level_str = parts.get(1).unwrap_or(&"INFO").trim();
    let level = match level_str.to_uppercase().as_str() {
        "INFO" => "info",
        "WARN" | "WARNING" => "warn",
        "ERROR" => "error",
        "DEBUG" | "TRACE" => "debug",
        _ => "info",
    };
    
    // Get message (rest of line after level)
    let message = parts.get(2).unwrap_or(&line).trim().to_string();
    
    // Detect category from message content
    let category = if message.contains("peer") || message.contains("Peer") || 
                      message.contains("P2P") || message.contains("Kademlia") ||
                      message.contains("Connected to") || message.contains("Disconnected") ||
                      message.contains("mDNS") || message.contains("DHT") {
        "p2p"
    } else if message.contains("block") || message.contains("Block") || 
              message.contains("height") || message.contains("sync") {
        "block"
    } else if message.contains("RPC") || message.contains("rpc") ||
              message.contains("JSON") || message.contains("request") {
        "rpc"
    } else if message.contains("mining") || message.contains("Mining") ||
              message.contains("Stratum") || message.contains("worker") ||
              message.contains("Worker") || message.contains("KAWPOW") {
        "mining"
    } else if message.contains("staking") || message.contains("Staking") ||
              message.contains("stake") || message.contains("ZK") {
        "staking"
    } else {
        "node"
    };
    
    (level.to_string(), category.to_string(), message)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatus {
    pub running: bool,
    pub connected: bool,
    pub syncing: bool,
    pub sync_progress: f64,
    pub peer_count: u32,
    pub block_height: u64,
    pub block_hash: String,
    pub network: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
    pub chain_id: u64,
    pub best_block_hash: String,
    pub best_block_height: u64,
    pub genesis_hash: String,
    pub difficulty: String,
    pub total_difficulty: String,
    pub peer_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub client_version: String,
    pub best_height: u64,
    pub latency_ms: u32,
    pub direction: String,
}

fn get_rpc_port(network: &crate::state::Network) -> u16 {
    match network {
        crate::state::Network::Mainnet => 8545,
        crate::state::Network::Testnet => 18545,
        crate::state::Network::Devnet => 28545,
    }
}

/// Get the remote RPC URL for a network (fallback only)
fn get_remote_rpc_url(network: &crate::state::Network) -> &'static str {
    match network {
        crate::state::Network::Mainnet => "https://rpc.pyrax.org",
        crate::state::Network::Testnet => "https://rpc.pyrax-testnet.org",
        crate::state::Network::Devnet => "http://209.38.137.105:28545", // Digital Ocean bootnode
    }
}

/// Get bootstrap P2P peers for a network
/// IMPORTANT: Addresses MUST include /p2p/<peer_id> suffix for relay reservation to work
fn get_bootstrap_peers(network: &crate::state::Network) -> Vec<&'static str> {
    match network {
        crate::state::Network::Mainnet => vec![
            "/ip4/bootstrap.pyrax.org/tcp/30303",
        ],
        crate::state::Network::Testnet => vec![
            "/ip4/bootstrap.pyrax-testnet.org/tcp/30303",
        ],
        crate::state::Network::Devnet => vec![
            // Full multiaddr with peer ID required for relay NAT traversal
            "/ip4/209.38.137.105/tcp/30303/p2p/12D3KooWJ5d89acu9cyQTBwRhE4kJ33HaVcPZ1Xr2gFozmS9jV1s",
        ],
    }
}

fn get_node_binary_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    let binary_name = "pyrax-node.exe";
    #[cfg(not(target_os = "windows"))]
    let binary_name = "pyrax-node";
    
    // Check current directory (where the app executable is)
    let current_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    
    // Check in same directory as executable
    let local_path = current_dir.join(binary_name);
    if local_path.exists() {
        info!("Found pyrax-node at: {:?}", local_path);
        return Some(local_path);
    }
    
    // Check in resources directory (Tauri bundle location)
    let resource_path = current_dir.join("resources").join(binary_name);
    if resource_path.exists() {
        info!("Found pyrax-node in resources: {:?}", resource_path);
        return Some(resource_path);
    }
    
    // Check in _up_/resources (dev mode)
    let dev_resource_path = current_dir.join("..").join("resources").join(binary_name);
    if dev_resource_path.exists() {
        info!("Found pyrax-node in dev resources: {:?}", dev_resource_path);
        return Some(dev_resource_path);
    }
    
    // Check src-tauri/resources (dev mode from target directory)
    let src_tauri_path = std::path::PathBuf::from("pyrax-desktop/src-tauri/resources").join(binary_name);
    if src_tauri_path.exists() {
        info!("Found pyrax-node in src-tauri/resources: {:?}", src_tauri_path);
        return Some(src_tauri_path);
    }
    
    // Try to find in PATH
    if let Ok(output) = std::process::Command::new("where").arg(binary_name).output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout);
            if let Some(first_line) = path_str.lines().next() {
                let path = std::path::PathBuf::from(first_line.trim());
                if path.exists() {
                    info!("Found pyrax-node in PATH: {:?}", path);
                    return Some(path);
                }
            }
        }
    }
    
    warn!("pyrax-node binary not found");
    None
}

#[tauri::command]
pub async fn start_node(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<NodeStatus, String> {
    let (network, rpc_port, data_dir, log_verbosity) = {
        let app_state = state.lock();
        if app_state.node_running {
            return Err("Node is already running".to_string());
        }
        (
            app_state.network.clone(),
            get_rpc_port(&app_state.network),
            app_state.data_dir.clone(),
            app_state.settings.log_verbosity,
        )
    };
    
    emit_log(&app, "info", "node", &format!("Starting PYRAX full node on {:?} network...", network));
    info!("Starting PYRAX full node on {:?} network...", network);
    
    // Check if a local node is already running on this port
    emit_log(&app, "info", "rpc", &format!("Checking for existing node on port {}...", rpc_port));
    let local_rpc = RpcClient::localhost(rpc_port);
    if local_rpc.is_connected().await {
        emit_log(&app, "info", "node", &format!("Found existing local node on port {}, connecting...", rpc_port));
        info!("Found existing local node on port {}, connecting...", rpc_port);
        {
            let mut app_state = state.lock();
            app_state.node_running = true;
            app_state.node_process = None; // External node
            app_state.rpc_port = rpc_port;
        }
        
        match local_rpc.get_chain_info().await {
            Ok(info) => {
                emit_log(&app, "info", "block", &format!("Connected to existing node: height={}", info.best_block_height));
                info!("Connected to existing node: network={}, height={}", info.network, info.best_block_height);
                return Ok(NodeStatus {
                    running: true,
                    connected: true,
                    syncing: info.syncing,
                    sync_progress: if info.syncing { 50.0 } else { 100.0 },
                    peer_count: 0,
                    block_height: info.best_block_height,
                    block_hash: info.best_block_hash,
                    network: info.network,
                    version: env!("CARGO_PKG_VERSION").to_string(),
                });
            }
            Err(e) => {
                warn!("Connected but failed to get chain info: {}", e);
            }
        }
    }
    
    // Try to spawn local pyrax-node binary (TRUE DECENTRALIZATION)
    if let Some(binary_path) = get_node_binary_path() {
        emit_log(&app, "info", "node", &format!("Spawning local full node from: {:?}", binary_path));
        info!("Spawning local full node from: {:?}", binary_path);
        
        let network_arg = match network {
            crate::state::Network::Mainnet => "mainnet",
            crate::state::Network::Testnet => "testnet",
            crate::state::Network::Devnet => "devnet",
        };
        
        let rpc_addr = format!("0.0.0.0:{}", rpc_port);
        // Use standard P2P port based on network - MUST match bootnode port for peer discovery
        let p2p_port = match network {
            crate::state::Network::Mainnet => 30303,
            crate::state::Network::Testnet => 30303,
            crate::state::Network::Devnet => 30303, // Must match bootnode P2P port
        };
        let p2p_addr = format!("/ip4/0.0.0.0/tcp/{}", p2p_port); // libp2p multiaddr format
        let staking_port = rpc_port + 2; // Staking RPC (e.g., 28545 -> 28547)
        let staking_addr = format!("0.0.0.0:{}", staking_port);
        // Use standard stratum port - same across all networks
        let stratum_port = 3333;
        let stratum_addr = format!("0.0.0.0:{}", stratum_port);
        
        // Get bootstrap peers for this network
        let bootstrap_peers = get_bootstrap_peers(&network);
        
        // Build command with all TriStream services enabled
        let mut cmd = Command::new(&binary_path);
        cmd.arg("--network").arg(network_arg)
           .arg("--rpc")
           .arg("--rpc-addr").arg(&rpc_addr)
           .arg("--p2p")
           .arg("--p2p-addr").arg(&p2p_addr)
           .arg("--stratum")
           .arg("--stratum-addr").arg(&stratum_addr)
           .arg("--staking")
           .arg("--staking-addr").arg(&staking_addr)
           .arg("--datadir").arg(&data_dir)
           .arg("--verbosity").arg(log_verbosity.to_string());
        
        // Add first bootstrap peer (--peer only takes one)
        if let Some(peer) = bootstrap_peers.first() {
            cmd.arg("--peer").arg(*peer);
        }
        
        // Ensure data directory exists
        let data_path = std::path::PathBuf::from(&data_dir);
        if let Err(e) = fs::create_dir_all(&data_path) {
            emit_log(&app, "error", "node", &format!("Failed to create data directory {:?}: {}", data_path, e));
        }
        emit_log(&app, "info", "node", &format!("Data directory: {:?}", data_path));
        
        // Capture stdout/stderr to stream logs to UI
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // On Windows, create the process without a window and detached
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        }
        
        // Log the full command for debugging
        emit_log(&app, "debug", "node", &format!("Command: {:?}", cmd));
        info!("Starting node with command: {:?}", cmd);
        
        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                
                // Spawn background thread to stream stdout logs to UI
                if let Some(stdout) = child.stdout.take() {
                    let app_clone = app.clone();
                    std::thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                // Parse tracing log format: "2026-01-16T05:12:43.406091Z  INFO message"
                                let (level, category, message) = parse_node_log(&line);
                                // Skip filtered logs (empty strings)
                                if !level.is_empty() && !message.is_empty() {
                                    emit_log(&app_clone, &level, &category, &message);
                                }
                            }
                        }
                    });
                }
                
                // Spawn background thread to stream stderr logs to UI
                if let Some(stderr) = child.stderr.take() {
                    let app_clone = app.clone();
                    std::thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                let (level, category, message) = parse_node_log(&line);
                                // Skip filtered logs (empty strings)
                                if !level.is_empty() && !message.is_empty() {
                                    emit_log(&app_clone, &level, &category, &message);
                                }
                            }
                        }
                    });
                }
                
                // Give the process a moment to start, then verify it's running
                std::thread::sleep(std::time::Duration::from_millis(500));
                
                match child.try_wait() {
                    Ok(Some(status)) => {
                        // Process exited immediately - this is a problem!
                        emit_log(&app, "error", "node", &format!("Node process exited immediately with status: {}", status));
                        return Err(format!("Node process exited immediately with status: {}", status));
                    }
                    Ok(None) => {
                        // Process is still running - good!
                        emit_log(&app, "info", "node", &format!("Node process {} is running", pid));
                    }
                    Err(e) => {
                        emit_log(&app, "warn", "node", &format!("Could not check process status: {}", e));
                    }
                }
                
                emit_log(&app, "info", "node", &format!("Node started with PID: {}", pid));
                emit_log(&app, "info", "p2p", &format!("P2P listening on {}", p2p_addr));
                emit_log(&app, "info", "rpc", &format!("Stream A RPC binding to {}", rpc_addr));
                emit_log(&app, "info", "mining", &format!("Stream B Stratum binding to {}", stratum_addr));
                emit_log(&app, "info", "staking", &format!("Stream C Staking RPC binding to {}", staking_addr));
                info!("Node started with PID: {} - syncing blockchain from P2P peers...", pid);
                
                // Update state
                {
                    let mut app_state = state.lock();
                    app_state.node_running = true;
                    app_state.node_process = Some(child);
                    app_state.rpc_port = rpc_port;
                }
                
                // Wait for node RPC to be ready
                emit_log(&app, "info", "rpc", "Waiting for RPC server to be ready...");
                let rpc = RpcClient::localhost(rpc_port);
                let mut attempts = 0;
                let max_attempts = 120; // Wait up to 60 seconds
                
                while attempts < max_attempts {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    
                    // Check if process is still running
                    {
                        let app_state = state.lock();
                        if let Some(ref _process) = app_state.node_process {
                            // Process handle exists - check if it's still alive
                        } else {
                            emit_log(&app, "error", "node", "Node process terminated unexpectedly");
                            break;
                        }
                    }
                    
                    if rpc.is_connected().await {
                        emit_log(&app, "info", "rpc", &format!("RPC server ready on port {}", rpc_port));
                        emit_log(&app, "info", "node", "Node fully operational - all streams active");
                        info!("Node RPC is ready!");
                        break;
                    }
                    
                    if attempts % 10 == 0 && attempts > 0 {
                        emit_log(&app, "debug", "rpc", &format!("Still waiting for RPC... ({}s)", attempts / 2));
                    }
                    attempts += 1;
                }
                
                if attempts >= max_attempts {
                    emit_log(&app, "warn", "rpc", "RPC timeout - falling back to remote RPC");
                }
                
                // Start connection watchdog for self-healing
                let _ = start_connection_watchdog_internal(app.clone(), state.inner().clone()).await;
                
                // Get initial status
                return get_node_status(state).await;
            }
            Err(e) => {
                emit_log(&app, "error", "node", &format!("Failed to spawn pyrax-node: {}", e));
                error!("Failed to spawn pyrax-node: {}", e);
                // Fall through to remote RPC fallback
            }
        }
    } else {
        emit_log(&app, "warn", "node", "pyrax-node binary not found, falling back to remote RPC");
        warn!("pyrax-node binary not found, falling back to remote RPC");
    }
    
    // Fallback: Connect to remote RPC (light client mode - NOT fully decentralized)
    let remote_url = get_remote_rpc_url(&network);
    emit_log(&app, "info", "rpc", &format!("Connecting to remote RPC: {}", remote_url));
    info!("Falling back to remote RPC (light client mode): {}", remote_url);
    let remote_rpc = RpcClient::new(remote_url);
    
    if remote_rpc.is_connected().await {
        emit_log(&app, "warn", "node", "Connected in LIGHT CLIENT mode (not fully decentralized)");
        warn!("Connected to remote RPC - running in LIGHT CLIENT mode (not fully decentralized)");
        
        {
            let mut app_state = state.lock();
            app_state.node_running = true;
            app_state.node_process = None;
            app_state.rpc_port = rpc_port;
        }
        
        match remote_rpc.get_chain_info().await {
            Ok(info) => {
                return Ok(NodeStatus {
                    running: true,
                    connected: true,
                    syncing: info.syncing,
                    sync_progress: if info.syncing { 50.0 } else { 100.0 },
                    peer_count: 1,
                    block_height: info.best_block_height,
                    block_hash: info.best_block_hash,
                    network: format!("{} (light)", info.network),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                });
            }
            Err(e) => {
                warn!("Remote RPC connected but failed to get chain info: {}", e);
                return Ok(NodeStatus {
                    running: true,
                    connected: true,
                    syncing: false,
                    sync_progress: 100.0,
                    peer_count: 1,
                    block_height: 0,
                    block_hash: String::new(),
                    network: format!("{} (light)", network.to_string()),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                });
            }
        }
    }
    
    Err(format!(
        "Cannot start node: pyrax-node binary not found and remote RPC {} is unavailable. \
         Please ensure pyrax-node.exe is in the application resources folder.",
        remote_url
    ))
}

#[tauri::command]
pub async fn stop_node(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.lock();
    
    // Allow stopping even if not "running" - reset state
    emit_log(&app, "info", "node", "Stopping node...");
    info!("Stopping node...");
    
    // Kill the node process if we have one
    if let Some(mut child) = app_state.node_process.take() {
        emit_log(&app, "info", "node", &format!("Stopping local node process (PID: {})", child.id()));
        info!("Stopping local node process (PID: {})", child.id());
        
        #[cfg(target_os = "windows")]
        {
            // On Windows, use taskkill for graceful shutdown
            let _ = Command::new("taskkill")
                .args(["/PID", &child.id().to_string(), "/T", "/F"])
                .output();
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // On Unix, send SIGTERM
            let _ = child.kill();
        }
        
        // Wait for process to exit
        let _ = child.wait();
        emit_log(&app, "info", "node", "Node process stopped successfully");
        info!("Node process stopped");
    } else if app_state.node_running {
        // Light client mode - just disconnect
        emit_log(&app, "info", "rpc", "Disconnecting from remote RPC (light client mode)");
        info!("Disconnecting from remote RPC (light client mode)");
    }
    
    app_state.node_running = false;
    Ok(())
}

#[tauri::command]
pub async fn get_node_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<NodeStatus, String> {
    let (running, network, rpc_port) = {
        let app_state = state.lock();
        (app_state.node_running, app_state.network.clone(), app_state.rpc_port)
    };
    
    if !running {
        return Ok(NodeStatus {
            running: false,
            connected: false,
            syncing: false,
            sync_progress: 0.0,
            peer_count: 0,
            block_height: 0,
            block_hash: String::new(),
            network: network.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        });
    }
    
    // Try remote RPC first, then local
    let remote_url = get_remote_rpc_url(&network);
    let remote_rpc = RpcClient::new(remote_url);
    let local_rpc = RpcClient::localhost(rpc_port);
    
    // Check which RPC is connected
    let (rpc, is_remote) = if remote_rpc.is_connected().await {
        (remote_rpc, true)
    } else if local_rpc.is_connected().await {
        (local_rpc, false)
    } else {
        return Ok(NodeStatus {
            running: true,
            connected: false,
            syncing: false,
            sync_progress: 0.0,
            peer_count: 0,
            block_height: 0,
            block_hash: String::new(),
            network: network.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        });
    };
    
    // Get chain info from connected node
    match rpc.get_chain_info().await {
        Ok(info) => {
            let syncing = info.syncing;
            let sync_progress = if syncing { 50.0 } else { 100.0 };
            
            Ok(NodeStatus {
                running: true,
                connected: true,
                syncing,
                sync_progress,
                peer_count: if is_remote { 1 } else { 0 }, // 1 peer if connected to remote
                block_height: info.best_block_height,
                block_hash: info.best_block_hash,
                network: info.network,
                version: env!("CARGO_PKG_VERSION").to_string(),
            })
        }
        Err(e) => {
            warn!("Failed to get chain info: {}", e);
            Ok(NodeStatus {
                running: true,
                connected: true,
                syncing: false,
                sync_progress: 0.0,
                peer_count: 0,
                block_height: 0,
                block_hash: String::new(),
                network: network.to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            })
        }
    }
}

#[tauri::command]
pub async fn get_chain_info(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ChainInfo, String> {
    let (running, rpc_port) = {
        let app_state = state.lock();
        (app_state.node_running, app_state.rpc_port)
    };
    
    if !running {
        return Err("Node is not running".to_string());
    }
    
    let rpc = RpcClient::localhost(rpc_port);
    
    match rpc.get_chain_info().await {
        Ok(info) => Ok(ChainInfo {
            chain_id: info.chain_id as u64,
            best_block_hash: info.best_block_hash,
            best_block_height: info.best_block_height,
            genesis_hash: info.genesis_hash,
            difficulty: info.difficulty.to_string(),
            total_difficulty: info.difficulty.to_string(),
            peer_count: 0, // Peers not yet exposed via RPC
        }),
        Err(e) => Err(format!("Failed to get chain info: {}", e)),
    }
}

#[tauri::command]
pub async fn get_peers(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<PeerInfo>, String> {
    let (running, rpc_port) = {
        let app_state = state.lock();
        (app_state.node_running, app_state.rpc_port)
    };
    
    if !running {
        return Err("Node is not running".to_string());
    }
    
    let rpc = RpcClient::localhost(rpc_port);
    
    match rpc.get_peers().await {
        Ok(peers) => Ok(peers.into_iter().map(|p| PeerInfo {
            id: p.id,
            address: p.address,
            client_version: p.client_version,
            best_height: p.best_height,
            latency_ms: p.latency_ms,
            direction: p.direction,
        }).collect()),
        Err(e) => Err(format!("Failed to get peers: {}", e)),
    }
}

/// Get remote bootnode server status and logs
#[tauri::command]
pub async fn get_remote_server_logs(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<RemoteServerStatus, String> {
    let network = {
        let app_state = state.lock();
        app_state.network.clone()
    };
    
    let status_url = match network {
        crate::state::Network::Mainnet => "https://rpc.pyrax.org/status",
        crate::state::Network::Testnet => "https://rpc.pyrax-testnet.org/status",
        crate::state::Network::Devnet => "http://209.38.137.105:28545", // Use RPC endpoint
    };
    
    emit_log(&app, "info", "rpc", &format!("Fetching remote server status from {}", status_url));
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    match client.get(status_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<RemoteServerStatus>().await {
                    Ok(status) => {
                        // Emit each log entry to the UI
                        for log in &status.logs {
                            emit_log(&app, &log.level, &log.category, &format!("[REMOTE] {}", log.message));
                        }
                        Ok(status)
                    }
                    Err(e) => {
                        // Server responded but not with expected format - try basic connectivity
                        emit_log(&app, "warn", "rpc", &format!("Server responded but status format unknown: {}", e));
                        Ok(RemoteServerStatus {
                            online: true,
                            streams: StreamStatus {
                                stream_a_rpc: true,
                                stream_b_stratum: false,
                                stream_c_staking: false,
                            },
                            peer_count: 0,
                            block_height: 0,
                            logs: vec![],
                        })
                    }
                }
            } else {
                emit_log(&app, "error", "rpc", &format!("Remote server returned status: {}", response.status()));
                Err(format!("Remote server returned status: {}", response.status()))
            }
        }
        Err(e) => {
            emit_log(&app, "error", "rpc", &format!("Failed to connect to remote server: {}", e));
            Err(format!("Failed to connect to remote server: {}", e))
        }
    }
}

/// Start streaming logs from remote bootnode
#[tauri::command]
pub async fn start_remote_log_stream(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let network = {
        let app_state = state.lock();
        app_state.network.clone()
    };
    
    let rpc_url = get_remote_rpc_url(&network);
    
    emit_log(&app, "info", "node", &format!("Connecting to remote bootnode: {}", rpc_url));
    
    // Try to get chain info from remote RPC to verify connection
    let rpc = RpcClient::new(rpc_url);
    
    if rpc.is_connected().await {
        emit_log(&app, "info", "rpc", "✓ Stream A (RPC) - Connected to remote bootnode");
        
        match rpc.get_chain_info().await {
            Ok(info) => {
                emit_log(&app, "info", "block", &format!("Chain: {} | Height: {} | Hash: {}", 
                    info.network, info.best_block_height, &info.best_block_hash[..16]));
                emit_log(&app, "info", "node", &format!("Genesis: {}", &info.genesis_hash[..16]));
                emit_log(&app, "info", "node", &format!("Difficulty: {}", info.difficulty));
                
                if info.syncing {
                    emit_log(&app, "info", "node", "Node is syncing...");
                } else {
                    emit_log(&app, "info", "node", "Node is fully synced");
                }
            }
            Err(e) => {
                emit_log(&app, "warn", "rpc", &format!("Connected but failed to get chain info: {}", e));
            }
        }
        
        // Check stratum port (Stream B)
        let stratum_port = match network {
            crate::state::Network::Mainnet => 3333,
            crate::state::Network::Testnet => 13333,
            crate::state::Network::Devnet => 3333,
        };
        emit_log(&app, "info", "mining", &format!("Stream B (Stratum) - Port {} configured", stratum_port));
        
        // Check staking port (Stream C)
        let staking_port = match network {
            crate::state::Network::Mainnet => 8547,
            crate::state::Network::Testnet => 18547,
            crate::state::Network::Devnet => 28547,
        };
        emit_log(&app, "info", "staking", &format!("Stream C (Staking) - Port {} configured", staking_port));
        
        emit_log(&app, "info", "node", "Remote bootnode connection established");
        Ok(())
    } else {
        emit_log(&app, "error", "rpc", &format!("Failed to connect to remote bootnode: {}", rpc_url));
        Err(format!("Failed to connect to remote bootnode: {}", rpc_url))
    }
}

/// Internal function to start the connection watchdog (called from start_node)
async fn start_connection_watchdog_internal(
    app: AppHandle,
    state: Arc<Mutex<AppState>>,
) -> Result<(), String> {
    let (network, rpc_port) = {
        let app_state = state.lock();
        (app_state.network.clone(), app_state.rpc_port)
    };
    
    let remote_url = get_remote_rpc_url(&network);
    let state_clone = state.clone();
    let app_clone = app.clone();
    
    emit_log(&app, "info", "node", "Starting connection watchdog for self-healing network recovery");
    
    start_watchdog_task(app_clone, state_clone, remote_url, rpc_port).await;
    
    Ok(())
}

/// Start the watchdog background task
async fn start_watchdog_task(
    app: AppHandle,
    state: Arc<Mutex<AppState>>,
    remote_url: &'static str,
    rpc_port: u16,
) {
    let state_clone = state;
    let app_clone = app;
    
    // Spawn background watchdog task
    tokio::spawn(async move {
        let mut consecutive_failures = 0;
        let max_failures = 3; // Restart after 3 consecutive failures (30 seconds)
        let check_interval = tokio::time::Duration::from_secs(10);
        
        loop {
            tokio::time::sleep(check_interval).await;
            
            // Check if node is supposed to be running
            let (node_running, has_process) = {
                let app_state = state_clone.lock();
                (app_state.node_running, app_state.node_process.is_some())
            };
            
            if !node_running {
                // Node is stopped, exit watchdog
                emit_log(&app_clone, "debug", "node", "Watchdog: Node stopped, exiting watchdog");
                break;
            }
            
            // Check local RPC connectivity
            let local_rpc = RpcClient::localhost(rpc_port);
            let local_connected = local_rpc.is_connected().await;
            
            // Check remote bootnode connectivity
            let remote_rpc = RpcClient::new(remote_url);
            let remote_connected = remote_rpc.is_connected().await;
            
            if !local_connected && has_process {
                consecutive_failures += 1;
                emit_log(&app_clone, "warn", "node", &format!(
                    "Watchdog: Local node RPC not responding ({}/{})", 
                    consecutive_failures, max_failures
                ));
                
                if consecutive_failures >= max_failures {
                    emit_log(&app_clone, "error", "node", "Watchdog: Node unresponsive - initiating auto-restart");
                    
                    // Kill the unresponsive node
                    {
                        let mut app_state = state_clone.lock();
                        if let Some(mut child) = app_state.node_process.take() {
                            emit_log(&app_clone, "info", "node", &format!("Watchdog: Killing unresponsive node (PID: {})", child.id()));
                            
                            #[cfg(target_os = "windows")]
                            {
                                let _ = std::process::Command::new("taskkill")
                                    .args(["/PID", &child.id().to_string(), "/T", "/F"])
                                    .output();
                            }
                            
                            #[cfg(not(target_os = "windows"))]
                            {
                                let _ = child.kill();
                            }
                            
                            let _ = child.wait();
                        }
                        app_state.node_running = false;
                    }
                    
                    // Emit disconnect event to UI
                    let _ = app_clone.emit_all("node-disconnected", serde_json::json!({
                        "reason": "Node unresponsive",
                        "will_restart": true
                    }));
                    
                    // Wait for network to stabilize
                    emit_log(&app_clone, "info", "node", "Watchdog: Waiting 5 seconds before restart attempt...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    
                    // Check if bootnode is reachable before restarting
                    if remote_rpc.is_connected().await {
                        emit_log(&app_clone, "info", "node", "Watchdog: Bootnode reachable - triggering auto-restart");
                        
                        // Emit restart event to UI (UI should call start_node)
                        let _ = app_clone.emit_all("node-restart-requested", serde_json::json!({
                            "reason": "Auto-recovery after disconnect"
                        }));
                    } else {
                        emit_log(&app_clone, "warn", "node", "Watchdog: Bootnode not reachable - waiting for network...");
                        
                        // Keep checking bootnode until it's available
                        let mut bootnode_wait_count = 0;
                        while bootnode_wait_count < 12 { // Wait up to 2 minutes
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                            bootnode_wait_count += 1;
                            
                            if remote_rpc.is_connected().await {
                                emit_log(&app_clone, "info", "node", "Watchdog: Bootnode now reachable - triggering auto-restart");
                                let _ = app_clone.emit_all("node-restart-requested", serde_json::json!({
                                    "reason": "Auto-recovery after network restoration"
                                }));
                                break;
                            }
                            
                            emit_log(&app_clone, "debug", "node", &format!(
                                "Watchdog: Still waiting for bootnode ({}/12)...", 
                                bootnode_wait_count
                            ));
                        }
                        
                        if bootnode_wait_count >= 12 {
                            emit_log(&app_clone, "error", "node", "Watchdog: Bootnode unreachable for 2 minutes - manual intervention may be required");
                            let _ = app_clone.emit_all("node-network-error", serde_json::json!({
                                "reason": "Bootnode unreachable",
                                "duration_seconds": 120
                            }));
                        }
                    }
                    
                    // Exit watchdog - a new one will start when node restarts
                    break;
                }
            } else if local_connected {
                // Reset failure counter on successful connection
                if consecutive_failures > 0 {
                    emit_log(&app_clone, "info", "node", "Watchdog: Node connection restored");
                    consecutive_failures = 0;
                }
            }
            
            // Also check bootnode connectivity periodically
            if !remote_connected && consecutive_failures == 0 {
                emit_log(&app_clone, "warn", "p2p", "Watchdog: Bootnode not reachable - monitoring...");
            }
        }
    });
}

/// Connection watchdog - monitors bootnode connectivity and auto-restarts node on disconnect
/// This runs as a background task and emits events to the UI
#[tauri::command]
pub async fn start_connection_watchdog(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    start_connection_watchdog_internal(app, state.inner().clone()).await
}

/// Stop connection watchdog (called when node is intentionally stopped)
#[tauri::command]
pub async fn stop_connection_watchdog(
    app: AppHandle,
) -> Result<(), String> {
    emit_log(&app, "info", "node", "Connection watchdog stopped");
    // The watchdog will exit on its own when it detects node_running = false
    Ok(())
}

/// Clear local blockchain data for a specific network
/// This removes the chain database to allow fresh sync
#[tauri::command]
pub async fn clear_local_data(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    network: String,
) -> Result<String, String> {
    // First, ensure the node is stopped
    {
        let state_guard = state.lock();
        if state_guard.node_running {
            return Err("Please stop the node before clearing data".to_string());
        }
    }
    
    // Get the data directory
    let app_data_dir = app.path_resolver()
        .app_data_dir()
        .ok_or("Failed to get app data directory")?;
    
    let data_path = app_data_dir.join("data").join(&network);
    
    emit_log(&app, "info", "system", &format!("Clearing local data for {} network...", network));
    
    if data_path.exists() {
        // Remove the entire network data directory
        match fs::remove_dir_all(&data_path) {
            Ok(_) => {
                emit_log(&app, "info", "system", &format!("Successfully cleared {} data at {:?}", network, data_path));
                Ok(format!("Cleared local data for {} network. The node will sync fresh on next start.", network))
            }
            Err(e) => {
                let error_msg = format!("Failed to clear data: {}", e);
                emit_log(&app, "error", "system", &error_msg);
                Err(error_msg)
            }
        }
    } else {
        emit_log(&app, "info", "system", &format!("No data found for {} network at {:?}", network, data_path));
        Ok(format!("No local data found for {} network.", network))
    }
}

/// Get the size of local data for a specific network
#[tauri::command]
pub async fn get_local_data_size(
    app: AppHandle,
    network: String,
) -> Result<String, String> {
    let app_data_dir = app.path_resolver()
        .app_data_dir()
        .ok_or("Failed to get app data directory")?;
    
    let data_path = app_data_dir.join("data").join(&network);
    
    if !data_path.exists() {
        return Ok("No data".to_string());
    }
    
    // Calculate directory size
    fn dir_size(path: &std::path::Path) -> u64 {
        let mut size = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    size += dir_size(&path);
                } else if let Ok(metadata) = entry.metadata() {
                    size += metadata.len();
                }
            }
        }
        size
    }
    
    let size = dir_size(&data_path);
    
    // Format size
    let formatted = if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    };
    
    Ok(formatted)
}
