use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfernoConfig {
    pub node: NodeConfig,
    pub network: NetworkConfig,
    pub mining: MiningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub instance_id: u32,
    pub data_dir: PathBuf,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub network: String,
    pub p2p_port: u16,
    pub rpc_port: u16,
    pub rpc_enabled: bool,
    pub bootnodes: Vec<String>,
    pub max_peers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    pub enabled: bool,
    pub threads: u32,
    pub wallet: String,
}

impl InfernoConfig {
    pub fn default_for_instance(instance_id: u32) -> Self {
        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".inferno")
            .join("devnet")
            .join(format!("instance-{}", instance_id));

        let p2p_port = 30303 + ((instance_id - 1) * 10) as u16;
        let rpc_port = 28545 + ((instance_id - 1) * 10) as u16;

        Self {
            node: NodeConfig {
                instance_id,
                data_dir,
                log_level: "info".to_string(),
            },
            network: NetworkConfig {
                network: "devnet".to_string(),
                p2p_port,
                rpc_port,
                rpc_enabled: true,
                bootnodes: vec![
                    "/ip4/209.38.137.105/tcp/30303/p2p/12D3KooWQGzw3hMqiL7bNDzRBfYKBjZ5vBrx9oQKGEMG4iczDH4W".to_string(),
                    "/ip4/137.184.118.228/tcp/30303/p2p/12D3KooWQGzw3hMqiL7bNDzRBfYKBjZ5vBrx9oQKGEMG4iczDH4W".to_string(),
                ],
                max_peers: 50,
            },
            mining: MiningConfig {
                enabled: false,
                threads: num_cpus(),
                wallet: String::new(),
            },
        }
    }
}

impl Default for InfernoConfig {
    fn default() -> Self {
        Self::default_for_instance(1)
    }
}

fn num_cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|p| p.get() as u32 / 2)
        .unwrap_or(2)
}
