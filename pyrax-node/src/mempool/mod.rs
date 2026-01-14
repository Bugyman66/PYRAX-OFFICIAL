//! Transaction Mempool for PYRAX
//!
//! Holds pending UTXO transactions ordered by fee rate.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info};

use crate::types::{Transaction, H256, OutPoint};

/// Mempool configuration
#[derive(Debug, Clone)]
pub struct MempoolConfig {
    /// Maximum number of transactions
    pub max_size: usize,
    /// Maximum transaction size in bytes
    pub max_tx_size: usize,
    /// Minimum fee rate (satoshis per byte)
    pub min_fee_rate: u64,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10_000,
            max_tx_size: 100_000,
            min_fee_rate: 1,
        }
    }
}

/// UTXO-based transaction mempool
#[derive(Clone)]
pub struct Mempool {
    inner: Arc<RwLock<MempoolInner>>,
    config: MempoolConfig,
}

struct MempoolInner {
    /// Transactions by txid
    by_hash: HashMap<H256, MempoolEntry>,
    /// Track spent outpoints to detect double-spends
    spent_outpoints: HashSet<OutPoint>,
    /// Total bytes of all transactions in mempool
    total_bytes: usize,
    /// Total fees of all transactions
    total_fees: u64,
}

struct MempoolEntry {
    tx: Transaction,
    fee: u64,
    size: usize,
    added_time: std::time::Instant,
}

