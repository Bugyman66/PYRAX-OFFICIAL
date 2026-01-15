use std::path::PathBuf;
use std::process::Child;
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub node_running: bool,
    pub miner_running: bool,
    pub wallet_unlocked: bool,
    pub data_dir: PathBuf,
    pub network: Network,
    pub settings: Settings,
    pub node_process: Option<Child>,
    pub miner_process: Option<Child>,
    pub rpc_port: u16,
    // Wallet state
    pub wallet_mnemonic: Option<String>,
    pub wallet_addresses: Vec<WalletAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddress {
    pub address: String,
    pub index: u32,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
    Devnet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub auto_start_node: bool,
    pub auto_start_miner: bool,
    pub miner_address: Option<String>,
    pub miner_threads: u32,
    pub cuda_device: i32,
    pub opencl_device: i32,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub max_peers: u32,
    pub theme: Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl AppState {
    pub fn new() -> Self {
        let data_dir = directories::ProjectDirs::from("org", "pyrax", "PYRAX Desktop")
            .map(|d| d.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./pyrax-data"));
        
        Self {
            node_running: false,
            miner_running: false,
            wallet_unlocked: false,
            data_dir,
            network: Network::Devnet, // Devnet is active, Testnet not yet deployed
            settings: Settings::default(),
            node_process: None,
            miner_process: None,
            rpc_port: 28545,
            wallet_mnemonic: None,
            wallet_addresses: Vec::new(),
        }
    }
    
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            ..Self::new()
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_start_node: false,
            auto_start_miner: false,
            miner_address: None,
            miner_threads: 0, // Auto-detect
            cuda_device: 0,
            opencl_device: -1,
            rpc_port: 8545,
            p2p_port: 30303,
            max_peers: 50,
            theme: Theme::System,
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Network::Devnet // Devnet is active, Testnet not yet deployed
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Testnet => write!(f, "testnet"),
            Network::Devnet => write!(f, "devnet"),
        }
    }
}

/// Network endpoint configuration for all three streams
#[derive(Debug, Clone)]
pub struct NetworkEndpoints {
    /// Stream A - BLAKE3 PoW RPC
    pub stream_a_rpc: String,
    /// Stream B - KAWPOW GPU Stratum
    pub stream_b_stratum: String,
    /// Stream C - ZK Staking RPC
    pub stream_c_rpc: String,
    /// P2P bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    /// Chain ID
    pub chain_id: u32,
}

impl Network {
    /// Get network endpoints for connecting to all streams
    pub fn endpoints(&self) -> NetworkEndpoints {
        match self {
            Network::Mainnet => NetworkEndpoints {
                stream_a_rpc: "https://rpc.pyrax.org".to_string(),
                stream_b_stratum: "stratum+tcp://stratum.pyrax.org:3333".to_string(),
                stream_c_rpc: "https://staking.pyrax.org".to_string(),
                bootstrap_nodes: vec![
                    "/dns4/node1.pyrax.org/tcp/30303".to_string(),
                    "/dns4/node2.pyrax.org/tcp/30303".to_string(),
                ],
                chain_id: 797292,
            },
            Network::Testnet => NetworkEndpoints {
                stream_a_rpc: "https://rpc.pyrax-testnet.org".to_string(),
                stream_b_stratum: "stratum+tcp://stratum.pyrax-testnet.org:3333".to_string(),
                stream_c_rpc: "https://staking.pyrax-testnet.org".to_string(),
                bootstrap_nodes: vec![
                    "/dns4/testnet.pyrax.org/tcp/30303".to_string(),
                ],
                chain_id: 7972920,
            },
            Network::Devnet => NetworkEndpoints {
                stream_a_rpc: "http://209.38.137.105:28545".to_string(),
                stream_b_stratum: "stratum+tcp://209.38.137.105:3333".to_string(),
                stream_c_rpc: "http://209.38.137.105:28547".to_string(),
                bootstrap_nodes: vec![
                    "/ip4/209.38.137.105/tcp/30303".to_string(),
                ],
                chain_id: 79729200,
            },
        }
    }

    /// Get local RPC port for embedded node
    pub fn local_rpc_port(&self) -> u16 {
        match self {
            Network::Mainnet => 8545,
            Network::Testnet => 18545,
            Network::Devnet => 28545,
        }
    }

    /// Get local Stratum port for embedded miner
    pub fn local_stratum_port(&self) -> u16 {
        match self {
            Network::Mainnet => 3333,
            Network::Testnet => 13333,
            Network::Devnet => 23333,
        }
    }
}
