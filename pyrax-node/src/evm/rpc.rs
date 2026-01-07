//! EVM JSON-RPC API
//!
//! Implements Ethereum-compatible JSON-RPC methods (eth_*)

use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::chain::EvmChain;
use super::types::{Address, B256, U256, EvmTransaction, SignedTransaction};
use super::executor::ExecutionResult;

/// EVM RPC Handler
pub struct EvmRpcHandler {
    chain: Arc<RwLock<EvmChain>>,
}

impl EvmRpcHandler {
    pub fn new(chain: Arc<RwLock<EvmChain>>) -> Self {
        Self { chain }
    }

    /// Handle JSON-RPC request
    pub fn handle_request(&self, method: &str, params: &[Value]) -> Result<Value, RpcError> {
        match method {
            "eth_chainId" => self.eth_chain_id(),
            "eth_blockNumber" => self.eth_block_number(),
            "eth_gasPrice" => self.eth_gas_price(),
            "eth_getBalance" => self.eth_get_balance(params),
            "eth_getTransactionCount" => self.eth_get_transaction_count(params),
            "eth_getCode" => self.eth_get_code(params),
            "eth_getStorageAt" => self.eth_get_storage_at(params),
            "eth_call" => self.eth_call(params),
            "eth_estimateGas" => self.eth_estimate_gas(params),
            "eth_sendRawTransaction" => self.eth_send_raw_transaction(params),
            "eth_getTransactionByHash" => self.eth_get_transaction_by_hash(params),
            "eth_getTransactionReceipt" => self.eth_get_transaction_receipt(params),
            "eth_getBlockByNumber" => self.eth_get_block_by_number(params),
            "eth_getBlockByHash" => self.eth_get_block_by_hash(params),
            "eth_getLogs" => self.eth_get_logs(params),
            "eth_accounts" => self.eth_accounts(),
            "eth_syncing" => self.eth_syncing(),
            "net_version" => self.net_version(),
            "net_listening" => self.net_listening(),
            "web3_clientVersion" => self.web3_client_version(),
            _ => Err(RpcError::MethodNotFound(method.to_string())),
        }
    }

    // === Chain Info Methods ===

    fn eth_chain_id(&self) -> Result<Value, RpcError> {
        let chain = self.chain.read();
        Ok(json!(format!("0x{:x}", chain.config().chain_id)))
    }

    fn eth_block_number(&self) -> Result<Value, RpcError> {
        let chain = self.chain.read();
        Ok(json!(format!("0x{:x}", chain.head_number())))
    }

    fn eth_gas_price(&self) -> Result<Value, RpcError> {
        let chain = self.chain.read();
        let block = chain.latest_block().ok_or(RpcError::Internal("No blocks".into()))?;
        Ok(json!(format!("0x{:x}", block.header.base_fee_per_gas)))
    }

    fn net_version(&self) -> Result<Value, RpcError> {
        let chain = self.chain.read();
        Ok(json!(chain.config().chain_id.to_string()))
    }

    fn net_listening(&self) -> Result<Value, RpcError> {
        Ok(json!(true))
    }

    fn web3_client_version(&self) -> Result<Value, RpcError> {
        Ok(json!("PYRAX-EVM/1.0.0"))
    }

    fn eth_syncing(&self) -> Result<Value, RpcError> {
        Ok(json!(false))
    }

    fn eth_accounts(&self) -> Result<Value, RpcError> {
        Ok(json!([]))
    }

    // === Account Methods ===

    fn eth_get_balance(&self, params: &[Value]) -> Result<Value, RpcError> {
        let address = parse_address(params.get(0))?;
        let chain = self.chain.read();
        let balance = chain.get_balance(&address);
        Ok(json!(format_u256(&balance)))
    }

    fn eth_get_transaction_count(&self, params: &[Value]) -> Result<Value, RpcError> {
        let address = parse_address(params.get(0))?;
        let chain = self.chain.read();
        let nonce = chain.get_nonce(&address);
        Ok(json!(format!("0x{:x}", nonce)))
    }

    fn eth_get_code(&self, params: &[Value]) -> Result<Value, RpcError> {
        let address = parse_address(params.get(0))?;
        let chain = self.chain.read();
        let code = chain.get_code(&address);
        Ok(json!(format!("0x{}", hex::encode(&code))))
    }

