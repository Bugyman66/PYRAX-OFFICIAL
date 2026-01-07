//! EVM ↔ WASM Interoperability
//!
//! Enables cross-VM calls between Solidity and Rust contracts

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::runtime::{ExecutionResult, ExecutionError};
use super::storage::ContractStorage;

/// Cross-VM call type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossVmCallType {
    /// Call from EVM to WASM
    EvmToWasm,
    /// Call from WASM to EVM
    WasmToEvm,
    /// Delegate call (uses caller's storage)
    DelegateCall,
    /// Static call (read-only)
    StaticCall,
}

/// Cross-VM call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossVmCall {
    /// Call type
    pub call_type: CrossVmCallType,
    /// Source contract address
    pub from: Address,
    /// Target contract address
    pub to: Address,
    /// Function selector (4 bytes for EVM, variable for WASM)
    pub selector: Vec<u8>,
    /// Function name (for WASM calls)
    pub function_name: Option<String>,
    /// Encoded arguments
    pub args: Vec<u8>,
    /// Value to transfer
    pub value: u64,
    /// Gas limit for this call
    pub gas_limit: u64,
    /// Call depth
    pub depth: u32,
}

impl CrossVmCall {
    /// Create EVM to WASM call
    pub fn evm_to_wasm(
        from: Address,
        to: Address,
        function_name: String,
        args: Vec<u8>,
        value: u64,
        gas_limit: u64,
    ) -> Self {
        Self {
            call_type: CrossVmCallType::EvmToWasm,
            from,
            to,
            selector: Vec::new(),
            function_name: Some(function_name),
            args,
            value,
            gas_limit,
            depth: 0,
        }
    }

    /// Create WASM to EVM call
    pub fn wasm_to_evm(
        from: Address,
        to: Address,
        selector: [u8; 4],
        args: Vec<u8>,
        value: u64,
        gas_limit: u64,
    ) -> Self {
        Self {
            call_type: CrossVmCallType::WasmToEvm,
            from,
            to,
            selector: selector.to_vec(),
            function_name: None,
            args,
            value,
            gas_limit,
            depth: 0,
        }
    }

    /// Create static call
    pub fn static_call(
        call_type: CrossVmCallType,
        from: Address,
        to: Address,
        args: Vec<u8>,
        gas_limit: u64,
    ) -> Self {
        Self {
            call_type: CrossVmCallType::StaticCall,
            from,
            to,
            selector: Vec::new(),
            function_name: None,
            args,
            value: 0,
            gas_limit,
            depth: 0,
        }
    }

    /// Set call depth
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    /// Check if call is read-only
    pub fn is_static(&self) -> bool {
        self.call_type == CrossVmCallType::StaticCall
    }
}

/// Cross-VM call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossVmResult {
    /// Success flag
    pub success: bool,
    /// Return data
    pub return_data: Vec<u8>,
    /// Gas used
    pub gas_used: u64,
    /// Error message if failed
    pub error: Option<String>,
    /// Logs emitted
    pub logs: Vec<CrossVmLog>,
}

impl CrossVmResult {
    pub fn success(return_data: Vec<u8>, gas_used: u64) -> Self {
        Self {
            success: true,
            return_data,
            gas_used,
            error: None,
            logs: Vec::new(),
        }
    }

    pub fn failure(error: String, gas_used: u64) -> Self {
        Self {
            success: false,
            return_data: Vec::new(),
            gas_used,
            error: Some(error),
            logs: Vec::new(),
        }
    }

    pub fn with_logs(mut self, logs: Vec<CrossVmLog>) -> Self {
        self.logs = logs;
        self
    }
}

/// Cross-VM log event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossVmLog {
    /// Source address
    pub address: Address,
    /// Log topics
    pub topics: Vec<[u8; 32]>,
    /// Log data
    pub data: Vec<u8>,
    /// Source VM type
    pub source_vm: VmType,
}

/// VM type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmType {
    Evm,
    Wasm,
}

/// ABI encoder/decoder for cross-VM calls
pub struct AbiCodec;

impl AbiCodec {
    /// Encode function call for EVM
    pub fn encode_evm_call(selector: [u8; 4], args: &[Vec<u8>]) -> Vec<u8> {
        let mut data = selector.to_vec();
        
        for arg in args {
            // Pad to 32 bytes
            let mut padded = vec![0u8; 32];
            let start = 32 - arg.len().min(32);
            padded[start..].copy_from_slice(&arg[..arg.len().min(32)]);
            data.extend_from_slice(&padded);
        }
        
        data
    }

    /// Decode EVM return data
    pub fn decode_evm_return(data: &[u8]) -> Vec<Vec<u8>> {
        let mut result = Vec::new();
        let mut offset = 0;
        
        while offset + 32 <= data.len() {
            result.push(data[offset..offset + 32].to_vec());
            offset += 32;
        }
        
        result
    }

    /// Encode function call for WASM (simple length-prefixed format)
    pub fn encode_wasm_call(function: &str, args: &[u8]) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Function name length + name
        let name_bytes = function.as_bytes();
        data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(name_bytes);
        
