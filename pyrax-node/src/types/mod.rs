//! PYRAX Core Types
//!
//! Production-ready types for the PYRAX blockchain.
//! No stubs, no mocks - devnet/testnet/mainnet ready.

use serde::{Deserialize, Serialize};
use std::fmt;

pub type BlockNumber = u64;

/// 256-bit hash
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct H256(pub [u8; 32]);

impl H256 {
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }
    
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut bytes = [0u8; 32];
        let len = slice.len().min(32);
        bytes[..len].copy_from_slice(&slice[..len]);
        Self(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
    
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl fmt::Debug for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

/// 160-bit address (20 bytes)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub const ZERO: Address = Address([0u8; 20]);
    
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut bytes = [0u8; 20];
        let len = slice.len().min(20);
        bytes[..len].copy_from_slice(&slice[..len]);
        Self(bytes)
    }
    
    pub fn from_pubkey_hash(pubkey: &[u8]) -> Self {
        use tiny_keccak::{Hasher, Keccak};
        let mut hasher = Keccak::v256();
        let mut hash = [0u8; 32];
        hasher.update(pubkey);
        hasher.finalize(&mut hash);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash[12..32]);
        Self(addr)
    }
    
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

/// Network/Chain identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkId(pub u32);

impl NetworkId {
    /// PYRAX Mainnet - Chain ID 79729 (PYRAX on phone keypad: 7-9-7-2-9)
    pub const MAINNET: NetworkId = NetworkId(79729);
    /// PYRAX Testnet - Chain ID 797291
    pub const TESTNET: NetworkId = NetworkId(797291);
    /// PYRAX Devnet - Chain ID 797292
    pub const DEVNET: NetworkId = NetworkId(797292);
    
    pub fn name(&self) -> &'static str {
        match self.0 {
            79729 => "mainnet",
            797291 => "testnet",
            797292 => "devnet",
            _ => "unknown",
        }
    }
}

/// Chain tip information - persisted to database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTip {
    pub height: BlockNumber,
    pub hash: H256,
    pub total_difficulty: u64,
}

impl Default for ChainTip {
    fn default() -> Self {
        Self {
            height: 0,
            hash: H256::zero(),
            total_difficulty: 0,
        }
    }
}

/// Reference to a transaction output (UTXO identifier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutPoint {
    /// Transaction hash
    pub txid: H256,
    /// Output index within the transaction
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: H256, vout: u32) -> Self {
        Self { txid, vout }
    }
    
    pub fn coinbase() -> Self {
        Self {
            txid: H256::zero(),
            vout: u32::MAX,
        }
    }
    
    pub fn is_coinbase(&self) -> bool {
        self.txid == H256::zero() && self.vout == u32::MAX
    }
    
    pub fn to_bytes(&self) -> [u8; 36] {
        let mut bytes = [0u8; 36];
        bytes[..32].copy_from_slice(&self.txid.0);
        bytes[32..36].copy_from_slice(&self.vout.to_le_bytes());
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 36 {
            return None;
        }
        let mut txid = [0u8; 32];
        txid.copy_from_slice(&bytes[..32]);
        let vout = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        Some(Self { txid: H256(txid), vout })
    }
}

/// Transaction output (the "value" side of UTXO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    /// Amount in base units (1 PYRAX = 10^8)
    pub value: u64,
    /// Locking script (P2PKH: 76 a9 14 <20-byte-hash> 88 ac)
    pub script_pubkey: Vec<u8>,
}

impl TxOutput {
    pub fn new(value: u64, script_pubkey: Vec<u8>) -> Self {
        Self { value, script_pubkey }
    }
    
    /// Create P2PKH output (Pay to Public Key Hash)
    pub fn p2pkh(value: u64, address: &Address) -> Self {
        let mut script = Vec::with_capacity(25);
        script.push(0x76); // OP_DUP
        script.push(0xa9); // OP_HASH160
        script.push(0x14); // Push 20 bytes
        script.extend_from_slice(&address.0);
        script.push(0x88); // OP_EQUALVERIFY
        script.push(0xac); // OP_CHECKSIG
        Self::new(value, script)
    }
    
    /// Extract address from P2PKH script
    pub fn get_address(&self) -> Option<Address> {
        if self.script_pubkey.len() == 25
            && self.script_pubkey[0] == 0x76
            && self.script_pubkey[1] == 0xa9
            && self.script_pubkey[2] == 0x14
            && self.script_pubkey[23] == 0x88
            && self.script_pubkey[24] == 0xac
        {
            Some(Address::from_slice(&self.script_pubkey[3..23]))
        } else {
            None
        }
    }
}

/// Transaction input (spending a UTXO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    /// Reference to the UTXO being spent
    pub previous_output: OutPoint,
    /// Unlocking script (signature + pubkey)
    pub script_sig: Vec<u8>,
    /// Sequence number (0xFFFFFFFF = final)
    pub sequence: u32,
}

impl TxInput {
    pub fn new(previous_output: OutPoint, script_sig: Vec<u8>) -> Self {
        Self {
            previous_output,
            script_sig,
            sequence: 0xFFFFFFFF,
        }
    }
    
