//! PYRAX Chain Observer - Configuration Module
//!
//! Handles loading and parsing configuration from TOML files.

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Observer settings
    pub observer: ObserverConfig,
    /// Node endpoints to monitor
    pub nodes: NodesConfig,
    /// Chain parameters
    pub chain: ChainConfig,
    /// Stream-specific settings
    pub streams: StreamsConfig,
    /// Canary transaction settings
    pub canary: CanaryConfig,
    /// Node discovery crawler settings
    #[serde(default)]
    pub crawler: CrawlerConfig,
    /// Prometheus metrics server settings
    pub metrics: MetricsConfig,
    /// Health/Status API settings
    pub api: ApiConfig,
    /// Logging settings
    pub logging: LoggingConfig,
    /// Telegram Alert settings
    #[serde(default)]
    pub telegram: TelegramConfig,
}

/// Telegram Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    /// Enable Telegram alerts
    #[serde(default)]
    pub enabled: bool,
    /// Bot API Token (can be set via TELEGRAM_BOT_TOKEN env)
    pub bot_token: Option<String>,
    /// Chat ID (can be set via TELEGRAM_CHAT_ID env)
    pub chat_id: Option<String>,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: None,
            chat_id: None,
        }
    }
}

/// Observer behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverConfig {
    /// Polling interval in milliseconds
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    /// Request timeout in milliseconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_ms: u64,
    /// Maximum retries per node
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

/// Node endpoints configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodesConfig {
    /// List of RPC endpoints to monitor
    pub endpoints: Vec<String>,
}

/// Chain parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Expected block time in milliseconds
    #[serde(default = "default_block_time")]
    pub expected_block_time_ms: u64,
    /// Stall threshold in milliseconds
    #[serde(default = "default_stall_threshold")]
    pub stall_threshold_ms: u64,
    /// Fork detection threshold in blocks
    #[serde(default = "default_fork_threshold")]
    pub fork_threshold_blocks: u64,
    /// Network name
    #[serde(default = "default_network")]
    pub network: String,
}

/// Stream-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamsConfig {
    /// Enable stream monitoring
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Stream IDs to monitor
    #[serde(default = "default_stream_ids")]
    pub stream_ids: Vec<String>,
    /// Stream A block time (ms)
    #[serde(default = "default_stream_a_block_time")]
    pub stream_a_block_time_ms: u64,
    /// Stream B block time (ms)
    #[serde(default = "default_stream_b_block_time")]
    pub stream_b_block_time_ms: u64,
    /// Stream C checkpoint interval (blocks)
    #[serde(default = "default_checkpoint_interval")]
    pub stream_c_checkpoint_interval: u64,
}

/// Canary transaction settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    /// Enable canary transactions
    #[serde(default)]
    pub enabled: bool,
    /// Canary interval in milliseconds
    #[serde(default = "default_canary_interval")]
    pub interval_ms: u64,
    /// Maximum pending canary transactions
    #[serde(default = "default_max_pending")]
    pub max_pending: u32,
}

/// Prometheus metrics server settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Listen address
    #[serde(default = "default_metrics_addr")]
    pub listen_addr: String,
    /// Enable histogram metrics
    #[serde(default)]
    pub enable_histograms: bool,
}

/// Health/Status API settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Listen address
    #[serde(default = "default_api_addr")]
    pub listen_addr: String,
    /// Enable CORS
    #[serde(default = "default_true")]
    pub enable_cors: bool,
}

/// Logging settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,
    /// JSON logging
    #[serde(default)]
    pub json: bool,
}

/// Node discovery crawler settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerConfig {
    /// Enable node discovery crawler
    #[serde(default)]
    pub enabled: bool,
    /// Crawl interval in milliseconds
    #[serde(default = "default_crawl_interval")]
    pub crawl_interval_ms: u64,
    /// Maximum nodes to discover
    #[serde(default = "default_max_nodes")]
    pub max_nodes: usize,
    /// Probe timeout in milliseconds
    #[serde(default = "default_probe_timeout")]
    pub probe_timeout_ms: u64,
}

