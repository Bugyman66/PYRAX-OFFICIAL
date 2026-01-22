use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use serde::Serialize;

use crate::instance::{InstanceManager, InstanceStatus};

#[derive(Args)]
pub struct StatusArgs {
    /// Instance number (default: 1)
    #[arg(short, long, default_value = "1")]
    pub instance: u32,

    /// Show all instances
    #[arg(long)]
    pub all: bool,
}

#[derive(Serialize)]
struct NodeStatus {
    instance_id: u32,
    status: String,
    pid: Option<u32>,
    network: String,
    p2p_port: u16,
    rpc_port: u16,
    peers: Option<u64>,
    block_height: Option<u64>,
    syncing: Option<bool>,
    mining: Option<bool>,
    uptime: Option<String>,
}

pub async fn run(args: StatusArgs, json: bool) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    if args.all {
        let instances = instance_manager.list_instances()?;
        for inst in instances {
            show_status(&instance_manager, inst.instance_id, json).await?;
            if !json {
                println!();
            }
        }
    } else {
        show_status(&instance_manager, args.instance, json).await?;
    }

    Ok(())
}

async fn show_status(instance_manager: &InstanceManager, instance_id: u32, json: bool) -> Result<()> {
    if !instance_manager.instance_exists(instance_id) {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "error": "Instance not found",
                    "instance_id": instance_id
                })
            );
        } else {
            println!(
                "  {} Instance {} not found",
                "!".bright_yellow(),
                instance_id.to_string().bright_cyan()
            );
        }
        return Ok(());
    }

    let config = instance_manager.load_config(instance_id)?;
    let pid = instance_manager.get_pid(instance_id)?;
    let is_running = pid.map(|p| instance_manager.is_process_running(p)).unwrap_or(false);

    let status = if is_running {
        InstanceStatus::Running
    } else {
        InstanceStatus::Stopped
    };

    // Try to get RPC data if running
    let (peers, block_height, syncing, mining) = if is_running {
        get_rpc_status(&config.network.rpc_port).await.unwrap_or((None, None, None, None))
    } else {
        (None, None, None, None)
    };

    let uptime = if is_running {
        pid.and_then(|p| get_process_uptime(p))
    } else {
        None
    };

    if json {
        let node_status = NodeStatus {
            instance_id,
            status: format!("{:?}", status),
            pid,
            network: config.network.network.clone(),
            p2p_port: config.network.p2p_port,
            rpc_port: config.network.rpc_port,
            peers,
            block_height,
            syncing,
            mining,
            uptime,
        };
        println!("{}", serde_json::to_string_pretty(&node_status)?);
    } else {
        print_status_display(
            instance_id,
            &status,
            pid,
            &config.network.network,
            config.network.p2p_port,
            config.network.rpc_port,
            peers,
            block_height,
            syncing,
            mining,
            uptime.as_deref(),
        );
    }

    Ok(())
}

