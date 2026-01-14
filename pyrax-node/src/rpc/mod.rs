//! PYRAX JSON-RPC Module
//!
//! Production-ready JSON-RPC server and client for wallet and explorer integration
//! Includes:
//! - Core chain RPC (blocks, transactions, balances)
//! - Mining RPC for stratum server control and GPU mining (Stream B)
//! - Staking RPC for ZK validation and checkpoints (Stream C)

mod server;
mod types;
pub mod client;
pub mod ai;
pub mod mining_rpc;
pub mod staking_rpc;

pub use server::{start_server, start_server_with_mempool, start_staking_server};
pub use types::*;
pub use client::{RpcClient, NetworkConfig, RpcClientError};
pub use ai::{AIRpcImpl, AIRpcServer};
pub use mining_rpc::{MiningRpc, DesktopMiningStatus, DesktopMiningCommand, DesktopMiningResponse, GpuDeviceInfo};
pub use staking_rpc::{StakingRpcImpl, StakingRpcServer};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Invalid params: {0}")]
    InvalidParams(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Server error: {0}")]
    ServerError(String),
}

impl From<RpcError> for jsonrpsee::types::ErrorObjectOwned {
    fn from(err: RpcError) -> Self {
        match err {
            RpcError::InternalError(msg) => {
                jsonrpsee::types::ErrorObject::owned(-32603, msg, None::<()>)
            }
            RpcError::InvalidParams(msg) => {
                jsonrpsee::types::ErrorObject::owned(-32602, msg, None::<()>)
            }
            RpcError::NotFound(msg) => {
                jsonrpsee::types::ErrorObject::owned(-32001, msg, None::<()>)
            }
            RpcError::ServerError(msg) => {
                jsonrpsee::types::ErrorObject::owned(-32000, msg, None::<()>)
            }
        }
    }
}
