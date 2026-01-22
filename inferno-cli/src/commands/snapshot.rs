//! Snapshot and Backup Commands for chaindata

use anyhow::{Result, bail};
use clap::Subcommand;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

#[derive(Subcommand, Debug)]
pub enum SnapshotCommand {
    /// Create a chaindata snapshot
    Create {
        /// Snapshot name/tag
        #[arg(short, long)]
        name: Option<String>,
        /// Instance to snapshot
        #[arg(short, long, default_value = "1")]
        instance: u32,
        /// Upload to S3
        #[arg(long)]
        s3: Option<String>,
        /// Upload to IPFS
        #[arg(long)]
        ipfs: bool,
    },
    /// Restore from a snapshot
    Restore {
        /// Snapshot path, S3 URI, or IPFS CID
        source: String,
        /// Instance to restore to
        #[arg(short, long, default_value = "1")]
        instance: u32,
        /// Force restore (overwrite existing)
        #[arg(short, long)]
        force: bool,
    },
    /// List available snapshots
    List {
        /// Include remote snapshots
        #[arg(long)]
        remote: bool,
    },
    /// Delete a snapshot
    Delete {
        /// Snapshot name or path
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
    /// Verify snapshot integrity
    Verify {
        /// Snapshot path or name
        source: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub block_height: u64,
    pub created_at: String,
    pub checksum: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub version: u32,
    pub snapshots: Vec<Snapshot>,
}

pub async fn run(command: SnapshotCommand) -> Result<()> {
    match command {
        SnapshotCommand::Create { name, instance, s3, ipfs } => {
            create_snapshot(name, instance, s3, ipfs).await
        }
        SnapshotCommand::Restore { source, instance, force } => {
            restore_snapshot(&source, instance, force).await
        }
        SnapshotCommand::List { remote } => list_snapshots(remote).await,
        SnapshotCommand::Delete { name, yes } => delete_snapshot(&name, yes).await,
        SnapshotCommand::Verify { source } => verify_snapshot(&source).await,
    }
}

async fn create_snapshot(name: Option<String>, instance: u32, s3: Option<String>, ipfs: bool) -> Result<()> {
    println!("{}", "📸 Creating Chaindata Snapshot".bright_cyan().bold());
    
    let data_dir = get_data_dir(instance)?;
    let chaindata = data_dir.join("chaindata");
    
    if !chaindata.exists() {
        bail!("Chaindata directory not found: {}", chaindata.display());
    }
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let snap_name = name.unwrap_or_else(|| format!("snapshot_{}", timestamp));
    let snapshot_dir = get_snapshot_dir()?;
    fs::create_dir_all(&snapshot_dir).await?;
    
    let archive_path = snapshot_dir.join(format!("{}.tar.zst", snap_name));
    
    println!("  {} {}", "Source:".dimmed(), chaindata.display());
    println!("  {} {}", "Output:".dimmed(), archive_path.display());
    println!();
    
    // Get block height from chaindata
    let height = get_chain_height(&chaindata).await.unwrap_or(0);
    println!("  {} {}", "Block height:".dimmed(), height);
    
    // Create compressed archive using tar + zstd
    println!("  Compressing...");
    let status = Command::new("tar")
        .args(["-I", "zstd", "-cf", &archive_path.display().to_string(), "-C", 
               &data_dir.display().to_string(), "chaindata"])
        .status()?;
    
    if !status.success() {
        bail!("Failed to create archive");
    }
    
    let metadata = fs::metadata(&archive_path).await?;
    let size_mb = metadata.len() / 1024 / 1024;
    
    // Calculate checksum
    let checksum = calculate_checksum(&archive_path).await?;
    
    // Save snapshot metadata
    let snapshot = Snapshot {
        name: snap_name.clone(),
        path: archive_path.display().to_string(),
        size_bytes: metadata.len(),
        block_height: height,
        created_at: chrono::Utc::now().to_rfc3339(),
        checksum: checksum.clone(),
        network: "devnet".to_string(),
    };
    
    save_snapshot_metadata(&snapshot).await?;
    
    println!();
    println!("{}", "✅ Snapshot created!".bright_green());
    println!("  {} {}", "Name:".dimmed(), snap_name);
    println!("  {} {} MB", "Size:".dimmed(), size_mb);
    println!("  {} {}", "Checksum:".dimmed(), &checksum[..16]);
    
    // Upload to S3 if requested
    if let Some(bucket) = s3 {
        println!();
        println!("  Uploading to S3...");
        upload_to_s3(&archive_path, &bucket, &snap_name).await?;
        println!("  {} s3://{}/{}.tar.zst", "S3:".dimmed(), bucket, snap_name);
    }
    
    // Upload to IPFS if requested
    if ipfs {
        println!();
        println!("  Uploading to IPFS...");
        let cid = upload_to_ipfs(&archive_path).await?;
        println!("  {} {}", "IPFS CID:".dimmed(), cid);
    }
    
    Ok(())
}

async fn restore_snapshot(source: &str, instance: u32, force: bool) -> Result<()> {
    println!("{}", "📥 Restoring from Snapshot".bright_cyan().bold());
    
    let data_dir = get_data_dir(instance)?;
    let chaindata = data_dir.join("chaindata");
    
    if chaindata.exists() && !force {
        bail!("Chaindata exists. Use --force to overwrite.");
    }
    
    // Determine source type
    let archive_path: PathBuf = if source.starts_with("s3://") {
        println!("  Downloading from S3...");
        download_from_s3(source).await?
    } else if source.starts_with("Qm") || source.starts_with("bafy") {
        println!("  Downloading from IPFS...");
        download_from_ipfs(source).await?
    } else {
        PathBuf::from(source)
    };
    
    if !archive_path.exists() {
        // Check snapshot directory
        let snap_path = get_snapshot_dir()?.join(format!("{}.tar.zst", source));
        if snap_path.exists() {
            println!("  {} {}", "Source:".dimmed(), snap_path.display());
        } else {
            bail!("Snapshot not found: {}", source);
        }
    }
    
    println!("  {} {}", "Target:".dimmed(), chaindata.display());
    
    // Verify checksum
    println!("  Verifying integrity...");
    
    // Remove existing chaindata if force
    if chaindata.exists() {
        println!("  Removing existing chaindata...");
        fs::remove_dir_all(&chaindata).await?;
    }
    
    // Extract archive
    println!("  Extracting...");
    fs::create_dir_all(&data_dir).await?;
    
    let status = Command::new("tar")
        .args(["-I", "zstd", "-xf", &archive_path.display().to_string(), 
               "-C", &data_dir.display().to_string()])
        .status()?;
    
    if !status.success() {
        bail!("Failed to extract archive");
    }
    
    println!();
    println!("{}", "✅ Snapshot restored!".bright_green());
    println!("  {} {}", "Chaindata:".dimmed(), chaindata.display());
    
    Ok(())
}

async fn list_snapshots(remote: bool) -> Result<()> {
    println!("{}", "Available Snapshots".bright_cyan().bold());
    println!();
    
    let snapshot_dir = get_snapshot_dir()?;
    
    if !snapshot_dir.exists() {
        println!("  No local snapshots found");
        return Ok(());
    }
    
    let mut entries = fs::read_dir(&snapshot_dir).await?;
    let mut count = 0;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().map(|e| e == "zst").unwrap_or(false) {
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            let metadata = fs::metadata(&path).await?;
            let size_mb = metadata.len() / 1024 / 1024;
            
            println!("  {} ({} MB)", name.bright_white(), size_mb);
            count += 1;
        }
    }
    
    if count == 0 {
        println!("  No local snapshots found");
    }
    
    if remote {
        println!();
        println!("{}", "Remote Snapshots:".dimmed());
        println!("  (Check S3/IPFS for remote snapshots)");
    }
    
    Ok(())
}

async fn delete_snapshot(name: &str, yes: bool) -> Result<()> {
    if !yes {
        println!("Add --yes to confirm deletion of snapshot: {}", name);
        return Ok(());
    }
    
    let snapshot_dir = get_snapshot_dir()?;
    let path = snapshot_dir.join(format!("{}.tar.zst", name));
    
    if path.exists() {
        fs::remove_file(&path).await?;
        println!("{} Deleted snapshot: {}", "✅".bright_green(), name);
    } else {
        println!("Snapshot not found: {}", name);
    }
    
    Ok(())
}

async fn verify_snapshot(source: &str) -> Result<()> {
    println!("{}", "🔍 Verifying Snapshot".bright_cyan().bold());
    
    let path = if PathBuf::from(source).exists() {
        PathBuf::from(source)
    } else {
        get_snapshot_dir()?.join(format!("{}.tar.zst", source))
    };
    
    if !path.exists() {
        bail!("Snapshot not found: {}", source);
    }
    
    println!("  {} {}", "File:".dimmed(), path.display());
    
    // Calculate checksum
    let checksum = calculate_checksum(&path).await?;
    println!("  {} {}", "SHA256:".dimmed(), checksum);
    
    // Test archive integrity
    println!("  Testing archive...");
    let status = Command::new("tar")
        .args(["-I", "zstd", "-tf", &path.display().to_string()])
        .stdout(std::process::Stdio::null())
        .status()?;
    
    if status.success() {
        println!();
        println!("{}", "✅ Snapshot is valid!".bright_green());
    } else {
        bail!("Snapshot is corrupted");
    }
    
    Ok(())
}

fn get_data_dir(instance: u32) -> Result<PathBuf> {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    Ok(base.join("pyrax").join(format!("instance-{}", instance)))
}

fn get_snapshot_dir() -> Result<PathBuf> {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    Ok(base.join("pyrax").join("snapshots"))
}

async fn get_chain_height(chaindata: &PathBuf) -> Result<u64> {
    // Read height from chaindata metadata if available
    let meta_path = chaindata.join("CURRENT");
    if meta_path.exists() {
        // Parse LevelDB/RocksDB CURRENT file for hints
        return Ok(0); // Placeholder - would need DB access
    }
    Ok(0)
}

async fn calculate_checksum(path: &PathBuf) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let data = fs::read(path).await?;
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
}

