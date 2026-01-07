//! Transaction Broadcaster for PYRAX
//!
//! Production-ready transaction broadcasting with real network connections.
//! Supports devnet, testnet, and mainnet.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use crate::rpc::{RpcClient, NetworkConfig, RpcClientError};
use crate::types::{H256, Address, Transaction};

/// Transaction broadcast result
#[derive(Debug, Clone)]
pub struct BroadcastResult {
    pub tx_hash: H256,
    pub network: String,
    pub success: bool,
    pub confirmations: u32,
    pub error: Option<String>,
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxStatus {
    Pending,
    Confirmed(u32),  // Number of confirmations
    Failed(String),
    Unknown,
}

/// Transaction broadcaster with real network connections
pub struct Broadcaster {
    client: Arc<RpcClient>,
    pending_txs: Arc<RwLock<Vec<PendingTx>>>,
}

#[derive(Debug, Clone)]
struct PendingTx {
    hash: H256,
    raw_tx: Vec<u8>,
    submitted_at: std::time::Instant,
    retries: u32,
}

impl Broadcaster {
    /// Create a new broadcaster for devnet
    pub fn devnet() -> Self {
        Self::new(NetworkConfig::devnet())
    }

    /// Create a new broadcaster for testnet
    pub fn testnet() -> Self {
        Self::new(NetworkConfig::testnet())
    }

    /// Create a new broadcaster for mainnet
    pub fn mainnet() -> Self {
        Self::new(NetworkConfig::mainnet())
    }

    /// Create a new broadcaster with custom config
    pub fn new(config: NetworkConfig) -> Self {
        info!("Broadcaster initialized for {}", config.name);
        Self {
            client: Arc::new(RpcClient::new(config)),
            pending_txs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the network name
    pub fn network_name(&self) -> &str {
        &self.client.network().name
    }

    /// Check if connected to the network
    pub async fn is_connected(&self) -> bool {
        self.client.is_connected().await
    }

    /// Broadcast a signed transaction
    pub async fn broadcast(&self, signed_tx: &[u8]) -> Result<BroadcastResult, BroadcastError> {
        // Check connection first
        if !self.is_connected().await {
            return Err(BroadcastError::NotConnected);
        }

        info!("Broadcasting transaction ({} bytes)", signed_tx.len());

        // Send to network
        match self.client.send_raw_transaction(signed_tx).await {
            Ok(tx_hash) => {
                info!("Transaction broadcast successful: {}", tx_hash);
                
                // Add to pending list for tracking
                self.pending_txs.write().await.push(PendingTx {
                    hash: tx_hash,
                    raw_tx: signed_tx.to_vec(),
                    submitted_at: std::time::Instant::now(),
                    retries: 0,
                });

                Ok(BroadcastResult {
                    tx_hash,
                    network: self.network_name().to_string(),
                    success: true,
                    confirmations: 0,
                    error: None,
                })
            }
            Err(e) => {
                error!("Transaction broadcast failed: {}", e);
                Err(BroadcastError::RpcError(e.to_string()))
            }
        }
    }

    /// Get transaction status
    pub async fn get_status(&self, tx_hash: &H256) -> TxStatus {
        match self.client.get_transaction_receipt(tx_hash).await {
            Ok(Some(receipt)) => {
                // Parse block number from receipt
                if let Ok(block_num) = u64::from_str_radix(
                    receipt.block_number.trim_start_matches("0x"), 16
                ) {
                    // Get current block to calculate confirmations
                    if let Ok(current_block) = self.client.get_block_number().await {
                        let confirmations = current_block.saturating_sub(block_num) as u32;
                        
                        // Check status
                        let status_ok = receipt.status == "0x1" || receipt.status == "1";
                        if status_ok {
                            return TxStatus::Confirmed(confirmations);
                        } else {
                            return TxStatus::Failed("Transaction reverted".to_string());
                        }
                    }
                }
                TxStatus::Confirmed(1)
            }
            Ok(None) => {
                // Check if transaction exists but not yet mined
                match self.client.get_transaction(tx_hash).await {
                    Ok(Some(_)) => TxStatus::Pending,
                    _ => TxStatus::Unknown,
                }
            }
            Err(e) => {
                warn!("Error checking transaction status: {}", e);
                TxStatus::Unknown
            }
        }
    }

    /// Wait for transaction confirmation
    pub async fn wait_for_confirmation(
        &self,
        tx_hash: &H256,
        required_confirmations: u32,
        timeout_secs: u64,
    ) -> Result<u32, BroadcastError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                return Err(BroadcastError::Timeout);
            }

            match self.get_status(tx_hash).await {
                TxStatus::Confirmed(confs) => {
                    if confs >= required_confirmations {
                        info!("Transaction {} confirmed with {} confirmations", tx_hash, confs);
                        return Ok(confs);
                    }
                    debug!("Transaction has {} confirmations, waiting for {}", confs, required_confirmations);
                }
                TxStatus::Failed(reason) => {
                    return Err(BroadcastError::TransactionFailed(reason));
                }
                TxStatus::Pending => {
                    debug!("Transaction pending, waiting...");
                }
                TxStatus::Unknown => {
                    warn!("Transaction status unknown, retrying...");
                }
            }

            // Wait before next check
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: &Address) -> Result<u64, BroadcastError> {
        self.client.get_balance(address).await
            .map_err(|e| BroadcastError::RpcError(e.to_string()))
    }

