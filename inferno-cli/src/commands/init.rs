use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::PathBuf;

use crate::config::{InfernoConfig, MiningConfig, NetworkConfig, NodeConfig};
use crate::instance::InstanceManager;
use crate::ports::PortManager;

#[derive(Args)]
pub struct InitArgs {
    /// Instance number to initialize (default: auto-detect next available)
    #[arg(short, long)]
    pub instance: Option<u32>,

    /// Use quick setup with defaults (non-interactive)
    #[arg(short, long)]
    pub quick: bool,

    /// Network to connect to
    #[arg(short, long, default_value = "devnet")]
    pub network: String,

    /// Custom data directory
    #[arg(short, long)]
    pub data_dir: Option<PathBuf>,

    /// Enable mining
    #[arg(long)]
    pub mining: bool,

    /// Mining wallet address
    #[arg(long)]
    pub wallet: Option<String>,
}

pub async fn run(args: InitArgs) -> Result<()> {
    let instance_manager = InstanceManager::new()?;
    let port_manager = PortManager::new();

    // Determine instance number
    let instance_id = args
        .instance
        .unwrap_or_else(|| instance_manager.next_available_instance());

    println!(
        "  {} Initializing instance {}...\n",
        "📦".bright_yellow(),
        instance_id.to_string().bright_cyan()
    );

    let config = if args.quick {
        quick_setup(&args, instance_id, &port_manager).await?
    } else {
        interactive_setup(&args, instance_id, &port_manager).await?
    };

    // Save configuration
    instance_manager.save_config(instance_id, &config)?;

    println!();
    println!(
        "  {} Configuration saved!",
        "✓".bright_green()
    );
    println!();
    println!("  {} Getting Started:", "📖".bright_blue());
    println!();
    println!(
        "    {} Start your node:",
        "1.".dimmed()
    );
    println!(
        "       {}",
        format!("inferno start --instance {}", instance_id).bright_white()
    );
    println!();
    println!(
        "    {} View logs:",
        "2.".dimmed()
    );
    println!(
        "       {}",
        format!("inferno logs --instance {}", instance_id).bright_white()
    );
    println!();
    println!(
        "    {} Open dashboard:",
        "3.".dimmed()
    );
    println!("       {}", "inferno dashboard --open".bright_white());
    println!();

    Ok(())
}

async fn quick_setup(
    args: &InitArgs,
    instance_id: u32,
    port_manager: &PortManager,
) -> Result<InfernoConfig> {
    println!("  {} Using quick setup with defaults...", "⚡".bright_yellow());

    let base_p2p_port = 30303 + ((instance_id - 1) * 10) as u16;
    let base_rpc_port = 28545 + ((instance_id - 1) * 10) as u16;

    let p2p_port = port_manager.find_available_port(base_p2p_port)?;
    let rpc_port = port_manager.find_available_port(base_rpc_port)?;

    let data_dir = args.data_dir.clone().unwrap_or_else(|| {
        let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join(".inferno")
            .join(&args.network)
            .join(format!("instance-{}", instance_id))
    });

    println!(
        "    {} Data directory: {}",
        "→".dimmed(),
        data_dir.display().to_string().bright_cyan()
    );
    println!(
        "    {} Network: {}",
        "→".dimmed(),
        args.network.bright_cyan()
    );
    println!(
        "    {} P2P Port: {}",
        "→".dimmed(),
        p2p_port.to_string().bright_cyan()
    );
    println!(
        "    {} RPC Port: {}",
        "→".dimmed(),
        rpc_port.to_string().bright_cyan()
    );

    Ok(InfernoConfig {
        node: NodeConfig {
            instance_id,
            data_dir,
            log_level: "info".to_string(),
        },
        network: NetworkConfig {
            network: args.network.clone(),
            p2p_port,
            rpc_port,
            rpc_enabled: true,
            bootnodes: get_default_bootnodes(&args.network),
            max_peers: 50,
        },
        mining: MiningConfig {
            enabled: args.mining,
            threads: num_cpus::get() as u32 / 2,
            wallet: args.wallet.clone().unwrap_or_default(),
        },
    })
}

