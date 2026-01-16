use crate::state::AppState;
use crate::rpc::RpcClient;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub hash: String,
    pub height: u64,
    pub parent_hash: String,
    pub timestamp: u64,
    pub difficulty: String,
    pub nonce: String,
    pub merkle_root: String,
    pub state_root: String,
    pub beneficiary: String,
    pub transaction_count: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub tx_type: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas_price: String,
    pub gas_limit: String,
    pub gas_used: Option<String>,
    pub nonce: u64,
    pub data: String,
    pub block_hash: Option<String>,
    pub block_number: Option<u64>,
    pub transaction_index: Option<usize>,
    pub status: Option<String>,
}

#[tauri::command]
pub async fn get_block(
    identifier: String, // hash or height
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<BlockInfo, String> {
    let (running, rpc_port) = {
        let app_state = state.lock();
        (app_state.node_running, app_state.rpc_port)
    };
    
    if !running {
        return Err("Node is not running".to_string());
    }
    
    let rpc = RpcClient::localhost(rpc_port);
    
    // Determine if identifier is a hash or height
    let block = if identifier.starts_with("0x") {
        // It's a hash
        rpc.get_block_by_hash(&identifier, false).await
            .map_err(|e| format!("Failed to get block: {}", e))?
    } else {
        // It's a height
        let height: u64 = identifier.parse()
            .map_err(|_| "Invalid block identifier".to_string())?;
        rpc.get_block_by_number(height, false).await
            .map_err(|e| format!("Failed to get block: {}", e))?
    };
    
    match block {
        Some(b) => {
            let height = u64::from_str_radix(b.number.trim_start_matches("0x"), 16).unwrap_or(0);
            let timestamp = u64::from_str_radix(b.timestamp.trim_start_matches("0x"), 16).unwrap_or(0);
            Ok(BlockInfo {
                hash: b.hash,
                height,
                parent_hash: b.parent_hash,
                timestamp,
                difficulty: b.difficulty,
                nonce: "0x0".to_string(),
                merkle_root: "0x0".to_string(),
                state_root: "0x0".to_string(),
                beneficiary: b.miner,
                transaction_count: b.transaction_count as usize,
                size: u64::from_str_radix(b.size.trim_start_matches("0x"), 16).unwrap_or(0) as usize,
            })
        }
        None => Err("Block not found".to_string()),
    }
}

#[tauri::command]
pub async fn get_transaction(
    hash: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<TransactionInfo, String> {
    let (running, rpc_port) = {
        let app_state = state.lock();
        (app_state.node_running, app_state.rpc_port)
    };
    
    if !running {
        return Err("Node is not running".to_string());
    }
    
    let rpc = RpcClient::localhost(rpc_port);
    
    match rpc.get_transaction(&hash).await {
        Ok(Some(tx)) => {
            let block_number = tx.block_number.as_ref()
                .and_then(|n| u64::from_str_radix(n.trim_start_matches("0x"), 16).ok());
            let tx_index = tx.transaction_index.as_ref()
                .and_then(|n| u64::from_str_radix(n.trim_start_matches("0x"), 16).ok().map(|v| v as usize));
            
            Ok(TransactionInfo {
                hash: tx.hash,
                tx_type: "transfer".to_string(),
                from: tx.from,
                to: tx.to,
                value: tx.value,
                gas_price: tx.gas_price,
                gas_limit: tx.gas,
                gas_used: None,
                nonce: u64::from_str_radix(tx.nonce.trim_start_matches("0x"), 16).unwrap_or(0),
                data: tx.input,
                block_hash: tx.block_hash,
                block_number,
                transaction_index: tx_index,
                status: Some("confirmed".to_string()),
            })
        }
        Ok(None) => Err("Transaction not found".to_string()),
        Err(e) => Err(format!("Failed to get transaction: {}", e)),
    }
}

#[tauri::command]
pub async fn get_recent_blocks(
    count: Option<u32>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<BlockInfo>, String> {
    let (running, rpc_port) = {
        let app_state = state.lock();
        (app_state.node_running, app_state.rpc_port)
    };
    
    if !running {
        return Err("Node is not running".to_string());
    }
    
    let block_count = count.unwrap_or(10).min(50) as u64; // Cap at 50 blocks
    let rpc = RpcClient::localhost(rpc_port);
    
    // Get current chain height
    let chain_info = rpc.get_chain_info().await
        .map_err(|e| format!("Failed to get chain info: {}", e))?;
    
    let current_height = chain_info.best_block_height;
    let mut blocks = Vec::new();
    
    // Fetch recent blocks from current height down
    for i in 0..block_count {
        let height = current_height.saturating_sub(i);
        if let Ok(Some(b)) = rpc.get_block_by_number(height, false).await {
            let block_height = u64::from_str_radix(b.number.trim_start_matches("0x"), 16).unwrap_or(height);
            let timestamp = u64::from_str_radix(b.timestamp.trim_start_matches("0x"), 16).unwrap_or(0);
            blocks.push(BlockInfo {
                hash: b.hash,
                height: block_height,
                parent_hash: b.parent_hash,
                timestamp,
                difficulty: b.difficulty,
                nonce: "0x0".to_string(),
                merkle_root: "0x0".to_string(),
                state_root: "0x0".to_string(),
                beneficiary: b.miner,
                transaction_count: b.transaction_count as usize,
                size: u64::from_str_radix(b.size.trim_start_matches("0x"), 16).unwrap_or(0) as usize,
            });
        }
        
        if height == 0 {
            break; // Reached genesis
        }
    }
    
    Ok(blocks)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootnodeInfo {
    pub id: String,
    pub url: String,
    #[serde(rename = "latencyMs")]
    pub latency_ms: u64,
    pub city: String,
    pub region: String,
    pub country: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    pub stream: String,
    pub online: bool,
}

// Bootnode configurations with location info
fn get_bootnode_configs() -> Vec<(String, String, String, String, String, String, String)> {
    vec![
        // (id, url, city, region, country, country_code, stream)
        ("node-us-east-1".to_string(), "https://rpc.pyrax-devnet.org".to_string(), "New York".to_string(), "NY".to_string(), "United States".to_string(), "US".to_string(), "A".to_string()),
        ("node-us-west-1".to_string(), "https://rpc2.pyrax-devnet.org".to_string(), "San Francisco".to_string(), "CA".to_string(), "United States".to_string(), "US".to_string(), "A".to_string()),
        ("node-eu-west-1".to_string(), "https://rpc-eu.pyrax-devnet.org".to_string(), "Frankfurt".to_string(), "HE".to_string(), "Germany".to_string(), "DE".to_string(), "B".to_string()),
        ("node-asia-1".to_string(), "https://rpc-asia.pyrax-devnet.org".to_string(), "Singapore".to_string(), "".to_string(), "Singapore".to_string(), "SG".to_string(), "B".to_string()),
        ("validator-1".to_string(), "https://validator.pyrax-devnet.org".to_string(), "London".to_string(), "".to_string(), "United Kingdom".to_string(), "GB".to_string(), "C".to_string()),
    ]
}

#[tauri::command]
pub async fn get_bootnode_info(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<BootnodeInfo>, String> {
    let running = {
        let app_state = state.lock();
        app_state.node_running
    };
    
    if !running {
        return Err("Node is not running".to_string());
    }
    
    let configs = get_bootnode_configs();
    let mut bootnodes = Vec::new();
    
    for (id, url, city, region, country, country_code, stream) in configs {
        // Measure latency by pinging the RPC endpoint
        let start = Instant::now();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();
        
        let (online, latency_ms) = match client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(r#"{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}"#)
            .send()
            .await
        {
            Ok(response) => {
                let latency = start.elapsed().as_millis() as u64;
                (response.status().is_success(), latency)
            }
            Err(_) => (false, 0),
        };
        
        bootnodes.push(BootnodeInfo {
            id,
            url,
            latency_ms,
            city,
            region,
            country,
            country_code,
            stream,
            online,
        });
    }
    
    Ok(bootnodes)
}
