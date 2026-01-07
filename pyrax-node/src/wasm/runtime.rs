//! WASM Runtime Engine
//!
//! Core execution engine using Wasmtime

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use wasmtime::{
    Config, Engine, Instance, Linker, Module, Store, TypedFunc,
    Caller, Memory, MemoryType,
};

use crate::types::{Address, H256};
use super::{
    MAX_CODE_SIZE, MAX_MEMORY_PAGES, DEFAULT_GAS_LIMIT, CALL_TIMEOUT_SECS,
    host::HostContext,
    metering::{GasMeter, GasConfig},
    storage::ContractStorage,
    contract::{Contract, ContractCode},
};

/// WASM runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Maximum code size in bytes
    pub max_code_size: usize,
    /// Maximum memory pages (64KB each)
    pub max_memory_pages: u32,
    /// Default gas limit
    pub default_gas_limit: u64,
    /// Call timeout in seconds
    pub call_timeout_secs: u64,
    /// Enable debug mode
    pub debug_mode: bool,
    /// Enable fuel-based metering
    pub fuel_metering: bool,
    /// Fuel to gas ratio
    pub fuel_to_gas_ratio: u64,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            max_code_size: MAX_CODE_SIZE,
            max_memory_pages: MAX_MEMORY_PAGES,
            default_gas_limit: DEFAULT_GAS_LIMIT,
            call_timeout_secs: CALL_TIMEOUT_SECS,
            debug_mode: false,
            fuel_metering: true,
            fuel_to_gas_ratio: 1,
        }
    }
}

/// Execution result from WASM call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Return data from the call
    pub return_data: Vec<u8>,
    /// Gas used
    pub gas_used: u64,
    /// Logs emitted
    pub logs: Vec<ContractLog>,
    /// State changes
    pub state_changes: Vec<StateChange>,
    /// Success flag
    pub success: bool,
    /// Revert reason (if any)
    pub revert_reason: Option<String>,
}

impl ExecutionResult {
    pub fn success(return_data: Vec<u8>, gas_used: u64) -> Self {
        Self {
            return_data,
            gas_used,
            logs: Vec::new(),
            state_changes: Vec::new(),
            success: true,
            revert_reason: None,
        }
    }

    pub fn revert(reason: String, gas_used: u64) -> Self {
        Self {
            return_data: Vec::new(),
            gas_used,
            logs: Vec::new(),
            state_changes: Vec::new(),
            success: false,
            revert_reason: Some(reason),
        }
    }
}

/// Contract log event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLog {
    /// Contract address
    pub address: Address,
    /// Log topics
    pub topics: Vec<[u8; 32]>,
    /// Log data
    pub data: Vec<u8>,
}

/// State change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// Contract address
    pub address: Address,
    /// Storage key
    pub key: [u8; 32],
    /// Old value
    pub old_value: Option<Vec<u8>>,
    /// New value
    pub new_value: Option<Vec<u8>>,
}

/// Execution errors
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Code too large: {size} bytes (max: {max})")]
    CodeTooLarge { size: usize, max: usize },

    #[error("Invalid WASM module: {0}")]
    InvalidModule(String),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Instantiation failed: {0}")]
    InstantiationFailed(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Out of gas: used {used}, limit {limit}")]
    OutOfGas { used: u64, limit: u64 },

    #[error("Execution timeout after {0} seconds")]
    Timeout(u64),

    #[error("Memory access out of bounds")]
    MemoryOutOfBounds,

    #[error("Stack overflow")]
    StackOverflow,

    #[error("Contract not found: {0:?}")]
    ContractNotFound(Address),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("Revert: {0}")]
    Revert(String),

    #[error("Host error: {0}")]
    HostError(String),
}

/// WASM Runtime - Main execution engine
pub struct WasmRuntime {
    /// Wasmtime engine
    engine: Engine,
    /// Runtime configuration
    config: WasmConfig,
    /// Compiled modules cache
    module_cache: Arc<RwLock<HashMap<H256, Module>>>,
    /// Contract storage
    storage: Arc<ContractStorage>,
    /// Gas configuration
    gas_config: GasConfig,
}

impl WasmRuntime {
    /// Create new WASM runtime
    pub fn new(config: WasmConfig, storage: Arc<ContractStorage>) -> Result<Self, ExecutionError> {
        let mut engine_config = Config::new();
        
        // Enable fuel-based metering for gas tracking
        if config.fuel_metering {
            engine_config.consume_fuel(true);
        }
        
        // Configure memory limits
        engine_config.max_wasm_stack(512 * 1024); // 512 KB stack
        engine_config.wasm_memory64(false);
        engine_config.wasm_threads(false);
        engine_config.wasm_simd(true);
        engine_config.wasm_bulk_memory(true);
        
        let engine = Engine::new(&engine_config)
            .map_err(|e| ExecutionError::CompilationFailed(e.to_string()))?;

        Ok(Self {
            engine,
            config,
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            storage,
            gas_config: GasConfig::default(),
        })
    }

