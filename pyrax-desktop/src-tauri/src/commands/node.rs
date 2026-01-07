use crate::state::AppState;
use crate::rpc::RpcClient;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::process::{Command, Child, Stdio};
use tauri::State;
use tracing::{info, error, warn};

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

fn get_node_binary_path() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    let binary_name = "pyrax-node.exe";
    #[cfg(not(target_os = "windows"))]
    let binary_name = "pyrax-node";
    
    // Check current directory first, then PATH
    let current_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    
    let local_path = current_dir.join(binary_name);
    if local_path.exists() {
        return local_path;
    }
    
    // Check in resources directory
    let resource_path = current_dir.join("resources").join(binary_name);
    if resource_path.exists() {
        return resource_path;
    }
    
    // Fall back to PATH
    std::path::PathBuf::from(binary_name)
}

#[tauri::command]
pub async fn start_node(
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
    
    info!("Starting pyrax-node on network {:?}, RPC port {}", network, rpc_port);
    
    // First check if a node is already running on this port
    let rpc = RpcClient::localhost(rpc_port);
    if rpc.is_connected().await {
        info!("Found existing node on port {}, connecting...", rpc_port);
        // Update state to mark as running (external node)
        {
            let mut app_state = state.lock();
            app_state.node_running = true;
            app_state.node_process = None; // External node, we don't own it
            app_state.rpc_port = rpc_port;
        }
        
        // Get chain info directly instead of calling get_node_status to avoid potential issues
        match rpc.get_chain_info().await {
            Ok(info) => {
                info!("Connected to node: network={}, height={}", info.network, info.best_block_height);
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
                return Ok(NodeStatus {
                    running: true,
                    connected: true,
                    syncing: false,
                    sync_progress: 100.0,
                    peer_count: 0,
                    block_height: 0,
                    block_hash: String::new(),
                    network: network.to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                });
            }
        }
    }
    
    let binary_path = get_node_binary_path();
    info!("Node binary path: {:?}", binary_path);
    
    // Build command arguments
    let network_arg = match network {
        crate::state::Network::Mainnet => "mainnet",
        crate::state::Network::Testnet => "testnet",
        crate::state::Network::Devnet => "devnet",
    };
    
    // Spawn the node process
    let rpc_addr = format!("127.0.0.1:{}", rpc_port);
    let child = Command::new(&binary_path)
        .arg("--network")
        .arg(network_arg)
        .arg("--rpc")
        .arg("--rpc-addr")
        .arg(&rpc_addr)
        .arg("--datadir")
        .arg(&data_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start node: {}. Make sure pyrax-node is in PATH or current directory.", e))?;
    
    let pid = child.id();
    info!("Node started with PID: {}", pid);
    
    // Update state
    {
        let mut app_state = state.lock();
        app_state.node_running = true;
        app_state.node_process = Some(child);
        app_state.rpc_port = rpc_port;
    }
    
    // Wait for node to be ready
    let mut attempts = 0;
    while attempts < 30 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        if rpc.is_connected().await {
            info!("Node RPC is ready");
            break;
        }
        attempts += 1;
    }
    
    // Get initial status
    get_node_status(state).await
}

#[tauri::command]
pub async fn stop_node(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.lock();
    
    if !app_state.node_running {
        return Err("Node is not running".to_string());
    }
    
    // Kill the node process
    if let Some(mut child) = app_state.node_process.take() {
        info!("Stopping node process");
        
        #[cfg(target_os = "windows")]
        {
            // On Windows, use taskkill for graceful shutdown
            let _ = Command::new("taskkill")
                .args(["/PID", &child.id().to_string(), "/T"])
                .output();
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // On Unix, send SIGTERM
            let _ = child.kill();
        }
        
        // Wait for process to exit
        let _ = child.wait();
        info!("Node process stopped");
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
    
    // Query real status from node
    let rpc = RpcClient::localhost(rpc_port);
    
    let connected = rpc.is_connected().await;
    if !connected {
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
    }
    
    // Get chain info from node
    match rpc.get_chain_info().await {
        Ok(info) => {
            let syncing = info.syncing;
            let sync_progress = if syncing { 50.0 } else { 100.0 };
            
            Ok(NodeStatus {
                running: true,
                connected: true,
                syncing,
                sync_progress,
                peer_count: 0, // Peers not yet exposed via RPC
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
