use anyhow::{Context, Result};
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogOutput, LogsOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use clap::Subcommand;
use colored::Colorize;
use futures::StreamExt;
use std::collections::HashMap;
use std::io::Write;

const DOCKER_IMAGE: &str = "ghcr.io/pyrax-chain/inferno-node";
const CONTAINER_PREFIX: &str = "inferno-node";

#[derive(Subcommand)]
pub enum DockerCommand {
    /// Run a node in a Docker container
    Run {
        /// Run container in background (detached)
        #[arg(short, long)]
        detach: bool,

        /// Instance number for the container
        #[arg(short, long, default_value = "1")]
        instance: u32,

        /// Network to connect to
        #[arg(short, long, default_value = "devnet")]
        network: String,

        /// P2P port
        #[arg(long, default_value = "30303")]
        p2p_port: u16,

        /// RPC port
        #[arg(long, default_value = "28545")]
        rpc_port: u16,

        /// Enable mining
        #[arg(long)]
        mining: bool,

        /// Custom container name
        #[arg(long)]
        name: Option<String>,

        /// Docker image tag
        #[arg(long, default_value = "latest")]
        tag: String,

        /// Mount volume for persistent data
        #[arg(short, long)]
        volume: Option<String>,
    },

    /// List running containers
    #[command(name = "ps")]
    Ps {
        /// Show all containers (including stopped)
        #[arg(short, long)]
        all: bool,
    },

    /// View container logs
    Logs {
        /// Container name or instance number
        #[arg(default_value = "1")]
        container: String,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        tail: String,
    },

    /// Stop a container
    Stop {
        /// Container name or instance number
        #[arg(default_value = "1")]
        container: String,

        /// Stop all inferno containers
        #[arg(long)]
        all: bool,
    },

    /// Remove a container
    #[command(name = "rm")]
    Remove {
        /// Container name or instance number
        #[arg(default_value = "1")]
        container: String,

        /// Force remove running container
        #[arg(short, long)]
        force: bool,

        /// Remove all inferno containers
        #[arg(long)]
        all: bool,
    },

    /// Open a shell in a container
    Shell {
        /// Container name or instance number
        #[arg(default_value = "1")]
        container: String,
    },

    /// Pull the latest Docker image
    Pull {
        /// Image tag
        #[arg(default_value = "latest")]
        tag: String,
    },

    /// Show Docker info and status
    Info,
}

pub async fn run(command: DockerCommand) -> Result<()> {
    match command {
        DockerCommand::Run {
            detach,
            instance,
            network,
            p2p_port,
            rpc_port,
            mining,
            name,
            tag,
            volume,
        } => {
            run_container(
                detach, instance, &network, p2p_port, rpc_port, mining, name, &tag, volume,
            )
            .await
        }
        DockerCommand::Ps { all } => list_containers(all).await,
        DockerCommand::Logs {
            container,
            follow,
            tail,
        } => show_logs(&container, follow, &tail).await,
        DockerCommand::Stop { container, all } => stop_container(&container, all).await,
        DockerCommand::Remove {
            container,
            force,
            all,
        } => remove_container(&container, force, all).await,
        DockerCommand::Shell { container } => open_shell(&container).await,
        DockerCommand::Pull { tag } => pull_image(&tag).await,
        DockerCommand::Info => show_info().await,
    }
}

async fn get_docker() -> Result<Docker> {
    Docker::connect_with_local_defaults().context(
        "Failed to connect to Docker. Is Docker running?\n\
         On Linux/macOS: sudo systemctl start docker\n\
         On Windows: Start Docker Desktop",
    )
}

