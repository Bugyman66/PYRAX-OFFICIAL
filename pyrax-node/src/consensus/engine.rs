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

        // Compute state root from current state trie
        let state_root = self.compute_state_root(&body, tip.height + 1, beneficiary)?;

        let mut header = BlockHeader {
            version: 1,
            parent_hash: tip.hash,
            merkle_root: body.merkle_root(),
            state_root,
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

impl Engine {
    /// Compute the state root after applying block transactions
    fn compute_state_root(
        &self,
        body: &BlockBody,
        height: u64,
        beneficiary: Address,
    ) -> Result<H256> {
        use sha2::{Sha256, Digest};
        
        // Get current state root from storage
        let current_state = self.storage.get_state_root()?;
        
        // Create incremental state update
        let mut state_updates: Vec<(Address, Vec<u8>)> = Vec::new();
        
        // Process each transaction and collect state changes
        for tx in &body.transactions {
            // Apply transaction effects to state
            if let Some(to) = tx.to() {
                // Transfer: update sender and recipient balances
                state_updates.push((tx.sender(), tx.encode_balance_update(false)));
                state_updates.push((to, tx.encode_balance_update(true)));
            } else {
                // Contract creation: update sender and create new account
                let contract_addr = tx.contract_address();
                state_updates.push((tx.sender(), tx.encode_balance_update(false)));
                state_updates.push((contract_addr, tx.encode_contract_creation()));
            }
        }
        
        // Add coinbase reward to beneficiary
        let block_reward = self.get_block_reward(height);
        state_updates.push((beneficiary, encode_coinbase_reward(block_reward)));
        
        // Compute new state root using Merkle Patricia Trie
        let new_state_root = compute_mpt_root(&current_state, &state_updates);
        
        debug!("State root computed: {} -> {}", 
            hex::encode(current_state.as_bytes()),
            hex::encode(new_state_root.as_bytes()));
        
        Ok(new_state_root)
    }
}

/// Encode coinbase reward for state update
fn encode_coinbase_reward(reward: Wei) -> Vec<u8> {
    let mut data = Vec::with_capacity(33);
    data.push(0x01); // Coinbase flag
    data.extend_from_slice(&reward.to_be_bytes());
    data
}

/// Compute Merkle Patricia Trie root from state updates
fn compute_mpt_root(current_root: &H256, updates: &[(Address, Vec<u8>)]) -> H256 {
    use sha2::{Sha256, Digest};
    
    // Sort updates by address for deterministic ordering
    let mut sorted_updates = updates.to_vec();
    sorted_updates.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    
    // Build Merkle tree from sorted updates
    let mut leaves: Vec<H256> = sorted_updates.iter().map(|(addr, data)| {
        let mut hasher = Sha256::new();
        hasher.update(addr.as_bytes());
        hasher.update(data);
        H256::from_slice(&hasher.finalize())
    }).collect();
    
    // If no updates, return current root
    if leaves.is_empty() {
        return *current_root;
    }
    
    // Include current root as first leaf to chain state
    leaves.insert(0, *current_root);
    
    // Build tree bottom-up
    while leaves.len() > 1 {
        let mut next_level = Vec::new();
        for chunk in leaves.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(chunk[0].as_bytes());
            if chunk.len() > 1 {
                hasher.update(chunk[1].as_bytes());
            } else {
                hasher.update(chunk[0].as_bytes()); // Duplicate if odd
            }
            next_level.push(H256::from_slice(&hasher.finalize()));
        }
        leaves = next_level;
    }
    
    leaves[0]
}

fn parse_address(s: &str) -> Result<Address> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)
        .map_err(|_| ConsensusError::InvalidChain("Invalid address format".into()))?;
    Address::from_slice(&bytes)
        .ok_or_else(|| ConsensusError::InvalidChain("Invalid address length".into()))
}