    fn eth_get_storage_at(&self, params: &[Value]) -> Result<Value, RpcError> {
        let address = parse_address(params.get(0))?;
        let key = parse_b256(params.get(1))?;
        let chain = self.chain.read();
        let value = chain.get_storage(&address, &key);
        Ok(json!(format_u256(&value)))
    }

    // === Transaction Methods ===

    fn eth_call(&self, params: &[Value]) -> Result<Value, RpcError> {
        let call_obj = params.get(0).ok_or(RpcError::InvalidParams("Missing call object".into()))?;
        
        let to = parse_address(call_obj.get("to"))?;
        let from = call_obj.get("from").and_then(|v| parse_address(Some(v)).ok());
        let data = parse_bytes(call_obj.get("data")).unwrap_or_default();
        let value = parse_u256(call_obj.get("value")).unwrap_or(U256::ZERO);

        let chain = self.chain.read();
        let result = chain.call(to, data, from, value);

        if result.success {
            Ok(json!(format!("0x{}", hex::encode(&result.output))))
        } else {
            Err(RpcError::ExecutionError(result.revert_reason.unwrap_or_default()))
        }
    }

    fn eth_estimate_gas(&self, params: &[Value]) -> Result<Value, RpcError> {
        let call_obj = params.get(0).ok_or(RpcError::InvalidParams("Missing call object".into()))?;
        
        let to = call_obj.get("to").and_then(|v| parse_address(Some(v)).ok());
        let from = parse_address(call_obj.get("from")).unwrap_or([0u8; 20]);
        let data = parse_bytes(call_obj.get("data")).unwrap_or_default();
        let value = parse_u256(call_obj.get("value")).unwrap_or(U256::ZERO);
        let gas_limit = parse_u64(call_obj.get("gas")).unwrap_or(30_000_000);

        let chain = self.chain.read();
        let tx = EvmTransaction {
            tx_type: 2,
            chain_id: chain.config().chain_id,
            nonce: chain.get_nonce(&from),
            max_priority_fee_per_gas: 0,
            max_fee_per_gas: chain.latest_block().map(|b| b.header.base_fee_per_gas).unwrap_or(1_000_000_000),
            gas_limit,
            to,
            value,
            input: data,
            access_list: Vec::new(),
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
        };

        match chain.estimate_gas(&tx, from) {
            Ok(gas) => Ok(json!(format!("0x{:x}", gas))),
            Err(e) => Err(RpcError::ExecutionError(e)),
        }
    }

    fn eth_send_raw_transaction(&self, params: &[Value]) -> Result<Value, RpcError> {
        let raw_tx = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or(RpcError::InvalidParams("Missing raw transaction".into()))?;

        let tx_bytes = hex::decode(raw_tx.trim_start_matches("0x"))
            .map_err(|_| RpcError::InvalidParams("Invalid hex".into()))?;

        // Decode and submit transaction
        let signed_tx = decode_signed_transaction(&tx_bytes)?;
        
        let mut chain = self.chain.write();
        let tx_hash = chain.submit_transaction(signed_tx)
            .map_err(|e| RpcError::ExecutionError(e))?;

        Ok(json!(format!("0x{}", hex::encode(&tx_hash))))
    }

    fn eth_get_transaction_by_hash(&self, params: &[Value]) -> Result<Value, RpcError> {
        let hash = parse_b256(params.get(0))?;
        let chain = self.chain.read();
        
        match chain.get_transaction(&hash) {
            Some((tx, block_num, tx_idx)) => {
                Ok(json!({
                    "hash": format!("0x{}", hex::encode(&hash)),
                    "blockNumber": format!("0x{:x}", block_num),
                    "transactionIndex": format!("0x{:x}", tx_idx),
                    "from": format!("0x{}", hex::encode(&tx.sender)),
                    "to": tx.transaction.to.map(|t| format!("0x{}", hex::encode(&t))),
                    "value": format_u256(&tx.transaction.value),
                    "gas": format!("0x{:x}", tx.transaction.gas_limit),
                    "gasPrice": format!("0x{:x}", tx.transaction.max_fee_per_gas),
                    "input": format!("0x{}", hex::encode(&tx.transaction.input)),
                    "nonce": format!("0x{:x}", tx.transaction.nonce),
                    "chainId": format!("0x{:x}", tx.transaction.chain_id),
                }))
            }
            None => Ok(json!(null)),
        }
    }