async fn run_container(
    detach: bool,
    instance: u32,
    network: &str,
    p2p_port: u16,
    rpc_port: u16,
    mining: bool,
    name: Option<String>,
    tag: &str,
    volume: Option<String>,
) -> Result<()> {
    let docker = get_docker().await?;

    let container_name = name.unwrap_or_else(|| format!("{}-{}", CONTAINER_PREFIX, instance));
    let image = format!("{}:{}", DOCKER_IMAGE, tag);

    println!(
        "  {} Starting container {}...",
        "🐳".bright_blue(),
        container_name.bright_cyan()
    );

    // Check if image exists, pull if not
    if docker.inspect_image(&image).await.is_err() {
        println!(
            "  {} Pulling image {}...",
            "→".dimmed(),
            image.bright_white()
        );
        pull_image(tag).await?;
    }

    // Build command arguments
    let mut cmd = vec![
        "--network".to_string(),
        network.to_string(),
        "--p2p-port".to_string(),
        "30303".to_string(),
        "--rpc-port".to_string(),
        "28545".to_string(),
        "--rpc".to_string(),
    ];

    if mining {
        cmd.push("--mining".to_string());
    }

    // Port bindings
    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        "30303/tcp".to_string(),
        Some(vec![bollard::service::PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(p2p_port.to_string()),
        }]),
    );
    port_bindings.insert(
        "28545/tcp".to_string(),
        Some(vec![bollard::service::PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(rpc_port.to_string()),
        }]),
    );

    // Volume bindings
    let binds = volume.map(|v| vec![format!("{}:/data", v)]);

    let host_config = bollard::service::HostConfig {
        port_bindings: Some(port_bindings),
        binds,
        restart_policy: Some(bollard::service::RestartPolicy {
            name: Some(bollard::service::RestartPolicyNameEnum::UNLESS_STOPPED),
            maximum_retry_count: None,
        }),
        ..Default::default()
    };

    let config = Config {
        image: Some(image.clone()),
        cmd: Some(cmd.iter().map(|s| s.as_str()).collect()),
        host_config: Some(host_config),
        exposed_ports: Some({
            let mut ports = HashMap::new();
            ports.insert("30303/tcp", HashMap::new());
            ports.insert("28545/tcp", HashMap::new());
            ports
        }),
        labels: Some({
            let mut labels = HashMap::new();
            labels.insert("inferno.instance", instance.to_string());
            labels.insert("inferno.network", network.to_string());
            labels
        }),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: container_name.as_str(),
        platform: None,
    };

    // Remove existing container with same name if exists
    let _ = docker
        .remove_container(
            &container_name,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await;

    let container = docker
        .create_container(Some(options), config)
        .await
        .context("Failed to create container")?;

    docker
        .start_container(&container.id, None::<StartContainerOptions<String>>)
        .await
        .context("Failed to start container")?;

    println!(
        "  {} Container {} started",
        "✓".bright_green(),
        container_name.bright_cyan()
    );
    println!(
        "    {} ID: {}",
        "→".dimmed(),
        container.id[..12].bright_yellow()
    );
    println!(
        "    {} P2P: {}",
        "→".dimmed(),
        format!("0.0.0.0:{}", p2p_port).bright_white()
    );
    println!(
        "    {} RPC: {}",
        "→".dimmed(),
        format!("http://localhost:{}", rpc_port).bright_white()
    );
    println!();
    println!(
        "  {} View logs: {}",
        "ℹ".bright_blue(),
        format!("inferno docker logs {}", container_name).bright_white()
    );

    if !detach {
        println!();
        println!(
            "  {} Attaching to logs. Press {} to detach.",
            "📋".bright_blue(),
            "Ctrl+C".bright_yellow()
        );
        println!();
        show_logs(&container_name, true, "50").await?;
    }

    Ok(())
}

