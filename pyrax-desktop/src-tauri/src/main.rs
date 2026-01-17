#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod node;
mod wallet;
mod miner;
mod state;
mod rpc;

use state::AppState;
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::Manager;
use tracing::{info, warn};
use tracing_subscriber;

/// Check if Visual C++ Runtime is installed (Windows only)
#[cfg(target_os = "windows")]
fn check_and_install_vcruntime() {
    use std::process::Command;
    
    // Check registry for VC++ 2015-2022 Redistributable
    let check_x64 = Command::new("reg")
        .args(["query", r"HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64", "/v", "Installed"])
        .output();
    
    let x64_installed = match check_x64 {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("0x1")
        }
        Err(_) => false,
    };
    
    if x64_installed {
        info!("Visual C++ Runtime (x64) is installed");
        return;
    }
    
    warn!("Visual C++ Runtime not found, attempting to install...");
    
    // Try to find bundled VC++ redistributable in resources
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    
    if let Some(dir) = exe_dir {
        let vcredist_path = dir.join("resources").join("vc_redist.x64.exe");
        if vcredist_path.exists() {
            info!("Installing Visual C++ Runtime from bundled installer...");
            let result = Command::new(&vcredist_path)
                .args(["/install", "/passive", "/norestart"])
                .status();
            
            match result {
                Ok(status) => {
                    if status.success() || status.code() == Some(1638) || status.code() == Some(3010) {
                        info!("Visual C++ Runtime installed successfully");
                    } else {
                        warn!("Visual C++ Runtime installation returned code: {:?}", status.code());
                    }
                }
                Err(e) => warn!("Failed to run VC++ installer: {}", e),
            }
            return;
        }
    }
    
    // If bundled installer not found, show a message dialog
    warn!("VC++ redistributable not bundled. User may need to install manually.");
    
    // Try to open Microsoft download page
    if let Err(e) = open::that("https://aka.ms/vs/17/release/vc_redist.x64.exe") {
        warn!("Failed to open VC++ download page: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn check_and_install_vcruntime() {
    // No-op on non-Windows platforms
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting PYRAX Desktop v{}", env!("CARGO_PKG_VERSION"));
    
    // Check and install VC++ runtime if needed (Windows only)
    check_and_install_vcruntime();

    // Initialize application state
    let app_state = Arc::new(Mutex::new(AppState::new()));

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Node commands
            commands::node::start_node,
            commands::node::stop_node,
            commands::node::get_node_status,
            commands::node::get_chain_info,
            commands::node::get_peers,
            
            // Wallet commands
            commands::wallet::create_wallet,
            commands::wallet::unlock_wallet,
            commands::wallet::lock_wallet,
            commands::wallet::get_addresses,
            commands::wallet::create_address,
            commands::wallet::get_balance,
            commands::wallet::send_transaction,
            commands::wallet::get_transactions,
            commands::wallet::import_mnemonic,
            commands::wallet::export_mnemonic,
            
            // Miner commands
            commands::miner::start_miner,
            commands::miner::stop_miner,
            commands::miner::get_miner_status,
            commands::miner::get_hashrate,
            commands::miner::detect_gpus,
            commands::miner::benchmark_gpu,
            
            // Explorer commands
            commands::explorer::get_block,
            commands::explorer::get_transaction,
            commands::explorer::get_recent_blocks,
            commands::explorer::get_bootnode_info,
            
            // Settings commands
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_data_dir,
            commands::settings::set_data_dir,
            commands::settings::browse_directory,
            
            // Remote logging commands
            commands::node::get_remote_server_logs,
            commands::node::start_remote_log_stream,
            
            // Connection watchdog commands (self-healing)
            commands::node::start_connection_watchdog,
            commands::node::stop_connection_watchdog,
            
            // Data management commands
            commands::node::clear_local_data,
            commands::node::get_local_data_size,
            
            // Updater commands
            commands::updater::check_for_updates,
            commands::updater::install_update,
            commands::updater::get_app_version,
        ])
        .setup(|app| {
            info!("Application setup complete");
            
            // Get data directory
            let app_handle = app.handle();
            let data_dir = app_handle.path_resolver()
                .app_data_dir()
                .expect("Failed to get app data directory");
            
            info!("Data directory: {:?}", data_dir);
            
            // Create data directory if it doesn't exist
            std::fs::create_dir_all(&data_dir).ok();
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running PYRAX Desktop");
}