    fn eth_get_transaction_receipt(&self, params: &[Value]) -> Result<Value, RpcError> {
        let hash = parse_b256(params.get(0))?;
        let chain = self.chain.read();
        
        match chain.get_receipt(&hash) {
            Some(receipt) => {
                Ok(json!({
                    "transactionHash": format!("0x{}", hex::encode(&receipt.transaction_hash)),
                    "transactionIndex": format!("0x{:x}", receipt.transaction_index),
                    "blockHash": format!("0x{}", hex::encode(&receipt.block_hash)),
                    "blockNumber": format!("0x{:x}", receipt.block_number),
                    "from": format!("0x{}", hex::encode(&receipt.from)),
                    "to": receipt.to.map(|t| format!("0x{}", hex::encode(&t))),
                    "cumulativeGasUsed": format!("0x{:x}", receipt.cumulative_gas_used),
                    "gasUsed": format!("0x{:x}", receipt.gas_used),
                    "contractAddress": receipt.contract_address.map(|a| format!("0x{}", hex::encode(&a))),
                    "logs": receipt.logs.iter().map(|l| json!({
                        "address": format!("0x{}", hex::encode(&l.address)),
                        "topics": l.topics.iter().map(|t| format!("0x{}", hex::encode(t))).collect::<Vec<_>>(),
                        "data": format!("0x{}", hex::encode(&l.data)),
                        "blockNumber": format!("0x{:x}", l.block_number),
                        "transactionHash": format!("0x{}", hex::encode(&l.transaction_hash)),
                        "transactionIndex": format!("0x{:x}", l.transaction_index),
                        "logIndex": format!("0x{:x}", l.log_index),
                    })).collect::<Vec<_>>(),
                    "status": format!("0x{:x}", receipt.status),
                    "effectiveGasPrice": format!("0x{:x}", receipt.effective_gas_price),
                }))
            }
            None => Ok(json!(null)),
        }
    }

    // === Block Methods ===

    fn eth_get_block_by_number(&self, params: &[Value]) -> Result<Value, RpcError> {
        let block_num = parse_block_number(params.get(0))?;
        let full_txs = params.get(1).and_then(|v| v.as_bool()).unwrap_or(false);
        
        let chain = self.chain.read();
        let number = match block_num {
            BlockNumber::Latest => chain.head_number(),
            BlockNumber::Number(n) => n,
            BlockNumber::Pending => chain.head_number() + 1,
            BlockNumber::Earliest => 0,
        };

        match chain.get_block(number) {
            Some(block) => Ok(format_block(&block, full_txs)),
            None => Ok(json!(null)),
        }
    }

    fn eth_get_block_by_hash(&self, params: &[Value]) -> Result<Value, RpcError> {
        let hash = parse_b256(params.get(0))?;
        let full_txs = params.get(1).and_then(|v| v.as_bool()).unwrap_or(false);
        
        let chain = self.chain.read();
        match chain.get_block_by_hash(&hash) {
            Some(block) => Ok(format_block(&block, full_txs)),
            None => Ok(json!(null)),
        }
    }

    fn eth_get_logs(&self, params: &[Value]) -> Result<Value, RpcError> {
        let filter = params.get(0).ok_or(RpcError::InvalidParams("Missing filter".into()))?;
        
        let from_block = filter.get("fromBlock")
            .and_then(|v| parse_block_number(Some(v)).ok())
            .unwrap_or(BlockNumber::Latest);
        let to_block = filter.get("toBlock")
            .and_then(|v| parse_block_number(Some(v)).ok())
            .unwrap_or(BlockNumber::Latest);
        let address_filter = filter.get("address")
            .and_then(|v| parse_address(Some(v)).ok());
        let topics: Vec<Option<B256>> = filter.get("topics")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|t| parse_b256(Some(t)).ok()).collect())
            .unwrap_or_default();

        let chain = self.chain.read();
        let head = chain.head_number();
        
        let start = match from_block {
            BlockNumber::Latest => head,
            BlockNumber::Number(n) => n,
            BlockNumber::Earliest => 0,
            BlockNumber::Pending => head,
        };
        let end = match to_block {
            BlockNumber::Latest => head,
            BlockNumber::Number(n) => n,
            BlockNumber::Earliest => 0,
            BlockNumber::Pending => head,
        };

        let mut logs = Vec::new();
        for block_num in start..=end {
            if let Some(block) = chain.get_block(block_num) {
                for receipt in &block.receipts {
                    for log in &receipt.logs {
                        // Apply filters
                        if let Some(addr) = address_filter {
                            if log.address != addr {
                                continue;
                            }
                        }
                        
                        let mut topic_match = true;
                        for (i, topic_filter) in topics.iter().enumerate() {
                            if let Some(expected) = topic_filter {
                                if log.topics.get(i) != Some(expected) {
                                    topic_match = false;
                                    break;
                                }
                            }
                        }
                        
                        if topic_match {
                            logs.push(json!({
                                "address": format!("0x{}", hex::encode(&log.address)),
                                "topics": log.topics.iter().map(|t| format!("0x{}", hex::encode(t))).collect::<Vec<_>>(),
                                "data": format!("0x{}", hex::encode(&log.data)),
                                "blockNumber": format!("0x{:x}", log.block_number),
                                "transactionHash": format!("0x{}", hex::encode(&log.transaction_hash)),
                                "transactionIndex": format!("0x{:x}", log.transaction_index),
                                "blockHash": format!("0x{}", hex::encode(&log.block_hash)),
                                "logIndex": format!("0x{:x}", log.log_index),
                                "removed": log.removed,
                            }));
                        }
                    }
                }
            }
        }

        Ok(json!(logs))
    }
}

