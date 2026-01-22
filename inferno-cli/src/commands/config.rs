use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::process::Command;

use crate::config::InfernoConfig;
use crate::instance::InstanceManager;

pub async fn show(instance: u32, json: bool) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    if !instance_manager.instance_exists(instance) {
        println!(
            "  {} Instance {} not found",
            "!".bright_yellow(),
            instance.to_string().bright_cyan()
        );
        return Ok(());
    }

    let config = instance_manager.load_config(instance)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&config)?);
    } else {
        println!();
        println!(
            "  {} Configuration for Instance {}",
            "⚙️".bright_blue(),
            instance.to_string().bright_cyan()
        );
        println!();

        println!("  {} Node", "[node]".bright_yellow());
        println!(
            "    {} instance_id = {}",
            "→".dimmed(),
            config.node.instance_id.to_string().bright_white()
        );
        println!(
            "    {} data_dir = {}",
            "→".dimmed(),
            format!("\"{}\"", config.node.data_dir.display()).bright_white()
        );
        println!(
            "    {} log_level = {}",
            "→".dimmed(),
            format!("\"{}\"", config.node.log_level).bright_white()
        );
        println!();

        println!("  {} Network", "[network]".bright_yellow());
        println!(
            "    {} network = {}",
            "→".dimmed(),
            format!("\"{}\"", config.network.network).bright_white()
        );
        println!(
            "    {} p2p_port = {}",
            "→".dimmed(),
            config.network.p2p_port.to_string().bright_white()
        );
        println!(
            "    {} rpc_port = {}",
            "→".dimmed(),
            config.network.rpc_port.to_string().bright_white()
        );
        println!(
            "    {} rpc_enabled = {}",
            "→".dimmed(),
            config.network.rpc_enabled.to_string().bright_white()
        );
        println!(
            "    {} max_peers = {}",
            "→".dimmed(),
            config.network.max_peers.to_string().bright_white()
        );
        println!(
            "    {} bootnodes = [{}]",
            "→".dimmed(),
            if config.network.bootnodes.is_empty() {
                "".to_string()
            } else {
                format!("{} entries", config.network.bootnodes.len())
            }
            .dimmed()
        );
        println!();

        println!("  {} Mining", "[mining]".bright_yellow());
        println!(
            "    {} enabled = {}",
            "→".dimmed(),
            config.mining.enabled.to_string().bright_white()
        );
        println!(
            "    {} threads = {}",
            "→".dimmed(),
            config.mining.threads.to_string().bright_white()
        );
        println!(
            "    {} wallet = {}",
            "→".dimmed(),
            if config.mining.wallet.is_empty() {
                "\"\"".dimmed()
            } else {
                format!("\"{}\"", config.mining.wallet).bright_white()
            }
        );
        println!();

        let config_path = instance_manager.config_path(instance);
        println!(
            "  {} Config file: {}",
            "📁".dimmed(),
            config_path.display().to_string().dimmed()
        );
    }

    Ok(())
}

pub async fn set(key: &str, value: &str, instance: u32) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    if !instance_manager.instance_exists(instance) {
        println!(
            "  {} Instance {} not found",
            "!".bright_yellow(),
            instance.to_string().bright_cyan()
        );
        return Ok(());
    }

    let mut config = instance_manager.load_config(instance)?;

    // Parse the key path (e.g., "network.p2p_port")
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["node", "log_level"] => {
            config.node.log_level = value.to_string();
        }
        ["network", "network"] => {
            config.network.network = value.to_string();
        }
        ["network", "p2p_port"] => {
            config.network.p2p_port = value
                .parse()
                .context("Invalid port number")?;
        }
        ["network", "rpc_port"] => {
            config.network.rpc_port = value
                .parse()
                .context("Invalid port number")?;
        }
        ["network", "rpc_enabled"] => {
            config.network.rpc_enabled = value
                .parse()
                .context("Invalid boolean value")?;
        }
        ["network", "max_peers"] => {
            config.network.max_peers = value
                .parse()
                .context("Invalid number")?;
        }
        ["mining", "enabled"] => {
            config.mining.enabled = value
                .parse()
                .context("Invalid boolean value")?;
        }
        ["mining", "threads"] => {
            config.mining.threads = value
                .parse()
                .context("Invalid number")?;
        }
        ["mining", "wallet"] => {
            config.mining.wallet = value.to_string();
        }
        _ => {
            anyhow::bail!(
                "Unknown configuration key: {}. Valid keys:\n  \
                node.log_level\n  \
                network.network\n  \
                network.p2p_port\n  \
                network.rpc_port\n  \
                network.rpc_enabled\n  \
                network.max_peers\n  \
                mining.enabled\n  \
                mining.threads\n  \
                mining.wallet",
                key
            );
        }
    }

    instance_manager.save_config(instance, &config)?;

    println!(
        "  {} Set {} = {}",
        "✓".bright_green(),
        key.bright_cyan(),
        value.bright_white()
    );
    println!(
        "    {} Restart the node for changes to take effect",
        "ℹ".bright_blue()
    );

    Ok(())
}