async fn interactive_setup(
    args: &InitArgs,
    instance_id: u32,
    port_manager: &PortManager,
) -> Result<InfernoConfig> {
    let theme = ColorfulTheme::default();

    println!(
        "  {} Welcome to the Inferno setup wizard!\n",
        "🔥".bright_red()
    );

    // Setup type selection
    let setup_options = vec![
        "Quick Setup (recommended defaults)",
        "Custom Setup (advanced users)",
    ];

    let setup_choice = Select::with_theme(&theme)
        .with_prompt("Select setup type")
        .items(&setup_options)
        .default(0)
        .interact()
        .context("Failed to get setup type selection")?;

    if setup_choice == 0 {
        return quick_setup(args, instance_id, port_manager).await;
    }

    println!();
    println!("  {} Custom Configuration", "⚙️".bright_blue());
    println!();

    // Data directory
    let default_data_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".inferno")
        .join(&args.network)
        .join(format!("instance-{}", instance_id));

    let data_dir_str: String = Input::with_theme(&theme)
        .with_prompt("Data directory")
        .default(default_data_dir.to_string_lossy().to_string())
        .interact_text()
        .context("Failed to get data directory")?;

    let data_dir = PathBuf::from(shellexpand::tilde(&data_dir_str).to_string());

    // Network selection
    let networks = vec!["devnet", "testnet (coming soon)", "mainnet (coming soon)"];
    let network_choice = Select::with_theme(&theme)
        .with_prompt("Select network")
        .items(&networks)
        .default(0)
        .interact()
        .context("Failed to get network selection")?;

    let network = match network_choice {
        0 => "devnet".to_string(),
        1 => "testnet".to_string(),
        _ => "mainnet".to_string(),
    };

    // Port configuration
    println!();
    println!("  {} Port Configuration", "🔌".bright_blue());
    println!();

    let base_p2p_port = 30303 + ((instance_id - 1) * 10) as u16;
    let suggested_p2p = port_manager.find_available_port(base_p2p_port)?;

    let p2p_port: u16 = Input::with_theme(&theme)
        .with_prompt("P2P port")
        .default(suggested_p2p)
        .interact_text()
        .context("Failed to get P2P port")?;

    // Validate P2P port
    if !port_manager.is_port_available(p2p_port) {
        println!(
            "    {} Port {} is in use, finding alternative...",
            "⚠".bright_yellow(),
            p2p_port
        );
        let alt_port = port_manager.find_available_port(p2p_port + 1)?;
        println!(
            "    {} Using port {} instead",
            "→".dimmed(),
            alt_port.to_string().bright_cyan()
        );
    }

    let base_rpc_port = 28545 + ((instance_id - 1) * 10) as u16;
    let suggested_rpc = port_manager.find_available_port(base_rpc_port)?;

    let rpc_port: u16 = Input::with_theme(&theme)
        .with_prompt("RPC port")
        .default(suggested_rpc)
        .interact_text()
        .context("Failed to get RPC port")?;

    let rpc_enabled = Confirm::with_theme(&theme)
        .with_prompt("Enable RPC API?")
        .default(true)
        .interact()
        .context("Failed to get RPC enabled setting")?;

    // Mining configuration
    println!();
    println!("  {} Mining Configuration", "⛏️".bright_blue());
    println!();

    let mining_enabled = Confirm::with_theme(&theme)
        .with_prompt("Enable mining?")
        .default(false)
        .interact()
        .context("Failed to get mining enabled setting")?;

    let (mining_threads, mining_wallet) = if mining_enabled {
        let default_threads = num_cpus::get() as u32 / 2;
        let threads: u32 = Input::with_theme(&theme)
            .with_prompt("Mining threads")
            .default(default_threads)
            .interact_text()
            .context("Failed to get mining threads")?;

        let wallet: String = Input::with_theme(&theme)
            .with_prompt("Mining wallet address (optional)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to get mining wallet")?;

        (threads, wallet)
    } else {
        (num_cpus::get() as u32 / 2, String::new())
    };

    // Max peers
    let max_peers: u32 = Input::with_theme(&theme)
        .with_prompt("Maximum peers")
        .default(50u32)
        .interact_text()
        .context("Failed to get max peers")?;

    // Log level
    let log_levels = vec!["info", "debug", "warn", "error", "trace"];
    let log_level_choice = Select::with_theme(&theme)
        .with_prompt("Log level")
        .items(&log_levels)
        .default(0)
        .interact()
        .context("Failed to get log level")?;

    let log_level = log_levels[log_level_choice].to_string();

    // Summary
    println!();
    println!("  {} Configuration Summary", "📋".bright_blue());
    println!();
    println!(
        "    {} Instance: {}",
        "→".dimmed(),
        instance_id.to_string().bright_cyan()
    );
    println!(
        "    {} Data Dir: {}",
        "→".dimmed(),
        data_dir.display().to_string().bright_cyan()
    );
    println!(
        "    {} Network: {}",
        "→".dimmed(),
        network.bright_cyan()
    );
    println!(
        "    {} P2P Port: {}",
        "→".dimmed(),
        p2p_port.to_string().bright_cyan()
    );
    println!(
        "    {} RPC Port: {} ({})",
        "→".dimmed(),
        rpc_port.to_string().bright_cyan(),
        if rpc_enabled { "enabled" } else { "disabled" }
    );
    println!(
        "    {} Mining: {}",
        "→".dimmed(),
        if mining_enabled {
            format!("enabled ({} threads)", mining_threads).bright_green()
        } else {
            "disabled".dimmed()
        }
    );
    println!(
        "    {} Max Peers: {}",
        "→".dimmed(),
        max_peers.to_string().bright_cyan()
    );
    println!(
        "    {} Log Level: {}",
        "→".dimmed(),
        log_level.bright_cyan()
    );
    println!();

    let confirm = Confirm::with_theme(&theme)
        .with_prompt("Save this configuration?")
        .default(true)
        .interact()
        .context("Failed to get confirmation")?;

    if !confirm {
        anyhow::bail!("Configuration cancelled by user");
    }

    Ok(InfernoConfig {
        node: NodeConfig {
            instance_id,
            data_dir,
            log_level,
        },
        network: NetworkConfig {
            network,
            p2p_port,
            rpc_port,
            rpc_enabled,
            bootnodes: get_default_bootnodes("devnet"),
            max_peers,
        },
        mining: MiningConfig {
            enabled: mining_enabled,
            threads: mining_threads,
            wallet: mining_wallet,
        },
    })
}

fn get_default_bootnodes(network: &str) -> Vec<String> {
    match network {
        "devnet" => vec![
            "/ip4/209.38.137.105/tcp/30303/p2p/12D3KooWQGzw3hMqiL7bNDzRBfYKBjZ5vBrx9oQKGEMG4iczDH4W".to_string(),
            "/ip4/137.184.118.228/tcp/30303/p2p/12D3KooWQGzw3hMqiL7bNDzRBfYKBjZ5vBrx9oQKGEMG4iczDH4W".to_string(),
        ],
        "testnet" => vec![],
        "mainnet" => vec![],
        _ => vec![],
    }
}

