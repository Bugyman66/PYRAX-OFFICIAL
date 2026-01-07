//! JSON-RPC Client for PYRAX Node
//!
//! Production-ready RPC client for real network connections.
//! Supports devnet, testnet, and mainnet endpoints.

use std::time::Duration;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::types::{H256, Address, Transaction, Block, BlockHeader};

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub explorer_url: Option<String>,
}

impl NetworkConfig {
    pub fn devnet() -> Self {
        Self {
            name: "devnet".to_string(),
            rpc_url: "http://127.0.0.1:8545".to_string(),
            chain_id: 31337,
            explorer_url: None,
        }
    }

    pub fn testnet() -> Self {
        Self {
            name: "testnet".to_string(),
            rpc_url: "https://testnet.pyrax.org/rpc".to_string(),
            chain_id: 7331,
            explorer_url: Some("https://testnet.pyrax.org/explorer".to_string()),
        }
    }

    pub fn mainnet() -> Self {
        Self {
            name: "mainnet".to_string(),
            rpc_url: "https://rpc.pyrax.org".to_string(),
            chain_id: 7330,
            explorer_url: Some("https://explorer.pyrax.org".to_string()),
        }
    }
}

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct RpcRequest<T: Serialize> {
    jsonrpc: &'static str,
    method: String,
    params: T,
    id: u64,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    jsonrpc: String,
    result: Option<T>,
    error: Option<RpcErrorResponse>,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct RpcErrorResponse {
    code: i32,
    message: String,
}

/// RPC client error
#[derive(Debug, thiserror::Error)]
pub enum RpcClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("RPC error ({code}): {message}")]
    Rpc { code: i32, message: String },
    #[error("Invalid response")]
    InvalidResponse,
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Timeout")]
    Timeout,
}

/// PYRAX RPC Client
pub struct RpcClient {
    client: reqwest::Client,
    config: NetworkConfig,
    request_id: std::sync::atomic::AtomicU64,
}