impl Mempool {
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(MempoolInner {
                by_hash: HashMap::new(),
                spent_outpoints: HashSet::new(),
                total_bytes: 0,
                total_fees: 0,
            })),
            config,
        }
    }

    /// Add a transaction to the mempool
    pub fn add(&self, tx: Transaction, input_value: u64) -> Result<(), MempoolError> {
        let txid = tx.txid();
        
        // Calculate fee
        let output_value = tx.total_output();
        if input_value < output_value {
            return Err(MempoolError::InsufficientFee);
        }
        let fee = input_value - output_value;
        
        // Check size
        let size = bincode::serialize(&tx).map(|v| v.len()).unwrap_or(0);
        if size > self.config.max_tx_size {
            return Err(MempoolError::TransactionTooLarge);
        }
        
        // Check fee rate
        let fee_rate = fee / size as u64;
        if fee_rate < self.config.min_fee_rate {
            return Err(MempoolError::FeeTooLow);
        }
        
        // Can't be coinbase
        if tx.is_coinbase() {
            return Err(MempoolError::CoinbaseNotAllowed);
        }

        let mut inner = self.inner.write();

        // Check capacity
        if inner.by_hash.len() >= self.config.max_size {
            return Err(MempoolError::PoolFull);
        }

        // Check for duplicate
        if inner.by_hash.contains_key(&txid) {
            return Err(MempoolError::DuplicateTransaction);
        }

        // Check for double-spend
        for input in &tx.inputs {
            if inner.spent_outpoints.contains(&input.previous_output) {
                return Err(MempoolError::DoubleSpend);
            }
        }

        // Add spent outpoints
        for input in &tx.inputs {
            inner.spent_outpoints.insert(input.previous_output);
        }

        // Update totals
        inner.total_bytes += size;
        inner.total_fees += fee;

        // Add entry
        inner.by_hash.insert(txid, MempoolEntry {
            tx,
            fee,
            size,
            added_time: std::time::Instant::now(),
        });

        debug!("Added tx {} to mempool (count: {}, bytes: {}, fee: {})", 
            txid, inner.by_hash.len(), inner.total_bytes, fee);
        Ok(())
    }

    /// Remove a transaction
    pub fn remove(&self, txid: &H256) {
        let mut inner = self.inner.write();
        if let Some(entry) = inner.by_hash.remove(txid) {
            inner.total_bytes = inner.total_bytes.saturating_sub(entry.size);
            inner.total_fees = inner.total_fees.saturating_sub(entry.fee);
            for input in &entry.tx.inputs {
                inner.spent_outpoints.remove(&input.previous_output);
            }
        }
    }

    /// Remove transactions that spend any of the given outpoints (after block commit)
    pub fn remove_confirmed(&self, spent: &[OutPoint]) {
        let mut inner = self.inner.write();
        let spent_set: HashSet<_> = spent.iter().collect();
        
        let to_remove: Vec<H256> = inner.by_hash.iter()
            .filter(|(_, entry)| {
                entry.tx.inputs.iter().any(|i| spent_set.contains(&i.previous_output))
            })
            .map(|(txid, _)| *txid)
            .collect();

        for txid in to_remove {
            if let Some(entry) = inner.by_hash.remove(&txid) {
                inner.total_bytes = inner.total_bytes.saturating_sub(entry.size);
                inner.total_fees = inner.total_fees.saturating_sub(entry.fee);
                for input in &entry.tx.inputs {
                    inner.spent_outpoints.remove(&input.previous_output);
                }
            }
        }
    }

    /// Get a transaction by txid
    pub fn get(&self, txid: &H256) -> Option<Transaction> {
        self.inner.read().by_hash.get(txid).map(|e| e.tx.clone())
    }

    /// Check if mempool contains transaction
    pub fn contains(&self, txid: &H256) -> bool {
        self.inner.read().by_hash.contains_key(txid)
    }

    /// Get mempool size
    pub fn len(&self) -> usize {
        self.inner.read().by_hash.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().by_hash.is_empty()
    }

    /// Get total bytes of all transactions in mempool
    pub fn total_bytes(&self) -> usize {
        self.inner.read().total_bytes
    }

    /// Get total fees of all transactions in mempool
    pub fn total_fees(&self) -> u64 {
        self.inner.read().total_fees
    }

    /// Get pending transactions sorted by fee rate (highest first)
    pub fn get_pending(&self, max_count: usize) -> Vec<Transaction> {
        let inner = self.inner.read();
        let mut entries: Vec<_> = inner.by_hash.values().collect();
        
        // Sort by fee rate descending
        entries.sort_by(|a, b| {
            let rate_a = a.fee / a.size as u64;
            let rate_b = b.fee / b.size as u64;
            rate_b.cmp(&rate_a)
        });

        entries.into_iter()
            .take(max_count)
            .map(|e| e.tx.clone())
            .collect()
    }

    /// Check if an outpoint would cause a double-spend
    pub fn would_double_spend(&self, outpoint: &OutPoint) -> bool {
        self.inner.read().spent_outpoints.contains(outpoint)
    }

    /// Clear all transactions
    pub fn clear(&self) {
        let mut inner = self.inner.write();
        inner.by_hash.clear();
        inner.spent_outpoints.clear();
        inner.total_bytes = 0;
        inner.total_fees = 0;
        info!("Mempool cleared");
    }

    /// Get mempool info for RPC
    pub fn get_info(&self) -> MempoolInfo {
        let inner = self.inner.read();
        MempoolInfo {
            size: inner.by_hash.len(),
            bytes: inner.total_bytes,
            total_fees: inner.total_fees,
            max_size: self.config.max_size,
            min_fee_rate: self.config.min_fee_rate,
        }
    }
}

/// Mempool statistics for RPC
#[derive(Debug, Clone, serde::Serialize)]
pub struct MempoolInfo {
    pub size: usize,
    pub bytes: usize,
    pub total_fees: u64,
    pub max_size: usize,
    pub min_fee_rate: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum MempoolError {
    #[error("Transaction too large")]
    TransactionTooLarge,
    #[error("Fee too low")]
    FeeTooLow,
    #[error("Insufficient fee (output > input)")]
    InsufficientFee,
    #[error("Duplicate transaction")]
    DuplicateTransaction,
    #[error("Mempool is full")]
    PoolFull,
    #[error("Double spend detected")]
    DoubleSpend,
    #[error("Coinbase not allowed in mempool")]
    CoinbaseNotAllowed,
}
