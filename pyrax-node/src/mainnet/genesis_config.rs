//! Genesis Configuration
//!
//! Production-grade genesis block and network configuration

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::{MAINNET_CHAIN_ID, TESTNET_CHAIN_ID, BLOCK_TIME_SECS, EPOCH_LENGTH};

/// Network type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    /// Mainnet production network
    Mainnet,
    /// Public testnet
    Testnet,
    /// Private development network
    Devnet,
    /// Local development
    Local,
}

impl NetworkType {
    /// Get chain ID for network type
    pub fn chain_id(&self) -> u64 {
        match self {
            NetworkType::Mainnet => MAINNET_CHAIN_ID,
            NetworkType::Testnet => TESTNET_CHAIN_ID,
            NetworkType::Devnet => 0x505952_FE,
            NetworkType::Local => 0x505952_00,
        }
    }

    /// Get network name
    pub fn name(&self) -> &'static str {
        match self {
            NetworkType::Mainnet => "PYRAX Mainnet",
            NetworkType::Testnet => "PYRAX Testnet",
            NetworkType::Devnet => "PYRAX Devnet",
            NetworkType::Local => "PYRAX Local",
        }
    }
}

/// Genesis account with initial balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAccount {
    /// Account address
    pub address: Address,
    /// Initial balance (smallest unit)
    pub balance: u64,
    /// Account nonce
    pub nonce: u64,
    /// Contract code (if any)
    pub code: Option<Vec<u8>>,
    /// Storage (if contract)
    pub storage: HashMap<H256, H256>,
    /// Account label (for documentation)
    pub label: String,
}

impl GenesisAccount {
    /// Create new genesis account
    pub fn new(address: Address, balance: u64, label: String) -> Self {
        Self {
            address,
            balance,
            nonce: 0,
            code: None,
            storage: HashMap::new(),
            label,
        }
    }

    /// Create contract account
    pub fn contract(address: Address, code: Vec<u8>, label: String) -> Self {
        Self {
            address,
            balance: 0,
            nonce: 1,
            code: Some(code),
            storage: HashMap::new(),
            label,
        }
    }

    /// Set storage
    pub fn with_storage(mut self, storage: HashMap<H256, H256>) -> Self {
        self.storage = storage;
        self
    }
}

/// Genesis validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisValidator {
    /// Validator address
    pub address: Address,
    /// Initial stake
    pub stake: u64,
    /// Commission rate (basis points, 100 = 1%)
    pub commission_bps: u16,
    /// Validator name
    pub name: String,
    /// Validator description
    pub description: String,
    /// Website URL
    pub website: String,
    /// Public key for consensus
    pub consensus_pubkey: Vec<u8>,
    /// Network endpoint
    pub endpoint: String,
}

impl GenesisValidator {
    /// Create new genesis validator
    pub fn new(
        address: Address,
        stake: u64,
        name: String,
        consensus_pubkey: Vec<u8>,
        endpoint: String,
    ) -> Self {
        Self {
            address,
            stake,
            commission_bps: 500, // 5% default
            name,
            description: String::new(),
            website: String::new(),
            consensus_pubkey,
            endpoint,
        }
    }

    /// Set commission
    pub fn with_commission(mut self, bps: u16) -> Self {
        self.commission_bps = bps;
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
}

/// Network configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Chain ID
    pub chain_id: u64,
    /// Network type
    pub network_type: NetworkType,
    /// Block time target (seconds)
    pub block_time_secs: u64,
    /// Epoch length (blocks)
    pub epoch_length: u64,
    /// Maximum validators
    pub max_validators: u32,
    /// Minimum stake to become validator
    pub min_validator_stake: u64,
    /// Unbonding period (epochs)
    pub unbonding_period_epochs: u64,
    /// Slashing rate for double-sign (basis points)
    pub double_sign_slash_bps: u16,
    /// Slashing rate for downtime (basis points)
    pub downtime_slash_bps: u16,
    /// Block reward (smallest unit)
    pub block_reward: u64,
    /// Transaction fee minimum
    pub min_tx_fee: u64,
    /// Gas price minimum
    pub min_gas_price: u64,
    /// Maximum block gas
    pub max_block_gas: u64,
    /// Enable EVM
    pub evm_enabled: bool,
    /// Enable WASM
    pub wasm_enabled: bool,
    /// Enable AI compute
    pub ai_enabled: bool,
    /// Enable ZK rollups
    pub zk_enabled: bool,
}

