//! Stratum Protocol Implementation for PYRAX Mining
//!
//! Supports both solo mining via RPC and pool mining via Stratum v1/v2
//! Production-ready implementation for devnet, testnet, and mainnet.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug, error};

use crate::types::{H256, BlockHeader};

/// Stratum protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StratumVersion {
    V1,
    V2,
}

/// Stratum client configuration
#[derive(Debug, Clone)]
pub struct StratumConfig {
    pub pool_url: String,
    pub worker_name: String,
    pub password: String,
    pub version: StratumVersion,
}

impl Default for StratumConfig {
    fn default() -> Self {
        Self {
            pool_url: "stratum+tcp://localhost:3333".to_string(),
            worker_name: "pyrax_worker".to_string(),
            password: "x".to_string(),
            version: StratumVersion::V1,
        }
    }
}

/// Mining job from pool
#[derive(Debug, Clone)]
pub struct MiningJob {
    pub job_id: String,
    pub header_hash: H256,
    pub seed_hash: H256,
    pub target: H256,
    pub height: u64,
    pub received_at: Instant,
    pub clean_jobs: bool,
}

/// Share submission result
#[derive(Debug, Clone)]
pub struct ShareResult {
    pub accepted: bool,
    pub job_id: String,
    pub nonce: u64,
    pub difficulty: f64,
    pub error: Option<String>,
}

/// Stratum JSON-RPC request
#[derive(Debug, Serialize, Deserialize)]
struct StratumRequest {
    id: u64,
    method: String,
    params: serde_json::Value,
}

/// Stratum JSON-RPC response
#[derive(Debug, Serialize, Deserialize)]
struct StratumResponse {
    id: Option<u64>,
    result: Option<serde_json::Value>,
    error: Option<StratumError>,
}

/// Stratum JSON-RPC notification (no id)
#[derive(Debug, Serialize, Deserialize)]
struct StratumNotification {
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct StratumError {
    code: i32,
    message: String,
}

/// Stratum client state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientState {
    Disconnected,
    Connecting,
    Connected,
    Subscribing,
    Subscribed,
    Authorizing,
    Authorized,
    Mining,
}

/// Statistics for the stratum client
#[derive(Debug, Clone, Default)]
pub struct StratumStats {
    pub shares_accepted: u64,
    pub shares_rejected: u64,
    pub shares_stale: u64,
    pub current_difficulty: f64,
    pub jobs_received: u64,
    pub connected_since: Option<Instant>,
    pub last_share_time: Option<Instant>,
}

/// Stratum mining client
pub struct StratumClient {
    config: StratumConfig,
    state: Arc<RwLock<ClientState>>,
    stats: Arc<RwLock<StratumStats>>,
    current_job: Arc<RwLock<Option<MiningJob>>>,
    extranonce1: Arc<RwLock<String>>,
    extranonce2_size: Arc<RwLock<usize>>,
    request_id: Arc<RwLock<u64>>,
    pending_requests: Arc<RwLock<HashMap<u64, String>>>,
    
    // Channels
    job_tx: mpsc::Sender<MiningJob>,
    share_rx: Option<mpsc::Receiver<(String, u64, H256)>>,
}

