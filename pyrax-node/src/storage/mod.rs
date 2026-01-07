//! Storage layer for PYRAX blockchain
//!
//! Production-ready RocksDB storage with:
//! - Block storage (by hash and height)
//! - UTXO set management
//! - Chain metadata persistence
//! - Transaction indexing

mod chaindb;
mod columns;

pub use chaindb::ChainDB;
pub use columns::{ColumnFamily, keys, CURRENT_SCHEMA_VERSION};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rocksdb::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    #[error("UTXO not found: {0}")]
    UtxoNotFound(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Chain error: {0}")]
    ChainError(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
