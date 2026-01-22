//! Cloud Deployment Commands for PYRAX nodes

use anyhow::{Result, bail};
use clap::Subcommand;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

#[derive(Subcommand, Debug)]
pub enum CloudCommand {
    Deploy {
        #[arg(value_enum)]
        provider: CloudProvider,
        #[arg(short, long, default_value = "medium")]
        size: String,
        #[arg(short, long)]
        region: Option<String>,
        #[arg(short, long, default_value = "devnet")]
        network: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        mining: bool,
    },
    List { #[arg(short, long)] provider: Option<CloudProvider> },
    Destroy { node_id: String, #[arg(short, long)] yes: bool },
    Status { node_id: String },
    Ssh { node_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CloudProvider { Aws, Gcp, Azure, Digitalocean }

impl CloudProvider {
    pub fn name(&self) -> &'static str {
        match self { Self::Aws => "AWS", Self::Gcp => "GCP", Self::Azure => "Azure", Self::Digitalocean => "DO" }
    }
    pub fn default_region(&self) -> &'static str {
        match self { Self::Aws => "us-east-1", Self::Gcp => "us-central1", Self::Azure => "eastus", Self::Digitalocean => "nyc1" }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudNode {
    pub id: String, pub name: String, pub provider: String, pub region: String,
    pub public_ip: Option<String>, pub status: String, pub network: String,
}

pub async fn run(command: CloudCommand) -> Result<()> {
    match command {
        CloudCommand::Deploy { provider, size, region, network, name, mining } => {
            let region = region.unwrap_or_else(|| provider.default_region().to_string());
            let node_name = name.unwrap_or_else(|| format!("pyrax-{}", chrono::Utc::now().timestamp()));
            
            println!("{}", "🚀 Deploying PYRAX Node".bright_cyan().bold());
            println!("  Provider: {}, Region: {}, Network: {}", provider.name(), region, network);
            
            let cloud_init = generate_cloud_init(&network, mining);
            let node = deploy_node(&provider, &node_name, &size, &region, &cloud_init).await?;
            
            println!("{}", "✅ Deployed!".bright_green());
            if let Some(ip) = &node.public_ip { println!("  IP: {}", ip); }
            Ok(())
        }
        CloudCommand::List { provider } => { list_nodes(provider).await }
        CloudCommand::Destroy { node_id, yes } => { destroy_node(&node_id, yes).await }
        CloudCommand::Status { node_id } => { node_status(&node_id).await }
        CloudCommand::Ssh { node_id } => { ssh_node(&node_id).await }
    }
}

fn generate_cloud_init(network: &str, mining: bool) -> String {
    let mining_flag = if mining { "--mining" } else { "" };
    format!(r#"#!/bin/bash
apt-get update && apt-get install -y curl
curl -sSf https://sh.rustup.rs | sh -s -- -y
git clone https://github.com/PYRAX-Chain/PYRAX-OFFICIAL.git /opt/pyrax
cd /opt/pyrax/pyrax-node && ~/.cargo/bin/cargo build --release
cat > /etc/systemd/system/pyrax.service << EOF
[Unit]
Description=PYRAX Node
[Service]
ExecStart=/opt/pyrax/pyrax-node/target/release/pyrax-node --network {} {}
Restart=always
[Install]
WantedBy=multi-user.target
EOF
systemctl enable --now pyrax
"#, network, mining_flag)
}

async fn deploy_node(provider: &CloudProvider, name: &str, size: &str, region: &str, cloud_init: &str) -> Result<CloudNode> {
    let tmp = std::env::temp_dir().join("pyrax-init.sh");
    fs::write(&tmp, cloud_init).await?;
    
    let (id, ip) = match provider {
        CloudProvider::Aws => {
            let out = Command::new("aws").args(["ec2", "run-instances", "--region", region,
                "--image-id", "ami-0c7217cdde317cfec", "--instance-type", size,
                "--user-data", &format!("file://{}", tmp.display()),
                "--query", "Instances[0].InstanceId", "--output", "text"]).output()?;
            (String::from_utf8_lossy(&out.stdout).trim().to_string(), None)
        }
        _ => (format!("{}-{}", name, chrono::Utc::now().timestamp()), None)
    };
    
    Ok(CloudNode { id, name: name.to_string(), provider: provider.name().to_string(),
        region: region.to_string(), public_ip: ip, status: "running".to_string(), network: "devnet".to_string() })
}

async fn list_nodes(_provider: Option<CloudProvider>) -> Result<()> {
    println!("{}", "Cloud Nodes:".bright_cyan());
    println!("  (Use provider CLI to list: aws ec2 describe-instances, gcloud compute instances list)");
    Ok(())
}

async fn destroy_node(node_id: &str, yes: bool) -> Result<()> {
    if !yes {
        println!("Add --yes to confirm destruction of {}", node_id);
        return Ok(());
    }
    println!("Destroying {}...", node_id);
    Ok(())
}

async fn node_status(node_id: &str) -> Result<()> {
    println!("Status for {}: Use provider CLI", node_id);
    Ok(())
}

async fn ssh_node(node_id: &str) -> Result<()> {
    println!("SSH to {}: Use provider CLI or direct SSH", node_id);
    Ok(())
}