impl NetworkConfig {
    /// Create mainnet config
    pub fn mainnet() -> Self {
        Self {
            chain_id: MAINNET_CHAIN_ID,
            network_type: NetworkType::Mainnet,
            block_time_secs: BLOCK_TIME_SECS,
            epoch_length: EPOCH_LENGTH,
            max_validators: 100,
            min_validator_stake: 100_000 * 100_000_000, // 100,000 PYRAX
            unbonding_period_epochs: 21, // ~21 days
            double_sign_slash_bps: 500, // 5%
            downtime_slash_bps: 10, // 0.1%
            block_reward: 2 * 100_000_000, // 2 PYRAX
            min_tx_fee: 21_000, // Minimum gas
            min_gas_price: 1_000_000_000, // 1 cinder
            max_block_gas: 30_000_000,
            evm_enabled: true,
            wasm_enabled: true,
            ai_enabled: true,
            zk_enabled: true,
        }
    }

    /// Create testnet config
    pub fn testnet() -> Self {
        Self {
            chain_id: TESTNET_CHAIN_ID,
            network_type: NetworkType::Testnet,
            block_time_secs: 3, // Faster for testing
            epoch_length: 1000, // Shorter epochs
            max_validators: 50,
            min_validator_stake: 1_000 * 100_000_000, // 1,000 PYRAX
            unbonding_period_epochs: 3, // ~3 days
            double_sign_slash_bps: 500,
            downtime_slash_bps: 10,
            block_reward: 10 * 100_000_000, // 10 PYRAX (more for testing)
            min_tx_fee: 21_000,
            min_gas_price: 1_000_000, // Lower for testing
            max_block_gas: 50_000_000,
            evm_enabled: true,
            wasm_enabled: true,
            ai_enabled: true,
            zk_enabled: true,
        }
    }

    /// Create devnet config
    pub fn devnet() -> Self {
        Self {
            chain_id: NetworkType::Devnet.chain_id(),
            network_type: NetworkType::Devnet,
            block_time_secs: 1,
            epoch_length: 100,
            max_validators: 21,
            min_validator_stake: 100 * 100_000_000,
            unbonding_period_epochs: 1,
            double_sign_slash_bps: 100,
            downtime_slash_bps: 1,
            block_reward: 100 * 100_000_000,
            min_tx_fee: 1000,
            min_gas_price: 1,
            max_block_gas: 100_000_000,
            evm_enabled: true,
            wasm_enabled: true,
            ai_enabled: true,
            zk_enabled: true,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.block_time_secs == 0 {
            return Err(ConfigError::InvalidValue("block_time_secs must be > 0".into()));
        }
        if self.epoch_length == 0 {
            return Err(ConfigError::InvalidValue("epoch_length must be > 0".into()));
        }
        if self.max_validators == 0 {
            return Err(ConfigError::InvalidValue("max_validators must be > 0".into()));
        }
        if self.double_sign_slash_bps > 10000 {
            return Err(ConfigError::InvalidValue("double_sign_slash_bps must be <= 10000".into()));
        }
        Ok(())
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::mainnet()
    }
}

/// Full genesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Genesis timestamp
    pub timestamp: u64,
    /// Network configuration
    pub network: NetworkConfig,
    /// Genesis accounts
    pub accounts: Vec<GenesisAccount>,
    /// Genesis validators
    pub validators: Vec<GenesisValidator>,
    /// System contracts
    pub system_contracts: Vec<SystemContract>,
    /// Genesis state root
    pub state_root: H256,
    /// Genesis block hash
    pub genesis_hash: H256,
    /// Extra data
    pub extra_data: Vec<u8>,
    /// Version
    pub version: String,
}

impl GenesisConfig {
    /// Create new genesis config
    pub fn new(network: NetworkConfig, timestamp: u64) -> Self {
        Self {
            timestamp,
            network,
            accounts: Vec::new(),
            validators: Vec::new(),
            system_contracts: Self::default_system_contracts(),
            state_root: H256::zero(),
            genesis_hash: H256::zero(),
            extra_data: b"PYRAX Genesis".to_vec(),
            version: "1.0.0".to_string(),
        }
    }

