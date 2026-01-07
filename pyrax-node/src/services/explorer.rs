//! Block Explorer Backend Service
//!
//! Production block explorer API for PYRAX blockchain

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::EXPLORER_PAGE_SIZE;

/// Explorer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerConfig {
    /// Enable explorer service
    pub enabled: bool,
    /// Page size for pagination
    pub page_size: u32,
    /// Cache TTL (seconds)
    pub cache_ttl_secs: u64,
    /// Maximum search results
    pub max_search_results: u32,
    /// Enable rich transaction data
    pub rich_tx_data: bool,
    /// Index internal transactions
    pub index_internal_txs: bool,
    /// Index token transfers
    pub index_token_transfers: bool,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            page_size: EXPLORER_PAGE_SIZE,
            cache_ttl_secs: 60,
            max_search_results: 100,
            rich_tx_data: true,
            index_internal_txs: true,
            index_token_transfers: true,
        }
    }
}

/// Block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    /// Block hash
    pub hash: H256,
    /// Block number
    pub number: u64,
    /// Parent hash
    pub parent_hash: H256,
    /// Timestamp
    pub timestamp: u64,
    /// Miner/validator address
    pub miner: Address,
    /// Difficulty
    pub difficulty: u64,
    /// Total difficulty
    pub total_difficulty: u64,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas used
    pub gas_used: u64,
    /// Transaction count
    pub tx_count: u32,
    /// Block size (bytes)
    pub size: u32,
    /// Transactions root
    pub transactions_root: H256,
    /// State root
    pub state_root: H256,
    /// Block reward
    pub reward: u64,
    /// Uncle/ommer count
    pub uncle_count: u32,
    /// Extra data
    pub extra_data: Vec<u8>,
    /// Transaction hashes
    pub transactions: Vec<H256>,
    /// Is finalized
    pub finalized: bool,
}

impl BlockInfo {
    /// Create new block info
    pub fn new(hash: H256, number: u64, miner: Address) -> Self {
        Self {
            hash,
            number,
            parent_hash: H256::zero(),
            timestamp: current_timestamp(),
            miner,
            difficulty: 0,
            total_difficulty: 0,
            gas_limit: 30_000_000,
            gas_used: 0,
            tx_count: 0,
            size: 0,
            transactions_root: H256::zero(),
            state_root: H256::zero(),
            reward: 0,
            uncle_count: 0,
            extra_data: Vec::new(),
            transactions: Vec::new(),
            finalized: false,
        }
    }
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    /// Transaction hash
    pub hash: H256,
    /// Block hash
    pub block_hash: Option<H256>,
    /// Block number
    pub block_number: Option<u64>,
    /// Transaction index in block
    pub tx_index: Option<u32>,
    /// From address
    pub from: Address,
    /// To address
    pub to: Option<Address>,
    /// Value transferred
    pub value: u64,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas price
    pub gas_price: u64,
    /// Gas used
    pub gas_used: Option<u64>,
    /// Input data
    pub input: Vec<u8>,
    /// Nonce
    pub nonce: u64,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Status (1 = success, 0 = failure)
    pub status: Option<u8>,
    /// Timestamp
    pub timestamp: u64,
    /// Contract created
    pub contract_created: Option<Address>,
    /// Logs count
    pub logs_count: u32,
    /// Internal transactions count
    pub internal_txs_count: u32,
    /// Token transfers
    pub token_transfers: Vec<TokenTransfer>,
    /// Is pending
    pub pending: bool,
    /// Confirmation count
    pub confirmations: u64,
}

impl TransactionInfo {
    /// Create new transaction info
    pub fn new(hash: H256, from: Address) -> Self {
        Self {
            hash,
            block_hash: None,
            block_number: None,
            tx_index: None,
            from,
            to: None,
            value: 0,
            gas_limit: 21000,
            gas_price: 0,
            gas_used: None,
            input: Vec::new(),
            nonce: 0,
            tx_type: TransactionType::Transfer,
            status: None,
            timestamp: current_timestamp(),
            contract_created: None,
            logs_count: 0,
            internal_txs_count: 0,
            token_transfers: Vec::new(),
            pending: true,
            confirmations: 0,
        }
    }

    /// Is confirmed
    pub fn is_confirmed(&self) -> bool {
        self.block_number.is_some() && self.confirmations > 0
    }

    /// Is success
    pub fn is_success(&self) -> bool {
        self.status == Some(1)
    }
}

/// Transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Native token transfer
    Transfer,
    /// Contract creation
    ContractCreation,
    /// Contract call
    ContractCall,
    /// Token transfer (ERC20)
    TokenTransfer,
    /// NFT transfer (ERC721)
    NftTransfer,
    /// Multi-token transfer (ERC1155)
    MultiTokenTransfer,
    /// Staking operation
    Staking,
    /// Governance vote
    Governance,
    /// Bridge operation
    Bridge,
}

