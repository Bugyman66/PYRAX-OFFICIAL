//! Mainnet Infrastructure
//!
//! Production-ready mainnet components:
//! - Chaos testing framework
//! - Security audit tools
//! - Genesis configuration
//! - Network monitoring
//! - Upgrade mechanism

mod chaos;
mod security;
mod genesis_config;
mod monitoring;
mod upgrade;
mod health;

pub use chaos::{ChaosEngine, ChaosConfig, ChaosEvent, ChaosResult};
pub use security::{SecurityAuditor, AuditConfig, AuditResult, Vulnerability};
pub use genesis_config::{GenesisConfig, GenesisAccount, GenesisValidator, NetworkConfig};
pub use monitoring::{NetworkMonitor, MonitorConfig, NodeMetrics, NetworkStats};
pub use upgrade::{UpgradeManager, UpgradeConfig, UpgradeProposal, UpgradeStatus};
pub use health::{HealthChecker, HealthStatus, HealthReport, ComponentHealth};

/// Mainnet chain ID
pub const MAINNET_CHAIN_ID: u64 = 0x505952_01; // "PYR" + 01

/// Testnet chain ID
pub const TESTNET_CHAIN_ID: u64 = 0x505952_FF; // "PYR" + FF

/// Minimum nodes for mainnet launch
pub const MIN_MAINNET_NODES: u32 = 21;

/// Minimum validators for mainnet
pub const MIN_MAINNET_VALIDATORS: u32 = 7;

/// Genesis timestamp (placeholder - set at launch)
pub const GENESIS_TIMESTAMP: u64 = 0;

/// Block time target (seconds)
pub const BLOCK_TIME_SECS: u64 = 6;

/// Epoch length (blocks)
pub const EPOCH_LENGTH: u64 = 14400; // ~24 hours

/// Upgrade delay (blocks after approval)
pub const UPGRADE_DELAY_BLOCKS: u64 = 7200; // ~12 hours

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MAINNET_CHAIN_ID, 0x505952_01);
        assert_eq!(MIN_MAINNET_VALIDATORS, 7);
        assert_eq!(EPOCH_LENGTH, 14400);
    }
}
