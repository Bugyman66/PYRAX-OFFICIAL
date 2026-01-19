use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Duration;

use crate::instance::InstanceManager;

#[derive(Args)]
pub struct LogsArgs {
    /// Instance number (default: 1)
    #[arg(short, long, default_value = "1")]
    pub instance: u32,

    /// Number of lines to show (0 = all)
    #[arg(short, long, default_value = "50")]
    pub tail: usize,

    /// Follow log output (like tail -f)
    #[arg(short, long)]
    pub follow: bool,

    /// Filter by log level (debug, info, warn, error)
    #[arg(short, long)]
    pub level: Option<String>,

    /// Search for text in logs
    #[arg(short, long)]
    pub search: Option<String>,

    /// Show raw output without formatting
    #[arg(long)]
    pub raw: bool,
}

pub async fn run(args: LogsArgs) -> Result<()> {
    let instance_manager = InstanceManager::new()?;

    if !instance_manager.instance_exists(args.instance) {
        println!(
            "  {} Instance {} not found",
            "!".bright_yellow(),
            args.instance.to_string().bright_cyan()
        );
        return Ok(());
    }

    let config = instance_manager.load_config(args.instance)?;
    let log_file = config.node.data_dir.join("node.log");

    if !log_file.exists() {
        // Try alternative log locations
        let alt_log_file = config.node.data_dir.join("logs").join("node.log");
        if alt_log_file.exists() {
            return show_logs(&alt_log_file, &args).await;
        }

        println!(
            "  {} Log file not found: {}",
            "!".bright_yellow(),
            log_file.display()
        );
        println!(
            "    {} The node may not have been started yet.",
            "→".dimmed()
        );
        return Ok(());
    }

    show_logs(&log_file, &args).await
}

async fn show_logs(log_file: &PathBuf, args: &LogsArgs) -> Result<()> {
    if args.follow {
        follow_logs(log_file, args).await
    } else {
        show_tail_logs(log_file, args)
    }
}

fn show_tail_logs(log_file: &PathBuf, args: &LogsArgs) -> Result<()> {
    let file = File::open(log_file).context("Failed to open log file")?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

    let start = if args.tail > 0 && lines.len() > args.tail {
        lines.len() - args.tail
    } else {
        0
    };

    for line in &lines[start..] {
        if should_show_line(line, args) {
            print_log_line(line, args.raw);
        }
    }

    Ok(())
}

async fn follow_logs(log_file: &PathBuf, args: &LogsArgs) -> Result<()> {
    println!(
        "  {} Following logs for instance {}. Press {} to exit.",
        "📋".bright_blue(),
        args.instance.to_string().bright_cyan(),
        "Ctrl+C".bright_yellow()
    );
    println!();

    let mut file = File::open(log_file).context("Failed to open log file")?;

    // Seek to end minus tail lines
    if args.tail > 0 {
        let metadata = file.metadata()?;
        let file_size = metadata.len();

        // Simple approximation: seek back tail * 200 bytes
        let seek_pos = file_size.saturating_sub((args.tail * 200) as u64);
        file.seek(SeekFrom::Start(seek_pos))?;

        // Skip partial first line if we seeked
        if seek_pos > 0 {
            let mut reader = BufReader::new(&file);
            let mut discard = String::new();
            let _ = reader.read_line(&mut discard);
        }
    }

    let mut reader = BufReader::new(file);
    let mut buffer = String::new();

    // Set up for non-blocking input detection
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        buffer.clear();

        match reader.read_line(&mut buffer) {
            Ok(0) => {
                // No new data, wait a bit
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Ok(_) => {
                let line = buffer.trim_end();
                if should_show_line(line, args) {
                    print_log_line(line, args.raw);
                }
            }
            Err(e) => {
                eprintln!("Error reading log: {}", e);
                break;
            }
        }
    }

    println!();
    println!("  {} Log following stopped.", "ℹ".bright_blue());

    Ok(())
}

fn should_show_line(line: &str, args: &LogsArgs) -> bool {
    // Filter by level
    if let Some(ref level) = args.level {
        let level_upper = level.to_uppercase();
        if !line.to_uppercase().contains(&level_upper) {
            return false;
        }
    }

    // Filter by search
    if let Some(ref search) = args.search {
        if !line.to_lowercase().contains(&search.to_lowercase()) {
            return false;
        }
    }

    true
}

fn print_log_line(line: &str, raw: bool) {
    if raw {
        println!("{}", line);
        return;
    }

    // Parse and colorize log line
    let colored_line = colorize_log_line(line);
    println!("{}", colored_line);
}

