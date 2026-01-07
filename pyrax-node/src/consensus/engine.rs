use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

use super::{Result, ConsensusError, validate_block, calculate_next_difficulty};
use crate::storage::ChainDB;
use crate::mempool::Mempool;
use crate::config::ConsensusConfig;
use crate::types::{Block, BlockHeader, BlockBody, H256, Address, Wei};

pub struct Engine {
    storage: ChainDB,
    mempool: Arc<Mempool>,
    config: ConsensusConfig,
    mining_enabled: Arc<RwLock<bool>>,
}

impl Engine {
    pub fn new(
        storage: ChainDB,
        mempool: Arc<Mempool>,
        config: ConsensusConfig,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            mempool,
            config,
            mining_enabled: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn run(self, mine_cpu: bool, miner_address: Option<String>) -> Result<()> {
        info!("Starting consensus engine");

        if mine_cpu {
            if let Some(addr_str) = miner_address {
                let addr = parse_address(&addr_str)?;
                info!("CPU mining enabled, rewards to {}", addr);
                *self.mining_enabled.write() = true;
                
                let engine = Arc::new(self);
                let mining_engine = engine.clone();
                
                tokio::spawn(async move {
                    mining_engine.mining_loop(addr).await
                });

                engine.validation_loop().await?;
            } else {
                warn!("CPU mining requested but no miner address provided");
                self.validation_loop().await?;
            }
        } else {
            self.validation_loop().await?;
        }

        Ok(())
    }

    async fn validation_loop(&self) -> Result<()> {
        info!("Starting validation loop");
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    async fn mining_loop(&self, beneficiary: Address) {
        info!("Starting mining loop");

        loop {
            if !*self.mining_enabled.read() {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }

            match self.mine_block(beneficiary).await {
                Ok(Some(block)) => {
                    info!("Mined block {} at height {}", 
                        hex::encode(block.hash().as_bytes()),
                        block.height()
                    );
                    
                    if let Err(e) = self.storage.commit_block(&block) {
                        error!("Failed to commit mined block: {}", e);
                    }
                }
                Ok(None) => {
                    // No block found this round
                }
                Err(e) => {
                    warn!("Mining error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn mine_block(&self, beneficiary: Address) -> Result<Option<Block>> {
        let tip = self.storage.load_tip()?;
        let parent = if tip.height > 0 {
            self.storage.get_block(&tip.hash)?
        } else {
            None
        };

        let parent_header = parent.as_ref().map(|b| &b.header);
        
        // Calculate next difficulty
        let difficulty = if let Some(parent) = &parent {
            let grandparent = if parent.height() > 0 {
                self.storage.get_block(&parent.header.parent_hash)?
                    .map(|b| b.header.timestamp)
            } else {
                None
            };
            calculate_next_difficulty(&parent.header, grandparent, &self.config)
        } else {
            self.config.initial_difficulty
        };

        // Get transactions from mempool
        let txs = self.mempool.get_pending_transactions(100);

        // Build block body
        let body = BlockBody::new(txs);

        // Build block header
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut header = BlockHeader {
            version: 1,
            parent_hash: tip.hash,
            merkle_root: body.merkle_root(),
            state_root: H256::zero(), // TODO: Compute state root
            timestamp,
            difficulty,
            nonce: 0,
            height: tip.height + 1,
            extra_nonce: rand::random(),
            beneficiary,
        };

        // CPU mining - try nonces
        let target = BlockHeader::difficulty_to_target(difficulty);
        
        for nonce in 0..1_000_000u64 {
            header.nonce = nonce;
            let hash = header.hash();
            
            if hash.as_bytes() <= target.as_bytes() {
                let block = Block::new(header, body);
                return Ok(Some(block));
            }

            // Yield periodically
            if nonce % 10_000 == 0 {
                tokio::task::yield_now().await;
            }
        }

        Ok(None)
    }

    pub fn process_block(&self, block: Block) -> Result<()> {
        let parent = if block.height() > 0 {
            self.storage.get_block(&block.header.parent_hash)?
        } else {
            None
        };

        validate_block(&block, parent.as_ref(), &self.config)?;
        self.storage.commit_block(&block)?;

        // Remove included transactions from mempool
        for tx in &block.body.transactions {
            self.mempool.remove(&tx.hash());
        }

        Ok(())
    }

    pub fn get_block_reward(&self, height: u64) -> Wei {
        let halvings = height / self.config.halving_interval;
        let reward = self.config.block_reward >> halvings;
        Wei::from(reward)
    }
}

fn parse_address(s: &str) -> Result<Address> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)
        .map_err(|_| ConsensusError::InvalidChain("Invalid address format".into()))?;
    Address::from_slice(&bytes)
        .ok_or_else(|| ConsensusError::InvalidChain("Invalid address length".into()))
}
