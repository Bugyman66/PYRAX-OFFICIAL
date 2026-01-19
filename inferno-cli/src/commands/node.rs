use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;
use std::process::Stdio;
use tokio::process::Command;

use crate::config::InfernoConfig;
use crate::instance::{InstanceInfo, InstanceManager, InstanceStatus};

/// Bootnode configuration with geolocation for proximity-based selection
#[derive(Clone)]
struct BootnodeConfig {
    ip: &'static str,
    p2p_port: u16,
    rpc_port: u16,
    lat: f64,
    lon: f64,
    region: &'static str,
}

/// Get bootnode configurations for devnet with geolocation data
fn get_bootnode_configs() -> Vec<BootnodeConfig> {
    vec![
        // Bootnode 1: Digital Ocean NYC
        BootnodeConfig { ip: "209.38.137.105", rpc_port: 28545, p2p_port: 30303, lat: 40.7128, lon: -74.0060, region: "NYC" },
        // Bootnode 2: Digital Ocean SFO
        BootnodeConfig { ip: "137.184.118.228", rpc_port: 28545, p2p_port: 30303, lat: 37.7749, lon: -122.4194, region: "SFO" },
    ]
}

/// User's geolocation from IP lookup
#[derive(Debug, Clone)]
struct UserLocation {
    lat: f64,
    lon: f64,
    city: String,
    country: String,
}

/// Fetch user's geolocation using ip-api.com (free, no API key required)
async fn get_user_location() -> Option<UserLocation> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    
    let response = client
        .get("http://ip-api.com/json/?fields=status,lat,lon,city,country")
        .send()
        .await
        .ok()?;
    
    let json: serde_json::Value = response.json().await.ok()?;
    
    if json.get("status")?.as_str()? != "success" {
        return None;
    }
    
    Some(UserLocation {
        lat: json.get("lat")?.as_f64()?,
        lon: json.get("lon")?.as_f64()?,
        city: json.get("city")?.as_str()?.to_string(),
        country: json.get("country")?.as_str()?.to_string(),
    })
}

/// Calculate distance between two points using Haversine formula (returns km)
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();
    
    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    
    EARTH_RADIUS_KM * c
}

/// Sort bootnodes by distance from user (closest first)
fn sort_bootnodes_by_distance(configs: Vec<BootnodeConfig>, user_loc: &UserLocation) -> Vec<BootnodeConfig> {
    let mut configs_with_distance: Vec<(BootnodeConfig, f64)> = configs
        .into_iter()
        .map(|config| {
            let distance = haversine_distance(user_loc.lat, user_loc.lon, config.lat, config.lon);
            (config, distance)
        })
        .collect();
    
    configs_with_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    configs_with_distance.into_iter().map(|(config, _)| config).collect()
}

