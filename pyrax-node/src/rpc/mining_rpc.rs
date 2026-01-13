//! Mining RPC Endpoints
//!
//! JSON-RPC endpoints for mining operations:
//! - getblocktemplate: Get current block template for mining
//! - submitblock: Submit a mined block
//! - getmininginfo: Get mining statistics
//! - getwork: Get work for GPU mining (ethash-style)
//! - submitwork: Submit GPU mining result
//!
//! Compatible with standard mining software and the PYRAX desktop app.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn, debug};

use crate::types::{H256, Address, Block, BlockHeader, Transaction, BlockNumber};
use crate::services::mining::{MiningService, MiningInfo, ChainStateProvider};
use crate::miner::BlockTemplate;
use crate::consensus::{get_epoch, compute_seed};

/// Mining RPC handler
pub struct MiningRpc {
    mining_service: Arc<MiningService>,
}

impl MiningRpc {
    pub fn new(mining_service: Arc<MiningService>) -> Self {
        Self { mining_service }
    }

    /// Handle mining RPC request
    pub async fn handle(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        match method {
            "pyrax_getBlockTemplate" | "getblocktemplate" => self.get_block_template(params).await,
            "pyrax_submitBlock" | "submitblock" => self.submit_block(params).await,
            "pyrax_getMiningInfo" | "getmininginfo" => self.get_mining_info().await,
            "pyrax_getWork" | "eth_getWork" => self.get_work().await,
            "pyrax_submitWork" | "eth_submitWork" => self.submit_work(params).await,
            "pyrax_getHashrate" | "eth_hashrate" => self.get_hashrate().await,
            "pyrax_mining" | "eth_mining" => self.is_mining().await,
            "pyrax_getStratumInfo" => self.get_stratum_info().await,
            _ => Err(RpcError::MethodNotFound(method.to_string())),
        }
    }

    /// Get block template for mining
    async fn get_block_template(&self, _params: Value) -> Result<Value, RpcError> {
        let template = self.mining_service.get_block_template();
        
        let epoch = get_epoch(template.height);
        let seed_hash = compute_seed(epoch);

        // Calculate total transaction fees (fees would be computed from inputs - outputs)
        // For now, fees are included in the coinbase value
        let total_fees: u64 = 0;

        Ok(json!({
            "height": template.height,
            "parentHash": format!("0x{}", hex::encode(template.parent_hash.as_bytes())),
            "timestamp": template.timestamp,
            "difficulty": template.difficulty,
            "difficultyHex": format!("0x{:x}", template.difficulty),
            "target": format!("0x{}", hex::encode(difficulty_to_target(template.difficulty).as_bytes())),
            "seedHash": format!("0x{}", hex::encode(&seed_hash)),
            "epoch": epoch,
            "coinbaseValue": template.coinbase_value,
            "transactions": template.transactions.len(),
            "transactionsFees": total_fees,
        }))
    }

    /// Submit a mined block
    async fn submit_block(&self, params: Value) -> Result<Value, RpcError> {
        let block_hex = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError::InvalidParams("Missing block data".to_string()))?;

        let block_bytes = hex::decode(block_hex.strip_prefix("0x").unwrap_or(block_hex))
            .map_err(|e| RpcError::InvalidParams(format!("Invalid hex: {}", e)))?;

        let block: Block = bincode::deserialize(&block_bytes)
            .map_err(|e| RpcError::InvalidParams(format!("Invalid block: {}", e)))?;