pub async fn get(key: &str, instance: u32, json: bool) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    if !instance_manager.instance_exists(instance) {
        println!(
            "  {} Instance {} not found",
            "!".bright_yellow(),
            instance.to_string().bright_cyan()
        );
        return Ok(());
    }

    let config = instance_manager.load_config(instance)?;

    let parts: Vec<&str> = key.split('.').collect();

    let value: String = match parts.as_slice() {
        ["node", "instance_id"] => config.node.instance_id.to_string(),
        ["node", "data_dir"] => config.node.data_dir.display().to_string(),
        ["node", "log_level"] => config.node.log_level.clone(),
        ["network", "network"] => config.network.network.clone(),
        ["network", "p2p_port"] => config.network.p2p_port.to_string(),
        ["network", "rpc_port"] => config.network.rpc_port.to_string(),
        ["network", "rpc_enabled"] => config.network.rpc_enabled.to_string(),
        ["network", "max_peers"] => config.network.max_peers.to_string(),
        ["network", "bootnodes"] => serde_json::to_string(&config.network.bootnodes)?,
        ["mining", "enabled"] => config.mining.enabled.to_string(),
        ["mining", "threads"] => config.mining.threads.to_string(),
        ["mining", "wallet"] => config.mining.wallet.clone(),
        _ => {
            anyhow::bail!("Unknown configuration key: {}", key);
        }
    };

    if json {
        println!("{}", serde_json::json!({ key: value }));
    } else {
        println!("{}", value);
    }

    Ok(())
}

pub async fn reset(instance: u32, yes: bool) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    if !instance_manager.instance_exists(instance) {
        println!(
            "  {} Instance {} not found",
            "!".bright_yellow(),
            instance.to_string().bright_cyan()
        );
        return Ok(());
    }

    if !yes {
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Reset configuration for instance {} to defaults?",
                instance
            ))
            .default(false)
            .interact()
            .context("Failed to get confirmation")?;

        if !confirm {
            println!("  {} Reset cancelled", "→".dimmed());
            return Ok(());
        }
    }

    let config = InfernoConfig::default_for_instance(instance);
    instance_manager.save_config(instance, &config)?;

    println!(
        "  {} Configuration reset to defaults",
        "✓".bright_green()
    );

    Ok(())
}

pub async fn edit(instance: u32) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    if !instance_manager.instance_exists(instance) {
        println!(
            "  {} Instance {} not found",
            "!".bright_yellow(),
            instance.to_string().bright_cyan()
        );
        return Ok(());
    }

    let config_path = instance_manager.config_path(instance);

    // Try to find an editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });

    println!(
        "  {} Opening {} in {}...",
        "📝".bright_blue(),
        config_path.display().to_string().bright_cyan(),
        editor.bright_white()
    );

    let status = Command::new(&editor)
        .arg(&config_path)
        .status()
        .context(format!("Failed to open editor: {}", editor))?;

    if status.success() {
        println!(
            "  {} Configuration saved",
            "✓".bright_green()
        );
        println!(
            "    {} Restart the node for changes to take effect",
            "ℹ".bright_blue()
        );
    } else {
        println!(
            "  {} Editor exited with error",
            "!".bright_yellow()
        );
    }

    Ok(())
}