    /// Create with default configuration
    pub fn with_defaults(storage: Arc<ContractStorage>) -> Result<Self, ExecutionError> {
        Self::new(WasmConfig::default(), storage)
    }

    /// Compile WASM bytecode to module
    pub fn compile(&self, code: &[u8]) -> Result<Module, ExecutionError> {
        if code.len() > self.config.max_code_size {
            return Err(ExecutionError::CodeTooLarge {
                size: code.len(),
                max: self.config.max_code_size,
            });
        }

        Module::new(&self.engine, code)
            .map_err(|e| ExecutionError::InvalidModule(e.to_string()))
    }

    /// Deploy a new contract
    pub fn deploy(
        &self,
        deployer: Address,
        code: &[u8],
        init_args: &[u8],
        gas_limit: u64,
    ) -> Result<(Address, ExecutionResult), ExecutionError> {
        // Compile the module
        let module = self.compile(code)?;
        
        // Generate contract address
        let contract_address = self.generate_contract_address(&deployer, code);
        
        // Cache the compiled module
        let code_hash = self.hash_code(code);
        self.module_cache.write().insert(code_hash, module.clone());
        
        // Store the contract code
        self.storage.store_code(contract_address, code.to_vec(), code_hash);
        
        // Call constructor if present
        let result = self.call_internal(
            &module,
            contract_address,
            deployer,
            "deploy",
            init_args,
            gas_limit,
            0,
        )?;

        if !result.success {
            // Rollback storage
            self.storage.remove_code(&contract_address);
            return Err(ExecutionError::Revert(
                result.revert_reason.unwrap_or_else(|| "Deployment failed".to_string())
            ));
        }

        Ok((contract_address, result))
    }

    /// Call a contract function
    pub fn call(
        &self,
        contract_address: Address,
        caller: Address,
        function: &str,
        args: &[u8],
        gas_limit: u64,
        value: u64,
    ) -> Result<ExecutionResult, ExecutionError> {
        // Get compiled module from cache or storage
        let module = self.get_module(&contract_address)?;
        
        self.call_internal(&module, contract_address, caller, function, args, gas_limit, value)
    }

    /// Internal call implementation
    fn call_internal(
        &self,
        module: &Module,
        contract_address: Address,
        caller: Address,
        function: &str,
        args: &[u8],
        gas_limit: u64,
        value: u64,
    ) -> Result<ExecutionResult, ExecutionError> {
        // Create host context
        let context = HostContext::new(
            contract_address,
            caller,
            value,
            gas_limit,
            self.storage.clone(),
        );

        // Create store with fuel
        let mut store = Store::new(&self.engine, context);
        if self.config.fuel_metering {
            store.set_fuel(gas_limit * self.config.fuel_to_gas_ratio)
                .map_err(|e| ExecutionError::ExecutionFailed(e.to_string()))?;
        }

        // Create linker and add host functions
        let mut linker = Linker::new(&self.engine);
        self.register_host_functions(&mut linker)?;

        // Instantiate module
        let instance = linker.instantiate(&mut store, module)
            .map_err(|e| ExecutionError::InstantiationFailed(e.to_string()))?;

        // Get memory export
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| ExecutionError::ExecutionFailed("No memory export".to_string()))?;

        // Allocate and write input data
        let input_ptr = self.allocate_input(&mut store, &instance, args)?;

        // Get the function
        let func = instance.get_typed_func::<(i32, i32), i32>(&mut store, function)
            .map_err(|_| ExecutionError::FunctionNotFound(function.to_string()))?;

        // Execute with timeout
        let start = Instant::now();
        let timeout = Duration::from_secs(self.config.call_timeout_secs);

        let result = func.call(&mut store, (input_ptr, args.len() as i32));

        if start.elapsed() > timeout {
            return Err(ExecutionError::Timeout(self.config.call_timeout_secs));
        }

        // Calculate gas used
        let gas_used = if self.config.fuel_metering {
            let remaining = store.get_fuel().unwrap_or(0);
            let initial = gas_limit * self.config.fuel_to_gas_ratio;
            (initial - remaining) / self.config.fuel_to_gas_ratio
        } else {
            0
        };

