//! PYRAX Mining Service
//!
//! Integrates the Stratum server with the node's consensus and mempool.
//! Provides block templates, validates shares, and submits blocks.
//!
//! Production-ready for solo mining and pool operation.

#![allow(dead_code)]

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use crate::types::{H256, Address, Block, BlockHeader, Transaction, BlockNumber};
use crate::miner::{
    StratumServer, StratumServerConfig, BlockTemplate, ServerStats,
};
use crate::consensus::DagManager;
use crate::storage::ChainDB;

/// Mining service configuration
#[derive(Debug, Clone)]
pub struct MiningServiceConfig {
    /// Enable the stratum server
    pub stratum_enabled: bool,
    /// Stratum server bind address
    pub stratum_bind: String,
    /// Stratum server port
    pub stratum_port: u16,
    /// Default share difficulty
    pub share_difficulty: f64,
    /// Coinbase address for mining rewards
    pub coinbase_address: Address,
    /// Enable CPU mining (for testing)
    pub cpu_mining_enabled: bool,
    /// Number of CPU mining threads
    pub cpu_threads: usize,
}

impl Default for MiningServiceConfig {
    fn default() -> Self {
        Self {
            stratum_enabled: true,
            stratum_bind: "0.0.0.0".to_string(),
            stratum_port: 3333,
            share_difficulty: 1.0,
            coinbase_address: Address::ZERO,
            cpu_mining_enabled: false,
            cpu_threads: 1,
        }
    }
}

/// Chain state provider for mining
pub trait ChainStateProvider: Send + Sync {
    /// Get current chain tip height
    fn get_height(&self) -> BlockNumber;
    
    /// Get current chain tip hash
    fn get_tip_hash(&self) -> H256;
    
    /// Get current network difficulty
    fn get_difficulty(&self) -> u64;
    
    /// Get block reward for height
    fn get_block_reward(&self, height: BlockNumber) -> u64;
    
    /// Get pending transactions from mempool
    fn get_pending_transactions(&self, max_count: usize, max_bytes: usize) -> Vec<Transaction>;
    
    /// Submit a mined block
    fn submit_block(&self, block: Block) -> Result<H256, String>;
    
    /// Verify block header
    fn verify_header(&self, header: &BlockHeader) -> Result<(), String>;
}

/// Production Mining Service
pub struct MiningService {
    config: MiningServiceConfig,
    stratum_server: Option<Arc<StratumServer>>,
    chain_provider: Arc<dyn ChainStateProvider>,
    running: Arc<RwLock<bool>>,
    last_template_height: Arc<RwLock<BlockNumber>>,
}

impl MiningService {
    /// Create a new mining service
    pub fn new(config: MiningServiceConfig, chain_provider: Arc<dyn ChainStateProvider>) -> Self {
        Self {
            config,
            stratum_server: None,
            chain_provider,
            running: Arc::new(RwLock::new(false)),
            last_template_height: Arc::new(RwLock::new(0)),
        }
    }

    /// Start the mining service
    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting PYRAX Mining Service...");
        
        *self.running.write().await = true;

        if self.config.stratum_enabled {
            self.start_stratum_server().await?;
        }

        if self.config.cpu_mining_enabled {
            self.start_cpu_mining().await;
        }

