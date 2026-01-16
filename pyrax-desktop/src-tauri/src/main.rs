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
use tracing::info;
use tracing_subscriber;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting PYRAX Desktop v{}", env!("CARGO_PKG_VERSION"));

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
