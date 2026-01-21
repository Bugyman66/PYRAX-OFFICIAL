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
use commands::neurax::NeuraxState;
use commands::neurax_commands::NeuraxStateWrapper;
use commands::neurax_email::{NeuraxErrorBuffer, NeuraxErrorBufferWrapper};
use commands::neurax_llm::{LlmState, LlmStateWrapper};

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
    
    // Initialize NEURAX AI state
    let neurax_state = Arc::new(NeuraxState::new());
    
    // Initialize NEURAX error buffer for Brevo email reporting
    let neurax_error_buffer = Arc::new(NeuraxErrorBuffer::new());
    
    // Initialize NEURAX LLM state for local AI inference
    let neurax_llm_state = Arc::new(LlmState::new());

    tauri::Builder::default()
        .manage(app_state)
        .manage(NeuraxStateWrapper(neurax_state.clone()))
        .manage(NeuraxErrorBufferWrapper(neurax_error_buffer.clone()))
        .manage(LlmStateWrapper(neurax_llm_state.clone()))
        .invoke_handler(tauri::generate_handler![
            // Node commands
            commands::node::start_node,
            commands::node::stop_node,
            commands::node::get_node_status,
            commands::node::get_chain_info,
            commands::node::get_peers,
            commands::node::get_network_mesh,
            commands::node::measure_bootnode_latency,
            
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
            commands::settings::check_firewall_status,
            commands::settings::configure_firewall,
            commands::settings::remove_firewall_rules,
            commands::settings::wipe_chain_data,
            commands::settings::get_chain_data_size,
            commands::settings::test_port_connectivity,
            commands::settings::get_network_diagnostics,
            
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
            commands::updater::check_version_compatibility,
            
            // NEURAX AI commands
            commands::neurax_commands::neurax_get_config,
            commands::neurax_commands::neurax_set_config,
            commands::neurax_commands::neurax_set_enabled,
            commands::neurax_commands::neurax_set_permissions,
            commands::neurax_commands::neurax_get_system_metrics,
            commands::neurax_commands::neurax_get_metrics_history,
            commands::neurax_commands::neurax_analyze_logs,
            commands::neurax_commands::neurax_generate_insights,
            commands::neurax_commands::neurax_get_insights,
            commands::neurax_commands::neurax_dismiss_insight,
            commands::neurax_commands::neurax_chat,
            commands::neurax_commands::neurax_get_chat_history,
            commands::neurax_commands::neurax_clear_chat,
            commands::neurax_commands::neurax_execute_action,
            commands::neurax_commands::neurax_get_quick_insights,
            
            // NEURAX Brevo email commands
            commands::neurax_email::neurax_get_brevo_config,
            commands::neurax_email::neurax_set_brevo_config,
            commands::neurax_email::neurax_get_error_stats,
            commands::neurax_email::neurax_test_email,
            
            // NEURAX LLM commands
            commands::neurax_llm::neurax_get_llm_config,
            commands::neurax_llm::neurax_set_llm_config,
            commands::neurax_llm::neurax_detect_gpus,
            commands::neurax_llm::neurax_get_available_models,
            commands::neurax_llm::neurax_download_model,
            commands::neurax_llm::neurax_get_download_status,
            commands::neurax_llm::neurax_is_model_loaded,
            
            // NEURAX Admin privilege commands
            commands::neurax_admin::neurax_check_admin,
            commands::neurax_admin::neurax_request_elevation,
            commands::neurax_admin::neurax_get_permission_info,
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
            
            // Load NEURAX config
            let neurax = app.state::<NeuraxStateWrapper>();
            neurax.0.load_config(&data_dir);
            info!("NEURAX AI system initialized");
            
            // Load Brevo config and start error reporter
            let error_buffer = app.state::<NeuraxErrorBufferWrapper>();
            let brevo_config_path = data_dir.join("neurax_brevo_config.json");
            if brevo_config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&brevo_config_path) {
                    if let Ok(mut config) = serde_json::from_str::<commands::neurax_email::BrevoConfig>(&content) {
                        // API key should come from environment variable
                        config.api_key = std::env::var("BREVO_API_KEY").ok();
                        error_buffer.0.set_config(config);
                    }
                }
            }
            
            // Start background error reporter (sends every 5 minutes if enabled)
            let buffer_clone = error_buffer.0.clone();
            commands::neurax_email::start_error_reporter(
                buffer_clone,
                || format!("PYRAX Desktop v{}", env!("CARGO_PKG_VERSION"))
            );
            info!("NEURAX error reporter initialized");
            
            // Load LLM config
            let llm_state = app.state::<LlmStateWrapper>();
            llm_state.0.load_config(&data_dir);
            info!("NEURAX LLM system initialized");
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running PYRAX Desktop");
}
