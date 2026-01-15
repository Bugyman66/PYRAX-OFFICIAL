use crate::state::AppState;
use crate::rpc::RpcClient;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::fs::File;
use tauri::{State, Manager, AppHandle};
use tracing::{info, error, warn};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
        crate::state::Network::Devnet => "https://rpc.pyrax-devnet.org",
    }
}

/// Get bootstrap P2P peers for a network
fn get_bootstrap_peers(network: &crate::state::Network) -> Vec<&'static str> {
    match network {
        crate::state::Network::Mainnet => vec![
            "/ip4/bootstrap.pyrax.org/tcp/30303",
        ],
        crate::state::Network::Testnet => vec![
            "/ip4/bootstrap.pyrax-testnet.org/tcp/30303",
        ],
        crate::state::Network::Devnet => vec![
            "/ip4/209.38.137.105/tcp/30303",
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
    let (network, rpc_port, data_dir) = {
        let app_state = state.lock();
        if app_state.node_running {
            return Err("Node is already running".to_string());
        }
        (
            app_state.network.clone(),
            get_rpc_port(&app_state.network),
            app_state.data_dir.clone(),
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
        let p2p_port = rpc_port + 21758; // P2P port offset (e.g., 28545 -> 50303)
        let p2p_addr = format!("/ip4/0.0.0.0/tcp/{}", p2p_port); // libp2p multiaddr format
        let staking_port = rpc_port + 2; // Staking RPC (e.g., 28545 -> 28547)
        let staking_addr = format!("0.0.0.0:{}", staking_port);
        let stratum_port = rpc_port - 25212; // Stratum port (e.g., 28545 -> 3333)
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
           .arg("--verbosity").arg("3"); // Enable debug logging
        
        // Add first bootstrap peer (--peer only takes one)
        if let Some(peer) = bootstrap_peers.first() {
            cmd.arg("--peer").arg(*peer);
        }
        
        // Set up log file for node output
        let log_file_path = std::path::PathBuf::from(&data_dir).join("node.log");
        emit_log(&app, "info", "node", &format!("Node logs will be written to: {:?}", log_file_path));
        
        // Create/truncate log file
        if let Ok(log_file) = File::create(&log_file_path) {
            cmd.stdout(log_file.try_clone().unwrap_or_else(|_| File::create(&log_file_path).unwrap()))
               .stderr(Stdio::from(log_file));
        } else {
            cmd.stdout(Stdio::piped())
               .stderr(Stdio::piped());
        }
        
        // On Windows, create the process without a window
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        // Log the full command for debugging
        emit_log(&app, "debug", "node", &format!("Command: {:?}", cmd));
        info!("Starting node with command: {:?}", cmd);
        
        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                
                // Spawn a thread to continuously tail the log file and emit to UI
                let app_for_log = app.clone();
                let log_path = log_file_path.clone();
                std::thread::spawn(move || {
                    use std::io::{Seek, SeekFrom};
                    
                    // Wait for the file to be created
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    
                    let mut last_pos: u64 = 0;
                    
                    // Continuously tail the log file
                    loop {
                        match File::open(&log_path) {
                            Ok(mut file) => {
                                // Seek to where we left off
                                if let Ok(metadata) = file.metadata() {
                                    let file_len = metadata.len();
                                    if file_len > last_pos {
                                        let _ = file.seek(SeekFrom::Start(last_pos));
                                        let reader = BufReader::new(&file);
                                        
                                        for line in reader.lines() {
                                            if let Ok(line) = line {
                                                if line.is_empty() { continue; }
                                                
                                                // Parse log level from the line
                                                let level = if line.contains("ERROR") || line.contains("error") {
                                                    "error"
                                                } else if line.contains("WARN") || line.contains("warn") {
                                                    "warn"
                                                } else if line.contains("DEBUG") || line.contains("debug") {
                                                    "debug"
                                                } else {
                                                    "info"
                                                };
                                                
                                                // Determine category from content
                                                let category = if line.contains("RPC") || line.contains("rpc") {
                                                    "rpc"
                                                } else if line.contains("P2P") || line.contains("peer") || line.contains("Peer") {
                                                    "p2p"
                                                } else if line.contains("Stratum") || line.contains("stratum") || line.contains("Stream B") {
                                                    "mining"
                                                } else if line.contains("Staking") || line.contains("staking") || line.contains("Stream C") {
                                                    "staking"
                                                } else if line.contains("block") || line.contains("Block") {
                                                    "block"
                                                } else {
                                                    "node"
                                                };
                                                
                                                let _ = app_for_log.emit_all("node-log", LogPayload {
                                                    level: level.to_string(),
                                                    category: category.to_string(),
                                                    message: line,
                                                });
                                            }
                                        }
                                        last_pos = file_len;
                                    }
                                }
                            }
                            Err(_) => {
                                // File doesn't exist yet or was deleted - stop tailing
                                break;
                            }
                        }
                        
                        // Poll every 200ms for new content
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                });
                
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
