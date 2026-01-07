use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Invalid configuration: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub datadir: PathBuf,
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub mempool: MempoolConfig,
    pub p2p: P2PConfig,
    pub rpc: RpcConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub chain_id: u32,
    pub genesis_hash: String,
    pub bootnodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub target_block_time: u64,
    pub difficulty_adjustment_window: u64,
    pub max_difficulty_change: f64,
    pub initial_difficulty: u64,
    pub block_reward: u64,
    pub halving_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolConfig {
    pub max_size: usize,
    pub max_tx_size: usize,
    pub min_gas_price: u64,
    pub eviction_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    pub listen_addr: String,
    pub max_peers: usize,
    pub connection_timeout: u64,
    pub ping_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub bind_addr: String,
    pub max_connections: usize,
    pub enable_debug: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            datadir: PathBuf::from("./data"),
            network: NetworkConfig::mainnet(),
            consensus: ConsensusConfig::default(),
            mempool: MempoolConfig::default(),
            p2p: P2PConfig::default(),
            rpc: RpcConfig::default(),
        }
    }
}

impl NetworkConfig {
    pub fn mainnet() -> Self {
        Self {
            chain_id: 1,
            genesis_hash: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            bootnodes: vec![],
        }
    }

    pub fn testnet() -> Self {
        Self {
            chain_id: 2,
            genesis_hash: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            bootnodes: vec![],
        }
    }

    pub fn devnet() -> Self {
        Self {
            chain_id: 3,
            genesis_hash: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            bootnodes: vec![],
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            target_block_time: 60,
            difficulty_adjustment_window: 720,
            max_difficulty_change: 0.25,
            initial_difficulty: 1_000_000,
            block_reward: 5000_000_000_000_000_000_000, // 5000 PYRAX in wei
            halving_interval: 2_100_000,
        }
    }
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10_000,
            max_tx_size: 128 * 1024,
            min_gas_price: 1_000_000_000,
            eviction_interval: 300,
        }
    }
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/30303".into(),
            max_peers: 50,
            connection_timeout: 30,
            ping_interval: 15,
        }
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8545".into(),
            max_connections: 100,
            enable_debug: false,
        }
    }
}

impl NodeConfig {
    pub fn load(path: &PathBuf, args: &super::Args) -> Result<Self, ConfigError> {
        let mut config = if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            toml::from_str(&contents)?
        } else {
            Self::default()
        };

        // Override with CLI args
        if let Some(datadir) = &args.datadir {
            config.datadir = datadir.clone();
        }

        config.network = match args.network.as_str() {
            "mainnet" => NetworkConfig::mainnet(),
            "testnet" => NetworkConfig::testnet(),
            "devnet" => NetworkConfig::devnet(),
            _ => return Err(ConfigError::ValidationError(
                format!("Unknown network: {}", args.network)
            )),
        };

        config.rpc.bind_addr = args.rpc_addr.clone();
        config.p2p.listen_addr = args.p2p_addr.clone();

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.consensus.target_block_time == 0 {
            return Err(ConfigError::ValidationError(
                "target_block_time must be > 0".into()
            ));
        }
        if self.mempool.max_size == 0 {
            return Err(ConfigError::ValidationError(
                "mempool.max_size must be > 0".into()
            ));
        }
        Ok(())
    }
}