fn print_status_display(
    instance_id: u32,
    status: &InstanceStatus,
    pid: Option<u32>,
    network: &str,
    p2p_port: u16,
    rpc_port: u16,
    peers: Option<u64>,
    block_height: Option<u64>,
    syncing: Option<bool>,
    mining: Option<bool>,
    uptime: Option<&str>,
) {
    let status_icon = match status {
        InstanceStatus::Running => "🟢",
        InstanceStatus::Stopped => "🔴",
        InstanceStatus::Unknown => "🟡",
    };

    let status_text = match status {
        InstanceStatus::Running => "Online".bright_green(),
        InstanceStatus::Stopped => "Offline".bright_red(),
        InstanceStatus::Unknown => "Unknown".bright_yellow(),
    };

    println!();
    println!(
        "  ╭─────────────────────────────────────────────────────────╮"
    );
    println!(
        "  │  {} {} Instance {}                                      │",
        status_icon,
        status_text,
        instance_id.to_string().bright_cyan()
    );
    println!(
        "  ╰─────────────────────────────────────────────────────────╯"
    );
    println!();

    // Basic info
    println!("  {} Node Information", "📊".bright_blue());
    println!(
        "    {} Network:    {}",
        "→".dimmed(),
        network.bright_white()
    );
    println!(
        "    {} P2P Port:   {}",
        "→".dimmed(),
        p2p_port.to_string().bright_white()
    );
    println!(
        "    {} RPC Port:   {}",
        "→".dimmed(),
        rpc_port.to_string().bright_white()
    );

    if let Some(pid) = pid {
        println!(
            "    {} PID:        {}",
            "→".dimmed(),
            pid.to_string().bright_yellow()
        );
    }

    if let Some(uptime) = uptime {
        println!(
            "    {} Uptime:     {}",
            "→".dimmed(),
            uptime.bright_white()
        );
    }

    println!();

    // Network status
    if matches!(status, InstanceStatus::Running) {
        println!("  {} Network Status", "🌐".bright_blue());

        if let Some(peers) = peers {
            let peers_color = if peers > 3 {
                peers.to_string().bright_green()
            } else if peers > 0 {
                peers.to_string().bright_yellow()
            } else {
                peers.to_string().bright_red()
            };
            println!("    {} Peers:      {}", "→".dimmed(), peers_color);
        } else {
            println!("    {} Peers:      {}", "→".dimmed(), "-".dimmed());
        }

        if let Some(height) = block_height {
            println!(
                "    {} Height:     {}",
                "→".dimmed(),
                height.to_string().bright_white()
            );
        }

        if let Some(syncing) = syncing {
            let sync_status = if syncing {
                "Syncing...".bright_yellow()
            } else {
                "Synced".bright_green()
            };
            println!("    {} Sync:       {}", "→".dimmed(), sync_status);
        }

        println!();

        // Mining status
        println!("  {} Mining", "⛏️".bright_blue());
        if let Some(mining) = mining {
            let mining_status = if mining {
                "Active".bright_green()
            } else {
                "Inactive".dimmed()
            };
            println!("    {} Status:     {}", "→".dimmed(), mining_status);
        } else {
            println!("    {} Status:     {}", "→".dimmed(), "-".dimmed());
        }
    }
}

async fn get_rpc_status(rpc_port: &u16) -> Result<(Option<u64>, Option<u64>, Option<bool>, Option<bool>)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()?;

    let rpc_url = format!("http://127.0.0.1:{}", rpc_port);

    // Get peer count
    let peers = match client
        .post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "net_peerCount",
            "params": [],
            "id": 1
        }))
        .send()
        .await
    {
        Ok(response) => {
            let result: serde_json::Value = response.json().await?;
            result
                .get("result")
                .and_then(|v| v.as_str())
                .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        }
        Err(_) => None,
    };

    // Get block number
    let block_height = match client
        .post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 2
        }))
        .send()
        .await
    {
        Ok(response) => {
            let result: serde_json::Value = response.json().await?;
            result
                .get("result")
                .and_then(|v| v.as_str())
                .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        }
        Err(_) => None,
    };

    // Get syncing status
    let syncing = match client
        .post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_syncing",
            "params": [],
            "id": 3
        }))
        .send()
        .await
    {
        Ok(response) => {
            let result: serde_json::Value = response.json().await?;
            result.get("result").map(|v| !v.is_boolean() || v.as_bool().unwrap_or(false))
        }
        Err(_) => None,
    };

    // Get mining status
    let mining = match client
        .post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_mining",
            "params": [],
            "id": 4
        }))
        .send()
        .await
    {
        Ok(response) => {
            let result: serde_json::Value = response.json().await?;
            result.get("result").and_then(|v| v.as_bool())
        }
        Err(_) => None,
    };

    Ok((peers, block_height, syncing, mining))
}

fn get_process_uptime(_pid: u32) -> Option<String> {
    // Platform-specific implementation would go here
    // For now, return None as a placeholder
    None
}
