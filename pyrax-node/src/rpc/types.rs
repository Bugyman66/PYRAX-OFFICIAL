//! RPC types for PYRAX JSON-RPC API
//! 
//! UTXO-based blockchain types for RPC responses

use serde::{Deserialize, Serialize};
use crate::types::{Block, Transaction, H256, Address, BlockNumber};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcBlock {
    pub hash: String,
    pub height: u64,
    pub parent_hash: String,
    pub timestamp: u64,
    pub difficulty: u64,
    pub nonce: String,
    pub extra_nonce: u64,
    pub merkle_root: String,
    pub utxo_commitment: String,
    pub beneficiary: String,
    pub version: u32,
    pub stream: u8,
    pub transaction_count: usize,
    pub transactions: Option<Vec<RpcTransaction>>,
}

impl RpcBlock {
    pub fn from_block(block: &Block) -> Self {
        Self {
            hash: format!("0x{}", hex::encode(&block.hash().0)),
            height: block.height(),
            parent_hash: format!("0x{}", hex::encode(&block.header.parent_hash.0)),
            timestamp: block.header.timestamp,
            difficulty: block.header.difficulty,
            nonce: format!("0x{:016x}", block.header.nonce),
            extra_nonce: block.header.extra_nonce,
            merkle_root: format!("0x{}", hex::encode(&block.header.merkle_root.0)),
            utxo_commitment: format!("0x{}", hex::encode(&block.header.utxo_commitment.0)),
            beneficiary: format!("0x{}", hex::encode(&block.header.beneficiary.0)),
            version: block.header.version,
            stream: block.header.stream,
            transaction_count: block.transactions.len(),
            transactions: None,
        }
    }

    pub fn with_transactions(mut self, block: &Block) -> Self {
        self.transactions = Some(
            block.transactions.iter()
                .enumerate()
                .map(|(i, tx)| RpcTransaction::from_tx(tx, Some(&block.hash()), Some(block.height()), Some(i)))
                .collect()
        );
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcTransaction {
    pub txid: String,
    pub version: u32,
    pub lock_time: u32,
    pub is_coinbase: bool,
    pub inputs: Vec<RpcTxInput>,
    pub outputs: Vec<RpcTxOutput>,
    pub block_hash: Option<String>,
    pub block_height: Option<u64>,
    pub tx_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcTxInput {
    pub txid: String,
    pub vout: u32,
    pub script_sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcTxOutput {
    pub value: u64,
    pub script_pubkey: String,
}

impl RpcTransaction {
    pub fn from_tx(tx: &Transaction, block_hash: Option<&H256>, block_height: Option<u64>, tx_index: Option<usize>) -> Self {
        Self {
            txid: format!("0x{}", hex::encode(&tx.txid().0)),
            version: tx.version,
            lock_time: tx.lock_time,
            is_coinbase: tx.is_coinbase(),
            inputs: tx.inputs.iter().map(|input| RpcTxInput {
                txid: format!("0x{}", hex::encode(&input.previous_output.txid.0)),
                vout: input.previous_output.vout,
                script_sig: format!("0x{}", hex::encode(&input.script_sig)),
            }).collect(),
            outputs: tx.outputs.iter().map(|output| RpcTxOutput {
                value: output.value,
                script_pubkey: format!("0x{}", hex::encode(&output.script_pubkey)),
            }).collect(),
            block_hash: block_hash.map(|h| format!("0x{}", hex::encode(&h.0))),
            block_height,
            tx_index,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcChainInfo {
    pub chain_id: u32,
    pub network: String,
    pub best_block_hash: String,
    pub best_block_height: u64,
    pub genesis_hash: String,
    pub difficulty: u64,
    pub utxo_count: u64,
    pub syncing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcPeerInfo {
    pub peer_id: String,
    pub address: String,
    pub ip: String,
    pub port: u16,
    pub protocol: String,
    pub direction: String,
    pub connected_secs: u64,
    pub last_seen: u64,
    pub version: String,
    pub block_height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcNetworkInfo {
    pub peer_count: usize,
    pub peers: Vec<RpcPeerInfo>,
    pub local_peer_id: String,
    pub listen_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMempoolInfo {
    pub size: usize,
    pub bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcUtxo {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub script_pubkey: String,
    pub height: u64,
    pub coinbase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcBalance {
    pub address: String,
    pub balance: u64,
    pub utxo_count: usize,
    pub utxos: Vec<RpcUtxo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcBlockTemplate {
    pub height: u64,
    pub parent_hash: String,
    pub timestamp: u64,
    pub difficulty: u64,
    pub target: String,
    pub transactions: Vec<String>,
    pub coinbase_value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcSubmitResult {
    pub accepted: bool,
    pub hash: Option<String>,
    pub error: Option<String>,
}
