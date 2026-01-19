use crate::state::{AppState, Settings, Network, Theme, ConnectionMode, PortPreset};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
use std::process::Command;
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
    /// Log verbosity: 0=error, 1=warn, 2=info, 3=debug, 4=trace
    pub log_verbosity: Option<u8>,
    
    // === MASS ADOPTION NETWORK SETTINGS ===
    /// Connection mode: "full", "relay", "auto"
    #[serde(default)]
    pub connection_mode: Option<String>,
    /// Port preset: "standard", "https", "althttp", "althttps", "custom"
    #[serde(default)]
    pub port_preset: Option<String>,
    /// Enable WebSocket transport
    #[serde(default)]
    pub enable_websocket: Option<bool>,
    /// Enable automatic port fallback
    #[serde(default)]
    pub auto_port_fallback: Option<bool>,
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
            log_verbosity: Some(state.settings.log_verbosity),
            // Mass adoption network settings
            connection_mode: Some(state.settings.connection_mode.to_string()),
            port_preset: Some(state.settings.port_preset.to_string()),
            enable_websocket: Some(state.settings.enable_websocket),
            auto_port_fallback: Some(state.settings.auto_port_fallback),
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
        log_verbosity: settings.log_verbosity.unwrap_or(3),
        // Mass adoption network settings
        connection_mode: match settings.connection_mode.as_deref() {
            Some("full") => ConnectionMode::FullNode,
            Some("relay") => ConnectionMode::RelayOnly,
            _ => ConnectionMode::Auto,
        },
        port_preset: match settings.port_preset.as_deref() {
            Some("https") => PortPreset::Https,
            Some("althttp") => PortPreset::AltHttp,
            Some("althttps") => PortPreset::AltHttps,
            Some("custom") => PortPreset::Custom,
            _ => PortPreset::Standard,
        },
        enable_websocket: settings.enable_websocket.unwrap_or(true),
        auto_port_fallback: settings.auto_port_fallback.unwrap_or(true),
        detected_nat_type: None,
        last_successful_transport: None,
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