fn colorize_log_line(line: &str) -> String {
    // Detect log level and colorize accordingly
    let line_upper = line.to_uppercase();

    // First, determine the log level
    let (level_prefix, message) = if line_upper.contains("ERROR") || line_upper.contains("ERR") {
        (format!("{}", "ERROR".bright_red().bold()), 
         line.replace("ERROR", "").replace("error", "").replace("ERR", "").trim().to_string())
    } else if line_upper.contains("WARN") {
        (format!("{}", "WARN".bright_yellow().bold()), 
         line.replace("WARN", "").replace("warn", "").trim().to_string())
    } else if line_upper.contains("DEBUG") {
        (format!("{}", "DEBUG".bright_blue()), 
         line.replace("DEBUG", "").replace("debug", "").trim().to_string())
    } else if line_upper.contains("TRACE") {
        (format!("{}", "TRACE".dimmed()), 
         line.replace("TRACE", "").replace("trace", "").trim().to_string())
    } else if line_upper.contains("INFO") {
        (format!("{}", "INFO".bright_green()), 
         line.replace("INFO", "").replace("info", "").trim().to_string())
    } else {
        (String::new(), line.to_string())
    };

    // Detect category and add appropriate icon
    // Categories match desktop app: p2p, block, rpc, mining, staking, node
    let category_icon = if message.contains("peer") || message.contains("Peer") || 
                           message.contains("P2P") || message.contains("Kademlia") ||
                           message.contains("Connected to") || message.contains("Disconnected") ||
                           message.contains("mDNS") || message.contains("DHT") ||
                           message.contains("GossipSub") || message.contains("mesh") ||
                           message.contains("MESH") || message.contains("relay") ||
                           message.contains("Relay") || message.contains("NAT") ||
                           message.contains("AutoNAT") || message.contains("bootnode") ||
                           message.contains("Bootnode") || message.contains("dial") ||
                           message.contains("Dial") || message.contains("listen") ||
                           message.contains("Listen") || message.contains("swarm") ||
                           message.contains("Swarm") || message.contains("circuit") ||
                           message.contains("reservation") || message.contains("UPnP") ||
                           message.contains("DCUtR") || message.contains("hole punch") {
        format!("{}", "[p2p]".bright_cyan())
    } else if message.contains("block") || message.contains("Block") || 
              message.contains("height") || message.contains("sync") ||
              message.contains("Sync") || message.contains("chain") ||
              message.contains("Chain") || message.contains("genesis") ||
              message.contains("Genesis") || message.contains("tip") ||
              message.contains("orphan") || message.contains("reorg") ||
              message.contains("fork") || message.contains("UTXO") {
        format!("{}", "[block]".bright_green())
    } else if message.contains("RPC") || message.contains("rpc") ||
              message.contains("JSON") || message.contains("request") ||
              message.contains("endpoint") || message.contains("API") ||
              message.contains("WebSocket") || message.contains("ws://") {
        format!("{}", "[rpc]".bright_blue())
    } else if message.contains("mining") || message.contains("Mining") ||
              message.contains("Stratum") || message.contains("worker") ||
              message.contains("Worker") || message.contains("KAWPOW") ||
              message.contains("BLAKE3") || message.contains("hashrate") ||
              message.contains("nonce") || message.contains("difficulty") ||
              message.contains("target") || message.contains("share") ||
              message.contains("mined") || message.contains("Mined") {
        format!("{}", "[mining]".bright_yellow())
    } else if message.contains("staking") || message.contains("Staking") ||
              message.contains("stake") || message.contains("ZK") ||
              message.contains("validator") || message.contains("Validator") ||
              message.contains("checkpoint") || message.contains("slash") ||
              message.contains("delegation") || message.contains("reward") {
        format!("{}", "[staking]".bright_magenta())
    } else {
        format!("{}", "[node]".white())
    };

    // Format the complete log line
    if level_prefix.is_empty() {
        format!("{} {}", category_icon, message)
    } else {
        format!("{:5} {} {}", level_prefix, category_icon, message)
    }
}

pub struct LogViewer {
    instance_id: u32,
    log_file: PathBuf,
    lines: Vec<String>,
    scroll_offset: usize,
    filter: Option<String>,
    search: Option<String>,
}

impl LogViewer {
    pub fn new(instance_id: u32, log_file: PathBuf) -> Self {
        Self {
            instance_id,
            log_file,
            lines: Vec::new(),
            scroll_offset: 0,
            filter: None,
            search: None,
        }
    }

    pub fn load_lines(&mut self) -> Result<()> {
        let file = File::open(&self.log_file)?;
        let reader = BufReader::new(file);
        self.lines = reader.lines().filter_map(|l| l.ok()).collect();
        Ok(())
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_offset = self.lines.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
    }

    pub fn scroll_to_end(&mut self) {
        self.scroll_offset = self.lines.len().saturating_sub(1);
    }

    pub fn set_filter(&mut self, filter: Option<String>) {
        self.filter = filter;
    }

    pub fn set_search(&mut self, search: Option<String>) {
        self.search = search;
    }

    pub fn get_visible_lines(&self, height: usize) -> Vec<&str> {
        let filtered: Vec<&str> = self
            .lines
            .iter()
            .filter(|line| {
                if let Some(ref filter) = self.filter {
                    line.to_uppercase().contains(&filter.to_uppercase())
                } else {
                    true
                }
            })
            .filter(|line| {
                if let Some(ref search) = self.search {
                    line.to_lowercase().contains(&search.to_lowercase())
                } else {
                    true
                }
            })
            .map(|s| s.as_str())
            .collect();

        let start = self.scroll_offset.min(filtered.len().saturating_sub(height));
        let end = (start + height).min(filtered.len());

        filtered[start..end].to_vec()
    }
}