/// Token transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    /// Token contract address
    pub token: Address,
    /// From address
    pub from: Address,
    /// To address
    pub to: Address,
    /// Amount (for ERC20) or token ID (for ERC721)
    pub value: String,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Log index
    pub log_index: u32,
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    /// Address
    pub address: Address,
    /// Balance
    pub balance: u64,
    /// Nonce
    pub nonce: u64,
    /// Is contract
    pub is_contract: bool,
    /// Contract creation tx
    pub creation_tx: Option<H256>,
    /// Contract creator
    pub creator: Option<Address>,
    /// Code hash
    pub code_hash: Option<H256>,
    /// Total transactions
    pub tx_count: u64,
    /// Total received
    pub total_received: u64,
    /// Total sent
    pub total_sent: u64,
    /// First seen block
    pub first_seen: Option<u64>,
    /// Last seen block
    pub last_seen: Option<u64>,
    /// Token holdings
    pub tokens: Vec<TokenBalance>,
    /// Is validator
    pub is_validator: bool,
    /// Validator info
    pub validator_info: Option<ValidatorInfo>,
}

impl AddressInfo {
    /// Create new address info
    pub fn new(address: Address) -> Self {
        Self {
            address,
            balance: 0,
            nonce: 0,
            is_contract: false,
            creation_tx: None,
            creator: None,
            code_hash: None,
            tx_count: 0,
            total_received: 0,
            total_sent: 0,
            first_seen: None,
            last_seen: None,
            tokens: Vec::new(),
            is_validator: false,
            validator_info: None,
        }
    }
}

/// Token balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    /// Token contract address
    pub token: Address,
    /// Balance
    pub balance: String,
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: String,
    /// Decimals
    pub decimals: u8,
    /// Token type
    pub token_type: TokenType,
}

/// Token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    ERC20,
    ERC721,
    ERC1155,
}

/// Validator info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator address
    pub address: Address,
    /// Stake amount
    pub stake: u64,
    /// Commission (basis points)
    pub commission_bps: u16,
    /// Is active
    pub active: bool,
    /// Blocks produced
    pub blocks_produced: u64,
    /// Uptime percentage
    pub uptime_percent: f64,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResult {
    Block(BlockInfo),
    Transaction(TransactionInfo),
    Address(AddressInfo),
    Token(TokenInfo),
    NotFound,
}

/// Token info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Contract address
    pub address: Address,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Decimals
    pub decimals: u8,
    /// Total supply
    pub total_supply: String,
    /// Holders count
    pub holders: u64,
    /// Transfers count
    pub transfers: u64,
    /// Token type
    pub token_type: TokenType,
}

/// Pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Current page (1-indexed)
    pub page: u32,
    /// Items per page
    pub per_page: u32,
    /// Total items
    pub total: u64,
    /// Total pages
    pub total_pages: u32,
    /// Has next page
    pub has_next: bool,
    /// Has previous page
    pub has_prev: bool,
}

impl Pagination {
    /// Create new pagination
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        Self {
            page,
            per_page,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// Explorer service
pub struct ExplorerService {
    /// Configuration
    config: ExplorerConfig,
    /// Block cache
    blocks: Arc<RwLock<HashMap<H256, BlockInfo>>>,
    /// Block by number
    blocks_by_number: Arc<RwLock<HashMap<u64, H256>>>,
    /// Transaction cache
    transactions: Arc<RwLock<HashMap<H256, TransactionInfo>>>,
    /// Address cache
    addresses: Arc<RwLock<HashMap<Address, AddressInfo>>>,
    /// Current block height
    current_height: Arc<RwLock<u64>>,
    /// Statistics
    stats: Arc<RwLock<ExplorerStats>>,
}

impl ExplorerService {
    /// Create new explorer service
    pub fn new(config: ExplorerConfig) -> Self {
        Self {
            config,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            blocks_by_number: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            addresses: Arc::new(RwLock::new(HashMap::new())),
            current_height: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(ExplorerStats::default())),
        }
    }

    /// Index a block
    pub fn index_block(&self, block: BlockInfo) {
        let hash = block.hash;
        let number = block.number;

        // Update current height
        if number > *self.current_height.read() {
            *self.current_height.write() = number;
        }

        // Index block
        self.blocks_by_number.write().insert(number, hash);
        self.blocks.write().insert(hash, block);

        self.stats.write().blocks_indexed += 1;
    }

    /// Index a transaction
    pub fn index_transaction(&self, tx: TransactionInfo) {
        self.transactions.write().insert(tx.hash, tx);
        self.stats.write().txs_indexed += 1;
    }

    /// Index an address
    pub fn index_address(&self, address: AddressInfo) {
        self.addresses.write().insert(address.address, address);
        self.stats.write().addresses_indexed += 1;
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &H256) -> Option<BlockInfo> {
        self.blocks.read().get(hash).cloned()
    }

    /// Get block by number
    pub fn get_block_by_number(&self, number: u64) -> Option<BlockInfo> {
        let hash = self.blocks_by_number.read().get(&number).copied()?;
        self.get_block(&hash)
    }

    /// Get latest block
    pub fn get_latest_block(&self) -> Option<BlockInfo> {
        let height = *self.current_height.read();
        self.get_block_by_number(height)
    }

    /// Get blocks (paginated)
    pub fn get_blocks(&self, page: u32) -> (Vec<BlockInfo>, Pagination) {
        let height = *self.current_height.read();
        let per_page = self.config.page_size;
        let start = height.saturating_sub(((page - 1) * per_page) as u64);
        let end = start.saturating_sub(per_page as u64);

        let mut blocks = Vec::new();
        for num in (end..=start).rev() {
            if let Some(block) = self.get_block_by_number(num) {
                blocks.push(block);
            }
        }

        let pagination = Pagination::new(page, per_page, height);
        (blocks, pagination)
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, hash: &H256) -> Option<TransactionInfo> {
        self.transactions.read().get(hash).cloned()
    }

    /// Get address info
    pub fn get_address(&self, address: &Address) -> Option<AddressInfo> {
        self.addresses.read().get(address).cloned()
    }

    /// Get address transactions (paginated)
    pub fn get_address_transactions(&self, address: &Address, page: u32) -> (Vec<TransactionInfo>, Pagination) {
        let txs: Vec<TransactionInfo> = self.transactions.read()
            .values()
            .filter(|tx| tx.from == *address || tx.to == Some(*address))
            .cloned()
            .collect();

        let total = txs.len() as u64;
        let per_page = self.config.page_size;
        let start = ((page - 1) * per_page) as usize;
        let end = (start + per_page as usize).min(txs.len());

        let page_txs = txs.get(start..end).unwrap_or(&[]).to_vec();
        let pagination = Pagination::new(page, per_page, total);

        (page_txs, pagination)
    }

    /// Search
    pub fn search(&self, query: &str) -> SearchResult {
        // Try as block hash
        if query.len() == 66 && query.starts_with("0x") {
            if let Ok(bytes) = hex::decode(&query[2..]) {
                if bytes.len() == 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&bytes);
                    let hash = H256(arr);
                    if let Some(block) = self.get_block(&hash) {
                        return SearchResult::Block(block);
                    }
                    if let Some(tx) = self.get_transaction(&hash) {
                        return SearchResult::Transaction(tx);
                    }
                }
            }
        }