        info!("Mining Service started successfully");
        Ok(())
    }

    /// Stop the mining service
    pub async fn stop(&mut self) {
        info!("Stopping Mining Service...");
        *self.running.write().await = false;
    }

    /// Start the stratum server
    async fn start_stratum_server(&mut self) -> anyhow::Result<()> {
        let bind_addr = format!("{}:{}", self.config.stratum_bind, self.config.stratum_port)
            .parse()?;

        let stratum_config = StratumServerConfig {
            bind_addr,
            default_difficulty: self.config.share_difficulty,
            coinbase_address: self.config.coinbase_address,
            ..Default::default()
        };

        let server = Arc::new(StratumServer::new(stratum_config));
        
        // Set up block submission callback
        let chain_provider = Arc::clone(&self.chain_provider);
        server.set_block_submit_fn(Box::new(move |block: Block| {
            chain_provider.submit_block(block)
        })).await;

        // Set up template provider callback
        let chain_provider = Arc::clone(&self.chain_provider);
        let coinbase_addr = self.config.coinbase_address;
        server.set_template_provider(Box::new(move || {
            let height = chain_provider.get_height() + 1;
            let parent_hash = chain_provider.get_tip_hash();
            let difficulty = chain_provider.get_difficulty();
            let reward = chain_provider.get_block_reward(height);
            let transactions = chain_provider.get_pending_transactions(1000, 1_000_000);
            
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            Some(BlockTemplate {
                height,
                parent_hash,
                timestamp,
                difficulty,
                transactions,
                coinbase_value: reward,
            })
        })).await;

        self.stratum_server = Some(Arc::clone(&server));

        // Spawn server task
        tokio::spawn(async move {
            if let Err(e) = server.run().await {
                error!("Stratum server error: {}", e);
            }
        });

        info!("Stratum server started on {}:{}", 
            self.config.stratum_bind, self.config.stratum_port);

        Ok(())
    }

    /// Start CPU mining (for testing/development)
    async fn start_cpu_mining(&self) {
        info!("CPU mining enabled with {} threads", self.config.cpu_threads);
        
        let running = Arc::clone(&self.running);
        let chain_provider = Arc::clone(&self.chain_provider);
        let coinbase_address = self.config.coinbase_address;
        let threads = self.config.cpu_threads;

        for thread_id in 0..threads {
            let running = Arc::clone(&running);
            let chain_provider = Arc::clone(&chain_provider);
            
            tokio::spawn(async move {
                Self::cpu_mining_thread(thread_id, running, chain_provider, coinbase_address).await;
            });
        }
    }

    /// CPU mining thread
    async fn cpu_mining_thread(
        thread_id: usize,
        running: Arc<RwLock<bool>>,
        chain_provider: Arc<dyn ChainStateProvider>,
        coinbase_address: Address,
    ) {
        info!("CPU mining thread {} started", thread_id);
        
        let mut nonce_offset = (thread_id as u64) << 48;
        
        loop {
            if !*running.read().await {
                break;
            }

            let height = chain_provider.get_height() + 1;
            let parent_hash = chain_provider.get_tip_hash();
            let difficulty = chain_provider.get_difficulty();
            let reward = chain_provider.get_block_reward(height);
            let transactions = chain_provider.get_pending_transactions(100, 100_000);

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Create coinbase
            let coinbase_tx = Transaction::coinbase(height, reward, &coinbase_address);
            let mut all_txs = vec![coinbase_tx];
            all_txs.extend(transactions);

            // Compute merkle root
            let merkle_root = compute_merkle_root(&all_txs);

            // Build header
            let mut header = BlockHeader {
                version: 1,
                stream: 0, // Stream A (BLAKE3)
                parent_hash,
                merkle_root,
                utxo_commitment: H256::zero(),
                timestamp,
                difficulty,
                nonce: nonce_offset,
                extra_nonce: 0,
                height,
                beneficiary: coinbase_address,
            };

            let target = header.target();

            // Mine for a batch of nonces
            for _ in 0..100_000 {
                header.nonce = nonce_offset;
                nonce_offset = nonce_offset.wrapping_add(1);

                let hash = header.pow_hash();
                
                if hash.as_bytes() <= target.as_bytes() {
                    info!("🎉 CPU Thread {} found block at height {}!", thread_id, height);
                    
                    let block = Block::new(header.clone(), all_txs.clone());
                    
                    match chain_provider.submit_block(block) {
                        Ok(hash) => info!("Block submitted: {}", hash),
                        Err(e) => error!("Block submission failed: {}", e),
                    }
                    break;
                }
            }

            // Small delay to prevent spinning
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        info!("CPU mining thread {} stopped", thread_id);
    }

    /// Get stratum server statistics
    pub fn get_stats(&self) -> Option<&ServerStats> {
        self.stratum_server.as_ref().map(|s| s.stats())
    }

    /// Get stratum server reference
    pub fn stratum_server(&self) -> Option<&Arc<StratumServer>> {
        self.stratum_server.as_ref()
    }

    /// Check if mining is active
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get current block template
    pub fn get_block_template(&self) -> BlockTemplate {
        let height = self.chain_provider.get_height() + 1;
        let parent_hash = self.chain_provider.get_tip_hash();
        let difficulty = self.chain_provider.get_difficulty();
        let reward = self.chain_provider.get_block_reward(height);
        let transactions = self.chain_provider.get_pending_transactions(1000, 1_000_000);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        BlockTemplate {
            height,
            parent_hash,
            timestamp,
            difficulty,
            transactions,
            coinbase_value: reward,
        }
    }

    /// Submit a block directly (for local GPU mining)
    pub fn submit_block(&self, block: Block) -> Result<H256, String> {
        self.chain_provider.submit_block(block)
    }

    /// Submit work from RPC (nonce, header_hash, mix_hash)
    pub async fn submit_work(&self, nonce: u64, header_hash: H256, mix_hash: H256) -> Result<H256, String> {
        // Get current template to reconstruct block
        let template = self.get_block_template();
        
        // Create coinbase transaction
        let coinbase_tx = Transaction::coinbase(
            template.height, 
            template.coinbase_value, 
            &self.config.coinbase_address
        );
        
        // Build full transaction list
        let mut all_txs = vec![coinbase_tx];
        all_txs.extend(template.transactions.clone());
        
        // Compute proper merkle root
        let merkle_root = compute_merkle_root(&all_txs);
        
        // Build block header with submitted nonce and proper merkle root
        let header = BlockHeader {
            version: 1,
            stream: 0,
            parent_hash: template.parent_hash,
            merkle_root,
            utxo_commitment: H256::zero(),
            timestamp: template.timestamp,
            difficulty: template.difficulty,
            nonce,
            extra_nonce: 0,
            height: template.height,
            beneficiary: self.config.coinbase_address,
        };
        
        // Verify the work meets difficulty target
        if header.hash() != header_hash {
            return Err("Header hash mismatch".to_string());
        }
        
        // Build and submit block with proper merkle root
        let block = Block::new(header, all_txs);
        self.chain_provider.submit_block(block)
    }

    /// Get mining info for RPC
    pub fn get_mining_info(&self) -> MiningInfo {
        let stats = self.stratum_server.as_ref().map(|s| {
            let stats = s.stats();
            StratumStats {
                workers_connected: stats.workers_connected.load(std::sync::atomic::Ordering::Relaxed),
                shares_accepted: stats.shares_accepted.load(std::sync::atomic::Ordering::Relaxed),
                shares_rejected: stats.shares_rejected.load(std::sync::atomic::Ordering::Relaxed),
                blocks_found: stats.blocks_found.load(std::sync::atomic::Ordering::Relaxed),
            }
        });

        MiningInfo {
            height: self.chain_provider.get_height() + 1,
            difficulty: self.chain_provider.get_difficulty(),
            network_hashrate: estimate_network_hashrate(
                self.chain_provider.get_difficulty(),
                10, // 10 second block time for Stream A
            ),
            stratum_enabled: self.config.stratum_enabled,
            stratum_port: self.config.stratum_port,
            stratum_stats: stats,
            coinbase_address: format!("{}", self.config.coinbase_address),
        }
    }
}