#[tauri::command]
pub async fn set_data_dir(
    path: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    
    // Validate path exists or can be created
    if !path_buf.exists() {
        fs::create_dir_all(&path_buf)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    // Validate path is a directory
    if !path_buf.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    let mut app_state = state.lock();
    app_state.data_dir = path_buf;
    
    info!("Data directory set to: {}", path);
    Ok(())
}

#[tauri::command]
pub async fn browse_directory(
    app: tauri::AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Option<String>, String> {
    use tauri::api::dialog::FileDialogBuilder;
    use std::sync::mpsc;
    
    let current_dir = {
        let app_state = state.lock();
        app_state.data_dir.clone()
    };
    
    let (tx, rx) = mpsc::channel();
    
    FileDialogBuilder::new()
        .set_title("Select Chain Data Directory")
        .set_directory(&current_dir)
        .pick_folder(move |path| {
            let _ = tx.send(path);
        });
    
    match rx.recv() {
        Ok(Some(path)) => {
            let path_str = path.to_string_lossy().to_string();
            
            // Update state with new path
            let mut app_state = state.lock();
            app_state.data_dir = path.clone();
            
            info!("User selected data directory: {}", path_str);
            Ok(Some(path_str))
        }
        Ok(None) => {
            // User cancelled
            Ok(None)
        }
        Err(e) => {
            error!("Failed to receive dialog result: {}", e);
            Err(format!("Dialog error: {}", e))
        }
    }
}

/// Check if Windows Firewall rule for Inferno Node P2P exists
#[tauri::command]
pub async fn check_firewall_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<FirewallStatus, String> {
    let p2p_port = {
        let app_state = state.lock();
        app_state.settings.p2p_port
    };
    
    #[cfg(target_os = "windows")]
    {
        // Check if firewall rule exists
        let output = Command::new("netsh")
            .args(["advfirewall", "firewall", "show", "rule", "name=Inferno Node P2P"])
            .output();
        
        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let rule_exists = stdout.contains("Inferno Node P2P");
                
                Ok(FirewallStatus {
                    rule_exists,
                    port: p2p_port,
                    requires_admin: true,
                    platform: "windows".to_string(),
                })
            }
            Err(e) => {
                warn!("Failed to check firewall status: {}", e);
                Ok(FirewallStatus {
                    rule_exists: false,
                    port: p2p_port,
                    requires_admin: true,
                    platform: "windows".to_string(),
                })
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // Check macOS Application Firewall status
        // socketfilterfw --getglobalstate shows if firewall is on
        let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
            .args(["--getglobalstate"])
            .output();
        
        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let firewall_enabled = stdout.contains("enabled");
                
                // If firewall is disabled, no rules needed
                if !firewall_enabled {
                    return Ok(FirewallStatus {
                        rule_exists: true,
                        port: p2p_port,
                        requires_admin: false,
                        platform: "macos".to_string(),
                    });
                }
                
                // Check if our app is allowed
                // For now, report that configuration might be needed
                Ok(FirewallStatus {
                    rule_exists: false, // User should check
                    port: p2p_port,
                    requires_admin: true,
                    platform: "macos".to_string(),
                })
            }
            Err(_) => {
                // Can't check, assume OK
                Ok(FirewallStatus {
                    rule_exists: true,
                    port: p2p_port,
                    requires_admin: false,
                    platform: "macos".to_string(),
                })
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Check for UFW (Ubuntu/Debian) or firewalld (Fedora/RHEL)
        let ufw_check = Command::new("ufw")
            .args(["status"])
            .output();
        
        if let Ok(result) = ufw_check {
            let stdout = String::from_utf8_lossy(&result.stdout);
            if stdout.contains("Status: active") {
                // UFW is active, check if port is allowed
                let port_allowed = stdout.contains(&format!("{}", p2p_port)) || 
                                   stdout.contains(&format!("{}/tcp", p2p_port));
                return Ok(FirewallStatus {
                    rule_exists: port_allowed,
                    port: p2p_port,
                    requires_admin: true,
                    platform: "linux-ufw".to_string(),
                });
            }
        }
        
        // Check for firewalld
        let firewalld_check = Command::new("firewall-cmd")
            .args(["--state"])
            .output();
        
        if let Ok(result) = firewalld_check {
            let stdout = String::from_utf8_lossy(&result.stdout);
            if stdout.contains("running") {
                // Check if port is open
                let port_check = Command::new("firewall-cmd")
                    .args(["--query-port", &format!("{}/tcp", p2p_port)])
                    .output();
                
                let port_allowed = port_check.map(|r| r.status.success()).unwrap_or(false);
                return Ok(FirewallStatus {
                    rule_exists: port_allowed,
                    port: p2p_port,
                    requires_admin: true,
                    platform: "linux-firewalld".to_string(),
                });
            }
        }
        
        // No firewall detected or inactive
        Ok(FirewallStatus {
            rule_exists: true,
            port: p2p_port,
            requires_admin: false,
            platform: "linux".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallStatus {
    pub rule_exists: bool,
    pub port: u16,
    pub requires_admin: bool,
    pub platform: String,
}

/// Configure Windows Firewall to allow Inferno Node P2P connections
/// This requires administrator privileges - uses PowerShell with elevation
#[tauri::command]
pub async fn configure_firewall(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<FirewallResult, String> {
    let p2p_port = {
        let app_state = state.lock();
        app_state.settings.p2p_port
    };
    
    #[cfg(target_os = "windows")]
    {
        info!("Configuring Windows Firewall for P2P port {}", p2p_port);
        
        // Build PowerShell commands to add firewall rules
        // We use Start-Process with -Verb RunAs to request elevation (UAC prompt)
        let tcp_rule = format!(
            "netsh advfirewall firewall add rule name='Inferno Node P2P' dir=in action=allow protocol=TCP localport={} profile=any description='Allow inbound P2P connections for Inferno Node blockchain sync'",
            p2p_port
        );
        let udp_rule = format!(
            "netsh advfirewall firewall add rule name='Inferno Node P2P UDP' dir=in action=allow protocol=UDP localport={} profile=any description='Allow UDP for P2P hole-punching'",
            p2p_port
        );
        
        // Combine commands into a single script
        let combined_script = format!(
            "{} ; {}",
            tcp_rule, udp_rule
        );
        
        // Use PowerShell Start-Process with -Verb RunAs to elevate
        // -Wait ensures we wait for completion, -WindowStyle Hidden hides the cmd window
        let ps_command = format!(
            "Start-Process -FilePath 'cmd.exe' -ArgumentList '/c {}' -Verb RunAs -Wait -WindowStyle Hidden",
            combined_script.replace("'", "''") // Escape single quotes for PowerShell
        );
        
        info!("Running elevated firewall command via PowerShell");
        
        let result = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_command])
            .output();
        
        match result {
            Ok(output) => {
                // Check if the rules now exist (since elevated process output is separate)
                std::thread::sleep(std::time::Duration::from_millis(500)); // Brief wait for rule to apply
                
                let check = Command::new("netsh")
                    .args(["advfirewall", "firewall", "show", "rule", "name=Inferno Node P2P"])
                    .output();
                
                match check {
                    Ok(check_result) => {
                        let stdout = String::from_utf8_lossy(&check_result.stdout);
                        if stdout.contains("Inferno Node P2P") {
                            info!("Firewall rules configured successfully for port {}", p2p_port);
                            Ok(FirewallResult {
                                success: true,
                                message: format!("✓ Firewall configured for port {} (TCP + UDP)", p2p_port),
                                requires_restart: false,
                            })
                        } else {
                            // User may have clicked No on UAC prompt
                            warn!("Firewall rules not found after configuration attempt");
                            Err("Firewall configuration was cancelled or failed. Please click 'Yes' on the Administrator prompt.".to_string())
                        }
                    }
                    Err(e) => {
                        error!("Failed to verify firewall rules: {}", e);
                        Err(format!("Failed to verify firewall configuration: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to execute PowerShell: {}", e);
                Err(format!("Failed to configure firewall: {}. Please ensure PowerShell is available.", e))
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        info!("Configuring macOS firewall for P2P port {}", p2p_port);
        
        // On macOS, we need to add the app to the firewall's allowed list
        // This typically requires running with sudo/admin privileges
        // The user may also be prompted by macOS to allow incoming connections
        
        // Try to add the app to allowed apps using socketfilterfw
        // Note: This requires the full path to the app bundle
        let app_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        
        if !app_path.is_empty() {
            let result = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
                .args(["--add", &app_path])
                .output();
            
            match result {
                Ok(output) => {
                    if output.status.success() {
                        // Also unblock the app
                        let _ = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
                            .args(["--unblockapp", &app_path])
                            .output();
                        
                        return Ok(FirewallResult {
                            success: true,
                            message: format!("Added Inferno Node to macOS firewall allowed apps (port {})", p2p_port),
                            requires_restart: false,
                        });
                    }
                }
                Err(_) => {}
            }
        }
        
        // If automatic configuration failed, provide manual instructions
        Ok(FirewallResult {
            success: true,
            message: format!(
                "macOS Firewall: When prompted, click 'Allow' to enable incoming connections on port {}. \
                You can also go to System Settings → Network → Firewall → Options and add Inferno Node to allowed apps.",
                p2p_port
            ),
            requires_restart: false,
        })
    }
    
    #[cfg(target_os = "linux")]
    {
        info!("Configuring Linux firewall for P2P port {}", p2p_port);
        
        // Try UFW first (Ubuntu/Debian)
        let ufw_result = Command::new("sudo")
            .args(["ufw", "allow", &format!("{}/tcp", p2p_port)])
            .output();
        
        if let Ok(output) = ufw_result {
            if output.status.success() {
                // Also allow UDP for hole-punching
                let _ = Command::new("sudo")
                    .args(["ufw", "allow", &format!("{}/udp", p2p_port)])
                    .output();
                
                return Ok(FirewallResult {
                    success: true,
                    message: format!("UFW: Allowed port {} (TCP + UDP)", p2p_port),
                    requires_restart: false,
                });
            }
        }
        
        // Try firewalld (Fedora/RHEL)
        let firewalld_result = Command::new("sudo")
            .args(["firewall-cmd", "--permanent", "--add-port", &format!("{}/tcp", p2p_port)])
            .output();
        
        if let Ok(output) = firewalld_result {
            if output.status.success() {
                // Also allow UDP
                let _ = Command::new("sudo")
                    .args(["firewall-cmd", "--permanent", "--add-port", &format!("{}/udp", p2p_port)])
                    .output();
                // Reload firewall
                let _ = Command::new("sudo")
                    .args(["firewall-cmd", "--reload"])
                    .output();
                
                return Ok(FirewallResult {
                    success: true,
                    message: format!("firewalld: Allowed port {} (TCP + UDP)", p2p_port),
                    requires_restart: false,
                });
            }
        }
        
        // Provide manual instructions
        Ok(FirewallResult {
            success: true,
            message: format!(
                "Linux Firewall: Please run one of these commands to open port {}:\n\
                Ubuntu/Debian: sudo ufw allow {}/tcp && sudo ufw allow {}/udp\n\
                Fedora/RHEL: sudo firewall-cmd --permanent --add-port={}/tcp && sudo firewall-cmd --reload",
                p2p_port, p2p_port, p2p_port, p2p_port
            ),
            requires_restart: false,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallResult {
    pub success: bool,
    pub message: String,
    pub requires_restart: bool,
}

/// Wipe chain data to prevent stale data reintroduction
/// This removes all blockchain data from the local storage
/// User will need to resync from the network after this
#[tauri::command]
pub async fn wipe_chain_data(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ChainWipeResult, String> {
    let data_dir = {
        let app_state = state.lock();
        
        // Check if node is running - cannot wipe while running
        if app_state.node_running {
            return Err("Cannot wipe chain data while node is running. Please stop the node first.".to_string());
        }
        
        app_state.data_dir.clone()
    };
    
    info!("Wiping chain data from {:?}", data_dir);
    
    // Define the directories/files to wipe
    let chain_dir = data_dir.join("chain");
    let blocks_dir = data_dir.join("blocks");
    let state_dir = data_dir.join("state");
    let db_dir = data_dir.join("db");
    let rocksdb_dir = data_dir.join("rocksdb");
    
    let mut wiped_items = Vec::new();
    let mut errors = Vec::new();
    
    // Wipe each directory if it exists
    for dir in [&chain_dir, &blocks_dir, &state_dir, &db_dir, &rocksdb_dir] {
        if dir.exists() {
            match fs::remove_dir_all(dir) {
                Ok(_) => {
                    info!("Wiped directory: {:?}", dir);
                    wiped_items.push(dir.to_string_lossy().to_string());
                }
                Err(e) => {
                    warn!("Failed to wipe {:?}: {}", dir, e);
                    errors.push(format!("{:?}: {}", dir, e));
                }
            }
        }
    }
    
    // Also wipe any .db files in the data directory
    if let Ok(entries) = fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "db" || ext == "ldb" || ext == "sst" {
                        match fs::remove_file(&path) {
                            Ok(_) => {
                                info!("Wiped file: {:?}", path);
                                wiped_items.push(path.to_string_lossy().to_string());
                            }
                            Err(e) => {
                                warn!("Failed to wipe {:?}: {}", path, e);
                                errors.push(format!("{:?}: {}", path, e));
                            }
                        }
                    }
                }
            }
        }
    }
    
    if wiped_items.is_empty() && errors.is_empty() {
        Ok(ChainWipeResult {
            success: true,
            message: "No chain data found to wipe".to_string(),
            wiped_items: vec![],
            errors: vec![],
        })
    } else if errors.is_empty() {
        Ok(ChainWipeResult {
            success: true,
            message: format!("Successfully wiped {} items. Node will resync from network on next start.", wiped_items.len()),
            wiped_items,
            errors: vec![],
        })
    } else {
        Ok(ChainWipeResult {
            success: false,
            message: format!("Wiped {} items with {} errors", wiped_items.len(), errors.len()),
            wiped_items,
            errors,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainWipeResult {
    pub success: bool,
    pub message: String,
    pub wiped_items: Vec<String>,
    pub errors: Vec<String>,
}

/// Get chain data size for display
#[tauri::command]
pub async fn get_chain_data_size(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ChainDataInfo, String> {
    let data_dir = {
        let app_state = state.lock();
        app_state.data_dir.clone()
    };
    
    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;
    
    // Calculate size of chain-related directories
    let dirs_to_check = ["chain", "blocks", "state", "db", "rocksdb"];
    
    for dir_name in dirs_to_check {
        let dir_path = data_dir.join(dir_name);
        if dir_path.exists() {
            if let Ok(size) = calculate_dir_size(&dir_path) {
                total_size += size.0;
                file_count += size.1;
            }
        }
    }
    
    // Also count .db files in root data dir
    if let Ok(entries) = fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "db" || ext == "ldb" || ext == "sst" {
                        if let Ok(meta) = fs::metadata(&path) {
                            total_size += meta.len();
                            file_count += 1;
                        }
                    }
                }
            }
        }
    }
    
    Ok(ChainDataInfo {
        data_dir: data_dir.to_string_lossy().to_string(),
        total_size_bytes: total_size,
        total_size_human: format_size(total_size),
        file_count,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainDataInfo {
    pub data_dir: String,
    pub total_size_bytes: u64,
    pub total_size_human: String,
    pub file_count: u64,
}

fn calculate_dir_size(path: &PathBuf) -> Result<(u64, u64), std::io::Error> {
    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;
    
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let (size, count) = calculate_dir_size(&path)?;
                total_size += size;
                file_count += count;
            } else {
                total_size += fs::metadata(&path)?.len();
                file_count += 1;
            }
        }
    }
    
    Ok((total_size, file_count))
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Remove Inferno Node firewall rules
#[tauri::command]
pub async fn remove_firewall_rules() -> Result<FirewallResult, String> {
    #[cfg(target_os = "windows")]
    {
        info!("Removing Windows Firewall rules for Inferno Node");
        
        // Build commands to remove firewall rules
        let remove_script = "netsh advfirewall firewall delete rule name='Inferno Node P2P' ; netsh advfirewall firewall delete rule name='Inferno Node P2P UDP'";
        
        // Use PowerShell Start-Process with -Verb RunAs to elevate
        let ps_command = format!(
            "Start-Process -FilePath 'cmd.exe' -ArgumentList '/c {}' -Verb RunAs -Wait -WindowStyle Hidden",
            remove_script.replace("'", "''")
        );
        
        info!("Running elevated firewall removal command via PowerShell");
        
        let result = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_command])
            .output();
        
        match result {
            Ok(_) => {
                // Brief wait then verify rules are gone
                std::thread::sleep(std::time::Duration::from_millis(500));
                
                let check = Command::new("netsh")
                    .args(["advfirewall", "firewall", "show", "rule", "name=Inferno Node P2P"])
                    .output();
                
                match check {
                    Ok(check_result) => {
                        let stdout = String::from_utf8_lossy(&check_result.stdout);
                        if !stdout.contains("Inferno Node P2P") || stdout.contains("No rules match") {
                            info!("Firewall rules removed successfully");
                            Ok(FirewallResult {
                                success: true,
                                message: "✓ Firewall rules removed".to_string(),
                                requires_restart: false,
                            })
                        } else {
                            warn!("Firewall rules still exist after removal attempt");
                            Err("Failed to remove firewall rules. Please click 'Yes' on the Administrator prompt.".to_string())
                        }
                    }
                    Err(_) => {
                        // If check fails, assume success (rule doesn't exist)
                        Ok(FirewallResult {
                            success: true,
                            message: "✓ Firewall rules removed".to_string(),
                            requires_restart: false,
                        })
                    }
                }
            }
            Err(e) => {
                error!("Failed to execute PowerShell: {}", e);
                Err(format!("Failed to remove firewall rules: {}", e))
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        info!("Removing macOS firewall rules for Inferno Node");
        
        let app_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        
        if !app_path.is_empty() {
            let _ = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
                .args(["--remove", &app_path])
                .output();
        }
        
        Ok(FirewallResult {
            success: true,
            message: "Removed Inferno Node from macOS firewall allowed apps".to_string(),
            requires_restart: false,
        })
    }
    
    #[cfg(target_os = "linux")]
    {
        info!("Removing Linux firewall rules for Inferno Node");
        
        // Try to get port from state (default to 30303)
        let p2p_port = 30303u16;
        
        // Try UFW
        let _ = Command::new("sudo")
            .args(["ufw", "delete", "allow", &format!("{}/tcp", p2p_port)])
            .output();
        let _ = Command::new("sudo")
            .args(["ufw", "delete", "allow", &format!("{}/udp", p2p_port)])
            .output();
        
        // Try firewalld
        let _ = Command::new("sudo")
            .args(["firewall-cmd", "--permanent", "--remove-port", &format!("{}/tcp", p2p_port)])
            .output();
        let _ = Command::new("sudo")
            .args(["firewall-cmd", "--permanent", "--remove-port", &format!("{}/udp", p2p_port)])
            .output();
        let _ = Command::new("sudo")
            .args(["firewall-cmd", "--reload"])
            .output();
        
        Ok(FirewallResult {
            success: true,
            message: format!("Removed firewall rules for port {}", p2p_port),
            requires_restart: false,
        })
    }
}

/// Test if a P2P port is reachable from the internet
/// Uses external port checking service to verify connectivity
#[tauri::command]
pub async fn test_port_connectivity(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<PortTestResult, String> {
    let p2p_port = {
        let app_state = state.lock();
        app_state.settings.p2p_port
    };
    
    info!("Testing port {} connectivity...", p2p_port);
    
    // First, check if port is listening locally
    let local_check = std::net::TcpListener::bind(format!("0.0.0.0:{}", p2p_port));
    let port_in_use = local_check.is_err();
    
    // Try to get external IP
    let external_ip = get_external_ip().await;
    
    // Use canyouseeme.org API or similar to check port from outside
    let external_reachable = if let Some(ref ip) = external_ip {
        check_port_external(ip, p2p_port).await
    } else {
        false
    };
    
    let status = if external_reachable {
        "open"
    } else if port_in_use {
        "in_use_but_blocked"
    } else {
        "blocked"
    };
    
    let recommendation = match status {
        "open" => "Your port is open and reachable! Full node mode will work.".to_string(),
        "in_use_but_blocked" => format!(
            "Port {} is in use locally but not reachable from internet. Try:\n\
            1. Enable port forwarding on your router\n\
            2. Switch to 'Relay Mode' in Network Settings\n\
            3. Try a different port (443, 8080, 8443)", 
            p2p_port
        ),
        _ => format!(
            "Port {} appears blocked. Options:\n\
            1. Switch to 'Relay Mode' (works behind any firewall)\n\
            2. Try 'Auto' mode with a stealth port (443, 8080)\n\
            3. Configure port forwarding on your router",
            p2p_port
        ),
    };
    
    Ok(PortTestResult {
        port: p2p_port,
        status: status.to_string(),
        external_ip,
        is_reachable: external_reachable,
        recommendation,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortTestResult {
    pub port: u16,
    pub status: String,
    pub external_ip: Option<String>,
    pub is_reachable: bool,
    pub recommendation: String,
}

/// Get external IP address
async fn get_external_ip() -> Option<String> {
    // Try multiple services for redundancy
    let services = [
        "https://api.ipify.org",
        "https://icanhazip.com",
        "https://ifconfig.me/ip",
    ];
    
    for service in services {
        if let Ok(response) = reqwest::get(service).await {
            if let Ok(ip) = response.text().await {
                let ip = ip.trim().to_string();
                if !ip.is_empty() && ip.len() < 50 {
                    return Some(ip);
                }
            }
        }
    }
    None
}

/// Check if port is reachable from external network
async fn check_port_external(ip: &str, port: u16) -> bool {
    // Use a port checking service
    // This is a simple implementation - in production you might use a more reliable service
    let url = format!("https://portchecker.co/check?ip={}&port={}", ip, port);
    
    match reqwest::get(&url).await {
        Ok(response) => {
            if let Ok(body) = response.text().await {
                // Check if response indicates port is open
                body.contains("open") || body.contains("reachable") || body.contains("success")
            } else {
                false
            }
        }
        Err(_) => {
            // If we can't check externally, try a simple TCP connect test
            // This won't work for NAT but gives some indication
            false
        }
    }
}

/// Get network diagnostics for troubleshooting
#[tauri::command]
pub async fn get_network_diagnostics(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<NetworkDiagnostics, String> {
    let settings = {
        let app_state = state.lock();
        app_state.settings.clone()
    };
    
    let external_ip = get_external_ip().await;
    
    Ok(NetworkDiagnostics {
        connection_mode: match settings.connection_mode {
            crate::state::ConnectionMode::FullNode => "Full Node".to_string(),
            crate::state::ConnectionMode::RelayOnly => "Relay Only".to_string(),
            crate::state::ConnectionMode::Auto => "Auto".to_string(),
        },
        p2p_port: settings.p2p_port,
        port_preset: match settings.port_preset {
            crate::state::PortPreset::Standard => "Standard (30303)".to_string(),
            crate::state::PortPreset::Https => "HTTPS (443)".to_string(),
            crate::state::PortPreset::AltHttp => "Alt HTTP (8080)".to_string(),
            crate::state::PortPreset::AltHttps => "Alt HTTPS (8443)".to_string(),
            crate::state::PortPreset::Custom => format!("Custom ({})", settings.p2p_port),
        },
        websocket_enabled: settings.enable_websocket,
        auto_fallback_enabled: settings.auto_port_fallback,
        external_ip,
        nat_type: settings.detected_nat_type.clone(),
        last_transport: settings.last_successful_transport.clone(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnostics {
    pub connection_mode: String,
    pub p2p_port: u16,
    pub port_preset: String,
    pub websocket_enabled: bool,
    pub auto_fallback_enabled: bool,
    pub external_ip: Option<String>,
    pub nat_type: Option<String>,
    pub last_transport: Option<String>,
}
