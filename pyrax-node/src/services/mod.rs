//! Production Services
//!
//! Post-launch operational services:
//! - Faucet service for testnet
//! - Block explorer backend
//! - Metrics dashboard
//! - API gateway
//! - CLI tools
//! - Mining service with stratum server

mod faucet;
mod explorer;
mod metrics;
mod api;
mod cli;
pub mod mining;

pub use faucet::{FaucetService, FaucetConfig, FaucetRequest, FaucetResponse};
pub use explorer::{ExplorerService, ExplorerConfig, BlockInfo, TransactionInfo, AddressInfo};
pub use metrics::{MetricsService, MetricsConfig, ChainMetrics, NetworkMetrics};
pub use api::{ApiGateway, ApiConfig, ApiResponse, RateLimiter};
pub use cli::{CliCommand, CliConfig, CliRunner};
pub use mining::{MiningService, MiningServiceConfig, MiningInfo, ChainStateProvider};

/// Default faucet amount (testnet only)
pub const DEFAULT_FAUCET_AMOUNT: u64 = 100 * 100_000_000; // 100 PYRAX

/// Faucet cooldown period (seconds)
pub const FAUCET_COOLDOWN_SECS: u64 = 86400; // 24 hours

/// Maximum API requests per minute
pub const MAX_REQUESTS_PER_MINUTE: u32 = 60;

/// Explorer page size
pub const EXPLORER_PAGE_SIZE: u32 = 25;

/// Metrics collection interval (seconds)
pub const METRICS_INTERVAL_SECS: u64 = 15;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_FAUCET_AMOUNT, 100 * 100_000_000);
        assert_eq!(FAUCET_COOLDOWN_SECS, 86400);
        assert_eq!(MAX_REQUESTS_PER_MINUTE, 60);
    }
}
