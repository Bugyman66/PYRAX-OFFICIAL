//! RPC Client Module
//!
//! HTTP client for polling pyrax-node RPC endpoints.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// RPC client for a single node
pub struct RpcClient {
    endpoint: String,
    client: Client,
    timeout: Duration,
}

/// Status of a single node
#[derive(Debug, Clone)]
pub struct NodeStatus {
    pub endpoint: String,
    pub reachable: bool,
    pub block_height: u64,
    pub block_hash: Option<String>,
    pub syncing: bool,
    pub peer_count: u32,
    pub latency_ms: u64,
    pub timestamp: u64,
    pub error: Option<String>,
}

/// Chain info from RPC
#[derive(Debug, Deserialize)]
pub struct ChainInfo {
    pub height: u64,
    pub best_hash: String,
    pub total_difficulty: u64,
}

/// Mining info from RPC
#[derive(Debug, Deserialize)]
pub struct MiningInfo {
    pub mining: bool,
    pub hashrate: u64,
    pub difficulty: u64,
    pub blocks_found: u64,
}

/// Sync status from RPC
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SyncStatus {
    Syncing {
        starting_block: u64,
        current_block: u64,
        highest_block: u64,
    },
    NotSyncing(bool),
}

impl RpcClient {
    /// Create a new RPC client
    pub fn new(endpoint: String, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            endpoint,
            client,
            timeout,
        }
    }
    
    /// Get node status (comprehensive health check)
    pub async fn get_status(&self) -> Result<NodeStatus> {
        let start = Instant::now();
        let timestamp = chrono::Utc::now().timestamp() as u64;
        
        // Try to get block number first (basic connectivity test)
        let block_height = match self.get_block_number().await {
            Ok(h) => h,
            Err(e) => {
                return Ok(NodeStatus {
                    endpoint: self.endpoint.clone(),
                    reachable: false,
                    block_height: 0,
                    block_hash: None,
                    syncing: false,
                    peer_count: 0,
                    latency_ms: start.elapsed().as_millis() as u64,
                    timestamp,
                    error: Some(e.to_string()),
                });
            }
        };
        
        // Get additional info
        let syncing = self.is_syncing().await.unwrap_or(false);
        let peer_count = self.get_peer_count().await.unwrap_or(0);
        let block_hash = self.get_block_hash(block_height).await.ok();
        
        Ok(NodeStatus {
            endpoint: self.endpoint.clone(),
            reachable: true,
            block_height,
            block_hash,
            syncing,
            peer_count,
            latency_ms: start.elapsed().as_millis() as u64,
            timestamp,
            error: None,
        })
    }
    
    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let result: Value = self.call("eth_blockNumber", json!([])).await?;
        
        let hex_str = result.as_str()
            .context("Expected hex string for block number")?;
        
        let height = u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .context("Failed to parse block number")?;
        
        Ok(height)
    }
    
    /// Check if node is syncing
    pub async fn is_syncing(&self) -> Result<bool> {
        let result: Value = self.call("eth_syncing", json!([])).await?;
        
        match result {
            Value::Bool(false) => Ok(false),
            Value::Object(_) => Ok(true),
            _ => Ok(false),
        }
    }
    
    /// Get peer count
    pub async fn get_peer_count(&self) -> Result<u32> {
        let result: Value = self.call("net_peerCount", json!([])).await?;
        
        let hex_str = result.as_str()
            .context("Expected hex string for peer count")?;
        
        let count = u32::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .context("Failed to parse peer count")?;
        
        Ok(count)
    }
    
    /// Get block hash by number
    pub async fn get_block_hash(&self, number: u64) -> Result<String> {
        let params = json!([format!("0x{:x}", number), false]);
        let result: Value = self.call("eth_getBlockByNumber", params).await?;
        
        let hash = result.get("hash")
            .and_then(|h| h.as_str())
            .context("Block hash not found")?
            .to_string();
        
        Ok(hash)
    }
    
    /// Get chain info (PYRAX custom RPC)
    pub async fn get_chain_info(&self) -> Result<ChainInfo> {
        let result: ChainInfo = self.call("pyrax_getChainInfo", json!([])).await?;
        Ok(result)
    }
    
    /// Get mining info (PYRAX custom RPC)
    pub async fn get_mining_info(&self) -> Result<MiningInfo> {
        let result: MiningInfo = self.call("pyrax_getMiningInfo", json!([])).await?;
        Ok(result)
    }
    
    /// Check node health
    pub async fn health(&self) -> Result<bool> {
        let result: Value = self.call("pyrax_health", json!([])).await?;
        Ok(result.as_str() == Some("ok"))
    }
    
    /// Make a JSON-RPC call
    async fn call<T: for<'de> Deserialize<'de>>(&self, method: &str, params: Value) -> Result<T> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        
        debug!("RPC call to {}: {}", self.endpoint, method);
        
        let response = self.client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await
            .with_context(|| format!("Failed to connect to {}", self.endpoint))?;
        
        let body: Value = response.json().await
            .with_context(|| "Failed to parse RPC response")?;
        
        if let Some(error) = body.get("error") {
            let msg = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown RPC error");
            anyhow::bail!("RPC error: {}", msg);
        }
        
        let result = body.get("result")
            .context("No result in RPC response")?
            .clone();
        
        serde_json::from_value(result)
            .with_context(|| "Failed to deserialize RPC result")
    }
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            reachable: false,
            block_height: 0,
            block_hash: None,
            syncing: false,
            peer_count: 0,
            latency_ms: 0,
            timestamp: 0,
            error: None,
        }
    }
}
