//! RocksDB Column Families for PYRAX blockchain storage
//!
//! Column families provide logical separation of data types for efficient
//! storage and retrieval. Each column family has its own LSM tree.

/// Database column families
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnFamily {
    /// Block headers and bodies indexed by block hash
    BlocksByHash,
    /// Maps block height -> block hash for the canonical chain
    BlockHeightToHash,
    /// Maps transaction hash -> (block_hash, tx_index)
    TransactionIndex,
    /// UTXO set: OutPoint -> Utxo
    UtxoSet,
    /// Spent UTXO index for reorg handling: OutPoint -> spending_block_hash
    SpentUtxos,
    /// Chain metadata (tip, genesis, etc.)
    ChainMeta,
    /// Block headers only (for light clients)
    BlockHeaders,
}

impl ColumnFamily {
    pub fn name(&self) -> &'static str {
        match self {
            ColumnFamily::BlocksByHash => "blocks_by_hash",
            ColumnFamily::BlockHeightToHash => "block_height_to_hash",
            ColumnFamily::TransactionIndex => "transaction_index",
            ColumnFamily::UtxoSet => "utxo_set",
            ColumnFamily::SpentUtxos => "spent_utxos",
            ColumnFamily::ChainMeta => "chain_meta",
            ColumnFamily::BlockHeaders => "block_headers",
        }
    }

    pub fn all() -> Vec<&'static str> {
        vec![
            ColumnFamily::BlocksByHash.name(),
            ColumnFamily::BlockHeightToHash.name(),
            ColumnFamily::TransactionIndex.name(),
            ColumnFamily::UtxoSet.name(),
            ColumnFamily::SpentUtxos.name(),
            ColumnFamily::ChainMeta.name(),
            ColumnFamily::BlockHeaders.name(),
        ]
    }
}

/// Well-known keys for chain metadata
pub mod keys {
    /// Current chain tip (height, hash, total_difficulty)
    pub const CHAIN_TIP: &[u8] = b"chain_tip";
    /// Genesis block hash
    pub const GENESIS_HASH: &[u8] = b"genesis_hash";
    /// Last finalized block height (for Stream C checkpoints)
    pub const LAST_FINALIZED: &[u8] = b"last_finalized";
    /// Network ID (mainnet=1, testnet=2, devnet=3)
    pub const NETWORK_ID: &[u8] = b"network_id";
    /// Database schema version for migrations
    pub const SCHEMA_VERSION: &[u8] = b"schema_version";
}

/// Current database schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 1;