/// Fetch peer ID from a bootnode's RPC endpoint for dynamic discovery
/// Uses explicit RPC port from config instead of deriving it
async fn fetch_peer_id_from_bootnode(ip: &str, rpc_port: u16) -> Option<String> {
    let url = format!("http://{}:{}", ip, rpc_port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(r#"{"jsonrpc":"2.0","method":"pyrax_getPeerId","params":[],"id":1}"#)
        .send()
        .await
        .ok()?;
    
    let json: serde_json::Value = response.json().await.ok()?;
    let peer_id = json.get("result")?.as_str()?;
    
    if peer_id.is_empty() || !peer_id.starts_with("12D3KooW") {
        return None;
    }
    
    Some(peer_id.to_string())
}

/// Resolve bootnodes by fetching peer IDs dynamically
/// Sorts bootnodes by proximity to user - connects to closest 2 first
async fn resolve_bootnodes(_bootnodes: &[String]) -> Vec<String> {
    let mut configs = get_bootnode_configs();
    
    // Get user's location and sort bootnodes by proximity
    println!("  {} Detecting your location for optimal bootnode selection...", "🌍".bright_blue());
    match get_user_location().await {
        Some(user_loc) => {
            println!("  {} Location: {}, {}", "📍".bright_green(), user_loc.city.bright_cyan(), user_loc.country);
            
            // Sort bootnodes by distance (closest first)
            configs = sort_bootnodes_by_distance(configs, &user_loc);
            
            // Show bootnode distances
            for (i, config) in configs.iter().enumerate() {
                let distance = haversine_distance(user_loc.lat, user_loc.lon, config.lat, config.lon);
                if i < 2 {
                    println!("  {} Priority #{}: {} ({}) - {:.0} km", 
                        "★".bright_yellow(), i + 1, config.ip.bright_cyan(), config.region, distance);
                } else {
                    println!("  {} Bootnode #{}: {} ({}) - {:.0} km", 
                        "○".dimmed(), i + 1, config.ip, config.region, distance);
                }
            }
        }
        None => {
            println!("  {} Could not determine location - using default order", "!".bright_yellow());
        }
    }
    
    let mut resolved = Vec::new();
    
    // Connect to bootnodes in proximity order using explicit RPC port from config
    for (i, config) in configs.iter().enumerate() {
        // FIX: Use explicit RPC port from config instead of deriving it
        match fetch_peer_id_from_bootnode(config.ip, config.rpc_port).await {
            Some(peer_id) => {
                let full_addr = format!("/ip4/{}/tcp/{}/p2p/{}", config.ip, config.p2p_port, peer_id);
                if i < 2 {
                    println!("  {} Connected to priority bootnode #{} ({})", "✓".bright_green(), i + 1, config.region);
                }
                resolved.push(full_addr);
            }
            None => {
                println!("  {} Could not connect to {} ({})", "✗".bright_red(), config.ip, config.region);
            }
        }
    }
    
    if resolved.is_empty() {
        println!("  {} No bootnodes available!", "!".bright_red());
    } else {
        println!("  {} Connecting to {} bootnodes (closest {} first)", 
            "🔗".bright_blue(), resolved.len(), std::cmp::min(2, resolved.len()));
    }
    
    resolved
}

pub async fn start(instance: u32, all: bool, foreground: bool, network: &str) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    if all {
        let instances = instance_manager.list_instances()?;
        if instances.is_empty() {
            println!(
                "  {} No instances configured. Run {} first.",
                "!".bright_yellow(),
                "inferno init".bright_cyan()
            );
            return Ok(());
        }

        println!(
            "  {} Starting all {} instances...",
            "🚀".bright_green(),
            instances.len()
        );

        for inst in instances {
            start_instance(&instance_manager, inst.instance_id, foreground).await?;
        }
    } else {
        if !instance_manager.instance_exists(instance) {
            println!(
                "  {} Instance {} not found. Run {} first.",
                "!".bright_yellow(),
                instance.to_string().bright_cyan(),
                format!("inferno init --instance {}", instance).bright_cyan()
            );
            return Ok(());
        }

        start_instance(&instance_manager, instance, foreground).await?;
    }

    Ok(())
}

async fn start_instance(
    instance_manager: &InstanceManager,
    instance_id: u32,
    foreground: bool,
) -> Result<()> {
    let config = instance_manager
        .load_config(instance_id)
        .context("Failed to load instance configuration")?;

    // Check if already running
    if let Some(pid) = instance_manager.get_pid(instance_id)? {
        if instance_manager.is_process_running(pid) {
            println!(
                "  {} Instance {} is already running (PID: {})",
                "→".dimmed(),
                instance_id.to_string().bright_cyan(),
                pid.to_string().bright_yellow()
            );
            return Ok(());
        }
    }

    // Find pyrax-node binary
    let node_binary = find_node_binary()?;

    println!(
        "  {} Starting instance {}...",
        "→".dimmed(),
        instance_id.to_string().bright_cyan()
    );

    // Build command arguments
    let mut args = vec![
        "--network".to_string(),
        config.network.network.clone(),
        "--p2p-port".to_string(),
        config.network.p2p_port.to_string(),
        "--rpc-port".to_string(),
        config.network.rpc_port.to_string(),
        "--data-dir".to_string(),
        config.node.data_dir.to_string_lossy().to_string(),
        "--log-level".to_string(),
        config.node.log_level.clone(),
    ];

    if config.network.rpc_enabled {
        args.push("--rpc".to_string());
    }

    if config.mining.enabled {
        args.push("--mining".to_string());
        args.push("--mining-threads".to_string());
        args.push(config.mining.threads.to_string());
        if !config.mining.wallet.is_empty() {
            args.push("--mining-wallet".to_string());
            args.push(config.mining.wallet.clone());
        }
    }

    // Dynamically resolve bootnode peer IDs before connecting
    println!("  {} Discovering bootnode peer IDs...", "🔍".bright_blue());
    let resolved_bootnodes = resolve_bootnodes(&config.network.bootnodes).await;
    
    // FIX: Use --peer argument (not --bootnode) as expected by pyrax-node
    for bootnode in &resolved_bootnodes {
        args.push("--peer".to_string());
        args.push(bootnode.clone());
    }

    if foreground {
        // Run in foreground
        println!(
            "  {} Running in foreground. Press Ctrl+C to stop.",
            "ℹ".bright_blue()
        );

        let status = Command::new(&node_binary)
            .args(&args)
            .status()
            .await
            .context("Failed to start node process")?;

        if !status.success() {
            anyhow::bail!("Node exited with error: {:?}", status.code());
        }
    } else {
        // Run as daemon
        let child = Command::new(&node_binary)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn node process")?;

        let pid = child.id().unwrap_or(0);
        instance_manager.save_pid(instance_id, pid)?;

        println!(
            "  {} Instance {} started (PID: {})",
            "✓".bright_green(),
            instance_id.to_string().bright_cyan(),
            pid.to_string().bright_yellow()
        );

        println!(
            "    {} View logs: {}",
            "→".dimmed(),
            format!("inferno logs --instance {}", instance_id).bright_white()
        );
    }

    Ok(())
}

pub async fn stop(instance: u32, all: bool, force: bool) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    if all {
        let instances = instance_manager.list_instances()?;
        println!(
            "  {} Stopping all instances...",
            "🛑".bright_red()
        );

        for inst in instances {
            stop_instance(&instance_manager, inst.instance_id, force).await?;
        }
    } else {
        stop_instance(&instance_manager, instance, force).await?;
    }

    Ok(())
}

