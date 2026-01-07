//! Dual State Bridge
//!
//! Provides seamless conversion between UTXO model (Stream A) and Account model (EVM).
//!
//! Key features:
//! - Lock UTXO → Mint EVM tokens
//! - Burn EVM tokens → Unlock UTXO
//! - Unified address format
//! - Atomic cross-model transactions
//! - Gas fee distribution across streams

mod dual_state;
mod unified_address;
mod fee_distribution;
mod cross_model_tx;

pub use dual_state::{
    DualStateBridge, BridgeState, ConversionRequest, ConversionStatus,
    ConversionDirection, BridgeError, LockedUtxo, MintedBalance,
};
pub use unified_address::{UnifiedAddress, AddressType, AddressError};
pub use fee_distribution::{FeeDistributor, FeeAllocation, StreamShare};
pub use cross_model_tx::{
    CrossModelTransaction, CrossModelTxType, CrossModelTxStatus,
    AtomicSwap, SwapState,
};

/// Bridge configuration
pub const UTXO_LOCK_CONFIRMATIONS: u64 = 6;
pub const EVM_BURN_CONFIRMATIONS: u64 = 12;
pub const BRIDGE_FEE_BASIS_POINTS: u64 = 10; // 0.1%
pub const MIN_BRIDGE_AMOUNT: u64 = 1_000_000; // 0.01 PYRAX
pub const MAX_BRIDGE_AMOUNT: u64 = 100_000_000_000_000; // 1M PYRAX

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(UTXO_LOCK_CONFIRMATIONS, 6);
        assert_eq!(BRIDGE_FEE_BASIS_POINTS, 10);
    }
}
