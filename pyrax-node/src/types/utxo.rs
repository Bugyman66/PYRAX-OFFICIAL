//! UTXO (Unspent Transaction Output) model for Stream A
//!
//! Stream A uses UTXO instead of account-based transactions because:
//! - Parallel validation (UTXOs are independent)
//! - Simple SPV proofs
//! - Privacy-friendly (CoinJoin compatible)
//! - No nonce tracking required
//!
//! This is the native transaction model for L1 value transfer.

use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};
use k256::ecdsa::{SigningKey, Signature, RecoveryId, VerifyingKey};

use super::{Address, H256};

/// A reference to a specific output in a previous transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutPoint {
    /// Transaction hash containing the output
    pub txid: H256,
    /// Index of the output within the transaction (0-based)
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: H256, vout: u32) -> Self {
        Self { txid, vout }
    }
    
    /// Coinbase outpoint (null txid, max vout)
    pub fn coinbase() -> Self {
        Self {
            txid: H256::zero(),
            vout: u32::MAX,
        }
    }
    
    pub fn is_coinbase(&self) -> bool {
        self.txid == H256::zero() && self.vout == u32::MAX
    }
    
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(36);
        buf.extend_from_slice(self.txid.as_bytes());
        buf.extend_from_slice(&self.vout.to_le_bytes());
        buf
    }
}

/// An input to a UTXO transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    /// Reference to the UTXO being spent
    pub previous_output: OutPoint,
    /// Signature script (unlocking script)
    pub script_sig: Vec<u8>,
    /// Sequence number (for future use: replace-by-fee, timelocks)
    pub sequence: u32,
}

impl TxInput {
    pub fn new(previous_output: OutPoint, script_sig: Vec<u8>) -> Self {
        Self {
            previous_output,
            script_sig,
            sequence: 0xFFFFFFFF, // Default: final
        }
    }
    
    /// Create a coinbase input (for mining rewards)
    pub fn coinbase(block_height: u64, extra_data: &[u8]) -> Self {
        let mut script_sig = Vec::new();
        // BIP34: block height in coinbase
        script_sig.extend_from_slice(&block_height.to_le_bytes());
        script_sig.extend_from_slice(extra_data);
        
        Self {
            previous_output: OutPoint::coinbase(),
            script_sig,
            sequence: 0xFFFFFFFF,
        }
    }
    
    pub fn is_coinbase(&self) -> bool {
        self.previous_output.is_coinbase()
    }
    
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.previous_output.encode());
        buf.extend_from_slice(&(self.script_sig.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.script_sig);
        buf.extend_from_slice(&self.sequence.to_le_bytes());
        buf
    }
}

/// An output of a UTXO transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    /// Amount in base units (1 PYRAX = 10^8 base units)
    pub value: u64,
    /// Locking script (typically P2PKH or P2SH)
    pub script_pubkey: Vec<u8>,
}

impl TxOutput {
    pub fn new(value: u64, script_pubkey: Vec<u8>) -> Self {
        Self { value, script_pubkey }
    }
    
    /// Create a Pay-to-Public-Key-Hash output
    pub fn p2pkh(value: u64, address: &Address) -> Self {
        // Script: OP_DUP OP_HASH160 <pubkey_hash> OP_EQUALVERIFY OP_CHECKSIG
        let mut script = Vec::with_capacity(25);
        script.push(0x76); // OP_DUP
        script.push(0xa9); // OP_HASH160
        script.push(0x14); // Push 20 bytes
        script.extend_from_slice(address.as_bytes());
        script.push(0x88); // OP_EQUALVERIFY
        script.push(0xac); // OP_CHECKSIG
        
        Self::new(value, script)
    }
    
    /// Extract the address from a P2PKH script
    pub fn extract_p2pkh_address(&self) -> Option<Address> {
        // P2PKH script: 76 a9 14 <20 bytes> 88 ac
        if self.script_pubkey.len() == 25
            && self.script_pubkey[0] == 0x76
            && self.script_pubkey[1] == 0xa9
            && self.script_pubkey[2] == 0x14
            && self.script_pubkey[23] == 0x88
            && self.script_pubkey[24] == 0xac
        {
            let mut addr_bytes = [0u8; 20];
            addr_bytes.copy_from_slice(&self.script_pubkey[3..23]);
            Some(Address::from_bytes(addr_bytes))
        } else {
            None
        }
    }
    
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.value.to_le_bytes());
        buf.extend_from_slice(&(self.script_pubkey.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.script_pubkey);
        buf
    }
}

/// A UTXO transaction for Stream A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoTransaction {
    /// Transaction version
    pub version: u32,
    /// Transaction inputs (UTXOs being spent)
    pub inputs: Vec<TxInput>,
    /// Transaction outputs (new UTXOs created)
    pub outputs: Vec<TxOutput>,
    /// Lock time (block height or timestamp)
    pub lock_time: u32,
}

