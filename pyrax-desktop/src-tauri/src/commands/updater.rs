use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{AppHandle, Manager};
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app.package_info().version.to_string();
    info!("Checking for updates... Current version: {}", current_version);
    
    // Emit log event
    let _ = app.emit_all("node-log", serde_json::json!({
        "level": "info",
        "category": "system",
        "message": format!("Checking for updates (current: v{})", current_version)
    }));
    
    match app.updater().check().await {
        Ok(update) => {
            if update.is_update_available() {
                let latest = update.latest_version().to_string();
                let notes = update.body().map(|s| s.to_string());
                
                info!("Update available: {} -> {}", current_version, latest);
                
                let _ = app.emit_all("node-log", serde_json::json!({
                    "level": "info",
                    "category": "system",
                    "message": format!("Update available: v{} -> v{}", current_version, latest)
                }));
                
                Ok(UpdateInfo {
                    available: true,
                    current_version,
                    latest_version: Some(latest),
                    release_notes: notes,
                    download_url: None,
                })
            } else {
                info!("No updates available");
                Ok(UpdateInfo {
                    available: false,
                    current_version,
                    latest_version: None,
                    release_notes: None,
                    download_url: None,
                })
            }
        }
        Err(e) => {
            error!("Failed to check for updates: {}", e);
            // Return current version info even on error
            Ok(UpdateInfo {
                available: false,
                current_version,
                latest_version: None,
                release_notes: None,
                download_url: None,
            })
        }
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    info!("Installing update...");
    
    let _ = app.emit_all("node-log", serde_json::json!({
        "level": "info",
        "category": "system",
        "message": "Downloading and installing update..."
    }));
    
    match app.updater().check().await {
        Ok(update) => {
            if update.is_update_available() {
                // CRITICAL: Wipe all blockchain data before installing update
                // This prevents stale/orphan block issues when users update
                // The node will sync fresh after the update
                wipe_all_blockchain_data(&app);
                
                match update.download_and_install().await {
                    Ok(_) => {
                        info!("Update installed successfully, restarting...");
                        let _ = app.emit_all("node-log", serde_json::json!({
                            "level": "info",
                            "category": "system",
                            "message": "Update installed! Restarting application..."
                        }));
                        // The app will restart automatically after install
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to install update: {}", e);
                        Err(format!("Failed to install update: {}", e))
                    }
                }
            } else {
                Err("No update available".to_string())
            }
        }
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

/// Wipe all blockchain data for all networks before update
/// This is a critical stop-gap to prevent stale/orphan block issues
/// when users update from older versions with incompatible chain data
fn wipe_all_blockchain_data(app: &AppHandle) {
    info!("CRITICAL: Wiping all blockchain data before update to prevent orphan block issues");
    
    let _ = app.emit_all("node-log", serde_json::json!({
        "level": "warn",
        "category": "system",
        "message": "Clearing all blockchain data before update (required for network compatibility)..."
    }));
    
    // Get the app data directory
    let app_data_dir = match app.path_resolver().app_data_dir() {
        Some(dir) => dir,
        None => {
            warn!("Could not get app data directory for cleanup");
            return;
        }
    };
    
    // Networks to clear
    let networks = ["testnet", "devnet", "mainnet"];
    
    for network in networks {
        let data_path = app_data_dir.join("data").join(network);
        
        if data_path.exists() {
            match fs::remove_dir_all(&data_path) {
                Ok(_) => {
                    info!("Cleared {} blockchain data at {:?}", network, data_path);
                    let _ = app.emit_all("node-log", serde_json::json!({
                        "level": "info",
                        "category": "system",
                        "message": format!("Cleared {} blockchain data", network)
                    }));
                }
                Err(e) => {
                    warn!("Failed to clear {} data: {} (will be overwritten on sync)", network, e);
                }
            }
        }
    }
    
    // Also clear any peer/node data that might have stale addresses
    let node_data_path = app_data_dir.join("node_data");
    if node_data_path.exists() {
        if let Err(e) = fs::remove_dir_all(&node_data_path) {
            warn!("Failed to clear node_data: {}", e);
        } else {
            info!("Cleared node_data directory");
        }
    }
    
    info!("Blockchain data wipe complete - node will sync fresh after update");
    let _ = app.emit_all("node-log", serde_json::json!({
        "level": "info",
        "category": "system",
        "message": "Data cleanup complete. Node will sync fresh after update."
    }));
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Minimum required version for network participation
/// This MUST match MIN_REQUIRED_VERSION in pyrax-node/src/p2p/mod.rs
const MIN_REQUIRED_VERSION: (u32, u32, u32) = (0, 2, 0);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionMismatch {
    pub detected: bool,
    pub current_version: String,
    pub minimum_required: String,
    pub message: String,
}

/// Check if the current app version meets the minimum network requirement
#[tauri::command]
pub fn check_version_compatibility(app: AppHandle) -> Result<VersionMismatch, String> {
    let current = app.package_info().version.clone();
    let (min_major, min_minor, min_patch) = MIN_REQUIRED_VERSION;
    
    // Parse current version
    let current_major = current.major as u32;
    let current_minor = current.minor as u32;
    let current_patch = current.patch as u32;
    
    // Check if current version meets minimum
    let meets_minimum = if current_major > min_major {
        true
    } else if current_major < min_major {
        false
    } else if current_minor > min_minor {
        true
    } else if current_minor < min_minor {
        false
    } else {
        current_patch >= min_patch
    };
    
    let current_str = format!("{}.{}.{}", current_major, current_minor, current_patch);
    let min_str = format!("{}.{}.{}", min_major, min_minor, min_patch);
    
    if meets_minimum {
        Ok(VersionMismatch {
            detected: false,
            current_version: current_str,
            minimum_required: min_str,
            message: "Version is compatible".to_string(),
        })
    } else {
        info!("VERSION MISMATCH: Current {} < Required {}", current_str, min_str);
        Ok(VersionMismatch {
            detected: true,
            current_version: current_str,
            minimum_required: min_str.clone(),
            message: format!("Your app version is outdated. Please update to v{} or later.", min_str),
        })
    }
}