        match self.mining_service.submit_block(block) {
            Ok(hash) => {
                info!("Block submitted via RPC: {}", hash);
                Ok(json!({
                    "success": true,
                    "hash": format!("0x{}", hex::encode(hash.as_bytes())),
                }))
            }
            Err(e) => {
                warn!("Block submission failed: {}", e);
                Ok(json!({
                    "success": false,
                    "error": e,
                }))
            }
        }
    }

    /// Get mining info
    async fn get_mining_info(&self) -> Result<Value, RpcError> {
        let info = self.mining_service.get_mining_info();
        
        Ok(json!({
            "height": info.height,
            "difficulty": info.difficulty,
            "networkHashrate": info.network_hashrate,
            "networkHashrateFormatted": format_hashrate(info.network_hashrate),
            "stratumEnabled": info.stratum_enabled,
            "stratumPort": info.stratum_port,
            "stratumStats": info.stratum_stats.map(|s| json!({
                "workersConnected": s.workers_connected,
                "sharesAccepted": s.shares_accepted,
                "sharesRejected": s.shares_rejected,
                "blocksFound": s.blocks_found,
            })),
            "coinbaseAddress": info.coinbase_address,
        }))
    }

    /// Get work for GPU mining (ethash/KAWPOW compatible)
    async fn get_work(&self) -> Result<Value, RpcError> {
        let template = self.mining_service.get_block_template();
        
        let epoch = get_epoch(template.height);
        let seed_hash = compute_seed(epoch);
        
        // Build header hash for mining
        let header = BlockHeader {
            version: 1,
            stream: 1, // Stream B (KAWPOW)
            parent_hash: template.parent_hash,
            merkle_root: H256::zero(), // Will be filled by miner
            utxo_commitment: H256::zero(),
            timestamp: template.timestamp,
            difficulty: template.difficulty,
            nonce: 0,
            extra_nonce: 0,
            height: template.height,
            beneficiary: Address::ZERO,
        };
        
        let header_hash = header.hash();
        let target = difficulty_to_target(template.difficulty);

        // Return in eth_getWork format: [header_hash, seed_hash, target]
        Ok(json!([
            format!("0x{}", hex::encode(header_hash.as_bytes())),
            format!("0x{}", hex::encode(&seed_hash)),
            format!("0x{}", hex::encode(target.as_bytes())),
            format!("0x{:x}", template.height),
        ]))
    }

    /// Submit work (GPU mining result)
    async fn submit_work(&self, params: Value) -> Result<Value, RpcError> {
        let params = params.as_array()
            .ok_or_else(|| RpcError::InvalidParams("Expected array".to_string()))?;

        if params.len() < 3 {
            return Err(RpcError::InvalidParams("Expected [nonce, header_hash, mix_hash]".to_string()));
        }

        let nonce_hex = params[0].as_str().unwrap_or("");
        let header_hash_hex = params[1].as_str().unwrap_or("");
        let mix_hash_hex = params[2].as_str().unwrap_or("");

        let nonce = u64::from_str_radix(
            nonce_hex.strip_prefix("0x").unwrap_or(nonce_hex),
            16
        ).unwrap_or(0);

        let header_hash = parse_h256(header_hash_hex);
        let mix_hash = parse_h256(mix_hash_hex);

        debug!("Work submitted: nonce={}, header={}, mix={}", 
            nonce, header_hash, mix_hash);

        // Validate and submit work through mining service
        match self.mining_service.submit_work(nonce, header_hash, mix_hash).await {
            Ok(block_hash) => {
                info!("Work accepted! Block hash: {}", block_hash);
                Ok(json!(true))
            }
            Err(e) => {
                debug!("Work rejected: {}", e);
                Ok(json!(false))
            }
        }
    }

    /// Get current hashrate
    async fn get_hashrate(&self) -> Result<Value, RpcError> {
        let info = self.mining_service.get_mining_info();
        
        if let Some(stats) = info.stratum_stats {
            // Estimate hashrate from share rate
            let shares_per_sec = stats.shares_accepted as f64 / 60.0; // Approximate
            let estimated_hashrate = (shares_per_sec * info.difficulty as f64 * 4294967296.0) as u64;
            
            Ok(json!(format!("0x{:x}", estimated_hashrate)))
        } else {
            Ok(json!("0x0"))
        }
    }

    /// Check if mining is active
    async fn is_mining(&self) -> Result<Value, RpcError> {
        let running = self.mining_service.is_running().await;
        Ok(json!(running))
    }

    /// Get stratum server info
    async fn get_stratum_info(&self) -> Result<Value, RpcError> {
        let info = self.mining_service.get_mining_info();
        
        if !info.stratum_enabled {
            return Ok(json!({
                "enabled": false,
            }));
        }

        let stats = info.stratum_stats.unwrap_or(crate::services::mining::StratumStats {
            workers_connected: 0,
            shares_accepted: 0,
            shares_rejected: 0,
            blocks_found: 0,
        });

        Ok(json!({
            "enabled": true,
            "port": info.stratum_port,
            "workers": stats.workers_connected,
            "sharesAccepted": stats.shares_accepted,
            "sharesRejected": stats.shares_rejected,
            "blocksFound": stats.blocks_found,
            "connectionUrl": format!("stratum+tcp://localhost:{}", info.stratum_port),
        }))
    }
}

