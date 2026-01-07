use crate::state::AppState;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

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
    let app_state = state.lock();
    
    if !app_state.node_running {
        return Err("Node is not running".to_string());
    }
    
    // TODO: Get real block from node RPC
    Ok(BlockInfo {
        hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        height: 0,
        parent_hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        timestamp: 1704067200,
        difficulty: "1000000".to_string(),
        nonce: "0x0".to_string(),
        merkle_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        state_root: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421".to_string(),
        beneficiary: "0x0000000000000000000000000000000000000000".to_string(),
        transaction_count: 0,
        size: 0,
    })
}

#[tauri::command]
pub async fn get_transaction(
    hash: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<TransactionInfo, String> {
    let app_state = state.lock();
    
    if !app_state.node_running {
        return Err("Node is not running".to_string());
    }
    
    // TODO: Get real transaction from node RPC
    Err("Transaction not found".to_string())
}

#[tauri::command]
pub async fn get_recent_blocks(
    count: Option<u32>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<BlockInfo>, String> {
    let app_state = state.lock();
    
    if !app_state.node_running {
        return Err("Node is not running".to_string());
    }
    
    let _count = count.unwrap_or(10);
    
    // TODO: Get real recent blocks from node RPC
    Ok(vec![BlockInfo {
        hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        height: 0,
        parent_hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        timestamp: 1704067200,
        difficulty: "1000000".to_string(),
        nonce: "0x0".to_string(),
        merkle_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        state_root: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421".to_string(),
        beneficiary: "0x0000000000000000000000000000000000000000".to_string(),
        transaction_count: 0,
        size: 0,
    }])
}