impl UtxoTransaction {
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        Self {
            version: 1,
            inputs,
            outputs,
            lock_time: 0,
        }
    }
    
    /// Create a coinbase transaction (mining reward)
    pub fn coinbase(block_height: u64, reward: u64, miner_address: &Address) -> Self {
        Self {
            version: 1,
            inputs: vec![TxInput::coinbase(block_height, b"PYRAX")],
            outputs: vec![TxOutput::p2pkh(reward, miner_address)],
            lock_time: 0,
        }
    }
    
    /// Encode transaction for hashing (without signatures)
    pub fn encode_for_signing(&self, input_index: usize, prev_script_pubkey: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&(self.inputs.len() as u32).to_le_bytes());
        
        for (i, input) in self.inputs.iter().enumerate() {
            buf.extend_from_slice(&input.previous_output.encode());
            if i == input_index {
                // Include the previous output's script for the input being signed
                buf.extend_from_slice(&(prev_script_pubkey.len() as u32).to_le_bytes());
                buf.extend_from_slice(prev_script_pubkey);
            } else {
                // Empty script for other inputs
                buf.extend_from_slice(&0u32.to_le_bytes());
            }
            buf.extend_from_slice(&input.sequence.to_le_bytes());
        }
        
        buf.extend_from_slice(&(self.outputs.len() as u32).to_le_bytes());
        for output in &self.outputs {
            buf.extend_from_slice(&output.encode());
        }
        
        buf.extend_from_slice(&self.lock_time.to_le_bytes());
        // SIGHASH_ALL
        buf.extend_from_slice(&1u32.to_le_bytes());
        
        buf
    }
    
    /// Compute the signing hash for a specific input
    pub fn signing_hash(&self, input_index: usize, prev_script_pubkey: &[u8]) -> H256 {
        let encoded = self.encode_for_signing(input_index, prev_script_pubkey);
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(&encoded);
        hasher.finalize(&mut output);
        H256::from_slice(&output)
    }
    
    /// Encode the full transaction
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&(self.inputs.len() as u32).to_le_bytes());
        
        for input in &self.inputs {
            buf.extend_from_slice(&input.encode());
        }
        
        buf.extend_from_slice(&(self.outputs.len() as u32).to_le_bytes());
        for output in &self.outputs {
            buf.extend_from_slice(&output.encode());
        }
        
        buf.extend_from_slice(&self.lock_time.to_le_bytes());
        buf
    }
    
    /// Compute the transaction hash (txid)
    pub fn txid(&self) -> H256 {
        let encoded = self.encode();
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(&encoded);
        hasher.finalize(&mut output);
        H256::from_slice(&output)
    }
    
    /// Check if this is a coinbase transaction
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].is_coinbase()
    }
    
    /// Total output value
    pub fn total_output(&self) -> u64 {
        self.outputs.iter().map(|o| o.value).sum()
    }
    
    /// Virtual size (for fee calculation)
    pub fn vsize(&self) -> usize {
        self.encode().len()
    }
}

/// A stored UTXO in the UTXO set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    /// The output data
    pub output: TxOutput,
    /// Block height where this UTXO was created
    pub height: u64,
    /// Whether this is from a coinbase transaction
    pub is_coinbase: bool,
}

impl Utxo {
    pub fn new(output: TxOutput, height: u64, is_coinbase: bool) -> Self {
        Self {
            output,
            height,
            is_coinbase,
        }
    }
    
    /// Check if this UTXO is mature (coinbase outputs need 100 confirmations)
    pub fn is_mature(&self, current_height: u64) -> bool {
        if !self.is_coinbase {
            return true;
        }
        // Coinbase maturity: 100 blocks
        current_height >= self.height + 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_outpoint_encode() {
        let outpoint = OutPoint::new(H256::zero(), 0);
        let encoded = outpoint.encode();
        assert_eq!(encoded.len(), 36); // 32 + 4
    }
    
    #[test]
    fn test_coinbase_transaction() {
        let miner = Address::ZERO;
        let tx = UtxoTransaction::coinbase(0, 50 * 100_000_000, &miner);
        
        assert!(tx.is_coinbase());
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.outputs[0].value, 50 * 100_000_000);
    }
    
    #[test]
    fn test_p2pkh_output() {
        let address = Address::ZERO;
        let output = TxOutput::p2pkh(1000, &address);
        
        // Should be able to extract the address back
        let extracted = output.extract_p2pkh_address().unwrap();
        assert_eq!(extracted, address);
    }
    
    #[test]
    fn test_txid_deterministic() {
        let miner = Address::ZERO;
        let tx1 = UtxoTransaction::coinbase(100, 50 * 100_000_000, &miner);
        let tx2 = UtxoTransaction::coinbase(100, 50 * 100_000_000, &miner);
        
        assert_eq!(tx1.txid(), tx2.txid());
    }
    
    #[test]
    fn test_utxo_maturity() {
        let output = TxOutput::p2pkh(1000, &Address::ZERO);
        
        // Regular UTXO is always mature
        let regular = Utxo::new(output.clone(), 100, false);
        assert!(regular.is_mature(100));
        assert!(regular.is_mature(0)); // Even at height 0
        
        // Coinbase needs 100 confirmations
        let coinbase = Utxo::new(output, 100, true);
        assert!(!coinbase.is_mature(100)); // Same block
        assert!(!coinbase.is_mature(199)); // 99 confirmations
        assert!(coinbase.is_mature(200));  // 100 confirmations
    }
}