async fn stop_instance(
    instance_manager: &InstanceManager,
    instance_id: u32,
    force: bool,
) -> Result<()> {
    let pid = match instance_manager.get_pid(instance_id)? {
        Some(pid) => pid,
        None => {
            println!(
                "  {} Instance {} is not running",
                "→".dimmed(),
                instance_id.to_string().bright_cyan()
            );
            return Ok(());
        }
    };

    if !instance_manager.is_process_running(pid) {
        println!(
            "  {} Instance {} is not running (stale PID file)",
            "→".dimmed(),
            instance_id.to_string().bright_cyan()
        );
        instance_manager.remove_pid(instance_id)?;
        return Ok(());
    }

    println!(
        "  {} Stopping instance {} (PID: {})...",
        "→".dimmed(),
        instance_id.to_string().bright_cyan(),
        pid.to_string().bright_yellow()
    );

    instance_manager.kill_process(pid, force)?;
    instance_manager.remove_pid(instance_id)?;

    println!(
        "  {} Instance {} stopped",
        "✓".bright_green(),
        instance_id.to_string().bright_cyan()
    );

    Ok(())
}

pub async fn restart(instance: u32, all: bool) -> Result<()> {
    println!("  {} Restarting...", "🔄".bright_blue());

    stop(instance, all, false).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    start(instance, all, false, "devnet").await?;

    Ok(())
}

pub async fn list(detailed: bool, json: bool) -> Result<()> {
    let instance_manager = InstanceManager::new()?;
    let instances = instance_manager.list_instances()?;

    if json {
        let output = serde_json::to_string_pretty(&instances)?;
        println!("{}", output);
        return Ok(());
    }

    if instances.is_empty() {
        println!(
            "  {} No instances configured. Run {} to create one.",
            "ℹ".bright_blue(),
            "inferno init".bright_cyan()
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} Instances ({} total)",
        "📦".bright_blue(),
        instances.len()
    );
    println!();

    for inst in &instances {
        let status_icon = match inst.status {
            InstanceStatus::Running => "🟢",
            InstanceStatus::Stopped => "🔴",
            InstanceStatus::Unknown => "🟡",
        };

        let status_text = match inst.status {
            InstanceStatus::Running => "Running".bright_green(),
            InstanceStatus::Stopped => "Stopped".bright_red(),
            InstanceStatus::Unknown => "Unknown".bright_yellow(),
        };

        println!(
            "  {} Instance {} - {}",
            status_icon,
            inst.instance_id.to_string().bright_cyan(),
            status_text
        );

        if detailed {
            if let Some(ref config) = inst.config {
                println!(
                    "      {} Network: {}",
                    "→".dimmed(),
                    config.network.network.bright_white()
                );
                println!(
                    "      {} P2P: {}, RPC: {}",
                    "→".dimmed(),
                    config.network.p2p_port.to_string().bright_white(),
                    config.network.rpc_port.to_string().bright_white()
                );
                println!(
                    "      {} Data: {}",
                    "→".dimmed(),
                    config.node.data_dir.display().to_string().dimmed()
                );
            }
            if let Some(pid) = inst.pid {
                println!(
                    "      {} PID: {}",
                    "→".dimmed(),
                    pid.to_string().bright_yellow()
                );
            }
            if let Some(ref uptime) = inst.uptime {
                println!(
                    "      {} Uptime: {}",
                    "→".dimmed(),
                    uptime.bright_white()
                );
            }
            println!();
        }
    }

    if !detailed {
        println!();
        println!(
            "  {} Use {} for more details",
            "ℹ".dimmed(),
            "inferno list --detailed".bright_white()
        );
    }

    Ok(())
}

