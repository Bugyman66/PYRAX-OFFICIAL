use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Subcommand)]
pub enum ServiceCommand {
    /// Install as system service
    Install {
        /// Instance number
        #[arg(short, long, default_value = "1")]
        instance: u32,

        /// Service user
        #[arg(short, long)]
        user: Option<String>,
    },

    /// Uninstall system service
    Uninstall {
        /// Instance number
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// Start the service
    Start {
        /// Instance number
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// Stop the service
    Stop {
        /// Instance number
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// Restart the service
    Restart {
        /// Instance number
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// Show service status
    Status {
        /// Instance number
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// Enable service to start on boot
    Enable {
        /// Instance number
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// Disable service from starting on boot
    Disable {
        /// Instance number
        #[arg(short, long, default_value = "1")]
        instance: u32,
    },

    /// View service logs via journalctl
    Logs {
        /// Instance number
        #[arg(short, long, default_value = "1")]
        instance: u32,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: u32,
    },
}

pub async fn run(command: ServiceCommand) -> Result<()> {
    // Check platform
    #[cfg(target_os = "macos")]
    {
        run_launchd(command).await
    }

    #[cfg(target_os = "linux")]
    {
        run_systemd(command).await
    }

    #[cfg(target_os = "windows")]
    {
        run_windows(command).await
    }
}

#[cfg(target_os = "linux")]
async fn run_systemd(command: ServiceCommand) -> Result<()> {
    match command {
        ServiceCommand::Install { instance, user } => install_systemd(instance, user).await,
        ServiceCommand::Uninstall { instance } => uninstall_systemd(instance).await,
        ServiceCommand::Start { instance } => systemctl("start", instance).await,
        ServiceCommand::Stop { instance } => systemctl("stop", instance).await,
        ServiceCommand::Restart { instance } => systemctl("restart", instance).await,
        ServiceCommand::Status { instance } => systemctl("status", instance).await,
        ServiceCommand::Enable { instance } => systemctl("enable", instance).await,
        ServiceCommand::Disable { instance } => systemctl("disable", instance).await,
        ServiceCommand::Logs {
            instance,
            follow,
            lines,
        } => journalctl(instance, follow, lines).await,
    }
}

#[cfg(target_os = "linux")]
async fn install_systemd(instance: u32, user: Option<String>) -> Result<()> {
    let service_name = format!("inferno@{}", instance);
    let service_path = PathBuf::from("/etc/systemd/system/inferno@.service");

    // Check if running as root
    if !is_root() {
        println!(
            "  {} This command requires root privileges.",
            "!".bright_yellow()
        );
        println!(
            "    {} Run: {}",
            "→".dimmed(),
            format!("sudo inferno service install --instance {}", instance).bright_white()
        );
        return Ok(());
    }

    let user = user.unwrap_or_else(|| std::env::var("SUDO_USER").unwrap_or_else(|_| "root".to_string()));

    // Find inferno binary path
    let inferno_path = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("/usr/local/bin/inferno"));

    let service_content = format!(
        r#"[Unit]
Description=Inferno Node Instance %i
Documentation=https://github.com/PYRAX-Chain/PYRAX-OFFICIAL
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User={user}
Group={user}
ExecStart={inferno} start --instance %i --foreground
ExecStop={inferno} stop --instance %i
Restart=always
RestartSec=10
TimeoutStopSec=30

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/inferno /home/{user}/.inferno

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
"#,
        user = user,
        inferno = inferno_path.display(),
    );

    println!(
        "  {} Installing systemd service...",
        "→".dimmed()
    );

    // Write service file
    fs::write(&service_path, service_content).context("Failed to write service file")?;

    // Reload systemd
    Command::new("systemctl")
        .arg("daemon-reload")
        .status()
        .context("Failed to reload systemd")?;

    println!(
        "  {} Service installed: {}",
        "✓".bright_green(),
        service_name.bright_cyan()
    );
    println!();
    println!("  {} Next steps:", "📖".bright_blue());
    println!(
        "    {} Enable on boot: {}",
        "1.".dimmed(),
        format!("sudo systemctl enable inferno@{}", instance).bright_white()
    );
    println!(
        "    {} Start service: {}",
        "2.".dimmed(),
        format!("sudo systemctl start inferno@{}", instance).bright_white()
    );
    println!(
        "    {} View status: {}",
        "3.".dimmed(),
        format!("sudo systemctl status inferno@{}", instance).bright_white()
    );

    Ok(())
}

#[cfg(target_os = "linux")]
async fn uninstall_systemd(instance: u32) -> Result<()> {
    if !is_root() {
        println!(
            "  {} This command requires root privileges.",
            "!".bright_yellow()
        );
        return Ok(());
    }

    let service_name = format!("inferno@{}", instance);

    // Stop and disable service
    let _ = Command::new("systemctl")
        .args(["stop", &service_name])
        .status();

    let _ = Command::new("systemctl")
        .args(["disable", &service_name])
        .status();

    // Remove service file (template)
    let service_path = PathBuf::from("/etc/systemd/system/inferno@.service");
    if service_path.exists() {
        fs::remove_file(&service_path)?;
    }

    // Reload systemd
    Command::new("systemctl")
        .arg("daemon-reload")
        .status()?;

    println!(
        "  {} Service uninstalled",
        "✓".bright_green()
    );

    Ok(())
}

#[cfg(target_os = "linux")]
async fn systemctl(action: &str, instance: u32) -> Result<()> {
    let service_name = format!("inferno@{}", instance);

    let need_sudo = !is_root() && matches!(action, "start" | "stop" | "restart" | "enable" | "disable");

    let status = if need_sudo {
        Command::new("sudo")
            .args(["systemctl", action, &service_name])
            .status()
    } else {
        Command::new("systemctl")
            .args([action, &service_name])
            .status()
    };

    match status {
        Ok(s) if s.success() => {
            let icon = match action {
                "start" => "🚀",
                "stop" => "🛑",
                "restart" => "🔄",
                "enable" => "✅",
                "disable" => "⭕",
                _ => "→",
            };
            println!(
                "  {} Service {} {}",
                icon,
                service_name.bright_cyan(),
                format!("{}ed", action).bright_green()
            );
        }
        Ok(_) => {
            println!(
                "  {} Action '{}' failed for {}",
                "!".bright_yellow(),
                action,
                service_name.bright_cyan()
            );
        }
        Err(e) => {
            anyhow::bail!("Failed to run systemctl: {}", e);
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
async fn journalctl(instance: u32, follow: bool, lines: u32) -> Result<()> {
    let service_name = format!("inferno@{}", instance);

    let mut args = vec![
        "-u".to_string(),
        service_name,
        "-n".to_string(),
        lines.to_string(),
        "--no-pager".to_string(),
    ];

    if follow {
        args.push("-f".to_string());
    }

    let status = Command::new("journalctl")
        .args(&args)
        .status()
        .context("Failed to run journalctl")?;

    if !status.success() {
        anyhow::bail!("journalctl exited with error");
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn run_launchd(command: ServiceCommand) -> Result<()> {
    match command {
        ServiceCommand::Install { instance, user: _ } => install_launchd(instance).await,
        ServiceCommand::Uninstall { instance } => uninstall_launchd(instance).await,
        ServiceCommand::Start { instance } => launchctl("load", instance).await,
        ServiceCommand::Stop { instance } => launchctl("unload", instance).await,
        ServiceCommand::Restart { instance } => {
            launchctl("unload", instance).await?;
            launchctl("load", instance).await
        }
        ServiceCommand::Status { instance } => launchctl_status(instance).await,
        ServiceCommand::Enable { instance } => {
            println!(
                "  {} macOS services are enabled by default when loaded",
                "ℹ".bright_blue()
            );
            Ok(())
        }
        ServiceCommand::Disable { instance } => launchctl("unload", instance).await,
        ServiceCommand::Logs {
            instance,
            follow,
            lines,
        } => show_macos_logs(instance, follow, lines).await,
    }
}

#[cfg(target_os = "macos")]
async fn install_launchd(instance: u32) -> Result<()> {
    let plist_name = format!("org.pyrax.inferno.{}", instance);
    let plist_path = dirs::home_dir()
        .context("Could not find home directory")?
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", plist_name));

    // Ensure directory exists
    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let inferno_path = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("/usr/local/bin/inferno"));

    let log_dir = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".inferno/logs");
    fs::create_dir_all(&log_dir)?;

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{inferno}</string>
        <string>start</string>
        <string>--instance</string>
        <string>{instance}</string>
        <string>--foreground</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log_dir}/instance-{instance}.log</string>
    <key>StandardErrorPath</key>
    <string>{log_dir}/instance-{instance}.err</string>
</dict>
</plist>
"#,
        label = plist_name,
        inferno = inferno_path.display(),
        instance = instance,
        log_dir = log_dir.display(),
    );

    fs::write(&plist_path, plist_content)?;

    println!(
        "  {} Service installed: {}",
        "✓".bright_green(),
        plist_name.bright_cyan()
    );
    println!();
    println!("  {} Start with: {}",
        "→".dimmed(),
        format!("inferno service start --instance {}", instance).bright_white()
    );

    Ok(())
}

#[cfg(target_os = "macos")]
async fn uninstall_launchd(instance: u32) -> Result<()> {
    let plist_name = format!("org.pyrax.inferno.{}", instance);
    let plist_path = dirs::home_dir()
        .context("Could not find home directory")?
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", plist_name));

    // Unload first
    let _ = launchctl("unload", instance).await;

    // Remove plist
    if plist_path.exists() {
        fs::remove_file(&plist_path)?;
    }

    println!(
        "  {} Service uninstalled",
        "✓".bright_green()
    );

    Ok(())
}

#[cfg(target_os = "macos")]
async fn launchctl(action: &str, instance: u32) -> Result<()> {
    let plist_name = format!("org.pyrax.inferno.{}", instance);
    let plist_path = dirs::home_dir()
        .context("Could not find home directory")?
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", plist_name));

    let status = Command::new("launchctl")
        .args([action, &plist_path.to_string_lossy()])
        .status()
        .context("Failed to run launchctl")?;

    if status.success() {
        let icon = if action == "load" { "🚀" } else { "🛑" };
        println!(
            "  {} Service {}",
            icon,
            if action == "load" { "started" } else { "stopped" }.bright_green()
        );
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn launchctl_status(instance: u32) -> Result<()> {
    let plist_name = format!("org.pyrax.inferno.{}", instance);

    let output = Command::new("launchctl")
        .args(["list", &plist_name])
        .output()
        .context("Failed to run launchctl")?;

    if output.status.success() {
        println!(
            "  {} Service {} is {}",
            "🟢",
            plist_name.bright_cyan(),
            "running".bright_green()
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    } else {
        println!(
            "  {} Service {} is {}",
            "🔴",
            plist_name.bright_cyan(),
            "not running".bright_red()
        );
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn show_macos_logs(instance: u32, follow: bool, lines: u32) -> Result<()> {
    let log_path = dirs::home_dir()
        .context("Could not find home directory")?
        .join(format!(".inferno/logs/instance-{}.log", instance));

    if !log_path.exists() {
        println!(
            "  {} Log file not found: {}",
            "!".bright_yellow(),
            log_path.display()
        );
        return Ok(());
    }

    let mut args = vec![
        "-n".to_string(),
        lines.to_string(),
    ];

    if follow {
        args.push("-f".to_string());
    }

    args.push(log_path.to_string_lossy().to_string());

    let status = Command::new("tail")
        .args(&args)
        .status()
        .context("Failed to run tail")?;

    if !status.success() {
        anyhow::bail!("tail exited with error");
    }

    Ok(())
}

#[cfg(target_os = "windows")]
async fn run_windows(command: ServiceCommand) -> Result<()> {
    println!(
        "  {} Windows service management is not yet implemented.",
        "ℹ".bright_blue()
    );
    println!(
        "    {} Use {} to run nodes on Windows.",
        "→".dimmed(),
        "inferno start".bright_white()
    );
    println!(
        "    {} For background operation, use: {}",
        "→".dimmed(),
        "Start-Process -WindowStyle Hidden inferno start".bright_white()
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn is_root() -> bool {
    unsafe { nix::libc::geteuid() == 0 }
}

#[cfg(not(target_os = "linux"))]
fn is_root() -> bool {
    false
}
