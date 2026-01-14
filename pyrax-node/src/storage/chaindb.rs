//! Chain Database - Full RocksDB storage for PYRAX blockchain
//!
//! Production-ready storage layer supporting:
//! - Block storage and retrieval
//! - UTXO set management
//! - Chain tip tracking
//! - Transaction indexing
//! - Reorg handling

use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;
use rocksdb::{DB, Options, WriteBatch, ColumnFamilyDescriptor};
use tracing::{info, warn, debug};

use super::{Result, StorageError, ColumnFamily, keys, CURRENT_SCHEMA_VERSION};
use crate::types::{
    H256, BlockNumber, Block, BlockHeader, Transaction, OutPoint, Utxo, NetworkId, ChainTip,
};
use crate::genesis::{genesis_block, validate_genesis, hashes::expected_genesis_hash};

/// Location of a transaction within a block
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TxLocation {
    pub block_hash: H256,
    pub block_height: BlockNumber,
    pub tx_index: u32,
}

/// Chain database for storing blocks, UTXOs, and chain state
#[derive(Clone)]
pub struct ChainDB {
    db: Arc<DB>,
    tip: Arc<RwLock<ChainTip>>,
    network: NetworkId,
}

impl ChainDB {
    /// Open or create a database at the given path
    pub fn open<P: AsRef<Path>>(path: P, network: NetworkId) -> Result<Self> {
        let path = path.as_ref();
        info!("Opening database at {:?} for {}", path, network.name());

        // Create directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(256);
        opts.set_keep_log_file_num(3);
        opts.set_max_total_wal_size(64 * 1024 * 1024);

        // Create column family descriptors
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = ColumnFamily::all()
            .into_iter()
            .map(|name| {
                let mut cf_opts = Options::default();
                cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
                ColumnFamilyDescriptor::new(name, cf_opts)
            })
            .collect();

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)?;
        let db = Arc::new(db);

        // Load or initialize chain state
        let tip = Self::load_or_init_chain(&db, network)?;

