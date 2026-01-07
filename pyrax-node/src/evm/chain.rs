//! EVM Sidechain Management
//!
//! Manages EVM blocks, chain state, and block production

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::state::StateDB;
use super::executor::{EvmExecutor, ExecutionResult, TransactionContext};
use super::types::{Address, B256, U256, EvmTransaction, SignedTransaction};
use super::{ChainConfig, EvmBlockHeader, TransactionReceipt, Log};

/// EVM Block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmBlock {
    pub header: EvmBlockHeader,
    pub transactions: Vec<SignedTransaction>,
    pub receipts: Vec<TransactionReceipt>,
}

impl EvmBlock {
    pub fn genesis(config: &ChainConfig) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            header: EvmBlockHeader {
                number: 0,
                hash: [0u8; 32],
                parent_hash: [0u8; 32],
                state_root: [0u8; 32],
                transactions_root: [0u8; 32],
                receipts_root: [0u8; 32],
                logs_bloom: vec![0u8; 256],
                difficulty: 0,
                gas_limit: config.block_gas_limit,
                gas_used: 0,
                timestamp,
                extra_data: b"PYRAX EVM Genesis".to_vec(),
                mix_hash: [0u8; 32],
                nonce: 0,
                base_fee_per_gas: config.base_fee_per_gas,
                withdrawals_root: None,
                blob_gas_used: None,
                excess_blob_gas: None,
            },
            transactions: Vec::new(),
            receipts: Vec::new(),
        }
    }

    pub fn compute_hash(&self) -> B256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.header.number.to_be_bytes());
        hasher.update(&self.header.parent_hash);
        hasher.update(&self.header.state_root);
        hasher.update(&self.header.transactions_root);
        hasher.update(&self.header.timestamp.to_be_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(result.as_bytes());
        hash
    }
}

/// Block producer for EVM sidechain
pub struct BlockProducer {
    config: ChainConfig,
    coinbase: Address,
    pending_transactions: Arc<RwLock<Vec<SignedTransaction>>>,
    last_block_time: Arc<RwLock<u64>>,
}

impl BlockProducer {
    pub fn new(config: ChainConfig, coinbase: Address) -> Self {
        Self {
            config,
            coinbase,
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
            last_block_time: Arc::new(RwLock::new(0)),
        }
    }

    /// Add transaction to pending pool
    pub fn add_transaction(&self, tx: SignedTransaction) -> Result<B256, String> {
        let tx_hash = tx.hash();
        self.pending_transactions.write().push(tx);
        Ok(tx_hash)
    }

    /// Get pending transaction count
    pub fn pending_count(&self) -> usize {
        self.pending_transactions.read().len()
    }