// === Helper Types and Functions ===

#[derive(Debug)]
pub enum RpcError {
    InvalidParams(String),
    MethodNotFound(String),
    ExecutionError(String),
    Internal(String),
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcError::InvalidParams(msg) => write!(f, "Invalid params: {}", msg),
            RpcError::MethodNotFound(method) => write!(f, "Method not found: {}", method),
            RpcError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            RpcError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

enum BlockNumber {
    Latest,
    Pending,
    Earliest,
    Number(u64),
}

fn parse_block_number(v: Option<&Value>) -> Result<BlockNumber, RpcError> {
    match v.and_then(|v| v.as_str()) {
        Some("latest") | None => Ok(BlockNumber::Latest),
        Some("pending") => Ok(BlockNumber::Pending),
        Some("earliest") => Ok(BlockNumber::Earliest),
        Some(s) => {
            let s = s.trim_start_matches("0x");
            u64::from_str_radix(s, 16)
                .map(BlockNumber::Number)
                .map_err(|_| RpcError::InvalidParams("Invalid block number".into()))
        }
    }
}

fn parse_address(v: Option<&Value>) -> Result<Address, RpcError> {
    let s = v.and_then(|v| v.as_str())
        .ok_or(RpcError::InvalidParams("Missing address".into()))?;
    let bytes = hex::decode(s.trim_start_matches("0x"))
        .map_err(|_| RpcError::InvalidParams("Invalid address hex".into()))?;
    if bytes.len() != 20 {
        return Err(RpcError::InvalidParams("Address must be 20 bytes".into()));
    }
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&bytes);
    Ok(addr)
}

fn parse_b256(v: Option<&Value>) -> Result<B256, RpcError> {
    let s = v.and_then(|v| v.as_str())
        .ok_or(RpcError::InvalidParams("Missing hash".into()))?;
    let bytes = hex::decode(s.trim_start_matches("0x"))
        .map_err(|_| RpcError::InvalidParams("Invalid hash hex".into()))?;
    if bytes.len() != 32 {
        return Err(RpcError::InvalidParams("Hash must be 32 bytes".into()));
    }
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes);
    Ok(hash)
}

fn parse_bytes(v: Option<&Value>) -> Result<Vec<u8>, RpcError> {
    let s = v.and_then(|v| v.as_str())
        .ok_or(RpcError::InvalidParams("Missing bytes".into()))?;
    hex::decode(s.trim_start_matches("0x"))
        .map_err(|_| RpcError::InvalidParams("Invalid hex".into()))
}

fn parse_u256(v: Option<&Value>) -> Result<U256, RpcError> {
    let s = v.and_then(|v| v.as_str())
        .ok_or(RpcError::InvalidParams("Missing value".into()))?;
    let s = s.trim_start_matches("0x");
    
    // Parse hex string to U256
    let mut bytes = [0u8; 32];
    let hex_bytes = hex::decode(format!("{:0>64}", s))
        .map_err(|_| RpcError::InvalidParams("Invalid U256 hex".into()))?;
    bytes.copy_from_slice(&hex_bytes);
    Ok(U256::from_be_bytes(bytes))
}

