//! RPC Client for real node connections
//!
//! Production-ready RPC client that connects to actual pyrax-node.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

/// RPC Client for connecting to pyrax-node
pub struct RpcClient {
    client: reqwest::Client,
    url: String,
}

impl RpcClient {
    /// Create a new RPC client
    pub fn new(url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            url: url.to_string(),
        }
    }

    /// Create client for default localhost
    pub fn localhost(port: u16) -> Self {
        Self::new(&format!("http://127.0.0.1:{}", port))
    }

    /// Send JSON-RPC request
    async fn request<P: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R, RpcError> {
        let request = RpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id: 1,
        };

        debug!("RPC request: {}", method);

        let response = self.client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| RpcError::Network(e.to_string()))?;

        let rpc_response: RpcResponse<R> = response
            .json()
            .await
            .map_err(|e| RpcError::Parse(e.to_string()))?;

        if let Some(error) = rpc_response.error {
            return Err(RpcError::Rpc {
                code: error.code,
                message: error.message,
            });
        }

        rpc_response.result.ok_or(RpcError::NoResult)
    }

    /// Check if node is reachable - uses simple health check first, falls back to chain info
    pub async fn is_connected(&self) -> bool {
        // Try simple health check first (no database access)
        if self.health_check().await.is_ok() {
            return true;
        }
        // Fall back to chain info check
        match self.get_block_number().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Simple health check that doesn't require database access
    pub async fn health_check(&self) -> Result<String, RpcError> {
        self.request("pyrax_health", ()).await
    }

    // ═══════════════════════════════════════════════════════════════
    // Chain Methods
    // ═══════════════════════════════════════════════════════════════

    /// Get current block number (uses chain info)
    pub async fn get_block_number(&self) -> Result<u64, RpcError> {
        let info: ChainInfoResponse = self.request("pyrax_getChainInfo", ()).await?;
        Ok(info.best_block_height)
    }

    /// Get chain info
    pub async fn get_chain_info(&self) -> Result<ChainInfoResponse, RpcError> {
        self.request("pyrax_getChainInfo", ()).await
    }

    /// Get block by number
    pub async fn get_block_by_number(&self, number: u64, full_txs: bool) -> Result<Option<BlockResponse>, RpcError> {
        self.request("pyrax_getBlockByNumber", (number, full_txs)).await
    }

    /// Get block by hash
    pub async fn get_block_by_hash(&self, hash: &str, full_txs: bool) -> Result<Option<BlockResponse>, RpcError> {
        self.request("pyrax_getBlockByHash", (hash, full_txs)).await
    }

    // ═══════════════════════════════════════════════════════════════
    // Account Methods
    // ═══════════════════════════════════════════════════════════════

    /// Get balance
    pub async fn get_balance(&self, address: &str) -> Result<u64, RpcError> {
        let result: BalanceResponse = self.request("pyrax_getBalance", (address,)).await?;
        Ok(result.balance)
    }

    /// Get transaction count (nonce) - not yet implemented in node
    pub async fn get_transaction_count(&self, address: &str) -> Result<u64, RpcError> {
        // PYRAX uses UTXO model, no nonce
        Ok(0)
    }

    /// Get UTXOs for address
    pub async fn get_utxos(&self, address: &str) -> Result<Vec<UtxoResponse>, RpcError> {
        self.request("pyrax_getUtxos", (address,)).await
    }

    // ═══════════════════════════════════════════════════════════════
    // Transaction Methods
    // ═══════════════════════════════════════════════════════════════

    /// Send raw transaction
    pub async fn send_raw_transaction(&self, tx_hex: &str) -> Result<String, RpcError> {
        let result: SubmitResponse = self.request("pyrax_sendRawTransaction", (tx_hex,)).await?;
        result.hash.ok_or(RpcError::NoResult)
    }

    /// Get transaction by hash
    pub async fn get_transaction(&self, hash: &str) -> Result<Option<TransactionResponse>, RpcError> {
        self.request("pyrax_getTransaction", (hash,)).await
    }

    /// Get transaction receipt - PYRAX uses UTXO, no receipts
    pub async fn get_transaction_receipt(&self, hash: &str) -> Result<Option<ReceiptResponse>, RpcError> {
        // UTXO model doesn't have receipts like account model
        Ok(None)
    }

    // ═══════════════════════════════════════════════════════════════
    // Network Methods
    // ═══════════════════════════════════════════════════════════════

    /// Get peer info
    pub async fn get_peers(&self) -> Result<Vec<PeerResponse>, RpcError> {
        self.request("pyrax_getPeers", ()).await
    }

    /// Get mempool info
    pub async fn get_mempool_info(&self) -> Result<MempoolResponse, RpcError> {
        self.request("pyrax_getMempoolInfo", ()).await
    }

    /// Check if syncing
    pub async fn is_syncing(&self) -> Result<SyncingResponse, RpcError> {
        let info: ChainInfoResponse = self.request("pyrax_getChainInfo", ()).await?;
        Ok(SyncingResponse { syncing: info.syncing })
    }

    // ═══════════════════════════════════════════════════════════════
    // Mining Methods
    // ═══════════════════════════════════════════════════════════════

    /// Get mining info
    pub async fn get_mining_info(&self) -> Result<MiningInfoResponse, RpcError> {
        self.request("pyrax_getMiningInfo", ()).await
    }

    /// Get block template
    pub async fn get_block_template(&self) -> Result<BlockTemplateResponse, RpcError> {
        self.request("pyrax_getBlockTemplate", ()).await
    }

    /// Submit block
    pub async fn submit_block(&self, block_hex: &str) -> Result<bool, RpcError> {
        self.request("pyrax_submitBlock", (block_hex,)).await
    }
}