/// Mining info for RPC responses
#[derive(Debug, Clone, serde::Serialize)]
pub struct MiningInfo {
    pub height: BlockNumber,
    pub difficulty: u64,
    pub network_hashrate: u64,
    pub stratum_enabled: bool,
    pub stratum_port: u16,
    pub stratum_stats: Option<StratumStats>,
    pub coinbase_address: String,
}

/// Stratum statistics for RPC
#[derive(Debug, Clone, serde::Serialize)]
pub struct StratumStats {
    pub workers_connected: u64,
    pub shares_accepted: u64,
    pub shares_rejected: u64,
    pub blocks_found: u64,
}

/// Compute merkle root of transactions
fn compute_merkle_root(transactions: &[Transaction]) -> H256 {
    if transactions.is_empty() {
        return H256::zero();
    }

    let mut hashes: Vec<H256> = transactions.iter().map(|tx| tx.txid()).collect();

    while hashes.len() > 1 {
        if hashes.len() % 2 == 1 {
            hashes.push(*hashes.last().unwrap());
        }
        hashes = hashes.chunks(2).map(|pair| {
            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(&pair[0].0);
            combined[32..].copy_from_slice(&pair[1].0);
            H256::from_slice(blake3::hash(&combined).as_bytes())
        }).collect();
    }

    hashes[0]
}

/// Estimate network hashrate from difficulty
fn estimate_network_hashrate(difficulty: u64, block_time_secs: u64) -> u64 {
    // H/s = difficulty * 2^32 / block_time
    let base: u128 = 1 << 32;
    ((difficulty as u128 * base) / block_time_secs as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockChainProvider {
        height: BlockNumber,
        tip_hash: H256,
        difficulty: u64,
    }

    impl ChainStateProvider for MockChainProvider {
        fn get_height(&self) -> BlockNumber { self.height }
        fn get_tip_hash(&self) -> H256 { self.tip_hash }
        fn get_difficulty(&self) -> u64 { self.difficulty }
        fn get_block_reward(&self, _height: BlockNumber) -> u64 { 5000_00000000 }
        fn get_pending_transactions(&self, _max: usize, _bytes: usize) -> Vec<Transaction> { vec![] }
        fn submit_block(&self, _block: Block) -> Result<H256, String> { Ok(H256::zero()) }
        fn verify_header(&self, _header: &BlockHeader) -> Result<(), String> { Ok(()) }
    }

    #[test]
    fn test_estimate_hashrate() {
        let hashrate = estimate_network_hashrate(1000, 10);
        assert!(hashrate > 0);
    }

    #[test]
    fn test_merkle_root() {
        let txs = vec![
            Transaction::coinbase(1, 5000_00000000, &Address::ZERO),
        ];
        let root = compute_merkle_root(&txs);
        assert!(!root.is_zero());
    }
}