fn parse_u64(v: Option<&Value>) -> Result<u64, RpcError> {
    let s = v.and_then(|v| v.as_str())
        .ok_or(RpcError::InvalidParams("Missing u64".into()))?;
    u64::from_str_radix(s.trim_start_matches("0x"), 16)
        .map_err(|_| RpcError::InvalidParams("Invalid u64".into()))
}

fn format_u256(v: &U256) -> String {
    let bytes = v.to_be_bytes();
    // Find first non-zero byte
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(31);
    format!("0x{}", hex::encode(&bytes[start..]))
}

fn format_block(block: &super::chain::EvmBlock, full_txs: bool) -> Value {
    let txs: Vec<Value> = if full_txs {
        block.transactions.iter().enumerate().map(|(i, tx)| {
            json!({
                "hash": format!("0x{}", hex::encode(&tx.hash())),
                "blockNumber": format!("0x{:x}", block.header.number),
                "transactionIndex": format!("0x{:x}", i),
                "from": format!("0x{}", hex::encode(&tx.sender)),
                "to": tx.transaction.to.map(|t| format!("0x{}", hex::encode(&t))),
                "value": format_u256(&tx.transaction.value),
                "gas": format!("0x{:x}", tx.transaction.gas_limit),
                "gasPrice": format!("0x{:x}", tx.transaction.max_fee_per_gas),
                "input": format!("0x{}", hex::encode(&tx.transaction.input)),
                "nonce": format!("0x{:x}", tx.transaction.nonce),
            })
        }).collect()
    } else {
        block.transactions.iter().map(|tx| {
            json!(format!("0x{}", hex::encode(&tx.hash())))
        }).collect()
    };

    json!({
        "number": format!("0x{:x}", block.header.number),
        "hash": format!("0x{}", hex::encode(&block.header.hash)),
        "parentHash": format!("0x{}", hex::encode(&block.header.parent_hash)),
        "stateRoot": format!("0x{}", hex::encode(&block.header.state_root)),
        "transactionsRoot": format!("0x{}", hex::encode(&block.header.transactions_root)),
        "receiptsRoot": format!("0x{}", hex::encode(&block.header.receipts_root)),
        "logsBloom": format!("0x{}", hex::encode(&block.header.logs_bloom)),
        "difficulty": format!("0x{:x}", block.header.difficulty),
        "gasLimit": format!("0x{:x}", block.header.gas_limit),
        "gasUsed": format!("0x{:x}", block.header.gas_used),
        "timestamp": format!("0x{:x}", block.header.timestamp),
        "extraData": format!("0x{}", hex::encode(&block.header.extra_data)),
        "mixHash": format!("0x{}", hex::encode(&block.header.mix_hash)),
        "nonce": format!("0x{:016x}", block.header.nonce),
        "baseFeePerGas": format!("0x{:x}", block.header.base_fee_per_gas),
        "transactions": txs,
    })
}

fn decode_signed_transaction(bytes: &[u8]) -> Result<SignedTransaction, RpcError> {
    // Simplified RLP decoding for EIP-1559 transactions
    if bytes.is_empty() {
        return Err(RpcError::InvalidParams("Empty transaction".into()));
    }

    // For now, create a basic transaction from raw bytes
    // In production, implement full RLP decoding
    if bytes[0] == 0x02 {
        // EIP-1559 transaction
        let tx = EvmTransaction {
            tx_type: 2,
            chain_id: 7777,
            nonce: 0,
            max_priority_fee_per_gas: 1_000_000_000,
            max_fee_per_gas: 2_000_000_000,
            gas_limit: 21000,
            to: None,
            value: U256::ZERO,
            input: bytes[1..].to_vec(),
            access_list: Vec::new(),
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
        };
        Ok(SignedTransaction::new(tx, [0u8; 20]))
    } else {
        Err(RpcError::InvalidParams("Unsupported transaction type".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let addr_str = json!("0x1234567890123456789012345678901234567890");
        let addr = parse_address(Some(&addr_str)).unwrap();
        assert_eq!(addr[0], 0x12);
        assert_eq!(addr[19], 0x90);
    }

    #[test]
    fn test_parse_block_number() {
        assert!(matches!(parse_block_number(Some(&json!("latest"))).unwrap(), BlockNumber::Latest));
        assert!(matches!(parse_block_number(Some(&json!("0x10"))).unwrap(), BlockNumber::Number(16)));
    }

    #[test]
    fn test_format_u256() {
        let v = U256::from_u64(0x1234);
        assert_eq!(format_u256(&v), "0x1234");
    }
}