async fn list_containers(all: bool) -> Result<()> {
    let docker = get_docker().await?;

    let mut filters = HashMap::new();
    filters.insert("name", vec![CONTAINER_PREFIX]);

    let options = ListContainersOptions {
        all,
        filters,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;

    if containers.is_empty() {
        println!(
            "  {} No inferno containers found",
            "ℹ".bright_blue()
        );
        if !all {
            println!(
                "    {} Use {} to show stopped containers",
                "→".dimmed(),
                "inferno docker ps --all".bright_white()
            );
        }
        return Ok(());
    }

    println!();
    println!(
        "  {} Inferno Containers ({} total)",
        "🐳".bright_blue(),
        containers.len()
    );
    println!();

    for container in containers {
        let id = container.id.map(|id| id[..12].to_string()).unwrap_or_default();
        let name = container
            .names
            .and_then(|n| n.first().cloned())
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_string();
        let state = container.state.unwrap_or_default();
        let status = container.status.unwrap_or_default();
        let image = container.image.unwrap_or_default();

        let state_icon = match state.as_str() {
            "running" => "🟢",
            "exited" => "🔴",
            "paused" => "🟡",
            _ => "⚪",
        };

        let state_text = match state.as_str() {
            "running" => state.bright_green(),
            "exited" => state.bright_red(),
            "paused" => state.bright_yellow(),
            _ => state.dimmed(),
        };

        println!(
            "  {} {} ({}) - {}",
            state_icon,
            name.bright_cyan(),
            id.bright_yellow(),
            state_text
        );
        println!(
            "      {} Image: {}",
            "→".dimmed(),
            image.dimmed()
        );
        println!(
            "      {} Status: {}",
            "→".dimmed(),
            status.dimmed()
        );

        // Show ports
        if let Some(ports) = container.ports {
            for port in ports {
                if let (Some(private), Some(public)) = (port.private_port, port.public_port) {
                    println!(
                        "      {} Port: {} → {}",
                        "→".dimmed(),
                        public.to_string().bright_white(),
                        private.to_string().dimmed()
                    );
                }
            }
        }
        println!();
    }

    Ok(())
}

async fn show_logs(container: &str, follow: bool, tail: &str) -> Result<()> {
    let docker = get_docker().await?;

    let container_name = resolve_container_name(container);

    let options = LogsOptions::<String> {
        follow,
        stdout: true,
        stderr: true,
        tail: tail.to_string(),
        ..Default::default()
    };

    let mut stream = docker.logs(&container_name, Some(options));

    while let Some(log) = stream.next().await {
        match log {
            Ok(output) => {
                let text = match output {
                    LogOutput::StdOut { message } => String::from_utf8_lossy(&message).to_string(),
                    LogOutput::StdErr { message } => String::from_utf8_lossy(&message).to_string(),
                    LogOutput::Console { message } => String::from_utf8_lossy(&message).to_string(),
                    LogOutput::StdIn { message } => String::from_utf8_lossy(&message).to_string(),
                };
                print!("{}", text);
                std::io::stdout().flush()?;
            }
            Err(e) => {
                eprintln!("Error reading logs: {}", e);
                break;
            }
        }
    }

    Ok(())
}

async fn stop_container(container: &str, all: bool) -> Result<()> {
    let docker = get_docker().await?;

    if all {
        let mut filters = HashMap::new();
        filters.insert("name", vec![CONTAINER_PREFIX]);

        let options = ListContainersOptions {
            all: false,
            filters,
            ..Default::default()
        };

        let containers = docker.list_containers(Some(options)).await?;

        println!(
            "  {} Stopping {} containers...",
            "🛑".bright_red(),
            containers.len()
        );

        for container in containers {
            if let Some(id) = container.id {
                let name = container
                    .names
                    .and_then(|n| n.first().cloned())
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string();

                docker
                    .stop_container(&id, Some(StopContainerOptions { t: 10 }))
                    .await?;

                println!(
                    "  {} Stopped {}",
                    "✓".bright_green(),
                    name.bright_cyan()
                );
            }
        }
    } else {
        let container_name = resolve_container_name(container);

        println!(
            "  {} Stopping {}...",
            "→".dimmed(),
            container_name.bright_cyan()
        );

        docker
            .stop_container(&container_name, Some(StopContainerOptions { t: 10 }))
            .await
            .context(format!("Failed to stop container: {}", container_name))?;

        println!(
            "  {} Container stopped",
            "✓".bright_green()
        );
    }

    Ok(())
}

async fn remove_container(container: &str, force: bool, all: bool) -> Result<()> {
    let docker = get_docker().await?;

    let options = RemoveContainerOptions {
        force,
        v: true,
        ..Default::default()
    };

    if all {
        let mut filters = HashMap::new();
        filters.insert("name", vec![CONTAINER_PREFIX]);

        let list_options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        let containers = docker.list_containers(Some(list_options)).await?;

        println!(
            "  {} Removing {} containers...",
            "🗑️".bright_red(),
            containers.len()
        );

        for container in containers {
            if let Some(id) = container.id {
                let name = container
                    .names
                    .and_then(|n| n.first().cloned())
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string();

                docker.remove_container(&id, Some(options.clone())).await?;

                println!(
                    "  {} Removed {}",
                    "✓".bright_green(),
                    name.bright_cyan()
                );
            }
        }
    } else {
        let container_name = resolve_container_name(container);

        println!(
            "  {} Removing {}...",
            "→".dimmed(),
            container_name.bright_cyan()
        );

        docker
            .remove_container(&container_name, Some(options))
            .await
            .context(format!("Failed to remove container: {}", container_name))?;

        println!(
            "  {} Container removed",
            "✓".bright_green()
        );
    }

    Ok(())
}

async fn open_shell(container: &str) -> Result<()> {
    let container_name = resolve_container_name(container);

    println!(
        "  {} Opening shell in {}...",
        "🐚".bright_blue(),
        container_name.bright_cyan()
    );
    println!();

    // Use docker exec directly via Command
    let status = std::process::Command::new("docker")
        .args(["exec", "-it", &container_name, "/bin/sh"])
        .status()
        .context("Failed to open shell. Is docker in PATH?")?;

    if !status.success() {
        anyhow::bail!("Shell exited with error");
    }

    Ok(())
}

async fn pull_image(tag: &str) -> Result<()> {
    let docker = get_docker().await?;

    let image = format!("{}:{}", DOCKER_IMAGE, tag);

    println!(
        "  {} Pulling {}...",
        "⬇️".bright_blue(),
        image.bright_cyan()
    );

    let options = CreateImageOptions {
        from_image: DOCKER_IMAGE,
        tag,
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(options), None, None);

    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => {
                if let Some(status) = info.status {
                    if let Some(progress) = info.progress {
                        print!("\r  {} {} {}", "→".dimmed(), status, progress);
                        std::io::stdout().flush()?;
                    } else {
                        println!("  {} {}", "→".dimmed(), status);
                    }
                }
            }
            Err(e) => {
                eprintln!("\n  {} Error: {}", "!".bright_red(), e);
            }
        }
    }

    println!();
    println!(
        "  {} Image pulled successfully",
        "✓".bright_green()
    );

    Ok(())
}