        match result {
            Ok(ret_ptr) => {
                // Read return data from memory
                let return_data = self.read_return_data(&store, &memory, ret_ptr)?;
                
                // Get logs and state changes from context
                let context = store.data();
                
                Ok(ExecutionResult {
                    return_data,
                    gas_used,
                    logs: context.logs.clone(),
                    state_changes: context.state_changes.clone(),
                    success: true,
                    revert_reason: None,
                })
            }
            Err(e) => {
                let reason = e.to_string();
                if reason.contains("out of fuel") {
                    Err(ExecutionError::OutOfGas { used: gas_used, limit: gas_limit })
                } else {
                    Ok(ExecutionResult::revert(reason, gas_used))
                }
            }
        }
    }

    /// Register host functions in linker
    fn register_host_functions(&self, linker: &mut Linker<HostContext>) -> Result<(), ExecutionError> {
        // Storage read
        linker.func_wrap("env", "storage_read", |mut caller: Caller<'_, HostContext>, key_ptr: i32, key_len: i32, value_ptr: i32| -> i32 {
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export");
            
            let key = read_memory(&caller, &memory, key_ptr as usize, key_len as usize);
            let value = caller.data().storage_read(&key);
            
            if let Some(v) = value {
                write_memory(&mut caller, &memory, value_ptr as usize, &v);
                v.len() as i32
            } else {
                -1
            }
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Storage write
        linker.func_wrap("env", "storage_write", |mut caller: Caller<'_, HostContext>, key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32| {
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export");
            
            let key = read_memory(&caller, &memory, key_ptr as usize, key_len as usize);
            let value = read_memory(&caller, &memory, value_ptr as usize, value_len as usize);
            
            caller.data_mut().storage_write(key, value);
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Emit log
        linker.func_wrap("env", "emit_log", |mut caller: Caller<'_, HostContext>, topics_ptr: i32, topics_count: i32, data_ptr: i32, data_len: i32| {
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export");
            
            let mut topics = Vec::new();
            for i in 0..topics_count {
                let topic_ptr = topics_ptr as usize + (i as usize * 32);
                let topic_data = read_memory(&caller, &memory, topic_ptr, 32);
                let mut topic = [0u8; 32];
                topic.copy_from_slice(&topic_data);
                topics.push(topic);
            }
            
            let data = read_memory(&caller, &memory, data_ptr as usize, data_len as usize);
            caller.data_mut().emit_log(topics, data);
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Get caller
        linker.func_wrap("env", "get_caller", |mut caller: Caller<'_, HostContext>, ptr: i32| {
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export");
            
            let caller_addr = caller.data().caller;
            let data = memory.data_mut(&mut caller);
            let slice = &mut data[ptr as usize..ptr as usize + 20];
            slice.copy_from_slice(&caller_addr.0);
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Get self address
        linker.func_wrap("env", "get_self", |mut caller: Caller<'_, HostContext>, ptr: i32| {
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export");
            
            let self_addr = caller.data().contract_address;
            let data = memory.data_mut(&mut caller);
            let slice = &mut data[ptr as usize..ptr as usize + 20];
            slice.copy_from_slice(&self_addr.0);
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Get value
        linker.func_wrap("env", "get_value", |caller: Caller<'_, HostContext>| -> i64 {
            caller.data().value as i64
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Get block height
        linker.func_wrap("env", "get_block_height", |caller: Caller<'_, HostContext>| -> i64 {
            caller.data().block_height as i64
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Get block timestamp
        linker.func_wrap("env", "get_block_timestamp", |caller: Caller<'_, HostContext>| -> i64 {
            caller.data().block_timestamp as i64
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // BLAKE3 hash
        linker.func_wrap("env", "blake3", |mut caller: Caller<'_, HostContext>, input_ptr: i32, input_len: i32, output_ptr: i32| {
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export");
            
            let input = read_memory(&caller, &memory, input_ptr as usize, input_len as usize);
            let hash = blake3::hash(&input);
            write_memory(&mut caller, &memory, output_ptr as usize, hash.as_bytes());
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Keccak256 hash
        linker.func_wrap("env", "keccak256", |mut caller: Caller<'_, HostContext>, input_ptr: i32, input_len: i32, output_ptr: i32| {
            use sha3::{Digest, Keccak256};
            
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export");
            
            let input = read_memory(&caller, &memory, input_ptr as usize, input_len as usize);
            let mut hasher = Keccak256::new();
            hasher.update(&input);
            let hash = hasher.finalize();
            write_memory(&mut caller, &memory, output_ptr as usize, &hash);
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Revert
        linker.func_wrap("env", "revert", |mut caller: Caller<'_, HostContext>, msg_ptr: i32, msg_len: i32| {
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export");
            
            let msg = read_memory(&caller, &memory, msg_ptr as usize, msg_len as usize);
            let reason = String::from_utf8_lossy(&msg).to_string();
            panic!("revert: {}", reason);
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        // Debug print
        linker.func_wrap("env", "debug_print", |mut caller: Caller<'_, HostContext>, msg_ptr: i32, msg_len: i32| {
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export");
            
            let msg = read_memory(&caller, &memory, msg_ptr as usize, msg_len as usize);
            let text = String::from_utf8_lossy(&msg);
            tracing::debug!(target: "wasm_contract", "{}", text);
        }).map_err(|e| ExecutionError::HostError(e.to_string()))?;

        Ok(())
    }

    /// Allocate memory for input data
    fn allocate_input(
        &self,
        store: &mut Store<HostContext>,
        instance: &Instance,
        data: &[u8],
    ) -> Result<i32, ExecutionError> {
        // Get memory first
        let memory = instance.get_memory(&mut *store, "memory")
            .ok_or_else(|| ExecutionError::ExecutionFailed("No memory".to_string()))?;

        // Try to call allocate function
        if let Ok(alloc) = instance.get_typed_func::<i32, i32>(&mut *store, "allocate") {
            let ptr = alloc.call(&mut *store, data.len() as i32)
                .map_err(|e| ExecutionError::ExecutionFailed(e.to_string()))?;
            
            // Write data to allocated memory
            memory.write(&mut *store, ptr as usize, data)
                .map_err(|_| ExecutionError::MemoryOutOfBounds)?;
            
            Ok(ptr)
        } else {
            // Fallback: write to beginning of memory
            memory.write(&mut *store, 0, data)
                .map_err(|_| ExecutionError::MemoryOutOfBounds)?;
            
            Ok(0)
        }
    }

    /// Read return data from memory
    fn read_return_data(
        &self,
        store: &Store<HostContext>,
        memory: &Memory,
        ret_ptr: i32,
    ) -> Result<Vec<u8>, ExecutionError> {
        if ret_ptr < 0 {
            return Ok(Vec::new());
        }

        // Read length prefix (4 bytes)
        let mut len_buf = [0u8; 4];
        memory.read(store, ret_ptr as usize, &mut len_buf)
            .map_err(|_| ExecutionError::MemoryOutOfBounds)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        if len == 0 {
            return Ok(Vec::new());
        }

        // Read data
        let mut data = vec![0u8; len];
        memory.read(store, ret_ptr as usize + 4, &mut data)
            .map_err(|_| ExecutionError::MemoryOutOfBounds)?;

        Ok(data)
    }

    /// Get compiled module for contract
    fn get_module(&self, address: &Address) -> Result<Module, ExecutionError> {
        // Check cache first
        if let Some(code_hash) = self.storage.get_code_hash(address) {
            if let Some(module) = self.module_cache.read().get(&code_hash) {
                return Ok(module.clone());
            }
        }

        // Load from storage and compile
        let code = self.storage.get_code(address)
            .ok_or_else(|| ExecutionError::ContractNotFound(*address))?;
        
        let module = self.compile(&code)?;
        
        // Cache it
        let code_hash = self.hash_code(&code);
        self.module_cache.write().insert(code_hash, module.clone());
        
        Ok(module)
    }

    /// Generate contract address
    fn generate_contract_address(&self, deployer: &Address, code: &[u8]) -> Address {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&deployer.0);
        hasher.update(code);
        hasher.update(&std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes());
        
        let hash = hasher.finalize();
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash.as_bytes()[12..32]);
        Address(addr)
    }

    /// Hash contract code
    fn hash_code(&self, code: &[u8]) -> H256 {
        let hash = blake3::hash(code);
        H256::from_slice(hash.as_bytes())
    }

    /// Get runtime statistics
    pub fn stats(&self) -> RuntimeStats {
        RuntimeStats {
            cached_modules: self.module_cache.read().len() as u64,
            config: self.config.clone(),
        }
    }
}

/// Runtime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    pub cached_modules: u64,
    pub config: WasmConfig,
}

/// Helper: Read from WASM memory
fn read_memory(caller: &Caller<'_, HostContext>, memory: &Memory, offset: usize, len: usize) -> Vec<u8> {
    let data = memory.data(caller);
    if offset + len <= data.len() {
        data[offset..offset + len].to_vec()
    } else {
        Vec::new()
    }
}

/// Helper: Write to WASM memory
fn write_memory(caller: &mut Caller<'_, HostContext>, memory: &Memory, offset: usize, data: &[u8]) {
    let mem_data = memory.data_mut(caller);
    if offset + data.len() <= mem_data.len() {
        mem_data[offset..offset + data.len()].copy_from_slice(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = WasmConfig::default();
        assert_eq!(config.max_code_size, MAX_CODE_SIZE);
        assert!(config.fuel_metering);
    }

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult::success(vec![1, 2, 3], 1000);
        assert!(result.success);
        assert_eq!(result.gas_used, 1000);
        assert_eq!(result.return_data, vec![1, 2, 3]);

        let result = ExecutionResult::revert("test error".to_string(), 500);
        assert!(!result.success);
        assert_eq!(result.revert_reason, Some("test error".to_string()));
    }
}