pub async fn peers(instance: u32, json: bool) -> Result<()> {
    let instance_manager = InstanceManager::new()?;
    let config = instance_manager.load_config(instance)?;

    let rpc_url = format!("http://127.0.0.1:{}", config.network.rpc_port);

    let client = reqwest::Client::new();
    let response = client
        .post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "net_peerCount",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to node RPC")?;

    let result: serde_json::Value = response.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if let Some(count) = result.get("result") {
            let count_str = count.as_str().unwrap_or("0");
            let count_num = u64::from_str_radix(count_str.trim_start_matches("0x"), 16).unwrap_or(0);
            println!(
                "  {} Connected peers: {}",
                "🔗".bright_blue(),
                count_num.to_string().bright_green()
            );
        }
    }

    Ok(())
}

pub async fn mining(instance: u32, start: bool, stop: bool, json: bool) -> Result<()> {
    let instance_manager = InstanceManager::new()?;
    let config = instance_manager.load_config(instance)?;

    let rpc_url = format!("http://127.0.0.1:{}", config.network.rpc_port);
    let client = reqwest::Client::new();

    if start {
        let response = client
            .post(&rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "miner_start",
                "params": [],
                "id": 1
            }))
            .send()
            .await
            .context("Failed to connect to node RPC")?;

        let result: serde_json::Value = response.json().await?;

        if json {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("  {} Mining started", "⛏️".bright_green());
        }
    } else if stop {
        let response = client
            .post(&rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "miner_stop",
                "params": [],
                "id": 1
            }))
            .send()
            .await
            .context("Failed to connect to node RPC")?;

        let result: serde_json::Value = response.json().await?;

        if json {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("  {} Mining stopped", "🛑".bright_red());
        }
    } else {
        // Show mining status
        let response = client
            .post(&rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_mining",
                "params": [],
                "id": 1
            }))
            .send()
            .await
            .context("Failed to connect to node RPC")?;

        let result: serde_json::Value = response.json().await?;

        if json {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            let is_mining = result
                .get("result")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if is_mining {
                println!("  {} Mining: {}", "⛏️".bright_green(), "Active".bright_green());
            } else {
                println!("  {} Mining: {}", "⛏️".dimmed(), "Inactive".dimmed());
            }
        }
    }

    Ok(())
}

pub async fn attach(instance: u32) -> Result<()> {
    let instance_manager = InstanceManager::new()?;
    let config = instance_manager.load_config(instance)?;

    println!(
        "  {} Attaching to instance {} RPC console...",
        "🔌".bright_blue(),
        instance.to_string().bright_cyan()
    );
    println!(
        "    {} RPC endpoint: {}",
        "→".dimmed(),
        format!("http://127.0.0.1:{}", config.network.rpc_port).bright_white()
    );
    println!();
    println!("  {} Interactive console not yet implemented.", "ℹ".bright_yellow());
    println!(
        "    {} Use {} to interact with the node.",
        "→".dimmed(),
        "curl or httpie".bright_white()
    );

    Ok(())
}

fn find_node_binary() -> Result<String> {
    // Check common locations
    let possible_paths = vec![
        "pyrax-node",
        "./pyrax-node",
        "../pyrax-node/target/release/pyrax-node",
        "/usr/local/bin/pyrax-node",
        "/usr/bin/pyrax-node",
    ];

    for path in possible_paths {
        if which::which(path).is_ok() {
            return Ok(path.to_string());
        }
    }

    // Check if it's in the same directory as inferno
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            let node_path = dir.join("pyrax-node");
            if node_path.exists() {
                return Ok(node_path.to_string_lossy().to_string());
            }
            #[cfg(windows)]
            {
                let node_path = dir.join("pyrax-node.exe");
                if node_path.exists() {
                    return Ok(node_path.to_string_lossy().to_string());
                }
            }
        }
    }

    anyhow::bail!(
        "Could not find pyrax-node binary. Please ensure it is installed and in your PATH."
    )
}