        // Try as block number
        if let Ok(number) = query.parse::<u64>() {
            if let Some(block) = self.get_block_by_number(number) {
                return SearchResult::Block(block);
            }
        }

        // Try as address
        if query.len() == 42 && query.starts_with("0x") {
            if let Ok(bytes) = hex::decode(&query[2..]) {
                if bytes.len() == 20 {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(&bytes);
                    let address = Address(arr);
                    if let Some(info) = self.get_address(&address) {
                        return SearchResult::Address(info);
                    }
                }
            }
        }

        SearchResult::NotFound
    }

    /// Get chain statistics
    pub fn get_chain_stats(&self) -> ChainStats {
        let height = *self.current_height.read();
        let total_txs = self.transactions.read().len() as u64;
        let total_addresses = self.addresses.read().len() as u64;

        ChainStats {
            block_height: height,
            total_transactions: total_txs,
            total_addresses,
            avg_block_time_ms: 6000,
            avg_gas_price: 1_000_000_000,
            tps: 0.0,
        }
    }

    /// Get current height
    pub fn current_height(&self) -> u64 {
        *self.current_height.read()
    }

    /// Get statistics
    pub fn stats(&self) -> ExplorerStats {
        self.stats.read().clone()
    }
}

impl Default for ExplorerService {
    fn default() -> Self {
        Self::new(ExplorerConfig::default())
    }
}

/// Chain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub block_height: u64,
    pub total_transactions: u64,
    pub total_addresses: u64,
    pub avg_block_time_ms: u64,
    pub avg_gas_price: u64,
    pub tps: f64,
}

/// Explorer statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExplorerStats {
    pub blocks_indexed: u64,
    pub txs_indexed: u64,
    pub addresses_indexed: u64,
    pub searches: u64,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_info() {
        let block = BlockInfo::new(H256([1u8; 32]), 100, Address([1u8; 20]));
        assert_eq!(block.number, 100);
    }

    #[test]
    fn test_transaction_info() {
        let tx = TransactionInfo::new(H256([1u8; 32]), Address([1u8; 20]));
        assert!(tx.pending);
        assert!(!tx.is_confirmed());
    }

    #[test]
    fn test_explorer_service() {
        let explorer = ExplorerService::default();
        
        let block = BlockInfo::new(H256([1u8; 32]), 1, Address([1u8; 20]));
        explorer.index_block(block);

        assert_eq!(explorer.current_height(), 1);
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(1, 25, 100);
        assert_eq!(pagination.total_pages, 4);
        assert!(pagination.has_next);
        assert!(!pagination.has_prev);
    }

    #[test]
    fn test_search() {
        let explorer = ExplorerService::default();
        
        let block = BlockInfo::new(H256([1u8; 32]), 1, Address([1u8; 20]));
        explorer.index_block(block);

        let result = explorer.search("1");
        assert!(matches!(result, SearchResult::Block(_)));
    }
}