// Default value functions
fn default_poll_interval() -> u64 { 5000 }
fn default_request_timeout() -> u64 { 3000 }
fn default_max_retries() -> u32 { 3 }
fn default_block_time() -> u64 { 10000 }
fn default_stall_threshold() -> u64 { 60000 }
fn default_fork_threshold() -> u64 { 2 }
fn default_network() -> String { "devnet".to_string() }
fn default_true() -> bool { true }
fn default_stream_ids() -> Vec<String> { vec!["a".to_string(), "b".to_string(), "c".to_string()] }
fn default_stream_a_block_time() -> u64 { 10000 }
fn default_stream_b_block_time() -> u64 { 60000 }
fn default_checkpoint_interval() -> u64 { 100 }
fn default_canary_interval() -> u64 { 60000 }
fn default_max_pending() -> u32 { 5 }
fn default_metrics_addr() -> String { "0.0.0.0:9092".to_string() }
fn default_api_addr() -> String { "0.0.0.0:8080".to_string() }
fn default_log_level() -> String { "info".to_string() }
fn default_crawl_interval() -> u64 { 300000 } // 5 minutes
fn default_max_nodes() -> usize { 100 }
fn default_probe_timeout() -> u64 { 5000 }

impl Config {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        
        let mut config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
        
        // Override Telegram settings from Environment Variables
        if let Ok(token) = std::env::var("TELEGRAM_BOT_TOKEN") {
            config.telegram.bot_token = Some(token);
        }
        if let Ok(chat_id) = std::env::var("TELEGRAM_CHAT_ID") {
            config.telegram.chat_id = Some(chat_id);
        }
        
        // Auto-enable if both are present
        if config.telegram.bot_token.is_some() && config.telegram.chat_id.is_some() {
             config.telegram.enabled = true;
        }

        config.validate()?;
        
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.nodes.endpoints.is_empty() {
            anyhow::bail!("At least one node endpoint must be configured");
        }
        
        if self.observer.poll_interval_ms < 100 {
            anyhow::bail!("Poll interval must be at least 100ms");
        }
        
        if self.observer.request_timeout_ms < 100 {
            anyhow::bail!("Request timeout must be at least 100ms");
        }
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            observer: ObserverConfig {
                poll_interval_ms: default_poll_interval(),
                request_timeout_ms: default_request_timeout(),
                max_retries: default_max_retries(),
            },
            nodes: NodesConfig {
                endpoints: vec!["http://localhost:8545".to_string()],
            },
            chain: ChainConfig {
                expected_block_time_ms: default_block_time(),
                stall_threshold_ms: default_stall_threshold(),
                fork_threshold_blocks: default_fork_threshold(),
                network: default_network(),
            },
            streams: StreamsConfig {
                enabled: true,
                stream_ids: default_stream_ids(),
                stream_a_block_time_ms: default_stream_a_block_time(),
                stream_b_block_time_ms: default_stream_b_block_time(),
                stream_c_checkpoint_interval: default_checkpoint_interval(),
            },
            canary: CanaryConfig {
                enabled: false,
                interval_ms: default_canary_interval(),
                max_pending: default_max_pending(),
            },
            metrics: MetricsConfig {
                listen_addr: default_metrics_addr(),
                enable_histograms: false,
            },
            api: ApiConfig {
                listen_addr: default_api_addr(),
                enable_cors: true,
            },
            logging: LoggingConfig {
                level: default_log_level(),
                json: false,
            },
            crawler: CrawlerConfig {
                enabled: false,
                crawl_interval_ms: default_crawl_interval(),
                max_nodes: default_max_nodes(),
                probe_timeout_ms: default_probe_timeout(),
            },
            telegram: TelegramConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_validation_no_endpoints() {
        let mut config = Config::default();
        config.nodes.endpoints.clear();
        assert!(config.validate().is_err());
    }
}
