use crate::state::{AppState, Settings, Network, Theme};
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
/// This requires administrator privileges
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
        
        // Create inbound rule for TCP
        let tcp_inbound = Command::new("netsh")
            .args([
                "advfirewall", "firewall", "add", "rule",
                "name=Inferno Node P2P",
                "dir=in",
                "action=allow",
                &format!("protocol=TCP"),
                &format!("localport={}", p2p_port),
                "profile=any",
                "description=Allow inbound P2P connections for Inferno Node blockchain sync"
            ])
            .output();
        
        // Create inbound rule for UDP (for hole-punching)
        let udp_inbound = Command::new("netsh")
            .args([
                "advfirewall", "firewall", "add", "rule",
                "name=Inferno Node P2P UDP",
                "dir=in",
                "action=allow",
                &format!("protocol=UDP"),
                &format!("localport={}", p2p_port),
                "profile=any",
                "description=Allow UDP for P2P hole-punching"
            ])
            .output();
        
        match (tcp_inbound, udp_inbound) {
            (Ok(tcp), Ok(udp)) => {
                let tcp_success = tcp.status.success();
                let udp_success = udp.status.success();
                
                if tcp_success && udp_success {
                    info!("Firewall rules configured successfully for port {}", p2p_port);
                    Ok(FirewallResult {
                        success: true,
                        message: format!("Firewall configured for port {} (TCP + UDP)", p2p_port),
                        requires_restart: false,
                    })
                } else if tcp_success || udp_success {
                    warn!("Partial firewall configuration");
                    Ok(FirewallResult {
                        success: true,
                        message: "Firewall partially configured. Some rules may require manual setup.".to_string(),
                        requires_restart: false,
                    })
                } else {
                    let stderr = String::from_utf8_lossy(&tcp.stderr);
                    if stderr.contains("requires elevation") || stderr.contains("Access is denied") {
                        Err("Administrator privileges required. Please run as Administrator.".to_string())
                    } else {
                        Err(format!("Failed to configure firewall: {}", stderr))
                    }
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                error!("Failed to execute netsh: {}", e);
                Err(format!("Failed to configure firewall: {}", e))
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

/// Remove Inferno Node firewall rules
#[tauri::command]
pub async fn remove_firewall_rules() -> Result<FirewallResult, String> {
    #[cfg(target_os = "windows")]
    {
        info!("Removing Windows Firewall rules for Inferno Node");
        
        let tcp_result = Command::new("netsh")
            .args(["advfirewall", "firewall", "delete", "rule", "name=Inferno Node P2P"])
            .output();
        
        let udp_result = Command::new("netsh")
            .args(["advfirewall", "firewall", "delete", "rule", "name=Inferno Node P2P UDP"])
            .output();
        
        match (tcp_result, udp_result) {
            (Ok(_), Ok(_)) => {
                info!("Firewall rules removed successfully");
                Ok(FirewallResult {
                    success: true,
                    message: "Firewall rules removed".to_string(),
                    requires_restart: false,
                })
            }
            _ => {
                Err("Failed to remove firewall rules. Administrator privileges may be required.".to_string())
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