    /// Produce a new block
    pub fn produce_block(
        &self,
        parent: &EvmBlock,
        executor: &mut EvmExecutor,
    ) -> EvmBlock {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Ensure minimum block time
        let min_timestamp = parent.header.timestamp + (self.config.block_time_ms / 1000);
        let block_timestamp = timestamp.max(min_timestamp);

        let ctx = TransactionContext {
            block_number: parent.header.number + 1,
            block_timestamp,
            block_coinbase: self.coinbase,
            block_gas_limit: self.config.block_gas_limit,
            block_difficulty: 0,
            block_basefee: self.calculate_base_fee(parent),
            chain_id: self.config.chain_id,
            prev_randao: parent.header.hash,
        };

        // Take pending transactions
        let pending: Vec<SignedTransaction> = {
            let mut pending = self.pending_transactions.write();
            std::mem::take(&mut *pending)
        };

        // Execute transactions
        let mut executed_txs = Vec::new();
        let mut receipts = Vec::new();
        let mut cumulative_gas = 0u64;
        let mut all_logs = Vec::new();

        for signed_tx in pending {
            if cumulative_gas >= self.config.block_gas_limit {
                // Re-add transaction to pending pool
                self.pending_transactions.write().push(signed_tx);
                continue;
            }

            let tx_hash = signed_tx.hash();
            let from = signed_tx.recover_sender().unwrap_or([0u8; 20]);
            
            let result = executor.execute_transaction(&signed_tx.transaction, from, &ctx);
            
            cumulative_gas += result.gas_used;

            let receipt = TransactionReceipt {
                transaction_hash: tx_hash,
                transaction_index: executed_txs.len() as u64,
                block_hash: [0u8; 32], // Will be set after block hash is computed
                block_number: ctx.block_number,
                from,
                to: signed_tx.transaction.to,
                cumulative_gas_used: cumulative_gas,
                gas_used: result.gas_used,
                contract_address: result.contract_address,
                logs: result.logs.clone(),
                logs_bloom: compute_logs_bloom(&result.logs),
                status: if result.success { 1 } else { 0 },
                effective_gas_price: signed_tx.transaction.effective_gas_price(ctx.block_basefee),
            };

            all_logs.extend(result.logs);
            receipts.push(receipt);
            executed_txs.push(signed_tx);
        }

        // Compute roots
        let state_root = executor.commit();
        let transactions_root = compute_transactions_root(&executed_txs);
        let receipts_root = compute_receipts_root(&receipts);

        let mut block = EvmBlock {
            header: EvmBlockHeader {
                number: ctx.block_number,
                hash: [0u8; 32],
                parent_hash: parent.header.hash,
                state_root,
                transactions_root,
                receipts_root,
                logs_bloom: compute_logs_bloom(&all_logs),
                difficulty: 0,
                gas_limit: self.config.block_gas_limit,
                gas_used: cumulative_gas,
                timestamp: block_timestamp,
                extra_data: Vec::new(),
                mix_hash: [0u8; 32],
                nonce: 0,
                base_fee_per_gas: ctx.block_basefee,
                withdrawals_root: None,
                blob_gas_used: None,
                excess_blob_gas: None,
            },
            transactions: executed_txs,
            receipts,
        };

        // Compute block hash
        block.header.hash = block.compute_hash();

        // Update receipt block hashes
        for receipt in &mut block.receipts {
            receipt.block_hash = block.header.hash;
        }

        *self.last_block_time.write() = block_timestamp;

        info!(
            "Produced EVM block #{} with {} txs, {} gas used",
            block.header.number,
            block.transactions.len(),
            block.header.gas_used
        );

        block
    }

    /// Calculate base fee for next block (EIP-1559)
    fn calculate_base_fee(&self, parent: &EvmBlock) -> u64 {
        let parent_gas_target = parent.header.gas_limit / 2;
        let parent_gas_used = parent.header.gas_used;
        let parent_base_fee = parent.header.base_fee_per_gas;

        if parent_gas_used == parent_gas_target {
            return parent_base_fee;
        }

        if parent_gas_used > parent_gas_target {
            let gas_used_delta = parent_gas_used - parent_gas_target;
            let base_fee_delta = parent_base_fee
                .saturating_mul(gas_used_delta)
                / parent_gas_target
                / 8;
            parent_base_fee.saturating_add(base_fee_delta.max(1))
        } else {
            let gas_used_delta = parent_gas_target - parent_gas_used;
            let base_fee_delta = parent_base_fee
                .saturating_mul(gas_used_delta)
                / parent_gas_target
                / 8;
            parent_base_fee.saturating_sub(base_fee_delta)
                .max(self.config.base_fee_per_gas) // Minimum base fee
        }
    }

    /// Check if it's time to produce a new block
    pub fn should_produce_block(&self, current_time: u64) -> bool {
        let last = *self.last_block_time.read();
        current_time >= last + (self.config.block_time_ms / 1000)
    }
}

/// EVM Chain - manages the full chain state
pub struct EvmChain {
    config: ChainConfig,
    blocks: Arc<RwLock<HashMap<u64, EvmBlock>>>,
    block_hashes: Arc<RwLock<HashMap<B256, u64>>>,
    head: Arc<RwLock<u64>>,
    state: Arc<RwLock<StateDB>>,
    producer: BlockProducer,
}