        // Args length + args
        data.extend_from_slice(&(args.len() as u32).to_le_bytes());
        data.extend_from_slice(args);
        
        data
    }

    /// Decode WASM call data
    pub fn decode_wasm_call(data: &[u8]) -> Result<(String, Vec<u8>), String> {
        if data.len() < 8 {
            return Err("Data too short".to_string());
        }

        // Read function name length
        let name_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if data.len() < 4 + name_len + 4 {
            return Err("Invalid name length".to_string());
        }

        // Read function name
        let name = String::from_utf8(data[4..4 + name_len].to_vec())
            .map_err(|e| e.to_string())?;

        // Read args length
        let args_offset = 4 + name_len;
        let args_len = u32::from_le_bytes([
            data[args_offset],
            data[args_offset + 1],
            data[args_offset + 2],
            data[args_offset + 3],
        ]) as usize;

        // Read args
        let args_start = args_offset + 4;
        if data.len() < args_start + args_len {
            return Err("Invalid args length".to_string());
        }

        let args = data[args_start..args_start + args_len].to_vec();

        Ok((name, args))
    }

    /// Compute function selector (first 4 bytes of keccak256)
    pub fn compute_selector(signature: &str) -> [u8; 4] {
        use sha3::{Digest, Keccak256};
        
        let mut hasher = Keccak256::new();
        hasher.update(signature.as_bytes());
        let hash = hasher.finalize();
        
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash[0..4]);
        selector
    }
}

/// Bridge between EVM and WASM VMs
pub struct EvmWasmBridge {
    /// WASM contract addresses
    wasm_contracts: Arc<RwLock<HashMap<Address, bool>>>,
    /// EVM contract addresses
    evm_contracts: Arc<RwLock<HashMap<Address, bool>>>,
    /// Call history for debugging
    call_history: Arc<RwLock<Vec<CrossVmCall>>>,
    /// Max call depth
    max_depth: u32,
    /// Storage reference
    storage: Arc<ContractStorage>,
}

impl EvmWasmBridge {
    /// Create new bridge
    pub fn new(storage: Arc<ContractStorage>) -> Self {
        Self {
            wasm_contracts: Arc::new(RwLock::new(HashMap::new())),
            evm_contracts: Arc::new(RwLock::new(HashMap::new())),
            call_history: Arc::new(RwLock::new(Vec::new())),
            max_depth: 10,
            storage,
        }
    }

    /// Register WASM contract
    pub fn register_wasm_contract(&self, address: Address) {
        self.wasm_contracts.write().insert(address, true);
    }

    /// Register EVM contract
    pub fn register_evm_contract(&self, address: Address) {
        self.evm_contracts.write().insert(address, true);
    }

    /// Check if address is WASM contract
    pub fn is_wasm_contract(&self, address: &Address) -> bool {
        self.wasm_contracts.read().contains_key(address)
    }

    /// Check if address is EVM contract
    pub fn is_evm_contract(&self, address: &Address) -> bool {
        self.evm_contracts.read().contains_key(address)
    }

    /// Get contract VM type
    pub fn get_vm_type(&self, address: &Address) -> Option<VmType> {
        if self.is_wasm_contract(address) {
            Some(VmType::Wasm)
        } else if self.is_evm_contract(address) {
            Some(VmType::Evm)
        } else {
            None
        }
    }

    /// Prepare cross-VM call
    pub fn prepare_call(&self, call: CrossVmCall) -> Result<PreparedCall, String> {
        // Check call depth
        if call.depth >= self.max_depth {
            return Err("Max call depth exceeded".to_string());
        }

        // Determine target VM
        let target_vm = self.get_vm_type(&call.to)
            .ok_or_else(|| "Unknown contract type".to_string())?;

        // Encode call data
        let encoded_data = match (&call.call_type, target_vm) {
            (CrossVmCallType::EvmToWasm, VmType::Wasm) => {
                let function = call.function_name.as_ref()
                    .ok_or_else(|| "Function name required for WASM call".to_string())?;
                AbiCodec::encode_wasm_call(function, &call.args)
            }
            (CrossVmCallType::WasmToEvm, VmType::Evm) => {
                if call.selector.len() < 4 {
                    return Err("Selector required for EVM call".to_string());
                }
                let mut selector = [0u8; 4];
                selector.copy_from_slice(&call.selector[..4]);
                AbiCodec::encode_evm_call(selector, &[call.args.clone()])
            }
            _ => call.args.clone(),
        };

        // Record call
        self.call_history.write().push(call.clone());

        Ok(PreparedCall {
            call,
            target_vm,
            encoded_data,
        })
    }

    /// Get call history
    pub fn get_call_history(&self) -> Vec<CrossVmCall> {
        self.call_history.read().clone()
    }

    /// Clear call history
    pub fn clear_history(&self) {
        self.call_history.write().clear();
    }