    /// Default system contracts
    fn default_system_contracts() -> Vec<SystemContract> {
        vec![
            SystemContract {
                name: "Staking".to_string(),
                address: Address([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
                code_hash: H256::zero(), // Set during deployment
                description: "Staking and validator management".to_string(),
            },
            SystemContract {
                name: "Governance".to_string(),
                address: Address([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]),
                code_hash: H256::zero(),
                description: "On-chain governance".to_string(),
            },
            SystemContract {
                name: "Bridge".to_string(),
                address: Address([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]),
                code_hash: H256::zero(),
                description: "L2 bridge contract".to_string(),
            },
            SystemContract {
                name: "AIMarketplace".to_string(),
                address: Address([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]),
                code_hash: H256::zero(),
                description: "AI compute marketplace".to_string(),
            },
        ]
    }

    /// Add genesis account
    pub fn add_account(&mut self, account: GenesisAccount) {
        self.accounts.push(account);
    }

    /// Add genesis validator
    pub fn add_validator(&mut self, validator: GenesisValidator) {
        self.validators.push(validator);
    }

    /// Calculate total supply
    pub fn total_supply(&self) -> u64 {
        self.accounts.iter().map(|a| a.balance).sum()
    }

    /// Calculate total staked
    pub fn total_staked(&self) -> u64 {
        self.validators.iter().map(|v| v.stake).sum()
    }

    /// Compute genesis hash
    pub fn compute_hash(&mut self) {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.network.chain_id.to_le_bytes());
        hasher.update(&self.state_root.0);
        hasher.update(&self.extra_data);
        
        for account in &self.accounts {
            hasher.update(&account.address.0);
            hasher.update(&account.balance.to_le_bytes());
        }
        
        for validator in &self.validators {
            hasher.update(&validator.address.0);
            hasher.update(&validator.stake.to_le_bytes());
        }

        self.genesis_hash = H256::from_slice(hasher.finalize().as_bytes());
    }

    /// Compute state root
    pub fn compute_state_root(&mut self) {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        
        for account in &self.accounts {
            hasher.update(&account.address.0);
            hasher.update(&account.balance.to_le_bytes());
            hasher.update(&account.nonce.to_le_bytes());
        }

        self.state_root = H256::from_slice(hasher.finalize().as_bytes());
    }

    /// Validate genesis config
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate network config
        self.network.validate()?;

        // Must have at least one validator
        if self.validators.is_empty() {
            return Err(ConfigError::NoValidators);
        }

        // Check validator stakes
        for validator in &self.validators {
            if validator.stake < self.network.min_validator_stake {
                return Err(ConfigError::InsufficientStake {
                    address: validator.address,
                    required: self.network.min_validator_stake,
                    actual: validator.stake,
                });
            }
        }

        // Check for duplicate addresses
        let mut seen_addresses = std::collections::HashSet::new();
        for account in &self.accounts {
            if !seen_addresses.insert(account.address) {
                return Err(ConfigError::DuplicateAddress(account.address));
            }
        }

        Ok(())
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))
    }

    /// Import from JSON
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))
    }

    /// Create mainnet genesis with proper tokenomics
    /// Total Supply: 100,000,000,000 PYRAX (100 Billion)
    pub fn mainnet_genesis() -> Self {
        let mut config = Self::new(NetworkConfig::mainnet(), 0);
        let decimals: u64 = 100_000_000; // 8 decimals
        
        // ============================================
        // PYRAX TOKENOMICS - Genesis Allocations
        // ============================================
        
        // Presale Pool: 15% (15,000,000,000 PYRAX) - 100% at TGE
        config.add_account(GenesisAccount::new(
            Address([0x01; 20]),
            15_000_000_000 * decimals,
            "Presale Pool".to_string(),
        ));

        // BDAG Community Pool: 10% (10,000,000,000 PYRAX) - 12mo cliff, 12mo vesting
        config.add_account(GenesisAccount::new(
            Address([0x02; 20]),
            10_000_000_000 * decimals,
            "BDAG Community Pool".to_string(),
        ));

        // Mining Rewards Pool: 35% (35,000,000,000 PYRAX) - Per block emission
        config.add_account(GenesisAccount::new(
            Address([0x03; 20]),
            35_000_000_000 * decimals,
            "Mining Rewards Pool (Stream A + B)".to_string(),
        ));

        // ZK Prover Rewards Pool: 5% (5,000,000,000 PYRAX) - Per attestation, 10% burned
        config.add_account(GenesisAccount::new(
            Address([0x04; 20]),
            5_000_000_000 * decimals,
            "ZK Prover Rewards Pool (Stream C)".to_string(),
        ));

        // Team & Founders Pool: 4% (4,000,000,000 PYRAX) - 12mo cliff, 48mo vesting
        config.add_account(GenesisAccount::new(
            Address([0x05; 20]),
            4_000_000_000 * decimals,
            "Team & Founders Pool".to_string(),
        ));

        // Advisors Pool: 3% (3,000,000,000 PYRAX) - 12mo cliff, 48mo vesting
        config.add_account(GenesisAccount::new(
            Address([0x06; 20]),
            3_000_000_000 * decimals,
            "Advisors Pool".to_string(),
        ));

        // Ecosystem Pool: 10% (10,000,000,000 PYRAX) - Milestone-based, DAO approval
        config.add_account(GenesisAccount::new(
            Address([0x07; 20]),
            10_000_000_000 * decimals,
            "Ecosystem Pool".to_string(),
        ));

        // Marketing Pool: 5% (5,000,000,000 PYRAX) - Multi-sig approval
        config.add_account(GenesisAccount::new(
            Address([0x08; 20]),
            5_000_000_000 * decimals,
            "Marketing Pool".to_string(),
        ));

        // Liquidity Pool: 10% (10,000,000,000 PYRAX) - 100% at TGE
        // CEX (60%), DEX (30%), Market Making (10%)
        config.add_account(GenesisAccount::new(
            Address([0x09; 20]),
            10_000_000_000 * decimals,
            "Liquidity Pool".to_string(),
        ));

        // Treasury Pool: 2% (2,000,000,000 PYRAX) - 75% DAO approval required
        config.add_account(GenesisAccount::new(
            Address([0x0A; 20]),
            2_000_000_000 * decimals,
            "Treasury Pool".to_string(),
        ));

        // Reserve Pool: 1% (1,000,000,000 PYRAX) - Protocol buffer
        config.add_account(GenesisAccount::new(
            Address([0x0B; 20]),
            1_000_000_000 * decimals,
            "Reserve Pool".to_string(),
        ));

        config.compute_state_root();
        config.compute_hash();

        config
    }

    /// Create testnet genesis
    pub fn testnet_genesis() -> Self {
        let mut config = Self::new(NetworkConfig::testnet(), 0);
        let decimals: u64 = 100_000_000;
        
        // Faucet with 10B test tokens
        config.add_account(GenesisAccount::new(
            Address([0xFF; 20]),
            10_000_000_000 * decimals,
            "Testnet Faucet".to_string(),
        ));

        // Test pools with same structure as mainnet but smaller amounts
        config.add_account(GenesisAccount::new(
            Address([0x01; 20]),
            1_000_000_000 * decimals,
            "Test Presale Pool".to_string(),
        ));

        config.add_account(GenesisAccount::new(
            Address([0x03; 20]),
            5_000_000_000 * decimals,
            "Test Mining Rewards Pool".to_string(),
        ));

        config.compute_state_root();
        config.compute_hash();

        config
    }
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self::mainnet_genesis()
    }
}

