//! Stream A Block - BLAKE3 PoW blocks with UTXO transactions
//!
//! Stream A is the fast value-transfer stream:
//! - BLAKE3 Proof-of-Work (ASIC-friendly, CPU-testable)
//! - 10-second block time
//! - 50 PYRAX block reward
//! - UTXO transaction model

use serde::{Deserialize, Serialize};
use super::{H256, Address, BlockNumber};
use super::utxo::UtxoTransaction;

// Re-export Blake3Pow functions inline to avoid circular dependency
pub struct Blake3Pow;

impl Blake3Pow {
    pub fn hash(header_bytes: &[u8], nonce: u64, extra_nonce: u64) -> H256 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(header_bytes);
        hasher.update(&nonce.to_le_bytes());
        hasher.update(&extra_nonce.to_le_bytes());
        let result = hasher.finalize();
        H256::from_slice(result.as_bytes())
    }
    
    pub fn meets_target(hash: &H256, target: &H256) -> bool {
        hash.as_bytes() <= target.as_bytes()
    }
    
    pub fn difficulty_to_target(difficulty: u64) -> H256 {
        if difficulty == 0 { return H256([0xff; 32]); }
        let max = ethereum_types::U256::MAX;
        let target = max / ethereum_types::U256::from(difficulty);
        let mut bytes = [0u8; 32];
        target.to_big_endian(&mut bytes);
        H256(bytes)
    }
    
    /// Mine a block (CPU reference implementation for testing)
    pub fn mine(header_bytes: &[u8], target: &H256, start_nonce: u64, max_iterations: u64) -> Option<(u64, u64, H256)> {
        for nonce in start_nonce..start_nonce + max_iterations {
            let hash = Self::hash(header_bytes, nonce, 0);
            if Self::meets_target(&hash, target) {
                return Some((nonce, 0, hash));
            }
        }
        None
    }
}

pub struct Stream;
impl Stream {
    pub const A: u8 = 0;
    pub fn block_reward() -> u64 { 50_00000000 } // 50 PYRAX
}

/// Stream A block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamAHeader {
    /// Protocol version
    pub version: u32,
    /// Stream identifier (always Stream::A)
    pub stream: u8,
    /// Hash of the previous Stream A block
    pub parent_hash: H256,
    /// Merkle root of transactions
    pub merkle_root: H256,
    /// UTXO commitment (root of UTXO set after this block)
    pub utxo_root: H256,
    /// Block timestamp (Unix seconds)
    pub timestamp: u64,
    /// Difficulty target
    pub difficulty: u64,
    /// PoW nonce
    pub nonce: u64,
    /// Extended nonce for larger search space
    pub extra_nonce: u64,
    /// Block height
    pub height: BlockNumber,
    /// Miner address (receives block reward)
    pub beneficiary: Address,
}

impl StreamAHeader {
    /// Create a new Stream A header
    pub fn new(
        parent_hash: H256,
        merkle_root: H256,
        utxo_root: H256,
        timestamp: u64,
        difficulty: u64,
        height: BlockNumber,
        beneficiary: Address,
    ) -> Self {
        Self {
            version: 1,
            stream: Stream::A as u8,
            parent_hash,
            merkle_root,
            utxo_root,
            timestamp,
            difficulty,
            nonce: 0,
            extra_nonce: 0,
            height,
            beneficiary,
        }
    }
    
    /// Create the genesis block header
    pub fn genesis() -> Self {
        Self {
            version: 1,
            stream: Stream::A as u8,
            parent_hash: H256::zero(),
            merkle_root: H256::zero(),
            utxo_root: H256::zero(),
            timestamp: 1735689600, // 2025-01-01 00:00:00 UTC
            difficulty: 1, // Very easy for initial mining
            nonce: 0,
            extra_nonce: 0,
            height: 0,
            beneficiary: Address::ZERO,
        }
    }
    
    /// Encode header bytes for hashing (excluding nonce fields)
    pub fn encode_for_pow(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(150);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.push(self.stream);
        buf.extend_from_slice(self.parent_hash.as_bytes());
        buf.extend_from_slice(self.merkle_root.as_bytes());
        buf.extend_from_slice(self.utxo_root.as_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&self.difficulty.to_le_bytes());
        buf.extend_from_slice(&self.height.to_le_bytes());
        buf.extend_from_slice(self.beneficiary.as_bytes());
        buf
    }
    
    /// Compute the PoW hash using BLAKE3
    pub fn pow_hash(&self) -> H256 {
        let header_bytes = self.encode_for_pow();
        Blake3Pow::hash(&header_bytes, self.nonce, self.extra_nonce)
    }
    
    /// Compute the block hash (includes nonce)
    pub fn hash(&self) -> H256 {
        let mut buf = self.encode_for_pow();
        buf.extend_from_slice(&self.nonce.to_le_bytes());
        buf.extend_from_slice(&self.extra_nonce.to_le_bytes());
        
        let result = blake3::hash(&buf);
        H256::from_slice(result.as_bytes())
    }
    
    /// Get the target from difficulty
    pub fn target(&self) -> H256 {
        Blake3Pow::difficulty_to_target(self.difficulty)
    }
    
    /// Check if the PoW is valid
    pub fn verify_pow(&self) -> bool {
        let hash = self.pow_hash();
        let target = self.target();
        Blake3Pow::meets_target(&hash, &target)
    }
    
