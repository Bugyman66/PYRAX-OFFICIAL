use crate::state::{AppState, Settings, Network, Theme};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
use tauri::State;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub network: String,
    pub auto_start_node: bool,
    pub auto_start_miner: bool,
    pub miner_address: Option<String>,
    pub miner_threads: u32,
    pub cuda_device: i32,
    pub opencl_device: i32,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub max_peers: u32,
    pub theme: String,
    pub data_dir: Option<String>,
}

impl From<&AppState> for AppSettings {
    fn from(state: &AppState) -> Self {
        Self {
            network: state.network.to_string(),
            auto_start_node: state.settings.auto_start_node,
            auto_start_miner: state.settings.auto_start_miner,
            miner_address: state.settings.miner_address.clone(),
            miner_threads: state.settings.miner_threads,
            cuda_device: state.settings.cuda_device,
            opencl_device: state.settings.opencl_device,
            rpc_port: state.settings.rpc_port,
            p2p_port: state.settings.p2p_port,
            max_peers: state.settings.max_peers,
            theme: match state.settings.theme {
                Theme::Light => "light".to_string(),
                Theme::Dark => "dark".to_string(),
                Theme::System => "system".to_string(),
            },
            data_dir: Some(state.data_dir.to_string_lossy().to_string()),
        }
    }
}

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<AppSettings, String> {
    let app_state = state.lock();
    Ok(AppSettings::from(&*app_state))
}

#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.lock();
    
    app_state.network = match settings.network.as_str() {
        "mainnet" => Network::Mainnet,
        "testnet" => Network::Testnet,
        "devnet" => Network::Devnet,
        _ => return Err("Invalid network".to_string()),
    };
    
    app_state.settings = Settings {
        auto_start_node: settings.auto_start_node,
        auto_start_miner: settings.auto_start_miner,
        miner_address: settings.miner_address,
        miner_threads: settings.miner_threads,
        cuda_device: settings.cuda_device,
        opencl_device: settings.opencl_device,
        rpc_port: settings.rpc_port,
        p2p_port: settings.p2p_port,
        max_peers: settings.max_peers,
        theme: match settings.theme.as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        },
    };
    
    if let Some(dir) = settings.data_dir {
        app_state.data_dir = PathBuf::from(dir);
    }
    
    // Persist settings to file
    let settings_path = app_state.data_dir.join("settings.json");
    let settings_to_save = AppSettings::from(&*app_state);
    
    // Ensure data directory exists
    if let Err(e) = fs::create_dir_all(&app_state.data_dir) {
        warn!("Failed to create data directory: {}", e);
    }
    
    match serde_json::to_string_pretty(&settings_to_save) {
        Ok(json) => {
            match fs::File::create(&settings_path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(json.as_bytes()) {
                        error!("Failed to write settings file: {}", e);
                        return Err(format!("Failed to save settings: {}", e));
                    }
                    info!("Settings saved to {:?}", settings_path);
                }
                Err(e) => {
                    error!("Failed to create settings file: {}", e);
                    return Err(format!("Failed to create settings file: {}", e));
                }
            }
        }
        Err(e) => {
            error!("Failed to serialize settings: {}", e);
            return Err(format!("Failed to serialize settings: {}", e));
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn get_data_dir(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let app_state = state.lock();
    
    Ok(app_state.data_dir.to_string_lossy().to_string())
}