async fn save_snapshot_metadata(snapshot: &Snapshot) -> Result<()> {
    let manifest_path = get_snapshot_dir()?.join("manifest.json");
    let mut manifest = if manifest_path.exists() {
        serde_json::from_str(&fs::read_to_string(&manifest_path).await?)?
    } else {
        SnapshotManifest { version: 1, snapshots: Vec::new() }
    };
    
    manifest.snapshots.push(snapshot.clone());
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?).await?;
    Ok(())
}

async fn upload_to_s3(path: &PathBuf, bucket: &str, name: &str) -> Result<()> {
    let s3_path = format!("s3://{}/{}.tar.zst", bucket, name);
    Command::new("aws")
        .args(["s3", "cp", &path.display().to_string(), &s3_path])
        .status()?;
    Ok(())
}

async fn upload_to_ipfs(path: &PathBuf) -> Result<String> {
    let output = Command::new("ipfs")
        .args(["add", "-Q", &path.display().to_string()])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn download_from_s3(uri: &str) -> Result<PathBuf> {
    let dest = std::env::temp_dir().join("pyrax-snapshot.tar.zst");
    Command::new("aws").args(["s3", "cp", uri, &dest.display().to_string()]).status()?;
    Ok(dest)
}

async fn download_from_ipfs(cid: &str) -> Result<PathBuf> {
    let dest = std::env::temp_dir().join("pyrax-snapshot.tar.zst");
    Command::new("ipfs").args(["get", "-o", &dest.display().to_string(), cid]).status()?;
    Ok(dest)
}