    /// Get bridge statistics
    pub fn stats(&self) -> BridgeStats {
        BridgeStats {
            wasm_contracts: self.wasm_contracts.read().len() as u64,
            evm_contracts: self.evm_contracts.read().len() as u64,
            total_calls: self.call_history.read().len() as u64,
            max_depth: self.max_depth,
        }
    }
}

/// Prepared cross-VM call
#[derive(Debug, Clone)]
pub struct PreparedCall {
    /// Original call
    pub call: CrossVmCall,
    /// Target VM type
    pub target_vm: VmType,
    /// Encoded call data
    pub encoded_data: Vec<u8>,
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    pub wasm_contracts: u64,
    pub evm_contracts: u64,
    pub total_calls: u64,
    pub max_depth: u32,
}

/// Precompile address for WASM calls from EVM
pub const WASM_PRECOMPILE_ADDRESS: [u8; 20] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x00, // 0x100
];

/// Precompile for calling WASM contracts from EVM
pub struct WasmPrecompile;

impl WasmPrecompile {
    /// Gas cost for WASM call
    pub const BASE_GAS: u64 = 1000;
    pub const GAS_PER_BYTE: u64 = 5;

    /// Calculate gas cost
    pub fn gas_cost(input_len: usize) -> u64 {
        Self::BASE_GAS + (input_len as u64 * Self::GAS_PER_BYTE)
    }

    /// Decode precompile input
    /// Format: [20 bytes target] [rest is call data]
    pub fn decode_input(input: &[u8]) -> Result<(Address, Vec<u8>), String> {
        if input.len() < 20 {
            return Err("Input too short".to_string());
        }

        let mut target = [0u8; 20];
        target.copy_from_slice(&input[0..20]);

        let data = input[20..].to_vec();

        Ok((Address(target), data))
    }

    /// Encode precompile input
    pub fn encode_input(target: &Address, data: &[u8]) -> Vec<u8> {
        let mut input = target.0.to_vec();
        input.extend_from_slice(data);
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_vm_call() {
        let call = CrossVmCall::evm_to_wasm(
            Address([1u8; 20]),
            Address([2u8; 20]),
            "transfer".to_string(),
            vec![1, 2, 3, 4],
            1000,
            100000,
        );

        assert_eq!(call.call_type, CrossVmCallType::EvmToWasm);
        assert_eq!(call.function_name, Some("transfer".to_string()));
        assert!(!call.is_static());
    }

    #[test]
    fn test_abi_codec_evm() {
        let selector = [0x01, 0x02, 0x03, 0x04];
        let args = vec![vec![0xff; 32]];
        
        let encoded = AbiCodec::encode_evm_call(selector, &args);
        
        assert_eq!(encoded.len(), 4 + 32);
        assert_eq!(&encoded[0..4], &selector);
    }

    #[test]
    fn test_abi_codec_wasm() {
        let function = "transfer";
        let args = vec![1, 2, 3, 4];
        
        let encoded = AbiCodec::encode_wasm_call(function, &args);
        let (decoded_fn, decoded_args) = AbiCodec::decode_wasm_call(&encoded).unwrap();
        
        assert_eq!(decoded_fn, function);
        assert_eq!(decoded_args, args);
    }

    #[test]
    fn test_compute_selector() {
        let selector = AbiCodec::compute_selector("transfer(address,uint256)");
        assert_eq!(selector.len(), 4);
    }

    #[test]
    fn test_bridge() {
        let storage = Arc::new(ContractStorage::new());
        let bridge = EvmWasmBridge::new(storage);

        let wasm_addr = Address([1u8; 20]);
        let evm_addr = Address([2u8; 20]);

        bridge.register_wasm_contract(wasm_addr);
        bridge.register_evm_contract(evm_addr);

        assert_eq!(bridge.get_vm_type(&wasm_addr), Some(VmType::Wasm));
        assert_eq!(bridge.get_vm_type(&evm_addr), Some(VmType::Evm));
    }

    #[test]
    fn test_prepare_call() {
        let storage = Arc::new(ContractStorage::new());
        let bridge = EvmWasmBridge::new(storage);

        let wasm_addr = Address([1u8; 20]);
        bridge.register_wasm_contract(wasm_addr);

        let call = CrossVmCall::evm_to_wasm(
            Address([2u8; 20]),
            wasm_addr,
            "test".to_string(),
            vec![1, 2, 3],
            0,
            100000,
        );

        let prepared = bridge.prepare_call(call).unwrap();
        assert_eq!(prepared.target_vm, VmType::Wasm);
    }

    #[test]
    fn test_wasm_precompile() {
        let target = Address([1u8; 20]);
        let data = vec![2, 3, 4, 5];

        let input = WasmPrecompile::encode_input(&target, &data);
        let (decoded_target, decoded_data) = WasmPrecompile::decode_input(&input).unwrap();

        assert_eq!(decoded_target, target);
        assert_eq!(decoded_data, data);
    }

    #[test]
    fn test_gas_cost() {
        let cost = WasmPrecompile::gas_cost(100);
        assert_eq!(cost, 1000 + 500);
    }
}
