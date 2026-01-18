use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};
use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum RemoteCommand {
    /// Add a remote node
    Add {
        /// Remote name (e.g., prod-1)
        name: String,

        /// SSH connection string (user@host)
        connection: String,

        /// SSH port
        #[arg(short, long, default_value = "22")]
        port: u16,

        /// Path to SSH identity file
        #[arg(short, long)]
        identity: Option<PathBuf>,
    },

    /// Remove a remote node
    Remove {
        /// Remote name
        name: String,
    },

    /// List all remotes
    List,

    /// Show status of remote node(s)
    Status {
        /// Remote name (or "all" for all remotes)
        name: String,
    },

    /// Start remote node
    Start {
        /// Remote name
        name: String,

        /// Instance number on remote
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// Stop remote node
    Stop {
        /// Remote name
        name: String,

        /// Instance number on remote
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// Restart remote node
    Restart {
        /// Remote name
        name: String,

        /// Instance number on remote
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// View logs from remote node
    Logs {
        /// Remote name
        name: String,

        /// Instance number on remote
        #[arg(short, long, default_value = "1")]
        instance: u32,

        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        tail: u32,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },

    /// Open SSH shell to remote
    Shell {
        /// Remote name
        name: String,
    },

    /// Deploy/update inferno on remote
    Deploy {
        /// Remote name (or "all" for all remotes)
        name: String,

        /// Version to deploy
        #[arg(short, long, default_value = "latest")]
        version: String,
    },

    /// Execute a command on remote
    Exec {
        /// Remote name
        name: String,

        /// Command to execute
        command: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoteConfig {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub identity: Option<PathBuf>,
    pub inferno_path: String,
    pub data_dir: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct RemotesFile {
    pub remotes: HashMap<String, RemoteConfig>,
}

impl RemotesFile {
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let remotes: RemotesFile = toml::from_str(&content)?;
        Ok(remotes)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn path() -> Result<PathBuf> {
        let base = dirs::home_dir().context("Could not find home directory")?;
        Ok(base.join(".inferno").join("remotes.toml"))
    }
}

pub async fn run(command: RemoteCommand) -> Result<()> {
    match command {
        RemoteCommand::Add {
            name,
            connection,
            port,
            identity,
        } => add_remote(&name, &connection, port, identity).await,
        RemoteCommand::Remove { name } => remove_remote(&name).await,
        RemoteCommand::List => list_remotes().await,
        RemoteCommand::Status { name } => show_status(&name).await,
        RemoteCommand::Start { name, instance } => start_remote(&name, instance).await,
        RemoteCommand::Stop { name, instance } => stop_remote(&name, instance).await,
        RemoteCommand::Restart { name, instance } => restart_remote(&name, instance).await,
        RemoteCommand::Logs {
            name,
            instance,
            tail,
            follow,
        } => show_logs(&name, instance, tail, follow).await,
        RemoteCommand::Shell { name } => open_shell(&name).await,
        RemoteCommand::Deploy { name, version } => deploy(&name, &version).await,
        RemoteCommand::Exec { name, command } => exec(&name, command).await,
    }
}

async fn add_remote(
    name: &str,
    connection: &str,
    port: u16,
    identity: Option<PathBuf>,
) -> Result<()> {
    let mut remotes = RemotesFile::load()?;

    // Parse connection string (user@host)
    let parts: Vec<&str> = connection.split('@').collect();
    let (user, host) = if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        anyhow::bail!(
            "Invalid connection string. Expected format: user@host, got: {}",
            connection
        );
    };

    let theme = ColorfulTheme::default();

    // Prompt for inferno path on remote
    let inferno_path: String = Input::with_theme(&theme)
        .with_prompt("Path to inferno binary on remote")
        .default("/usr/local/bin/inferno".to_string())
        .interact_text()?;

    // Prompt for data directory on remote
    let data_dir: String = Input::with_theme(&theme)
        .with_prompt("Data directory on remote")
        .default("/var/lib/inferno".to_string())
        .interact_text()?;

    let config = RemoteConfig {
        host,
        user,
        port,
        identity,
        inferno_path,
        data_dir,
    };

    // Test connection
    println!(
        "  {} Testing connection to {}...",
        "→".dimmed(),
        name.bright_cyan()
    );

    match test_connection(&config).await {
        Ok(_) => {
            println!(
                "  {} Connection successful!",
                "✓".bright_green()
            );
        }
        Err(e) => {
            println!(
                "  {} Connection failed: {}",
                "!".bright_yellow(),
                e
            );
            let proceed = Confirm::with_theme(&theme)
                .with_prompt("Save remote anyway?")
                .default(false)
                .interact()?;

            if !proceed {
                return Ok(());
            }
        }
    }

    remotes.remotes.insert(name.to_string(), config);
    remotes.save()?;

    println!(
        "  {} Remote '{}' added",
        "✓".bright_green(),
        name.bright_cyan()
    );

    Ok(())
}

async fn remove_remote(name: &str) -> Result<()> {
    let mut remotes = RemotesFile::load()?;

    if remotes.remotes.remove(name).is_none() {
        println!(
            "  {} Remote '{}' not found",
            "!".bright_yellow(),
            name.bright_cyan()
        );
        return Ok(());
    }

    remotes.save()?;

    println!(
        "  {} Remote '{}' removed",
        "✓".bright_green(),
        name.bright_cyan()
    );

    Ok(())
}

async fn list_remotes() -> Result<()> {
    let remotes = RemotesFile::load()?;

    if remotes.remotes.is_empty() {
        println!(
            "  {} No remotes configured",
            "ℹ".bright_blue()
        );
        println!(
            "    {} Add one with: {}",
            "→".dimmed(),
            "inferno remote add <name> <user@host>".bright_white()
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} Remotes ({} total)",
        "🌐".bright_blue(),
        remotes.remotes.len()
    );
    println!();

    for (name, config) in &remotes.remotes {
        println!(
            "  {} {}",
            "→".bright_cyan(),
            name.bright_white().bold()
        );
        println!(
            "      Host: {}@{}:{}",
            config.user.dimmed(),
            config.host.bright_white(),
            config.port.to_string().dimmed()
        );
        if let Some(ref identity) = config.identity {
            println!(
                "      Identity: {}",
                identity.display().to_string().dimmed()
            );
        }
        println!(
            "      Inferno: {}",
            config.inferno_path.dimmed()
        );
        println!();
    }

    Ok(())
}

async fn show_status(name: &str) -> Result<()> {
    let remotes = RemotesFile::load()?;

    if name == "all" {
        for (remote_name, config) in &remotes.remotes {
            show_remote_status(remote_name, config).await?;
            println!();
        }
    } else {
        let config = remotes
            .remotes
            .get(name)
            .context(format!("Remote '{}' not found", name))?;
        show_remote_status(name, config).await?;
    }

    Ok(())
}

async fn show_remote_status(name: &str, config: &RemoteConfig) -> Result<()> {
    println!(
        "  {} Checking {}...",
        "→".dimmed(),
        name.bright_cyan()
    );

    let session = connect_ssh(config).await?;

    // Check if inferno is installed
    let output = exec_ssh(&session, &format!("which {} 2>/dev/null || echo 'not found'", config.inferno_path))?;

    if output.trim() == "not found" {
        println!(
            "    {} Inferno: {}",
            "→".dimmed(),
            "Not installed".bright_red()
        );
        return Ok(());
    }

    println!(
        "    {} Inferno: {}",
        "→".dimmed(),
        "Installed".bright_green()
    );

    // Get version
    let version = exec_ssh(&session, &format!("{} --version 2>/dev/null || echo 'unknown'", config.inferno_path))?;
    println!(
        "    {} Version: {}",
        "→".dimmed(),
        version.trim().bright_white()
    );

    // Check running instances
    let status = exec_ssh(&session, &format!("{} list --json 2>/dev/null || echo '[]'", config.inferno_path))?;

    if let Ok(instances) = serde_json::from_str::<Vec<serde_json::Value>>(&status) {
        if instances.is_empty() {
            println!(
                "    {} Instances: {}",
                "→".dimmed(),
                "None running".dimmed()
            );
        } else {
            for inst in instances {
                let id = inst.get("instance_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let status = inst.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
                let status_color = if status == "Running" {
                    "🟢"
                } else {
                    "🔴"
                };
                println!(
                    "    {} Instance {}: {}",
                    status_color,
                    id.to_string().bright_cyan(),
                    status.bright_white()
                );
            }
        }
    }

    Ok(())
}

async fn start_remote(name: &str, instance: u32) -> Result<()> {
    let remotes = RemotesFile::load()?;
    let config = remotes
        .remotes
        .get(name)
        .context(format!("Remote '{}' not found", name))?;

    println!(
        "  {} Starting instance {} on {}...",
        "🚀".bright_green(),
        instance.to_string().bright_cyan(),
        name.bright_white()
    );

    let session = connect_ssh(config).await?;
    let output = exec_ssh(
        &session,
        &format!("{} start --instance {}", config.inferno_path, instance),
    )?;

    println!("{}", output);

    Ok(())
}

async fn stop_remote(name: &str, instance: u32) -> Result<()> {
    let remotes = RemotesFile::load()?;
    let config = remotes
        .remotes
        .get(name)
        .context(format!("Remote '{}' not found", name))?;

    println!(
        "  {} Stopping instance {} on {}...",
        "🛑".bright_red(),
        instance.to_string().bright_cyan(),
        name.bright_white()
    );

    let session = connect_ssh(config).await?;
    let output = exec_ssh(
        &session,
        &format!("{} stop --instance {}", config.inferno_path, instance),
    )?;

    println!("{}", output);

    Ok(())
}

async fn restart_remote(name: &str, instance: u32) -> Result<()> {
    let remotes = RemotesFile::load()?;
    let config = remotes
        .remotes
        .get(name)
        .context(format!("Remote '{}' not found", name))?;

    println!(
        "  {} Restarting instance {} on {}...",
        "🔄".bright_blue(),
        instance.to_string().bright_cyan(),
        name.bright_white()
    );

    let session = connect_ssh(config).await?;
    let output = exec_ssh(
        &session,
        &format!("{} restart --instance {}", config.inferno_path, instance),
    )?;

    println!("{}", output);

    Ok(())
}

async fn show_logs(name: &str, instance: u32, tail: u32, follow: bool) -> Result<()> {
    let remotes = RemotesFile::load()?;
    let config = remotes
        .remotes
        .get(name)
        .context(format!("Remote '{}' not found", name))?;

    if follow {
        println!(
            "  {} Following logs on {}. Press {} to stop.",
            "📋".bright_blue(),
            name.bright_cyan(),
            "Ctrl+C".bright_yellow()
        );
        println!();

        // For follow mode, use SSH directly
        let ssh_args = build_ssh_args(config);
        let cmd = format!(
            "{} logs --instance {} --tail {} --follow",
            config.inferno_path, instance, tail
        );

        let mut child = std::process::Command::new("ssh")
            .args(&ssh_args)
            .arg(&cmd)
            .spawn()
            .context("Failed to start SSH")?;

        child.wait()?;
    } else {
        let session = connect_ssh(config).await?;
        let output = exec_ssh(
            &session,
            &format!(
                "{} logs --instance {} --tail {}",
                config.inferno_path, instance, tail
            ),
        )?;

        println!("{}", output);
    }

    Ok(())
}

async fn open_shell(name: &str) -> Result<()> {
    let remotes = RemotesFile::load()?;
    let config = remotes
        .remotes
        .get(name)
        .context(format!("Remote '{}' not found", name))?;

    println!(
        "  {} Opening SSH shell to {}...",
        "🐚".bright_blue(),
        name.bright_cyan()
    );
    println!();

    let ssh_args = build_ssh_args(config);

    let status = std::process::Command::new("ssh")
        .args(&ssh_args)
        .status()
        .context("Failed to start SSH")?;

    if !status.success() {
        anyhow::bail!("SSH exited with error");
    }

    Ok(())
}

async fn deploy(name: &str, version: &str) -> Result<()> {
    let remotes = RemotesFile::load()?;

    if name == "all" {
        for (remote_name, config) in &remotes.remotes {
            deploy_to_remote(remote_name, config, version).await?;
            println!();
        }
    } else {
        let config = remotes
            .remotes
            .get(name)
            .context(format!("Remote '{}' not found", name))?;
        deploy_to_remote(name, config, version).await?;
    }

    Ok(())
}

async fn deploy_to_remote(name: &str, config: &RemoteConfig, version: &str) -> Result<()> {
    println!(
        "  {} Deploying to {}...",
        "📦".bright_blue(),
        name.bright_cyan()
    );

    let session = connect_ssh(config).await?;

    // Determine architecture
    let arch = exec_ssh(&session, "uname -m")?;
    let arch = arch.trim();

    let binary_name = match arch {
        "x86_64" => "inferno-cli-linux-x86_64.tar.gz",
        "aarch64" | "arm64" => "inferno-cli-linux-aarch64.tar.gz",
        _ => anyhow::bail!("Unsupported architecture: {}", arch),
    };

    let url = if version == "latest" {
        format!(
            "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/latest/download/{}",
            binary_name
        )
    } else {
        format!(
            "https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/download/{}/{}",
            version, binary_name
        )
    };

    // Download and install
    println!(
        "    {} Downloading {}...",
        "→".dimmed(),
        version.bright_white()
    );

    let install_script = format!(
        r#"
        cd /tmp && \
        curl -fsSL -o inferno.tar.gz '{}' && \
        tar -xzf inferno.tar.gz && \
        sudo mv inferno {} && \
        sudo chmod +x {} && \
        rm -f inferno.tar.gz && \
        {} --version
        "#,
        url, config.inferno_path, config.inferno_path, config.inferno_path
    );

    let output = exec_ssh(&session, &install_script)?;
    println!("{}", output);

    println!(
        "  {} Deploy complete",
        "✓".bright_green()
    );

    Ok(())
}

async fn exec(name: &str, command: Vec<String>) -> Result<()> {
    let remotes = RemotesFile::load()?;
    let config = remotes
        .remotes
        .get(name)
        .context(format!("Remote '{}' not found", name))?;

    let cmd = command.join(" ");

    println!(
        "  {} Executing on {}: {}",
        "→".dimmed(),
        name.bright_cyan(),
        cmd.bright_white()
    );
    println!();

    let session = connect_ssh(config).await?;
    let output = exec_ssh(&session, &cmd)?;

    println!("{}", output);

    Ok(())
}

async fn test_connection(config: &RemoteConfig) -> Result<()> {
    let session = connect_ssh(config).await?;
    exec_ssh(&session, "echo ok")?;
    Ok(())
}

async fn connect_ssh(config: &RemoteConfig) -> Result<Session> {
    let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
        .context("Failed to connect to SSH server")?;

    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    // Try identity file first
    if let Some(ref identity) = config.identity {
        let expanded = shellexpand::tilde(&identity.to_string_lossy());
        let path = PathBuf::from(expanded.to_string());
        if path.exists() {
            session.userauth_pubkey_file(&config.user, None, &path, None)?;
            if session.authenticated() {
                return Ok(session);
            }
        }
    }

    // Try SSH agent
    if let Ok(mut agent) = session.agent() {
        if agent.connect().is_ok() {
            let _ = agent.list_identities();
            for identity in agent.identities().unwrap_or_default() {
                if agent.userauth(&config.user, &identity).is_ok() && session.authenticated() {
                    return Ok(session);
                }
            }
        }
    }

    // Try default key locations
    let default_keys = vec![
        dirs::home_dir().map(|h| h.join(".ssh").join("id_rsa")),
        dirs::home_dir().map(|h| h.join(".ssh").join("id_ed25519")),
    ];

    for key_path in default_keys.into_iter().flatten() {
        if key_path.exists() {
            if session
                .userauth_pubkey_file(&config.user, None, &key_path, None)
                .is_ok()
                && session.authenticated()
            {
                return Ok(session);
            }
        }
    }

    // Fall back to password authentication
    let password = Password::new()
        .with_prompt(format!("Password for {}@{}", config.user, config.host))
        .interact()?;

    session.userauth_password(&config.user, &password)?;

    if !session.authenticated() {
        anyhow::bail!("SSH authentication failed");
    }

    Ok(session)
}

fn exec_ssh(session: &Session, command: &str) -> Result<String> {
    let mut channel = session.channel_session()?;
    channel.exec(command)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;

    channel.wait_close()?;

    Ok(output)
}

fn build_ssh_args(config: &RemoteConfig) -> Vec<String> {
    let mut args = vec![
        "-p".to_string(),
        config.port.to_string(),
    ];

    if let Some(ref identity) = config.identity {
        args.push("-i".to_string());
        args.push(identity.to_string_lossy().to_string());
    }

    args.push(format!("{}@{}", config.user, config.host));

    args
}
