//! Contract Management
//!
//! Contract deployment, instantiation, and lifecycle management

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::storage::ContractStorage;

/// Contract code wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCode {
    /// Raw WASM bytecode
    pub bytecode: Vec<u8>,
    /// Code hash
    pub hash: H256,
    /// ABI (JSON schema)
    pub abi: Option<String>,
    /// Source code hash (for verification)
    pub source_hash: Option<H256>,
    /// Compiler version
    pub compiler_version: Option<String>,
}

impl ContractCode {
    /// Create from bytecode
    pub fn from_bytecode(bytecode: Vec<u8>) -> Self {
        let hash = Self::compute_hash(&bytecode);
        Self {
            bytecode,
            hash,
            abi: None,
            source_hash: None,
            compiler_version: None,
        }
    }

    /// Create with full metadata
    pub fn new(
        bytecode: Vec<u8>,
        abi: Option<String>,
        source_hash: Option<H256>,
        compiler_version: Option<String>,
    ) -> Self {
        let hash = Self::compute_hash(&bytecode);
        Self {
            bytecode,
            hash,
            abi,
            source_hash,
            compiler_version,
        }
    }

    /// Compute code hash
    fn compute_hash(bytecode: &[u8]) -> H256 {
        H256::from_slice(blake3::hash(bytecode).as_bytes())
    }

    /// Verify WASM magic bytes
    pub fn is_valid_wasm(&self) -> bool {
        self.bytecode.len() >= 8 
            && self.bytecode[0..4] == [0x00, 0x61, 0x73, 0x6d] // \0asm
            && self.bytecode[4..8] == [0x01, 0x00, 0x00, 0x00] // version 1
    }

    /// Get bytecode size
    pub fn size(&self) -> usize {
        self.bytecode.len()
    }
}

/// Deployed contract instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Contract address
    pub address: Address,
    /// Code hash
    pub code_hash: H256,
    /// Deployer address
    pub deployer: Address,
    /// Deployment block
    pub deploy_block: u64,
    /// Deployment timestamp
    pub deploy_timestamp: u64,
    /// Contract balance
    pub balance: u64,
    /// Is initialized
    pub initialized: bool,
    /// Is paused
    pub paused: bool,
    /// Admin address (for upgrades)
    pub admin: Option<Address>,
}

impl Contract {
    /// Create new contract
    pub fn new(
        address: Address,
        code_hash: H256,
        deployer: Address,
        deploy_block: u64,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        Self {
            address,
            code_hash,
            deployer,
            deploy_block,
            deploy_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            balance: 0,
            initialized: false,
            paused: false,
            admin: Some(deployer),
        }
    }

    /// Mark as initialized
    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    /// Pause contract
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Unpause contract
    pub fn unpause(&mut self) {
        self.paused = false;
    }

    /// Update balance
    pub fn update_balance(&mut self, amount: u64, is_credit: bool) {
        if is_credit {
            self.balance = self.balance.saturating_add(amount);
        } else {
            self.balance = self.balance.saturating_sub(amount);
        }
    }

    /// Transfer admin
    pub fn transfer_admin(&mut self, new_admin: Address) {
        self.admin = Some(new_admin);
    }

    /// Renounce admin
    pub fn renounce_admin(&mut self) {
        self.admin = None;
    }
}

/// Contract instance for execution
pub struct ContractInstance {
    /// Contract metadata
    pub contract: Contract,
    /// Contract code
    pub code: ContractCode,
    /// Storage reference
    pub storage: Arc<ContractStorage>,
}

impl ContractInstance {
    /// Create new instance
    pub fn new(contract: Contract, code: ContractCode, storage: Arc<ContractStorage>) -> Self {
        Self {
            contract,
            code,
            storage,
        }
    }

    /// Read from storage
    pub fn storage_get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(&self.contract.address, key)
    }

    /// Write to storage
    pub fn storage_set(&self, key: &[u8], value: Vec<u8>) {
        self.storage.set(&self.contract.address, key, value);
    }

    /// Delete from storage
    pub fn storage_delete(&self, key: &[u8]) {
        self.storage.delete(&self.contract.address, key);
    }
}