/// System contract definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContract {
    /// Contract name
    pub name: String,
    /// Contract address
    pub address: Address,
    /// Code hash
    pub code_hash: H256,
    /// Description
    pub description: String,
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("No validators in genesis")]
    NoValidators,

    #[error("Insufficient stake for validator {address:?}: required {required}, actual {actual}")]
    InsufficientStake {
        address: Address,
        required: u64,
        actual: u64,
    },

    #[error("Duplicate address: {0:?}")]
    DuplicateAddress(Address),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_type() {
        assert_eq!(NetworkType::Mainnet.chain_id(), MAINNET_CHAIN_ID);
        assert_eq!(NetworkType::Testnet.chain_id(), TESTNET_CHAIN_ID);
    }

    #[test]
    fn test_network_config() {
        let config = NetworkConfig::mainnet();
        assert!(config.validate().is_ok());
        assert_eq!(config.chain_id, MAINNET_CHAIN_ID);
    }

    #[test]
    fn test_genesis_account() {
        let account = GenesisAccount::new(
            Address([1u8; 20]),
            1000,
            "Test".to_string(),
        );
        assert_eq!(account.balance, 1000);
        assert!(account.code.is_none());
    }

    #[test]
    fn test_genesis_validator() {
        let validator = GenesisValidator::new(
            Address([1u8; 20]),
            100_000 * 100_000_000,
            "Test Validator".to_string(),
            vec![1, 2, 3],
            "https://validator.test".to_string(),
        );
        assert_eq!(validator.commission_bps, 500);
    }

    #[test]
    fn test_genesis_config() {
        let config = GenesisConfig::mainnet_genesis();
        assert!(!config.accounts.is_empty());
        assert_ne!(config.genesis_hash, H256::zero());
    }

    #[test]
    fn test_genesis_json() {
        let config = GenesisConfig::testnet_genesis();
        let json = config.to_json().unwrap();
        let parsed = GenesisConfig::from_json(&json).unwrap();
        assert_eq!(parsed.network.chain_id, config.network.chain_id);
    }
}