    /// Get UTXOs for an address
    pub async fn get_utxos(&self, address: &Address) -> Result<Vec<crate::rpc::client::Utxo>, BroadcastError> {
        self.client.get_utxos(address).await
            .map_err(|e| BroadcastError::RpcError(e.to_string()))
    }

    /// Get current nonce for an address
    pub async fn get_nonce(&self, address: &Address) -> Result<u64, BroadcastError> {
        self.client.get_transaction_count(address).await
            .map_err(|e| BroadcastError::RpcError(e.to_string()))
    }

    /// Get current gas price
    pub async fn get_gas_price(&self) -> Result<u64, BroadcastError> {
        self.client.gas_price().await
            .map_err(|e| BroadcastError::RpcError(e.to_string()))
    }

    /// Get chain info
    pub async fn get_chain_info(&self) -> Result<crate::rpc::client::ChainInfo, BroadcastError> {
        self.client.get_chain_info().await
            .map_err(|e| BroadcastError::RpcError(e.to_string()))
    }

    /// Retry failed pending transactions
    pub async fn retry_pending(&self) -> Vec<BroadcastResult> {
        let mut results = Vec::new();
        let mut pending = self.pending_txs.write().await;
        
        let mut i = 0;
        while i < pending.len() {
            let tx = &mut pending[i];
            
            // Skip if too many retries
            if tx.retries >= 3 {
                pending.remove(i);
                continue;
            }

            // Check if already confirmed
            match self.get_status(&tx.hash).await {
                TxStatus::Confirmed(confs) => {
                    results.push(BroadcastResult {
                        tx_hash: tx.hash,
                        network: self.network_name().to_string(),
                        success: true,
                        confirmations: confs,
                        error: None,
                    });
                    pending.remove(i);
                    continue;
                }
                TxStatus::Failed(reason) => {
                    results.push(BroadcastResult {
                        tx_hash: tx.hash,
                        network: self.network_name().to_string(),
                        success: false,
                        confirmations: 0,
                        error: Some(reason),
                    });
                    pending.remove(i);
                    continue;
                }
                _ => {}
            }

            // Retry if old enough
            if tx.submitted_at.elapsed() > std::time::Duration::from_secs(30) {
                tx.retries += 1;
                if let Ok(hash) = self.client.send_raw_transaction(&tx.raw_tx).await {
                    debug!("Retried transaction: {} (attempt {})", hash, tx.retries);
                }
            }

            i += 1;
        }

        results
    }
}

/// Broadcast error types
#[derive(Debug, thiserror::Error)]
pub enum BroadcastError {
    #[error("Not connected to network")]
    NotConnected,
    #[error("RPC error: {0}")]
    RpcError(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Timeout waiting for confirmation")]
    Timeout,
    #[error("Invalid transaction")]
    InvalidTransaction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster_creation() {
        let broadcaster = Broadcaster::devnet();
        assert_eq!(broadcaster.network_name(), "devnet");

        let broadcaster = Broadcaster::testnet();
        assert_eq!(broadcaster.network_name(), "testnet");

        let broadcaster = Broadcaster::mainnet();
        assert_eq!(broadcaster.network_name(), "mainnet");
    }

    #[test]
    fn test_tx_status_enum() {
        assert_eq!(TxStatus::Pending, TxStatus::Pending);
        assert_eq!(TxStatus::Confirmed(5), TxStatus::Confirmed(5));
        assert_ne!(TxStatus::Confirmed(5), TxStatus::Confirmed(6));
    }
}