    pub fn coinbase(height: u64, extra: &[u8]) -> Self {
        let mut script = Vec::new();
        script.extend_from_slice(&height.to_le_bytes());
        script.extend_from_slice(extra);
        Self {
            previous_output: OutPoint::coinbase(),
            script_sig: script,
            sequence: 0xFFFFFFFF,
        }
    }
    
    pub fn is_coinbase(&self) -> bool {
        self.previous_output.is_coinbase()
    }
}

/// UTXO Transaction (Stream A native format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Version (currently 1)
    pub version: u32,
    /// Inputs (UTXOs being spent)
    pub inputs: Vec<TxInput>,
    /// Outputs (new UTXOs created)
    pub outputs: Vec<TxOutput>,
    /// Lock time (block height or timestamp)
    pub lock_time: u32,
}

impl Transaction {
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        Self {
            version: 1,
            inputs,
            outputs,
            lock_time: 0,
        }
    }
    
    /// Create coinbase transaction
    pub fn coinbase(height: u64, reward: u64, miner: &Address) -> Self {
        Self {
            version: 1,
            inputs: vec![TxInput::coinbase(height, b"PYRAX")],
            outputs: vec![TxOutput::p2pkh(reward, miner)],
            lock_time: 0,
        }
    }
    
    /// Compute transaction hash
    pub fn txid(&self) -> H256 {
        let encoded = bincode::serialize(self).unwrap_or_default();
        let hash = blake3::hash(&encoded);
        H256::from_slice(hash.as_bytes())
    }
    
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].is_coinbase()
    }
    
    pub fn total_output(&self) -> u64 {
        self.outputs.iter().map(|o| o.value).sum()
    }
}

/// Stored UTXO with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    pub output: TxOutput,
    pub height: BlockNumber,
    pub is_coinbase: bool,
}

impl Utxo {
    pub fn new(output: TxOutput, height: BlockNumber, is_coinbase: bool) -> Self {
        Self { output, height, is_coinbase }
    }
    
    /// Coinbase maturity check (100 blocks)
    pub fn is_mature(&self, current_height: BlockNumber) -> bool {
        if !self.is_coinbase {
            return true;
        }
        current_height >= self.height + 100
    }
}

/// Block header for Stream A (BLAKE3 PoW)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub stream: u8,
    pub parent_hash: H256,
    pub merkle_root: H256,
    pub utxo_commitment: H256,
    pub timestamp: u64,
    pub difficulty: u64,
    pub nonce: u64,
    pub extra_nonce: u64,
    pub height: BlockNumber,
    pub beneficiary: Address,
}

impl BlockHeader {
    /// Encode header for PoW hashing
    pub fn pow_input(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(150);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.push(self.stream);
        buf.extend_from_slice(&self.parent_hash.0);
        buf.extend_from_slice(&self.merkle_root.0);
        buf.extend_from_slice(&self.utxo_commitment.0);
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&self.difficulty.to_le_bytes());
        buf.extend_from_slice(&self.height.to_le_bytes());
        buf.extend_from_slice(&self.beneficiary.0);
        buf
    }
    
    /// Compute PoW hash (BLAKE3)
    pub fn pow_hash(&self) -> H256 {
        let input = self.pow_input();
        let mut hasher = blake3::Hasher::new();
        hasher.update(&input);
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.extra_nonce.to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }
    
    /// Compute block hash (includes nonce)
    pub fn hash(&self) -> H256 {
        let mut buf = self.pow_input();
        buf.extend_from_slice(&self.nonce.to_le_bytes());
        buf.extend_from_slice(&self.extra_nonce.to_le_bytes());
        H256::from_slice(blake3::hash(&buf).as_bytes())
    }
    
    /// Get difficulty target
    pub fn target(&self) -> H256 {
        difficulty_to_target(self.difficulty)
    }
    
    /// Verify PoW meets target
    pub fn verify_pow(&self) -> bool {
        let hash = self.pow_hash();
        hash.as_bytes() <= self.target().as_bytes()
    }
}

/// Complete block (header + transactions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self { header, transactions }
    }
    
    pub fn hash(&self) -> H256 {
        self.header.hash()
    }
    
    pub fn height(&self) -> BlockNumber {
        self.header.height
    }
    
    /// Compute merkle root of transactions
    pub fn compute_merkle_root(&self) -> H256 {
        if self.transactions.is_empty() {
            return H256::zero();
        }
        
        let mut hashes: Vec<H256> = self.transactions.iter().map(|tx| tx.txid()).collect();
        
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
    
    /// Verify merkle root matches transactions
    pub fn verify_merkle(&self) -> bool {
        self.header.merkle_root == self.compute_merkle_root()
    }
}

/// Convert difficulty to target (MAX / difficulty)
pub fn difficulty_to_target(difficulty: u64) -> H256 {
    if difficulty == 0 {
        return H256([0xff; 32]);
    }
    
    let max = ethereum_types::U256::MAX;
    let target = max / ethereum_types::U256::from(difficulty);
    let mut bytes = [0u8; 32];
    target.to_big_endian(&mut bytes);
    H256(bytes)
}