async fn show_info() -> Result<()> {
    let docker = get_docker().await?;

    let info = docker.info().await?;
    let version = docker.version().await?;

    println!();
    println!("  {} Docker Information", "🐳".bright_blue());
    println!();

    if let Some(v) = version.version {
        println!(
            "    {} Version: {}",
            "→".dimmed(),
            v.bright_white()
        );
    }

    if let Some(os) = info.operating_system {
        println!(
            "    {} OS: {}",
            "→".dimmed(),
            os.bright_white()
        );
    }

    if let Some(containers) = info.containers {
        println!(
            "    {} Containers: {}",
            "→".dimmed(),
            containers.to_string().bright_white()
        );
    }

    if let Some(images) = info.images {
        println!(
            "    {} Images: {}",
            "→".dimmed(),
            images.to_string().bright_white()
        );
    }

    println!();
    println!("  {} Inferno Image", "📦".bright_blue());
    println!(
        "    {} {}",
        "→".dimmed(),
        DOCKER_IMAGE.bright_white()
    );

    // Check if image is pulled
    let image = format!("{}:latest", DOCKER_IMAGE);
    match docker.inspect_image(&image).await {
        Ok(img) => {
            if let Some(size) = img.size {
                let size_mb = size / 1024 / 1024;
                println!(
                    "    {} Size: {} MB",
                    "→".dimmed(),
                    size_mb.to_string().bright_white()
                );
            }
            println!(
                "    {} Status: {}",
                "→".dimmed(),
                "Available".bright_green()
            );
        }
        Err(_) => {
            println!(
                "    {} Status: {}",
                "→".dimmed(),
                "Not pulled".bright_yellow()
            );
            println!(
                "    {} Run: {}",
                "ℹ".bright_blue(),
                "inferno docker pull".bright_white()
            );
        }
    }

    Ok(())
}

fn resolve_container_name(container: &str) -> String {
    // If it's a number, treat it as an instance ID
    if let Ok(instance) = container.parse::<u32>() {
        format!("{}-{}", CONTAINER_PREFIX, instance)
    } else {
        container.to_string()
    }
}