/// Contract errors
#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("Contract not found: {0:?}")]
    NotFound(Address),

    #[error("Contract already exists: {0:?}")]
    AlreadyExists(Address),

    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(String),

    #[error("Contract paused")]
    Paused,

    #[error("Not authorized")]
    NotAuthorized,

    #[error("Not initialized")]
    NotInitialized,

    #[error("Already initialized")]
    AlreadyInitialized,

    #[error("Upgrade not allowed")]
    UpgradeNotAllowed,

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
}

/// Contract registry for managing deployed contracts
pub struct ContractRegistry {
    /// Contracts by address
    contracts: Arc<RwLock<HashMap<Address, Contract>>>,
    /// Code registry
    code_registry: Arc<RwLock<HashMap<H256, ContractCode>>>,
    /// Storage
    storage: Arc<ContractStorage>,
}

impl ContractRegistry {
    /// Create new registry
    pub fn new(storage: Arc<ContractStorage>) -> Self {
        Self {
            contracts: Arc::new(RwLock::new(HashMap::new())),
            code_registry: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }

    /// Register contract code
    pub fn register_code(&self, code: ContractCode) -> H256 {
        let hash = code.hash;
        self.code_registry.write().insert(hash, code);
        hash
    }

    /// Get code by hash
    pub fn get_code(&self, hash: &H256) -> Option<ContractCode> {
        self.code_registry.read().get(hash).cloned()
    }

    /// Deploy contract
    pub fn deploy(
        &self,
        address: Address,
        code_hash: H256,
        deployer: Address,
        block: u64,
    ) -> Result<Contract, ContractError> {
        // Check if contract already exists
        if self.contracts.read().contains_key(&address) {
            return Err(ContractError::AlreadyExists(address));
        }

        // Check if code exists
        if !self.code_registry.read().contains_key(&code_hash) {
            return Err(ContractError::InvalidBytecode("Code not registered".to_string()));
        }

        let contract = Contract::new(address, code_hash, deployer, block);
        self.contracts.write().insert(address, contract.clone());

        Ok(contract)
    }

    /// Get contract
    pub fn get_contract(&self, address: &Address) -> Option<Contract> {
        self.contracts.read().get(address).cloned()
    }

    /// Get contract instance
    pub fn get_instance(&self, address: &Address) -> Result<ContractInstance, ContractError> {
        let contract = self.contracts.read()
            .get(address)
            .cloned()
            .ok_or_else(|| ContractError::NotFound(*address))?;

        let code = self.code_registry.read()
            .get(&contract.code_hash)
            .cloned()
            .ok_or_else(|| ContractError::InvalidBytecode("Code not found".to_string()))?;

        Ok(ContractInstance::new(contract, code, self.storage.clone()))
    }

    /// Update contract
    pub fn update_contract(&self, address: &Address, f: impl FnOnce(&mut Contract)) -> Result<(), ContractError> {
        let mut contracts = self.contracts.write();
        let contract = contracts.get_mut(address)
            .ok_or_else(|| ContractError::NotFound(*address))?;
        f(contract);
        Ok(())
    }

    /// Upgrade contract code
    pub fn upgrade(
        &self,
        address: &Address,
        new_code_hash: H256,
        caller: &Address,
    ) -> Result<(), ContractError> {
        let mut contracts = self.contracts.write();
        let contract = contracts.get_mut(address)
            .ok_or_else(|| ContractError::NotFound(*address))?;

        // Check authorization
        if contract.admin != Some(*caller) {
            return Err(ContractError::NotAuthorized);
        }

        // Check code exists
        if !self.code_registry.read().contains_key(&new_code_hash) {
            return Err(ContractError::InvalidBytecode("New code not registered".to_string()));
        }

        contract.code_hash = new_code_hash;
        Ok(())
    }

    /// Remove contract
    pub fn remove(&self, address: &Address) -> Result<Contract, ContractError> {
        self.contracts.write()
            .remove(address)
            .ok_or_else(|| ContractError::NotFound(*address))
    }

    /// Get all contracts
    pub fn list_contracts(&self) -> Vec<Address> {
        self.contracts.read().keys().cloned().collect()
    }

    /// Get contracts by deployer
    pub fn get_by_deployer(&self, deployer: &Address) -> Vec<Contract> {
        self.contracts.read()
            .values()
            .filter(|c| &c.deployer == deployer)
            .cloned()
            .collect()
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let contracts = self.contracts.read();
        let code_registry = self.code_registry.read();

        RegistryStats {
            total_contracts: contracts.len() as u64,
            total_code_entries: code_registry.len() as u64,
            active_contracts: contracts.values().filter(|c| !c.paused).count() as u64,
            paused_contracts: contracts.values().filter(|c| c.paused).count() as u64,
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_contracts: u64,
    pub total_code_entries: u64,
    pub active_contracts: u64,
    pub paused_contracts: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_code() {
        let wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let code = ContractCode::from_bytecode(wasm.clone());
        
        assert!(code.is_valid_wasm());
        assert_eq!(code.size(), 8);
    }

    #[test]
    fn test_invalid_wasm() {
        let invalid = vec![0x01, 0x02, 0x03, 0x04];
        let code = ContractCode::from_bytecode(invalid);
        
        assert!(!code.is_valid_wasm());
    }

    #[test]
    fn test_contract_creation() {
        let address = Address([1u8; 20]);
        let code_hash = H256([2u8; 32]);
        let deployer = Address([3u8; 20]);
        
        let contract = Contract::new(address, code_hash, deployer, 100);
        
        assert_eq!(contract.address, address);
        assert_eq!(contract.deployer, deployer);
        assert!(!contract.initialized);
        assert!(!contract.paused);
    }

    #[test]
    fn test_contract_lifecycle() {
        let address = Address([1u8; 20]);
        let code_hash = H256([2u8; 32]);
        let deployer = Address([3u8; 20]);
        
        let mut contract = Contract::new(address, code_hash, deployer, 100);
        
        contract.initialize();
        assert!(contract.initialized);
        
        contract.pause();
        assert!(contract.paused);
        
        contract.unpause();
        assert!(!contract.paused);
    }

    #[test]
    fn test_contract_balance() {
        let mut contract = Contract::new(
            Address([1u8; 20]),
            H256([2u8; 32]),
            Address([3u8; 20]),
            100,
        );
        
        contract.update_balance(1000, true);
        assert_eq!(contract.balance, 1000);
        
        contract.update_balance(400, false);
        assert_eq!(contract.balance, 600);
    }

    #[test]
    fn test_contract_registry() {
        let storage = Arc::new(ContractStorage::new());
        let registry = ContractRegistry::new(storage);
        
        // Register code
        let code = ContractCode::from_bytecode(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
        let code_hash = registry.register_code(code);
        
        // Deploy contract
        let address = Address([1u8; 20]);
        let deployer = Address([2u8; 20]);
        let result = registry.deploy(address, code_hash, deployer, 100);
        
        assert!(result.is_ok());
        assert!(registry.get_contract(&address).is_some());
    }

    #[test]
    fn test_contract_upgrade() {
        let storage = Arc::new(ContractStorage::new());
        let registry = ContractRegistry::new(storage);
        
        // Register codes
        let code1 = ContractCode::from_bytecode(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01]);
        let code2 = ContractCode::from_bytecode(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x02]);
        let hash1 = registry.register_code(code1);
        let hash2 = registry.register_code(code2);
        
        // Deploy
        let address = Address([1u8; 20]);
        let deployer = Address([2u8; 20]);
        registry.deploy(address, hash1, deployer, 100).unwrap();
        
        // Upgrade
        let result = registry.upgrade(&address, hash2, &deployer);
        assert!(result.is_ok());
        
        let contract = registry.get_contract(&address).unwrap();
        assert_eq!(contract.code_hash, hash2);
    }
}
