use crate::state::AppState;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStatus {
    pub running: bool,
    pub hashrate: f64,
    pub hashrate_unit: String,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub blocks_found: u64,
    pub temperature: Option<u32>,
    pub fan_speed: Option<u32>,
    pub power_usage: Option<u32>,
    pub device_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinerConfig {
    pub address: String,
    pub threads: Option<u32>,
    pub cuda_device: Option<i32>,
    pub opencl_device: Option<i32>,
    pub pool_url: Option<String>,
}

#[tauri::command]
pub async fn start_miner(
    config: MinerConfig,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<MinerStatus, String> {
    let mut app_state = state.lock();
    
    if app_state.miner_running {
        return Err("Miner is already running".to_string());
    }
    
    if !app_state.node_running && config.pool_url.is_none() {
        return Err("Node must be running for solo mining, or provide pool URL".to_string());
    }
    
    // Validate address
    if config.address.is_empty() {
        return Err("Miner address is required".to_string());
    }
    
    // TODO: Actually start miner process
    app_state.miner_running = true;
    app_state.settings.miner_address = Some(config.address);
    
    Ok(MinerStatus {
        running: true,
        hashrate: 0.0,
        hashrate_unit: "MH/s".to_string(),
        accepted_shares: 0,
        rejected_shares: 0,
        blocks_found: 0,
        temperature: None,
        fan_speed: None,
        power_usage: None,
        device_name: "Initializing...".to_string(),
    })
}

#[tauri::command]
pub async fn stop_miner(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.lock();
    
    if !app_state.miner_running {
        return Err("Miner is not running".to_string());
    }
    
    // TODO: Actually stop miner process
    app_state.miner_running = false;
    
    Ok(())
}

#[tauri::command]
pub async fn get_miner_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<MinerStatus, String> {
    let app_state = state.lock();
    
    if !app_state.miner_running {
        return Ok(MinerStatus {
            running: false,
            hashrate: 0.0,
            hashrate_unit: "MH/s".to_string(),
            accepted_shares: 0,
            rejected_shares: 0,
            blocks_found: 0,
            temperature: None,
            fan_speed: None,
            power_usage: None,
            device_name: "Not mining".to_string(),
        });
    }
    
    // TODO: Get real miner status
    Ok(MinerStatus {
        running: true,
        hashrate: 25.5,
        hashrate_unit: "MH/s".to_string(),
        accepted_shares: 100,
        rejected_shares: 2,
        blocks_found: 0,
        temperature: Some(65),
        fan_speed: Some(70),
        power_usage: Some(120),
        device_name: "GPU 0".to_string(),
    })
}

#[tauri::command]
pub async fn get_hashrate(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<f64, String> {
    let app_state = state.lock();
    
    if !app_state.miner_running {
        return Ok(0.0);
    }
    
    // TODO: Get real hashrate
    Ok(25.5)
}
