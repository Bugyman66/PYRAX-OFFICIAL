//! WASM Smart Contract Runtime
//!
//! Provides execution environment for Rust/WASM smart contracts on PYRAX.
//!
//! Key features:
//! - Wasmtime-based execution engine
//! - Gas metering and limits
//! - Host functions for storage, crypto, and events
//! - EVM ↔ WASM interoperability
//! - Contract deployment and invocation

#![allow(dead_code)]

mod runtime;
mod host;
mod metering;
mod storage;
mod contract;
mod interop;

pub use runtime::{WasmRuntime, WasmConfig, ExecutionResult, ExecutionError};
pub use host::{HostFunctions, HostContext};
pub use metering::{GasMeter, GasConfig, GasCost};
pub use storage::{ContractStorage, StorageKey, StorageValue};
pub use contract::{Contract, ContractCode, ContractInstance, ContractError};
pub use interop::{EvmWasmBridge, CrossVmCall, CrossVmResult};

/// Maximum contract code size (1 MB)
pub const MAX_CODE_SIZE: usize = 1024 * 1024;

/// Maximum memory pages (256 = 16 MB)
pub const MAX_MEMORY_PAGES: u32 = 256;

/// Default gas limit per call
pub const DEFAULT_GAS_LIMIT: u64 = 10_000_000;

/// Minimum gas for contract deployment
pub const MIN_DEPLOY_GAS: u64 = 100_000;

/// Contract call timeout (seconds)
pub const CALL_TIMEOUT_SECS: u64 = 30;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MAX_CODE_SIZE, 1024 * 1024);
        assert_eq!(MAX_MEMORY_PAGES, 256);
        assert!(DEFAULT_GAS_LIMIT > MIN_DEPLOY_GAS);
    }
}