/// RPC error types
#[derive(Debug)]
pub enum RpcError {
    MethodNotFound(String),
    InvalidParams(String),
    InternalError(String),
}

impl RpcError {
    pub fn code(&self) -> i32 {
        match self {
            RpcError::MethodNotFound(_) => -32601,
            RpcError::InvalidParams(_) => -32602,
            RpcError::InternalError(_) => -32603,
        }
    }

    pub fn message(&self) -> String {
        match self {
            RpcError::MethodNotFound(m) => format!("Method not found: {}", m),
            RpcError::InvalidParams(m) => format!("Invalid params: {}", m),
            RpcError::InternalError(m) => format!("Internal error: {}", m),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "code": self.code(),
            "message": self.message(),
        })
    }
}

/// Convert difficulty to target hash
fn difficulty_to_target(difficulty: u64) -> H256 {
    if difficulty == 0 {
        return H256([0xff; 32]);
    }
    
    let max = ethereum_types::U256::MAX;
    let target = max / ethereum_types::U256::from(difficulty);
    let mut bytes = [0u8; 32];
    target.to_big_endian(&mut bytes);
    H256(bytes)
}

/// Parse hex string to H256
fn parse_h256(s: &str) -> H256 {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s).unwrap_or_default();
    H256::from_slice(&bytes)
}

/// Format hashrate for display
fn format_hashrate(hashrate: u64) -> String {
    const UNITS: [&str; 6] = ["H/s", "KH/s", "MH/s", "GH/s", "TH/s", "PH/s"];
    
    let mut value = hashrate as f64;
    let mut unit_idx = 0;
    
    while value >= 1000.0 && unit_idx < UNITS.len() - 1 {
        value /= 1000.0;
        unit_idx += 1;
    }
    
    format!("{:.2} {}", value, UNITS[unit_idx])
}

/// Desktop app mining interface
/// Provides direct integration for the PYRAX desktop wallet's built-in miner
#[derive(Debug, Clone, Serialize)]
pub struct DesktopMiningStatus {
    pub is_mining: bool,
    pub hashrate: u64,
    pub hashrate_formatted: String,
    pub shares_accepted: u64,
    pub shares_rejected: u64,
    pub blocks_found: u64,
    pub current_difficulty: u64,
    pub estimated_earnings_per_day: f64,
    pub stratum_connected: bool,
    pub stratum_url: String,
}

/// Desktop app mining commands
#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum DesktopMiningCommand {
    Start {
        coinbase_address: String,
        gpu_devices: Vec<usize>,
        intensity: u8,
    },
    Stop,
    GetStatus,
    SetIntensity { intensity: u8 },
    BenchmarkDevices,
}

/// Response for desktop mining commands
#[derive(Debug, Serialize)]
pub struct DesktopMiningResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DesktopMiningStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<Vec<GpuDeviceInfo>>,
}

/// GPU device info for desktop app
#[derive(Debug, Serialize)]
pub struct GpuDeviceInfo {
    pub index: usize,
    pub name: String,
    pub vendor: String,
    pub memory_mb: u64,
    pub compute_units: u32,
    pub max_hashrate: u64,
    pub supported: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hashrate() {
        assert_eq!(format_hashrate(1000), "1.00 KH/s");
        assert_eq!(format_hashrate(1_000_000), "1.00 MH/s");
        assert_eq!(format_hashrate(1_000_000_000), "1.00 GH/s");
    }

    #[test]
    fn test_difficulty_to_target() {
        let target1 = difficulty_to_target(1);
        let target100 = difficulty_to_target(100);
        
        // Higher difficulty = lower target
        assert!(target100 < target1);
    }

    #[test]
    fn test_rpc_error() {
        let err = RpcError::MethodNotFound("test".to_string());
        assert_eq!(err.code(), -32601);
    }
}
