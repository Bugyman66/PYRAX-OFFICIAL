use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{info, error};

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

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}
