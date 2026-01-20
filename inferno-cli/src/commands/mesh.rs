//! VPN Mesh Commands for private P2P networks

use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::process::Command;

#[derive(Subcommand, Debug)]
pub enum MeshCommand {
    /// Create a new VPN mesh network
    Create {
        /// Mesh network name
        name: String,
        /// CIDR range for mesh (e.g., 10.100.0.0/24)
        #[arg(short, long, default_value = "10.100.0.0/24")]
        cidr: String,
    },
    /// Join an existing mesh network
    Join {
        /// Mesh invite token or peer address
        invite: String,
    },
    /// Leave the current mesh network
    Leave {
        /// Mesh network name
        name: String,
    },
    /// List mesh network members
    Members {
        /// Mesh network name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Show mesh network status
    Status,
    /// Generate invite token for mesh
    Invite {
        /// Mesh network name
        name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshNetwork {
    pub name: String,
    pub cidr: String,
    pub my_ip: String,
    pub peers: Vec<MeshPeer>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshPeer {
    pub name: String,
    pub mesh_ip: String,
    pub public_ip: String,
    pub connected: bool,
}

pub async fn run(command: MeshCommand) -> Result<()> {
    match command {
        MeshCommand::Create { name, cidr } => create_mesh(&name, &cidr).await,
        MeshCommand::Join { invite } => join_mesh(&invite).await,
        MeshCommand::Leave { name } => leave_mesh(&name).await,
        MeshCommand::Members { name } => list_members(name.as_deref()).await,
        MeshCommand::Status => mesh_status().await,
        MeshCommand::Invite { name } => generate_invite(&name).await,
    }
}

async fn create_mesh(name: &str, cidr: &str) -> Result<()> {
    println!("{}", "🔗 Creating VPN Mesh Network".bright_cyan().bold());
    println!("  {} {}", "Name:".dimmed(), name);
    println!("  {} {}", "CIDR:".dimmed(), cidr);
    
    // Check for WireGuard
    if !check_wireguard() {
        println!("{}", "⚠ WireGuard not found. Installing...".yellow());
        install_wireguard()?;
    }
    
    // Generate WireGuard keys
    let private_key = generate_wg_key()?;
    let public_key = derive_wg_pubkey(&private_key)?;
    
    // Assign first IP in range
    let my_ip = assign_mesh_ip(cidr, 1)?;
    
    // Create WireGuard config
    let config = format!(r#"[Interface]
PrivateKey = {}
Address = {}/24
ListenPort = 51820

# Peers will be added dynamically
"#, private_key, my_ip);
    
    let config_path = dirs::config_dir()
        .unwrap_or_default()
        .join("pyrax")
        .join("mesh")
        .join(format!("{}.conf", name));
    
    tokio::fs::create_dir_all(config_path.parent().unwrap()).await?;
    tokio::fs::write(&config_path, &config).await?;
    
    // Save mesh info
    let mesh = MeshNetwork {
        name: name.to_string(),
        cidr: cidr.to_string(),
        my_ip: my_ip.to_string(),
        peers: Vec::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    let mesh_file = config_path.with_extension("json");
    tokio::fs::write(&mesh_file, serde_json::to_string_pretty(&mesh)?).await?;
    
    println!();
    println!("{}", "✅ Mesh network created!".bright_green());
    println!("  {} {}", "Your mesh IP:".dimmed(), my_ip);
    println!("  {} {}", "Public key:".dimmed(), public_key);
    println!();
    println!("Share this invite with peers:");
    println!("  inferno mesh join {}@{}", public_key, my_ip);
    
    Ok(())
}

async fn join_mesh(invite: &str) -> Result<()> {
    println!("{}", "🔗 Joining VPN Mesh Network".bright_cyan().bold());
    
    // Parse invite (format: pubkey@mesh_ip or just peer address)
    let parts: Vec<&str> = invite.split('@').collect();
    let (peer_pubkey, peer_ip) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        return Err(anyhow::anyhow!("Invalid invite format. Expected: pubkey@ip"));
    };
    
    println!("  {} {}", "Peer:".dimmed(), peer_ip);
    
    // Generate our keys
    let private_key = generate_wg_key()?;
    let public_key = derive_wg_pubkey(&private_key)?;
    
    // Create config with peer
    let config = format!(r#"[Interface]
PrivateKey = {}
Address = 10.100.0.2/24

[Peer]
PublicKey = {}
AllowedIPs = 10.100.0.0/24
Endpoint = {}:51820
PersistentKeepalive = 25
"#, private_key, peer_pubkey, peer_ip);
    
    let config_path = dirs::config_dir()
        .unwrap_or_default()
        .join("pyrax")
        .join("mesh")
        .join("joined.conf");
    
    tokio::fs::create_dir_all(config_path.parent().unwrap()).await?;
    tokio::fs::write(&config_path, &config).await?;
    
    // Bring up interface
    bring_up_wg("pyrax-mesh", &config_path)?;
    
    println!("{}", "✅ Joined mesh network!".bright_green());
    println!("  {} {}", "Your public key:".dimmed(), public_key);
    
    Ok(())
}

async fn leave_mesh(name: &str) -> Result<()> {
    println!("Leaving mesh network: {}", name);
    
    // Bring down WireGuard interface
    let _ = Command::new("wg-quick").args(["down", "pyrax-mesh"]).output();
    
    // Remove config
    let config_path = dirs::config_dir()
        .unwrap_or_default()
        .join("pyrax")
        .join("mesh")
        .join(format!("{}.conf", name));
    
    if config_path.exists() {
        tokio::fs::remove_file(&config_path).await?;
    }
    
    println!("{}", "✅ Left mesh network".bright_green());
    Ok(())
}

async fn list_members(name: Option<&str>) -> Result<()> {
    println!("{}", "Mesh Network Members".bright_cyan().bold());
    
    // Read mesh config
    let mesh_dir = dirs::config_dir()
        .unwrap_or_default()
        .join("pyrax")
        .join("mesh");
    
    if !mesh_dir.exists() {
        println!("  No mesh networks configured");
        return Ok(());
    }
    
    // Show WireGuard peer info
    if let Ok(output) = Command::new("wg").args(["show"]).output() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    
    Ok(())
}

async fn mesh_status() -> Result<()> {
    println!("{}", "VPN Mesh Status".bright_cyan().bold());
    
    if let Ok(output) = Command::new("wg").args(["show"]).output() {
        if output.stdout.is_empty() {
            println!("  No active mesh connections");
        } else {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        println!("  WireGuard not available");
    }
    
    Ok(())
}

async fn generate_invite(name: &str) -> Result<()> {
    let config_path = dirs::config_dir()
        .unwrap_or_default()
        .join("pyrax")
        .join("mesh")
        .join(format!("{}.json", name));
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Mesh network '{}' not found", name));
    }
    
    let mesh: MeshNetwork = serde_json::from_str(&tokio::fs::read_to_string(&config_path).await?)?;
    
    // Get public key from WireGuard
    let pubkey = if let Ok(output) = Command::new("wg").args(["show", "pyrax-mesh", "public-key"]).output() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        "PUBKEY".to_string()
    };
    
    // Get external IP
    let external_ip = get_external_ip().await.unwrap_or_else(|_| "YOUR_IP".to_string());
    
    println!("{}", "Mesh Invite Token".bright_cyan().bold());
    println!();
    println!("Share this with peers to join:");
    println!("  {} mesh join {}@{}", "inferno".bright_white(), pubkey, external_ip);
    
    Ok(())
}

fn check_wireguard() -> bool {
    Command::new("wg").arg("--version").output().is_ok()
}

fn install_wireguard() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        Command::new("apt-get").args(["install", "-y", "wireguard"]).output()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("brew").args(["install", "wireguard-tools"]).output()?;
    }
    Ok(())
}

fn generate_wg_key() -> Result<String> {
    let output = Command::new("wg").args(["genkey"]).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn derive_wg_pubkey(private_key: &str) -> Result<String> {
    use std::io::Write;
    let mut child = Command::new("wg")
        .args(["pubkey"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    
    child.stdin.as_mut().unwrap().write_all(private_key.as_bytes())?;
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn assign_mesh_ip(cidr: &str, index: u8) -> Result<String> {
    let parts: Vec<&str> = cidr.split('/').collect();
    let base: Vec<&str> = parts[0].split('.').collect();
    Ok(format!("{}.{}.{}.{}", base[0], base[1], base[2], index))
}

fn bring_up_wg(interface: &str, config_path: &std::path::Path) -> Result<()> {
    Command::new("wg-quick")
        .args(["up", &config_path.display().to_string()])
        .output()?;
    Ok(())
}

async fn get_external_ip() -> Result<String> {
    let resp = reqwest::get("https://api.ipify.org").await?.text().await?;
    Ok(resp.trim().to_string())
}