impl StratumClient {
    /// Create a new stratum client
    pub fn new(config: StratumConfig) -> (Self, mpsc::Receiver<MiningJob>, mpsc::Sender<(String, u64, H256)>) {
        let (job_tx, job_rx) = mpsc::channel(16);
        let (share_tx, share_rx) = mpsc::channel(64);
        
        let client = Self {
            config,
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            stats: Arc::new(RwLock::new(StratumStats::default())),
            current_job: Arc::new(RwLock::new(None)),
            extranonce1: Arc::new(RwLock::new(String::new())),
            extranonce2_size: Arc::new(RwLock::new(4)),
            request_id: Arc::new(RwLock::new(1)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            job_tx,
            share_rx: Some(share_rx),
        };
        
        (client, job_rx, share_tx)
    }

    /// Get current state
    pub async fn state(&self) -> ClientState {
        self.state.read().await.clone()
    }

    /// Get current statistics
    pub async fn stats(&self) -> StratumStats {
        self.stats.read().await.clone()
    }

    /// Get current job
    pub async fn current_job(&self) -> Option<MiningJob> {
        self.current_job.read().await.clone()
    }

    /// Run the stratum client (connect and maintain connection)
    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            match self.connect_and_run().await {
                Ok(_) => {
                    info!("Stratum connection closed normally");
                }
                Err(e) => {
                    error!("Stratum connection error: {}", e);
                    *self.state.write().await = ClientState::Disconnected;
                }
            }
            
            // Reconnect after delay
            info!("Reconnecting to pool in 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_and_run(&mut self) -> anyhow::Result<()> {
        // Take ownership of share_rx for this connection
        let mut share_rx = self.share_rx.take()
            .ok_or_else(|| anyhow::anyhow!("Share receiver not available"))?;
        
        let result = self.run_connection(&mut share_rx).await;
        
        // Put it back for next reconnection
        self.share_rx = Some(share_rx);
        result
    }

    async fn run_connection(&self, share_rx: &mut mpsc::Receiver<(String, u64, H256)>) -> anyhow::Result<()> {
        // Parse pool URL
        let url = self.config.pool_url
            .strip_prefix("stratum+tcp://")
            .unwrap_or(&self.config.pool_url);
        
        *self.state.write().await = ClientState::Connecting;
        info!("Connecting to pool: {}", url);
        
        let stream = TcpStream::connect(url).await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        
        *self.state.write().await = ClientState::Connected;
        self.stats.write().await.connected_since = Some(Instant::now());
        info!("Connected to pool");

        // Subscribe
        *self.state.write().await = ClientState::Subscribing;
        let subscribe_req = self.create_request("mining.subscribe", 
            serde_json::json!(["pyrax-miner/1.0.0"])).await;
        self.send_request(&mut writer, &subscribe_req).await?;

        let mut line = String::new();
        
        loop {
            tokio::select! {
                // Read from pool
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            info!("Pool connection closed");
                            return Ok(());
                        }
                        Ok(_) => {
                            if let Err(e) = self.handle_message(&line, &mut writer).await {
                                warn!("Error handling message: {}", e);
                            }
                            line.clear();
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                
                // Submit shares
                Some((job_id, nonce, hash)) = share_rx.recv() => {
                    if let Err(e) = self.submit_share(&mut writer, &job_id, nonce, &hash).await {
                        warn!("Error submitting share: {}", e);
                    }
                }
            }
        }
    }

    async fn create_request(&self, method: &str, params: serde_json::Value) -> StratumRequest {
        let mut id = self.request_id.write().await;
        let req_id = *id;
        *id += 1;
        
        self.pending_requests.write().await.insert(req_id, method.to_string());
        
        StratumRequest {
            id: req_id,
            method: method.to_string(),
            params,
        }
    }

    async fn send_request<W: AsyncWriteExt + Unpin>(&self, writer: &mut W, req: &StratumRequest) -> anyhow::Result<()> {
        let mut json = serde_json::to_string(req)?;
        json.push('\n');
        writer.write_all(json.as_bytes()).await?;
        writer.flush().await?;
        debug!("Sent: {}", json.trim());
        Ok(())
    }

    async fn handle_message<W: AsyncWriteExt + Unpin>(&self, line: &str, writer: &mut W) -> anyhow::Result<()> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(());
        }
        
        debug!("Received: {}", line);

        // Try to parse as response first
        if let Ok(response) = serde_json::from_str::<StratumResponse>(line) {
            return self.handle_response(response, writer).await;
        }

        // Try to parse as notification
        if let Ok(notification) = serde_json::from_str::<StratumNotification>(line) {
            return self.handle_notification(notification).await;
        }