impl EvmChain {
    /// Create a new EVM chain
    pub fn new(config: ChainConfig, coinbase: Address) -> Self {
        let genesis = EvmBlock::genesis(&config);
        let genesis_hash = genesis.header.hash;
        
        let mut blocks = HashMap::new();
        blocks.insert(0, genesis);
        
        let mut block_hashes = HashMap::new();
        block_hashes.insert(genesis_hash, 0);

        Self {
            config: config.clone(),
            blocks: Arc::new(RwLock::new(blocks)),
            block_hashes: Arc::new(RwLock::new(block_hashes)),
            head: Arc::new(RwLock::new(0)),
            state: Arc::new(RwLock::new(StateDB::new())),
            producer: BlockProducer::new(config, coinbase),
        }
    }

    /// Get current head block number
    pub fn head_number(&self) -> u64 {
        *self.head.read()
    }

    /// Get block by number
    pub fn get_block(&self, number: u64) -> Option<EvmBlock> {
        self.blocks.read().get(&number).cloned()
    }

    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &B256) -> Option<EvmBlock> {
        let number = self.block_hashes.read().get(hash).copied()?;
        self.get_block(number)
    }

    /// Get latest block
    pub fn latest_block(&self) -> Option<EvmBlock> {
        self.get_block(self.head_number())
    }

    /// Submit transaction
    pub fn submit_transaction(&self, tx: SignedTransaction) -> Result<B256, String> {
        // Basic validation
        if tx.transaction.chain_id != self.config.chain_id {
            return Err("Invalid chain ID".to_string());
        }
        
        self.producer.add_transaction(tx)
    }

    /// Produce next block
    pub fn produce_next_block(&self) -> Option<EvmBlock> {
        let parent = self.latest_block()?;
        
        let mut state = self.state.write();
        let mut executor = EvmExecutor::new(self.config.clone(), state.clone());
        
        let block = self.producer.produce_block(&parent, &mut executor);
        
        // Update chain state
        *state = executor.state().clone();
        
        // Store block
        let block_num = block.header.number;
        let block_hash = block.header.hash;
        
        self.blocks.write().insert(block_num, block.clone());
        self.block_hashes.write().insert(block_hash, block_num);
        *self.head.write() = block_num;

        Some(block)
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, hash: &B256) -> Option<(SignedTransaction, u64, u64)> {
        let blocks = self.blocks.read();
        for block in blocks.values() {
            for (idx, tx) in block.transactions.iter().enumerate() {
                if &tx.hash() == hash {
                    return Some((tx.clone(), block.header.number, idx as u64));
                }
            }
        }
        None
    }

    /// Get transaction receipt
    pub fn get_receipt(&self, hash: &B256) -> Option<TransactionReceipt> {
        let blocks = self.blocks.read();
        for block in blocks.values() {
            for receipt in &block.receipts {
                if &receipt.transaction_hash == hash {
                    return Some(receipt.clone());
                }
            }
        }
        None
    }

    /// Get account balance
    pub fn get_balance(&self, address: &Address) -> U256 {
        self.state.read().get_balance(address)
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &Address) -> u64 {
        self.state.read().get_nonce(address)
    }

    /// Get contract code
    pub fn get_code(&self, address: &Address) -> Vec<u8> {
        self.state.read().get_code(address)
    }

    /// Get storage value
    pub fn get_storage(&self, address: &Address, key: &B256) -> U256 {
        self.state.read().get_storage(address, key)
    }

    /// Execute call (read-only)
    pub fn call(
        &self,
        to: Address,
        data: Vec<u8>,
        from: Option<Address>,
        value: U256,
    ) -> ExecutionResult {
        let state = self.state.read().clone();
        let executor = EvmExecutor::new(self.config.clone(), state);
        
        let latest = self.latest_block().unwrap_or_else(|| EvmBlock::genesis(&self.config));
        let ctx = TransactionContext {
            block_number: latest.header.number + 1,
            block_timestamp: latest.header.timestamp + (self.config.block_time_ms / 1000),
            block_coinbase: [0u8; 20],
            block_gas_limit: self.config.block_gas_limit,
            block_difficulty: 0,
            block_basefee: latest.header.base_fee_per_gas,
            chain_id: self.config.chain_id,
            prev_randao: latest.header.hash,
        };

        executor.call(to, data, from, value, &ctx)
    }

    /// Estimate gas for transaction
    pub fn estimate_gas(&self, tx: &EvmTransaction, from: Address) -> Result<u64, String> {
        let state = self.state.read().clone();
        let executor = EvmExecutor::new(self.config.clone(), state);
        
        let latest = self.latest_block().unwrap_or_else(|| EvmBlock::genesis(&self.config));
        let ctx = TransactionContext {
            block_number: latest.header.number + 1,
            block_timestamp: latest.header.timestamp + (self.config.block_time_ms / 1000),
            block_coinbase: [0u8; 20],
            block_gas_limit: self.config.block_gas_limit,
            block_difficulty: 0,
            block_basefee: latest.header.base_fee_per_gas,
            chain_id: self.config.chain_id,
            prev_randao: latest.header.hash,
        };

        executor.estimate_gas(tx, from, &ctx)
    }

    /// Get chain config
    pub fn config(&self) -> &ChainConfig {
        &self.config
    }

    /// Get pending transaction count
    pub fn pending_transaction_count(&self) -> usize {
        self.producer.pending_count()
    }
}