    /// Set nonce values (for mining)
    pub fn set_nonce(&mut self, nonce: u64, extra_nonce: u64) {
        self.nonce = nonce;
        self.extra_nonce = extra_nonce;
    }
    
    /// Block reward in base units
    pub fn block_reward(&self) -> u64 {
        Stream::block_reward()
    }
}

/// Stream A block body containing UTXO transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamABody {
    /// Transactions in this block (first is always coinbase)
    pub transactions: Vec<UtxoTransaction>,
}

impl StreamABody {
    pub fn new(transactions: Vec<UtxoTransaction>) -> Self {
        Self { transactions }
    }
    
    pub fn empty() -> Self {
        Self { transactions: vec![] }
    }
    
    /// Create body with just a coinbase transaction
    pub fn coinbase_only(height: u64, reward: u64, miner: &Address) -> Self {
        Self {
            transactions: vec![UtxoTransaction::coinbase(height, reward, miner)],
        }
    }
    
    /// Compute the merkle root of transactions
    pub fn merkle_root(&self) -> H256 {
        if self.transactions.is_empty() {
            return H256::zero();
        }
        
        let mut hashes: Vec<H256> = self.transactions
            .iter()
            .map(|tx| tx.txid())
            .collect();
        
        // Build merkle tree
        while hashes.len() > 1 {
            if hashes.len() % 2 == 1 {
                hashes.push(*hashes.last().unwrap());
            }
            
            hashes = hashes
                .chunks(2)
                .map(|pair| {
                    let mut combined = Vec::with_capacity(64);
                    combined.extend_from_slice(pair[0].as_bytes());
                    combined.extend_from_slice(pair[1].as_bytes());
                    let result = blake3::hash(&combined);
                    H256::from_slice(result.as_bytes())
                })
                .collect();
        }
        
        hashes[0]
    }
    
    /// Total fees from all transactions (excluding coinbase)
    pub fn total_fees(&self, utxo_values: &[u64]) -> u64 {
        // For UTXO model, fee = sum(inputs) - sum(outputs)
        // This requires knowing the input values from the UTXO set
        // For now, return 0 - proper implementation needs UTXO lookup
        let _ = utxo_values;
        0
    }
}

/// Complete Stream A block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamABlock {
    pub header: StreamAHeader,
    pub body: StreamABody,
}

impl StreamABlock {
    pub fn new(header: StreamAHeader, body: StreamABody) -> Self {
        Self { header, body }
    }
    
    /// Create the genesis block
    pub fn genesis() -> Self {
        Self {
            header: StreamAHeader::genesis(),
            body: StreamABody::empty(),
        }
    }
    
    pub fn hash(&self) -> H256 {
        self.header.hash()
    }
    
    pub fn height(&self) -> BlockNumber {
        self.header.height
    }
    
    pub fn verify_pow(&self) -> bool {
        self.header.verify_pow()
    }
    
    /// Verify merkle root matches transactions
    pub fn verify_merkle(&self) -> bool {
        self.header.merkle_root == self.body.merkle_root()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_genesis_block() {
        let genesis = StreamABlock::genesis();
        assert_eq!(genesis.height(), 0);
        assert_eq!(genesis.header.parent_hash, H256::zero());
        assert_eq!(genesis.header.stream, Stream::A as u8);
    }
    
    #[test]
    fn test_genesis_pow_easy() {
        let genesis = StreamABlock::genesis();
        // Genesis has difficulty 1, should be very easy
        assert!(genesis.verify_pow(), "Genesis block should have valid PoW");
    }
    
    #[test]
    fn test_header_hash_deterministic() {
        let header1 = StreamAHeader::genesis();
        let header2 = StreamAHeader::genesis();
        assert_eq!(header1.hash(), header2.hash());
    }
    
    #[test]
    fn test_mining_easy_block() {
        let mut header = StreamAHeader::new(
            H256::zero(),
            H256::zero(),
            H256::zero(),
            1735689600,
            1, // Easy difficulty
            1,
            Address::ZERO,
        );
        
        let header_bytes = header.encode_for_pow();
        let target = header.target();
        
        // Should find solution quickly at difficulty 1
        let result = Blake3Pow::mine(&header_bytes, &target, 0, 1000);
        assert!(result.is_some(), "Should find solution at difficulty 1");
        
        let (nonce, extra_nonce, _hash) = result.unwrap();
        header.set_nonce(nonce, extra_nonce);
        assert!(header.verify_pow(), "Mined block should have valid PoW");
    }
    
    #[test]
    fn test_merkle_root_single_tx() {
        let miner = Address::ZERO;
        let body = StreamABody::coinbase_only(1, 50 * 100_000_000, &miner);
        
        let root = body.merkle_root();
        assert_ne!(root, H256::zero(), "Merkle root should not be zero with transactions");
        
        // Same transactions = same root
        let body2 = StreamABody::coinbase_only(1, 50 * 100_000_000, &miner);
        assert_eq!(body.merkle_root(), body2.merkle_root());
    }
    
    #[test]
    fn test_block_reward() {
        let header = StreamAHeader::genesis();
        assert_eq!(header.block_reward(), 50 * 100_000_000); // 50 PYRAX
    }
}