impl RpcClient {
    /// Create a new RPC client for the specified network
    pub fn new(config: NetworkConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        info!("RPC client initialized for {} at {}", config.name, config.rpc_url);

        Self {
            client,
            config,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Create client for devnet
    pub fn devnet() -> Self {
        Self::new(NetworkConfig::devnet())
    }

    /// Create client for testnet
    pub fn testnet() -> Self {
        Self::new(NetworkConfig::testnet())
    }

    /// Create client for mainnet
    pub fn mainnet() -> Self {
        Self::new(NetworkConfig::mainnet())
    }

    /// Get current network configuration
    pub fn network(&self) -> &NetworkConfig {
        &self.config
    }

    /// Send a raw RPC request
    async fn request<P: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R, RpcClientError> {
        let id = self.request_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let request = RpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id,
        };

        debug!("RPC request: {} (id={})", method, id);

        let response = self.client
            .post(&self.config.rpc_url)
            .json(&request)
            .send()
            .await?;

        let rpc_response: RpcResponse<R> = response.json().await?;

        if let Some(error) = rpc_response.error {
            error!("RPC error: {} ({})", error.message, error.code);
            return Err(RpcClientError::Rpc {
                code: error.code,
                message: error.message,
            });
        }

        rpc_response.result.ok_or(RpcClientError::InvalidResponse)
    }

    // ═══════════════════════════════════════════════════════════════
    // Chain Methods
    // ═══════════════════════════════════════════════════════════════

    /// Get chain info
    pub async fn get_chain_info(&self) -> Result<ChainInfo, RpcClientError> {
        self.request("pyrax_chainInfo", ()).await
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64, RpcClientError> {
        let result: String = self.request("eth_blockNumber", ()).await?;
        Ok(u64::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(0))
    }

    /// Get block by number
    pub async fn get_block_by_number(&self, number: u64, full_txs: bool) -> Result<Option<RpcBlock>, RpcClientError> {
        let num_hex = format!("0x{:x}", number);
        self.request("eth_getBlockByNumber", (num_hex, full_txs)).await
    }

    /// Get block by hash
    pub async fn get_block_by_hash(&self, hash: &H256, full_txs: bool) -> Result<Option<RpcBlock>, RpcClientError> {
        let hash_hex = format!("0x{}", hex::encode(hash.as_bytes()));
        self.request("eth_getBlockByHash", (hash_hex, full_txs)).await
    }

    // ═══════════════════════════════════════════════════════════════
    // Account Methods
    // ═══════════════════════════════════════════════════════════════

    /// Get balance of an address
    pub async fn get_balance(&self, address: &Address) -> Result<u64, RpcClientError> {
        let addr_hex = format!("0x{}", hex::encode(&address.0));
        let result: String = self.request("eth_getBalance", (addr_hex, "latest")).await?;
        Ok(u64::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(0))
    }

    /// Get transaction count (nonce) for an address
    pub async fn get_transaction_count(&self, address: &Address) -> Result<u64, RpcClientError> {
        let addr_hex = format!("0x{}", hex::encode(&address.0));
        let result: String = self.request("eth_getTransactionCount", (addr_hex, "latest")).await?;
        Ok(u64::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(0))
    }

    /// Get UTXOs for an address (PYRAX-specific)
    pub async fn get_utxos(&self, address: &Address) -> Result<Vec<Utxo>, RpcClientError> {
        let addr_hex = format!("0x{}", hex::encode(&address.0));
        self.request("pyrax_getUtxos", (addr_hex,)).await
    }

    // ═══════════════════════════════════════════════════════════════
    // Transaction Methods
    // ═══════════════════════════════════════════════════════════════

    /// Send raw transaction
    pub async fn send_raw_transaction(&self, tx_bytes: &[u8]) -> Result<H256, RpcClientError> {
        let tx_hex = format!("0x{}", hex::encode(tx_bytes));
        let result: String = self.request("eth_sendRawTransaction", (tx_hex,)).await?;
        
        let hash_bytes = hex::decode(result.trim_start_matches("0x"))
            .map_err(|_| RpcClientError::InvalidResponse)?;
        
        Ok(H256::from_slice(&hash_bytes))
    }

    /// Get transaction by hash
    pub async fn get_transaction(&self, hash: &H256) -> Result<Option<RpcTransaction>, RpcClientError> {
        let hash_hex = format!("0x{}", hex::encode(hash.as_bytes()));
        self.request("eth_getTransactionByHash", (hash_hex,)).await
    }

    /// Get transaction receipt
    pub async fn get_transaction_receipt(&self, hash: &H256) -> Result<Option<TxReceipt>, RpcClientError> {
        let hash_hex = format!("0x{}", hex::encode(hash.as_bytes()));
        self.request("eth_getTransactionReceipt", (hash_hex,)).await
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<u64, RpcClientError> {
        let result: String = self.request("eth_estimateGas", (tx,)).await?;
        Ok(u64::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(21000))
    }

    /// Get current gas price
    pub async fn gas_price(&self) -> Result<u64, RpcClientError> {
        let result: String = self.request("eth_gasPrice", ()).await?;
        Ok(u64::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(1_000_000_000))
    }

    // ═══════════════════════════════════════════════════════════════
    // Mining Methods
    // ═══════════════════════════════════════════════════════════════

    /// Get block template for mining
    pub async fn get_block_template(&self) -> Result<BlockTemplate, RpcClientError> {
        self.request("pyrax_getBlockTemplate", ()).await
    }

    /// Submit mined block
    pub async fn submit_block(&self, block_hex: &str) -> Result<bool, RpcClientError> {
        self.request("pyrax_submitBlock", (block_hex,)).await
    }

    /// Get mining info
    pub async fn get_mining_info(&self) -> Result<MiningInfo, RpcClientError> {
        self.request("pyrax_getMiningInfo", ()).await
    }

    // ═══════════════════════════════════════════════════════════════
    // Network Methods
    // ═══════════════════════════════════════════════════════════════

    /// Get connected peers
    pub async fn get_peers(&self) -> Result<Vec<PeerInfo>, RpcClientError> {
        self.request("pyrax_getPeers", ()).await
    }

    /// Get network version
    pub async fn net_version(&self) -> Result<String, RpcClientError> {
        self.request("net_version", ()).await
    }

    /// Check if node is syncing
    pub async fn syncing(&self) -> Result<SyncStatus, RpcClientError> {
        self.request("eth_syncing", ()).await
    }

    /// Check connection to node
    pub async fn is_connected(&self) -> bool {
        match self.get_block_number().await {
            Ok(_) => true,
            Err(e) => {
                debug!("Connection check failed: {}", e);
                false
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Response Types
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
    pub chain_id: u64,
    pub network_name: String,
    pub latest_block: u64,
    pub latest_block_hash: String,
    pub total_difficulty: String,
    pub peer_count: u32,
    pub syncing: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlock {
    pub number: String,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: String,
    pub miner: String,
    pub difficulty: String,
    pub total_difficulty: String,
    pub size: String,
    pub gas_used: String,
    pub gas_limit: String,
    pub transactions: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransaction {
    pub hash: String,
    pub nonce: String,
    pub block_hash: Option<String>,
    pub block_number: Option<String>,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
    pub input: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxReceipt {
    pub transaction_hash: String,
    pub block_hash: String,
    pub block_number: String,
    pub status: String,
    pub gas_used: String,
    pub cumulative_gas_used: String,
    pub logs: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    pub from: String,
    pub to: Option<String>,
    pub value: Option<String>,
    pub data: Option<String>,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
    pub nonce: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub script_pubkey: String,
    pub confirmations: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTemplate {
    pub previous_block_hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub transactions: Vec<String>,
    pub coinbase_value: u64,
    pub target: String,
    pub stream: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningInfo {
    pub blocks: u64,
    pub current_difficulty: f64,
    pub network_hashrate: f64,
    pub pool_difficulty: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub version: String,
    pub height: u64,
    pub latency_ms: u32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SyncStatus {
    Syncing {
        starting_block: String,
        current_block: String,
        highest_block: String,
    },
    NotSyncing(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_configs() {
        let devnet = NetworkConfig::devnet();
        assert_eq!(devnet.chain_id, 31337);
        
        let testnet = NetworkConfig::testnet();
        assert_eq!(testnet.chain_id, 7331);
        
        let mainnet = NetworkConfig::mainnet();
        assert_eq!(mainnet.chain_id, 7330);
    }

    #[test]
    fn test_client_creation() {
        let client = RpcClient::devnet();
        assert_eq!(client.network().name, "devnet");
    }
}