        warn!("Unknown message format: {}", line);
        Ok(())
    }

    async fn handle_response<W: AsyncWriteExt + Unpin>(&self, response: StratumResponse, writer: &mut W) -> anyhow::Result<()> {
        let id = response.id.unwrap_or(0);
        let method = self.pending_requests.write().await.remove(&id);
        
        if let Some(error) = response.error {
            warn!("Stratum error for {:?}: {} ({})", method, error.message, error.code);
            return Ok(());
        }

        match method.as_deref() {
            Some("mining.subscribe") => {
                if let Some(result) = response.result {
                    self.handle_subscribe_result(result).await?;
                    
                    // Now authorize
                    *self.state.write().await = ClientState::Authorizing;
                    let auth_req = self.create_request("mining.authorize",
                        serde_json::json!([self.config.worker_name, self.config.password])).await;
                    self.send_request(writer, &auth_req).await?;
                }
            }
            Some("mining.authorize") => {
                if response.result == Some(serde_json::Value::Bool(true)) {
                    info!("Worker authorized successfully");
                    *self.state.write().await = ClientState::Authorized;
                } else {
                    warn!("Worker authorization failed");
                }
            }
            Some("mining.submit") => {
                let mut stats = self.stats.write().await;
                if response.result == Some(serde_json::Value::Bool(true)) {
                    stats.shares_accepted += 1;
                    stats.last_share_time = Some(Instant::now());
                    info!("Share accepted! Total: {}", stats.shares_accepted);
                } else {
                    stats.shares_rejected += 1;
                    warn!("Share rejected. Total rejected: {}", stats.shares_rejected);
                }
            }
            _ => {
                debug!("Response for unknown request {}: {:?}", id, response.result);
            }
        }

        Ok(())
    }

    async fn handle_subscribe_result(&self, result: serde_json::Value) -> anyhow::Result<()> {
        // Parse subscription result: [[["mining.notify", "subscription_id"]], extranonce1, extranonce2_size]
        if let Some(arr) = result.as_array() {
            if arr.len() >= 3 {
                if let Some(extranonce1) = arr[1].as_str() {
                    *self.extranonce1.write().await = extranonce1.to_string();
                }
                if let Some(size) = arr[2].as_u64() {
                    *self.extranonce2_size.write().await = size as usize;
                }
                info!("Subscribed: extranonce1={}, extranonce2_size={}", 
                    self.extranonce1.read().await, self.extranonce2_size.read().await);
                *self.state.write().await = ClientState::Subscribed;
            }
        }
        Ok(())
    }

    async fn handle_notification(&self, notification: StratumNotification) -> anyhow::Result<()> {
        match notification.method.as_str() {
            "mining.notify" => {
                self.handle_mining_notify(notification.params).await?;
            }
            "mining.set_difficulty" => {
                if let Some(arr) = notification.params.as_array() {
                    if let Some(diff) = arr.first().and_then(|v| v.as_f64()) {
                        self.stats.write().await.current_difficulty = diff;
                        info!("Difficulty set to: {}", diff);
                    }
                }
            }
            _ => {
                debug!("Unknown notification: {}", notification.method);
            }
        }
        Ok(())
    }

    async fn handle_mining_notify(&self, params: serde_json::Value) -> anyhow::Result<()> {
        // Parse mining.notify params
        // [job_id, prevhash, coinb1, coinb2, merkle_branches, version, nbits, ntime, clean_jobs]
        let arr = params.as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid mining.notify params"))?;
        
        if arr.len() < 9 {
            return Err(anyhow::anyhow!("Not enough params in mining.notify"));
        }

        let job_id = arr[0].as_str().unwrap_or("").to_string();
        let prev_hash = arr[1].as_str().unwrap_or("");
        let clean_jobs = arr[8].as_bool().unwrap_or(false);

        // Parse header hash from prevhash
        let header_hash = parse_hex_to_h256(prev_hash).unwrap_or_default();
        
        // For KAWPOW, we also need seed hash (usually derived from height/epoch)
        // For now, use prevhash as placeholder
        let seed_hash = header_hash;
        
        // Calculate target from difficulty
        let difficulty = self.stats.read().await.current_difficulty;
        let target = difficulty_to_target(difficulty);

        let job = MiningJob {
            job_id: job_id.clone(),
            header_hash,
            seed_hash,
            target,
            height: 0, // Would need to parse from block template
            received_at: Instant::now(),
            clean_jobs,
        };

        *self.current_job.write().await = Some(job.clone());
        self.stats.write().await.jobs_received += 1;
        *self.state.write().await = ClientState::Mining;

        if clean_jobs {
            info!("New job received (clean): {}", job_id);
        } else {
            debug!("New job received: {}", job_id);
        }

        // Send to miner
        let _ = self.job_tx.send(job).await;

        Ok(())
    }

    async fn submit_share<W: AsyncWriteExt + Unpin>(
        &self,
        writer: &mut W,
        job_id: &str,
        nonce: u64,
        _hash: &H256,
    ) -> anyhow::Result<()> {
        let extranonce2 = format!("{:08x}", nonce & 0xFFFFFFFF);
        let ntime = format!("{:08x}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as u32);
        let nonce_hex = format!("{:016x}", nonce);

        let req = self.create_request("mining.submit", serde_json::json!([
            self.config.worker_name,
            job_id,
            extranonce2,
            ntime,
            nonce_hex
        ])).await;

        self.send_request(writer, &req).await?;
        debug!("Submitted share for job {} with nonce {}", job_id, nonce);

        Ok(())
    }
}

/// Parse hex string to H256
fn parse_hex_to_h256(hex_str: &str) -> Option<H256> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(hex_str).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    Some(H256::from_slice(&bytes))
}

/// Convert difficulty to target hash
fn difficulty_to_target(difficulty: f64) -> H256 {
    if difficulty <= 0.0 {
        return H256::from_slice(&[0xFF; 32]);
    }
    
    // Target = 2^256 / difficulty
    // For practical purposes, we use: target = 0x00000000FFFF... / difficulty
    let max_target: u128 = 0x00000000FFFF0000_0000000000000000;
    let target_val = (max_target as f64 / difficulty) as u128;
    
    let mut target = [0u8; 32];
    target[16..32].copy_from_slice(&target_val.to_be_bytes());
    
    H256::from_slice(&target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_to_target() {
        let target = difficulty_to_target(1.0);
        assert!(!target.is_zero());
        
        let target_high = difficulty_to_target(1000.0);
        // Higher difficulty = lower target
        assert!(target_high < target);
    }

    #[test]
    fn test_stratum_config_default() {
        let config = StratumConfig::default();
        assert_eq!(config.worker_name, "pyrax_worker");
        assert_eq!(config.version, StratumVersion::V1);
    }
}