fn parse_hex_u64(s: &str) -> Result<u64, RpcError> {
    let s = s.trim_start_matches("0x");
    u64::from_str_radix(s, 16).map_err(|e| RpcError::Parse(e.to_string()))
}

// ═══════════════════════════════════════════════════════════════
// Request/Response Types
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize)]
struct RpcRequest<P: Serialize> {
    jsonrpc: &'static str,
    method: String,
    params: P,
    id: u64,
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    jsonrpc: String,
    result: Option<T>,
    error: Option<RpcErrorResponse>,
    id: u64,
}

#[derive(Deserialize)]
struct RpcErrorResponse {
    code: i32,
    message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("RPC error ({code}): {message}")]
    Rpc { code: i32, message: String },
    #[error("No result")]
    NoResult,
}

// ═══════════════════════════════════════════════════════════════
// Response Types
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChainInfoResponse {
    pub chain_id: u32,
    pub network: String,
    pub best_block_height: u64,
    pub best_block_hash: String,
    pub genesis_hash: String,
    pub difficulty: u64,
    pub utxo_count: u64,
    pub syncing: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
    pub utxo_count: u64,
    pub utxos: Vec<UtxoResponse>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubmitResponse {
    pub accepted: bool,
    pub hash: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockResponse {
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
    pub transaction_count: u32,
    pub transactions: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub hash: String,
    pub nonce: String,
    pub block_hash: Option<String>,
    pub block_number: Option<String>,
    pub transaction_index: Option<String>,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
    pub input: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptResponse {
    pub transaction_hash: String,
    pub block_hash: String,
    pub block_number: String,
    pub status: String,
    pub gas_used: String,
    pub cumulative_gas_used: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UtxoResponse {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub script_pubkey: String,
    pub height: u64,
    pub coinbase: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PeerResponse {
    pub id: String,
    pub address: String,
    pub client_version: String,
    pub best_height: u64,
    pub latency_ms: u32,
    pub direction: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MempoolResponse {
    pub size: u32,
    pub bytes: u64,
    pub pending_count: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncingResponse {
    pub syncing: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MiningInfoResponse {
    pub mining: bool,
    pub hashrate: f64,
    pub difficulty: f64,
    pub blocks_found: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockTemplateResponse {
    pub previous_block_hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub target: String,
    pub coinbase_value: u64,
}