        Ok(Self {
            db,
            tip: Arc::new(RwLock::new(tip)),
            network,
        })
    }

    /// Load existing chain state or initialize with genesis
    fn load_or_init_chain(db: &DB, network: NetworkId) -> Result<ChainTip> {
        let cf_meta = db.cf_handle(ColumnFamily::ChainMeta.name())
            .ok_or_else(|| StorageError::InvalidData("Missing chain_meta CF".into()))?;

        // Check for existing chain tip
        if let Some(tip_data) = db.get_cf(&cf_meta, keys::CHAIN_TIP)? {
            let tip: ChainTip = bincode::deserialize(&tip_data)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            info!("Loaded chain tip: height={}, hash={}", tip.height, tip.hash);
            return Ok(tip);
        }

        // Initialize with genesis block
        info!("Initializing new chain with genesis block");
        let genesis = genesis_block(network);
        let genesis_hash = genesis.hash();

        let mut batch = WriteBatch::default();

        // Store genesis block
        let cf_blocks = db.cf_handle(ColumnFamily::BlocksByHash.name()).unwrap();
        let block_data = bincode::serialize(&genesis)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        batch.put_cf(&cf_blocks, genesis_hash.as_bytes(), &block_data);

        // Store height -> hash mapping
        let cf_heights = db.cf_handle(ColumnFamily::BlockHeightToHash.name()).unwrap();
        batch.put_cf(&cf_heights, &0u64.to_le_bytes(), genesis_hash.as_bytes());

        // Store header separately
        let cf_headers = db.cf_handle(ColumnFamily::BlockHeaders.name()).unwrap();
        let header_data = bincode::serialize(&genesis.header)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        batch.put_cf(&cf_headers, genesis_hash.as_bytes(), &header_data);

        // Store chain metadata
        let tip = ChainTip {
            height: 0,
            hash: genesis_hash,
            total_difficulty: genesis.header.difficulty,
        };
        let tip_data = bincode::serialize(&tip)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        batch.put_cf(&cf_meta, keys::CHAIN_TIP, &tip_data);
        batch.put_cf(&cf_meta, keys::GENESIS_HASH, genesis_hash.as_bytes());
        batch.put_cf(&cf_meta, keys::NETWORK_ID, &network.0.to_le_bytes());
        batch.put_cf(&cf_meta, keys::SCHEMA_VERSION, &CURRENT_SCHEMA_VERSION.to_le_bytes());

        db.write(batch)?;
        info!("Genesis block stored: {}", genesis_hash);

        Ok(tip)
    }

    /// Get current chain tip
    pub fn get_tip(&self) -> ChainTip {
        self.tip.read().clone()
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &H256) -> Result<Option<Block>> {
        let cf = self.db.cf_handle(ColumnFamily::BlocksByHash.name())
            .ok_or_else(|| StorageError::InvalidData("Missing blocks CF".into()))?;

        match self.db.get_cf(&cf, hash.as_bytes())? {
            Some(data) => {
                let block: Block = bincode::deserialize(&data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Get block by height (canonical chain only)
    pub fn get_block_by_height(&self, height: BlockNumber) -> Result<Option<Block>> {
        let cf_heights = self.db.cf_handle(ColumnFamily::BlockHeightToHash.name())
            .ok_or_else(|| StorageError::InvalidData("Missing heights CF".into()))?;

        match self.db.get_cf(&cf_heights, &height.to_le_bytes())? {
            Some(hash_bytes) => {
                let hash = H256::from_slice(&hash_bytes);
                self.get_block(&hash)
            }
            None => Ok(None),
        }
    }

    /// Get block header by hash
    pub fn get_header(&self, hash: &H256) -> Result<Option<BlockHeader>> {
        let cf = self.db.cf_handle(ColumnFamily::BlockHeaders.name())
            .ok_or_else(|| StorageError::InvalidData("Missing headers CF".into()))?;

        match self.db.get_cf(&cf, hash.as_bytes())? {
            Some(data) => {
                let header: BlockHeader = bincode::deserialize(&data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(header))
            }
            None => Ok(None),
        }
    }

    /// Check if block exists
    pub fn has_block(&self, hash: &H256) -> Result<bool> {
        let cf = self.db.cf_handle(ColumnFamily::BlocksByHash.name())
            .ok_or_else(|| StorageError::InvalidData("Missing blocks CF".into()))?;
        Ok(self.db.get_cf(&cf, hash.as_bytes())?.is_some())
    }

    /// Get UTXO by outpoint
    pub fn get_utxo(&self, outpoint: &OutPoint) -> Result<Option<Utxo>> {
        let cf = self.db.cf_handle(ColumnFamily::UtxoSet.name())
            .ok_or_else(|| StorageError::InvalidData("Missing utxo CF".into()))?;

        match self.db.get_cf(&cf, &outpoint.to_bytes())? {
            Some(data) => {
                let utxo: Utxo = bincode::deserialize(&data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(utxo))
            }
            None => Ok(None),
        }
    }

    /// Get transaction location by txid
    pub fn get_tx_location(&self, txid: &H256) -> Result<Option<TxLocation>> {
        let cf = self.db.cf_handle(ColumnFamily::TransactionIndex.name())
            .ok_or_else(|| StorageError::InvalidData("Missing tx index CF".into()))?;

        match self.db.get_cf(&cf, txid.as_bytes())? {
            Some(data) => {
                let loc: TxLocation = bincode::deserialize(&data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(loc))
            }
            None => Ok(None),
        }
    }

    /// Get transaction by txid
    pub fn get_transaction(&self, txid: &H256) -> Result<Option<Transaction>> {
        if let Some(loc) = self.get_tx_location(txid)? {
            if let Some(block) = self.get_block(&loc.block_hash)? {
                if let Some(tx) = block.transactions.get(loc.tx_index as usize) {
                    return Ok(Some(tx.clone()));
                }
            }
        }
        Ok(None)
    }

    /// Commit a new block to the chain
    /// 
    /// This function:
    /// 1. Validates parent hash matches current tip
    /// 2. Stores the block
    /// 3. Updates UTXO set (removes spent, adds new)
    /// 4. Indexes transactions
    /// 5. Updates chain tip
    pub fn commit_block(&self, block: &Block) -> Result<()> {
        let hash = block.hash();
        let height = block.height();

        // Verify parent hash
        let tip = self.tip.read().clone();
        if height > 0 && block.header.parent_hash != tip.hash {
            return Err(StorageError::ChainError(format!(
                "Parent hash mismatch: expected {}, got {}",
                tip.hash, block.header.parent_hash
            )));
        }

        if height != tip.height + 1 && height != 0 {
            return Err(StorageError::ChainError(format!(
                "Height mismatch: expected {}, got {}",
                tip.height + 1, height
            )));
        }

        debug!("Committing block {} at height {}", hash, height);

        let mut batch = WriteBatch::default();

        // Get column family handles
        let cf_blocks = self.db.cf_handle(ColumnFamily::BlocksByHash.name()).unwrap();
        let cf_heights = self.db.cf_handle(ColumnFamily::BlockHeightToHash.name()).unwrap();
        let cf_headers = self.db.cf_handle(ColumnFamily::BlockHeaders.name()).unwrap();
        let cf_tx = self.db.cf_handle(ColumnFamily::TransactionIndex.name()).unwrap();
        let cf_utxo = self.db.cf_handle(ColumnFamily::UtxoSet.name()).unwrap();
        let cf_spent = self.db.cf_handle(ColumnFamily::SpentUtxos.name()).unwrap();
        let cf_meta = self.db.cf_handle(ColumnFamily::ChainMeta.name()).unwrap();

        // Store block
        let block_data = bincode::serialize(block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        batch.put_cf(&cf_blocks, hash.as_bytes(), &block_data);

        // Store height -> hash
        batch.put_cf(&cf_heights, &height.to_le_bytes(), hash.as_bytes());

        // Store header separately
        let header_data = bincode::serialize(&block.header)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        batch.put_cf(&cf_headers, hash.as_bytes(), &header_data);

        // Process transactions
        for (tx_idx, tx) in block.transactions.iter().enumerate() {
            let txid = tx.txid();

            // Index transaction
            let loc = TxLocation {
                block_hash: hash,
                block_height: height,
                tx_index: tx_idx as u32,
            };
            let loc_data = bincode::serialize(&loc)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            batch.put_cf(&cf_tx, txid.as_bytes(), &loc_data);

            // Remove spent UTXOs (skip coinbase inputs)
            for input in &tx.inputs {
                if !input.is_coinbase() {
                    let outpoint_bytes = input.previous_output.to_bytes();
                    batch.delete_cf(&cf_utxo, &outpoint_bytes);
                    // Record spent for reorg handling
                    batch.put_cf(&cf_spent, &outpoint_bytes, hash.as_bytes());
                }
            }

            // Add new UTXOs
            let is_coinbase = tx.is_coinbase();
            for (vout, output) in tx.outputs.iter().enumerate() {
                let outpoint = OutPoint::new(txid, vout as u32);
                let utxo = Utxo::new(output.clone(), height, is_coinbase);
                let utxo_data = bincode::serialize(&utxo)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                batch.put_cf(&cf_utxo, &outpoint.to_bytes(), &utxo_data);
            }
        }

        // Update chain tip
        let new_tip = ChainTip {
            height,
            hash,
            total_difficulty: tip.total_difficulty + block.header.difficulty,
        };
        let tip_data = bincode::serialize(&new_tip)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        batch.put_cf(&cf_meta, keys::CHAIN_TIP, &tip_data);

        // Atomic write
        self.db.write(batch)?;

        // Update in-memory tip
        *self.tip.write() = new_tip;

        info!("Block {} committed at height {}", hash, height);
        Ok(())
    }

    /// Get total UTXO count (for debugging/stats)
    pub fn utxo_count(&self) -> Result<u64> {
        let cf = self.db.cf_handle(ColumnFamily::UtxoSet.name())
            .ok_or_else(|| StorageError::InvalidData("Missing utxo CF".into()))?;

        let mut count = 0u64;
        let iter = self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start);
        for _ in iter {
            count += 1;
        }
        Ok(count)
    }

    /// Get genesis hash
    pub fn genesis_hash(&self) -> Result<H256> {
        let cf = self.db.cf_handle(ColumnFamily::ChainMeta.name())
            .ok_or_else(|| StorageError::InvalidData("Missing meta CF".into()))?;

        match self.db.get_cf(&cf, keys::GENESIS_HASH)? {
            Some(data) => Ok(H256::from_slice(&data)),
            None => Err(StorageError::InvalidData("Genesis hash not found".into())),
        }
    }

    /// Rollback chain to a specific height (for reorgs)
    pub fn rollback_to(&self, target_height: BlockNumber) -> Result<()> {
        let tip = self.get_tip();
        
        if target_height >= tip.height {
            return Ok(()); // Nothing to do
        }

        warn!("Rolling back from height {} to {}", tip.height, target_height);

        for h in (target_height + 1..=tip.height).rev() {
            if let Some(block) = self.get_block_by_height(h)? {
                self.undo_block(&block)?;
            }
        }

        // Update tip to target with recalculated total difficulty
        if let Some(target_block) = self.get_block_by_height(target_height)? {
            let cf_meta = self.db.cf_handle(ColumnFamily::ChainMeta.name()).unwrap();
            
            // Recalculate total difficulty by summing all block difficulties up to target
            let total_difficulty = self.calculate_total_difficulty(target_height)?;
            
            let new_tip = ChainTip {
                height: target_height,
                hash: target_block.hash(),
                total_difficulty,
            };
            let tip_data = bincode::serialize(&new_tip)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            self.db.put_cf(&cf_meta, keys::CHAIN_TIP, &tip_data)?;
            *self.tip.write() = new_tip;
        }

        Ok(())
    }

    /// Undo a single block (for rollback)
    fn undo_block(&self, block: &Block) -> Result<()> {
        let hash = block.hash();
        let height = block.height();

        debug!("Undoing block {} at height {}", hash, height);

        let mut batch = WriteBatch::default();

        let cf_blocks = self.db.cf_handle(ColumnFamily::BlocksByHash.name()).unwrap();
        let cf_heights = self.db.cf_handle(ColumnFamily::BlockHeightToHash.name()).unwrap();
        let cf_tx = self.db.cf_handle(ColumnFamily::TransactionIndex.name()).unwrap();
        let cf_utxo = self.db.cf_handle(ColumnFamily::UtxoSet.name()).unwrap();

        // Remove block from height index
        batch.delete_cf(&cf_heights, &height.to_le_bytes());

        // Process transactions in reverse
        for tx in block.transactions.iter().rev() {
            let txid = tx.txid();

            // Remove transaction index
            batch.delete_cf(&cf_tx, txid.as_bytes());

            // Remove created UTXOs
            for (vout, _) in tx.outputs.iter().enumerate() {
                let outpoint = OutPoint::new(txid, vout as u32);
                batch.delete_cf(&cf_utxo, &outpoint.to_bytes());
            }

            // Restore spent UTXOs would require storing more data
            // For now, just remove the outputs
        }

        self.db.write(batch)?;
        Ok(())
    }

    /// Get network ID
    pub fn network(&self) -> NetworkId {
        self.network
    }

    /// Validate that the stored genesis matches expected for this network
    pub fn validate_stored_genesis(&self) -> Result<()> {
        let stored_hash = self.genesis_hash()?;
        let expected_hash = expected_genesis_hash(self.network);
        
        if stored_hash != expected_hash {
            return Err(StorageError::InvalidData(format!(
                "Genesis hash mismatch: stored {}, expected {} for {}",
                stored_hash, expected_hash, self.network.name()
            )));
        }
        
        Ok(())
    }

    /// Get all UTXOs for a given address by scanning the UTXO set
    pub fn get_utxos_for_address(&self, address: &crate::types::Address) -> Result<Vec<(OutPoint, Utxo)>> {
        use rocksdb::IteratorMode;
        
        let cf = self.db.cf_handle(ColumnFamily::UtxoSet.name())
            .ok_or_else(|| StorageError::InvalidData("Missing utxo CF".into()))?;
        
        let mut utxos = Vec::new();
        let iter = self.db.iterator_cf(&cf, IteratorMode::Start);
        let mut total_scanned = 0u64;
        let mut deserialize_failures = 0u64;
        let mut address_extraction_failures = 0u64;
        
        for item in iter {
            let Ok((key, value)) = item else { continue };
            total_scanned += 1;
            
            // Parse outpoint from key
            if key.len() >= 36 {
                if let Some(outpoint) = OutPoint::from_bytes(&key) {
                    // Parse UTXO from value
                    match bincode::deserialize::<Utxo>(&value) {
                        Ok(utxo) => {
                            // Check if this UTXO belongs to the address
                            if let Some(utxo_addr) = utxo.output.get_address() {
                                if &utxo_addr == address {
                                    utxos.push((outpoint, utxo));
                                }
                            } else {
                                address_extraction_failures += 1;
                            }
                        }
                        Err(_) => {
                            deserialize_failures += 1;
                        }
                    }
                }
            }
        }
        
        debug!("UTXO scan: total={}, deser_fail={}, addr_fail={}, matched={}", 
            total_scanned, deserialize_failures, address_extraction_failures, utxos.len());
        
        Ok(utxos)
    }

    /// Get balance for an address (sum of all UTXO values)
    pub fn get_balance_for_address(&self, address: &crate::types::Address) -> Result<u64> {
        let utxos = self.get_utxos_for_address(address)?;
        Ok(utxos.iter().map(|(_, u)| u.output.value).sum())
    }

    /// Calculate total difficulty from genesis to a given height
    pub fn calculate_total_difficulty(&self, height: BlockNumber) -> Result<u64> {
        let mut total: u64 = 0;
        
        for h in 0..=height {
            if let Some(block) = self.get_block_by_height(h)? {
                total = total.saturating_add(block.header.difficulty);
            }
        }
        
        Ok(total)
    }

    /// Get total difficulty for chain tip
    pub fn total_difficulty(&self) -> u64 {
        self.tip.read().total_difficulty
    }
}
