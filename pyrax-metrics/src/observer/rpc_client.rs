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
    // Mining metrics
    pub hashrate: u64,
    pub difficulty: u64,
    pub blocks_found: u64,
}

/// Chain info from RPC (matches pyrax-node RpcChainInfo)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
    pub chain_id: u32,
    pub network: String,
    pub best_block_hash: String,
    pub best_block_height: u64,
    pub genesis_hash: String,
    pub difficulty: u64,
    pub utxo_count: u64,
    pub syncing: bool,
    pub node_version: String,
}

impl ChainInfo {
    /// Get height (alias for best_block_height)
    pub fn height(&self) -> u64 {
        self.best_block_height
    }
    
    /// Get best hash (alias for best_block_hash)
    pub fn best_hash(&self) -> &str {
        &self.best_block_hash
    }
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

/// Peer information from pyrax_getPeers
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: String,
    /// IP address
    #[serde(default)]
    pub ip: Option<String>,
    /// Port number
    #[serde(default)]
    pub port: Option<u16>,
    /// Full address
    #[serde(default)]
    pub address: Option<String>,
    /// Peer's best block height
    #[serde(default)]
    pub block_height: u64,
    /// Node version
    #[serde(default)]
    pub version: Option<String>,
    /// Direction (inbound/outbound)
    #[serde(default)]
    pub direction: Option<String>,
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
        
        // Try to get chain info first (PYRAX native method)
        let (block_height, block_hash) = match self.get_chain_info().await {
            Ok(info) => (info.best_block_height, Some(info.best_block_hash.clone())),
            Err(e) => {
                // Fallback to eth_blockNumber if pyrax_getChainInfo fails
                match self.get_block_number().await {
                    Ok(h) => (h, None),
                    Err(_) => {
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
                            hashrate: 0,
                            difficulty: 0,
                            blocks_found: 0,
                        });
                    }
                }
            }
        };
        
        // Get additional info (syncing and peer_count may use eth_ methods as fallback)
        let syncing = self.is_syncing().await.unwrap_or(false);
        let peer_count = self.get_peer_count().await.unwrap_or(0);
        
        // Get mining info
        let (hashrate, difficulty, blocks_found) = match self.get_mining_info().await {
            Ok(info) => (info.hashrate, info.difficulty, info.blocks_found),
            Err(_) => (0, 0, 0),
        };
        
        Ok(NodeStatus {
            endpoint: self.endpoint.clone(),
            reachable: true,
            block_height,
            block_hash, // Already obtained from get_chain_info
            syncing,
            peer_count,
            latency_ms: start.elapsed().as_millis() as u64,
            timestamp,
            error: None,
            hashrate,
            difficulty,
            blocks_found,
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
    
    /// Get connected peers (PYRAX custom RPC for node discovery)
    pub async fn get_peers(&self) -> Result<Vec<PeerInfo>> {
        let result: Vec<PeerInfo> = self.call("pyrax_getPeers", json!([])).await?;
        Ok(result)
    }
    
    /// Get endpoint URL
    pub fn endpoint(&self) -> &str {
        &self.endpoint
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
            hashrate: 0,
            difficulty: 0,
            blocks_found: 0,
        }
    }
}