// Helper functions

fn compute_transactions_root(txs: &[SignedTransaction]) -> B256 {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    for tx in txs {
        hasher.update(&tx.hash());
    }
    let result = hasher.finalize();
    let mut root = [0u8; 32];
    root.copy_from_slice(result.as_bytes());
    root
}

fn compute_receipts_root(receipts: &[TransactionReceipt]) -> B256 {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    for receipt in receipts {
        hasher.update(&receipt.transaction_hash);
        hasher.update(&[receipt.status]);
        hasher.update(&receipt.cumulative_gas_used.to_be_bytes());
    }
    let result = hasher.finalize();
    let mut root = [0u8; 32];
    root.copy_from_slice(result.as_bytes());
    root
}

fn compute_logs_bloom(logs: &[Log]) -> Vec<u8> {
    let mut bloom = vec![0u8; 256];
    for log in logs {
        // Simple bloom filter implementation
        let addr_hash = blake3::hash(&log.address);
        let idx = (addr_hash.as_bytes()[0] as usize) % 256;
        bloom[idx / 8] |= 1 << (idx % 8);
        
        for topic in &log.topics {
            let topic_hash = blake3::hash(topic);
            let idx = (topic_hash.as_bytes()[0] as usize) % 256;
            bloom[idx / 8] |= 1 << (idx % 8);
        }
    }
    bloom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let config = ChainConfig::default();
        let genesis = EvmBlock::genesis(&config);
        assert_eq!(genesis.header.number, 0);
        assert_eq!(genesis.header.gas_limit, config.block_gas_limit);
    }

    #[test]
    fn test_chain_creation() {
        let config = ChainConfig::default();
        let coinbase = [1u8; 20];
        let chain = EvmChain::new(config, coinbase);
        
        assert_eq!(chain.head_number(), 0);
        assert!(chain.latest_block().is_some());
    }

    #[test]
    fn test_block_production() {
        let config = ChainConfig::default();
        let coinbase = [1u8; 20];
        let chain = EvmChain::new(config, coinbase);
        
        // Produce empty block
        let block = chain.produce_next_block().unwrap();
        assert_eq!(block.header.number, 1);
        assert_eq!(chain.head_number(), 1);
    }

    #[test]
    fn test_base_fee_calculation() {
        let config = ChainConfig::default();
        let producer = BlockProducer::new(config.clone(), [0u8; 20]);
        
        let mut parent = EvmBlock::genesis(&config);
        
        // Exactly at target - base fee unchanged
        parent.header.gas_used = parent.header.gas_limit / 2;
        let base_fee = producer.calculate_base_fee(&parent);
        assert_eq!(base_fee, parent.header.base_fee_per_gas);
        
        // Above target - base fee increases
        parent.header.gas_used = parent.header.gas_limit;
        let base_fee = producer.calculate_base_fee(&parent);
        assert!(base_fee > parent.header.base_fee_per_gas);
        
        // Below target - base fee decreases
        parent.header.gas_used = 0;
        let base_fee = producer.calculate_base_fee(&parent);
        assert!(base_fee <= parent.header.base_fee_per_gas);
    }
}
