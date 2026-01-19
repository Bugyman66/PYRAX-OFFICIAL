//! Faucet Service
//!
//! Production testnet faucet with rate limiting and anti-abuse

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::net::IpAddr;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::{DEFAULT_FAUCET_AMOUNT, FAUCET_COOLDOWN_SECS};

/// Faucet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaucetConfig {
    /// Faucet enabled
    pub enabled: bool,
    /// Amount to dispense per request
    pub amount: u64,
    /// Cooldown period between requests (seconds)
    pub cooldown_secs: u64,
    /// Maximum requests per IP per day
    pub max_requests_per_ip: u32,
    /// Require captcha verification
    pub require_captcha: bool,
    /// Captcha site key
    pub captcha_site_key: Option<String>,
    /// Captcha secret key
    pub captcha_secret_key: Option<String>,
    /// Faucet wallet address
    pub faucet_address: Address,
    /// Minimum balance to maintain
    pub min_balance: u64,
    /// Alert threshold
    pub alert_threshold: u64,
}

impl Default for FaucetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            amount: DEFAULT_FAUCET_AMOUNT,
            cooldown_secs: FAUCET_COOLDOWN_SECS,
            max_requests_per_ip: 3,
            require_captcha: true,
            captcha_site_key: None,
            captcha_secret_key: None,
            faucet_address: Address([0xFF; 20]),
            min_balance: 1_000_000 * 100_000_000, // 1M PYRAX
            alert_threshold: 10_000_000 * 100_000_000, // 10M PYRAX
        }
    }
}

/// Faucet request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaucetRequest {
    /// Recipient address
    pub address: Address,
    /// Requester IP
    pub ip_address: String,
    /// Captcha response (if required)
    pub captcha_response: Option<String>,
    /// Request timestamp
    pub timestamp: u64,
    /// User agent
    pub user_agent: Option<String>,
}

impl FaucetRequest {
    /// Create new faucet request
    pub fn new(address: Address, ip_address: String) -> Self {
        Self {
            address,
            ip_address,
            captcha_response: None,
            timestamp: current_timestamp(),
            user_agent: None,
        }
    }

    /// With captcha response
    pub fn with_captcha(mut self, response: String) -> Self {
        self.captcha_response = Some(response);
        self
    }
}

/// Faucet response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaucetResponse {
    /// Success status
    pub success: bool,
    /// Transaction hash (if successful)
    pub tx_hash: Option<H256>,
    /// Amount sent
    pub amount: u64,
    /// Message
    pub message: String,
    /// Next eligible time
    pub next_eligible: Option<u64>,
    /// Remaining requests for IP
    pub remaining_requests: u32,
}

impl FaucetResponse {
    /// Create success response
    pub fn success(tx_hash: H256, amount: u64) -> Self {
        Self {
            success: true,
            tx_hash: Some(tx_hash),
            amount,
            message: format!("Successfully sent {} PYRAX", amount / 100_000_000),
            next_eligible: Some(current_timestamp() + FAUCET_COOLDOWN_SECS),
            remaining_requests: 0,
        }
    }

    /// Create error response
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            tx_hash: None,
            amount: 0,
            message,
            next_eligible: None,
            remaining_requests: 0,
        }
    }

    /// Create cooldown response
    pub fn cooldown(next_eligible: u64) -> Self {
        let wait_secs = next_eligible.saturating_sub(current_timestamp());
        Self {
            success: false,
            tx_hash: None,
            amount: 0,
            message: format!("Please wait {} seconds before requesting again", wait_secs),
            next_eligible: Some(next_eligible),
            remaining_requests: 0,
        }
    }
}

/// Rate limit entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Last request times
    requests: Vec<u64>,
    /// Total requests today
    daily_count: u32,
    /// Day start timestamp
    day_start: u64,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            daily_count: 0,
            day_start: current_day_start(),
        }
    }

    fn add_request(&mut self) {
        let now = current_timestamp();
        
        // Reset if new day
        if now >= self.day_start + 86400 {
            self.requests.clear();
            self.daily_count = 0;
            self.day_start = current_day_start();
        }

        self.requests.push(now);
        self.daily_count += 1;
    }

    fn last_request_time(&self) -> Option<u64> {
        self.requests.last().copied()
    }

    fn requests_today(&self) -> u32 {
        let now = current_timestamp();
        if now >= self.day_start + 86400 {
            0
        } else {
            self.daily_count
        }
    }
}

/// Faucet service
pub struct FaucetService {
    /// Configuration
    config: FaucetConfig,
    /// Rate limits by address
    address_limits: Arc<RwLock<HashMap<Address, RateLimitEntry>>>,
    /// Rate limits by IP
    ip_limits: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    /// Transaction queue
    tx_queue: Arc<RwLock<Vec<PendingFaucetTx>>>,
    /// Faucet balance
    balance: Arc<RwLock<u64>>,
    /// Statistics
    stats: Arc<RwLock<FaucetStats>>,
}

/// Pending faucet transaction
#[derive(Debug, Clone)]
struct PendingFaucetTx {
    address: Address,
    amount: u64,
    tx_hash: H256,
    created_at: u64,
    confirmed: bool,
}

impl FaucetService {
    /// Create new faucet service
    pub fn new(config: FaucetConfig) -> Self {
        Self {
            config,
            address_limits: Arc::new(RwLock::new(HashMap::new())),
            ip_limits: Arc::new(RwLock::new(HashMap::new())),
            tx_queue: Arc::new(RwLock::new(Vec::new())),
            balance: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(FaucetStats::default())),
        }
    }

    /// Set faucet balance
    pub fn set_balance(&self, balance: u64) {
        *self.balance.write() = balance;
    }

    /// Get faucet balance
    pub fn get_balance(&self) -> u64 {
        *self.balance.read()
    }

    /// Process faucet request
    pub fn request(&self, req: FaucetRequest) -> FaucetResponse {
        // Check if enabled
        if !self.config.enabled {
            return FaucetResponse::error("Faucet is currently disabled".into());
        }

        // Check balance
        let balance = self.get_balance();
        if balance < self.config.amount {
            return FaucetResponse::error("Faucet is empty, please try later".into());
        }

        // Check captcha if required
        if self.config.require_captcha {
            if req.captcha_response.is_none() {
                return FaucetResponse::error("Captcha verification required".into());
            }
            if !self.verify_captcha(req.captcha_response.as_ref().unwrap()) {
                return FaucetResponse::error("Captcha verification failed".into());
            }
        }

        // Check address rate limit
        if let Some(response) = self.check_address_limit(&req.address) {
            return response;
        }

        // Check IP rate limit
        if let Some(response) = self.check_ip_limit(&req.ip_address) {
            return response;
        }

        // Process the request
        match self.send_tokens(&req.address, self.config.amount) {
            Ok(tx_hash) => {
                // Update rate limits
                self.record_request(&req.address, &req.ip_address);
                
                // Update balance
                *self.balance.write() -= self.config.amount;
                
                // Update stats
                {
                    let mut stats = self.stats.write();
                    stats.requests_fulfilled += 1;
                    stats.total_dispensed += self.config.amount;
                }

                // Check low balance alert
                if self.get_balance() < self.config.alert_threshold {
                    self.stats.write().low_balance_alerts += 1;
                }

                FaucetResponse::success(tx_hash, self.config.amount)
            }
            Err(e) => {
                self.stats.write().requests_failed += 1;
                FaucetResponse::error(format!("Failed to send tokens: {}", e))
            }
        }
    }

    /// Check address rate limit
    fn check_address_limit(&self, address: &Address) -> Option<FaucetResponse> {
        let limits = self.address_limits.read();
        if let Some(entry) = limits.get(address) {
            if let Some(last_request) = entry.last_request_time() {
                let next_eligible = last_request + self.config.cooldown_secs;
                if current_timestamp() < next_eligible {
                    return Some(FaucetResponse::cooldown(next_eligible));
                }
            }
        }
        None
    }

    /// Check IP rate limit
    fn check_ip_limit(&self, ip: &str) -> Option<FaucetResponse> {
        let limits = self.ip_limits.read();
        if let Some(entry) = limits.get(ip) {
            if entry.requests_today() >= self.config.max_requests_per_ip {
                return Some(FaucetResponse::error(
                    format!("Maximum {} requests per day exceeded", self.config.max_requests_per_ip)
                ));
            }
        }
        None
    }

    /// Record a successful request
    fn record_request(&self, address: &Address, ip: &str) {
        // Update address limit
        {
            let mut limits = self.address_limits.write();
            limits.entry(*address)
                .or_insert_with(RateLimitEntry::new)
                .add_request();
        }

        // Update IP limit
        {
            let mut limits = self.ip_limits.write();
            limits.entry(ip.to_string())
                .or_insert_with(RateLimitEntry::new)
                .add_request();
        }
    }

    /// Send tokens to address
    fn send_tokens(&self, to: &Address, amount: u64) -> Result<H256, FaucetError> {
        // Create transaction hash
        let tx_hash = {
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(&self.config.faucet_address.0);
            hasher.update(&to.0);
            hasher.update(&amount.to_le_bytes());
            hasher.update(&current_timestamp().to_le_bytes());
            H256::from_slice(hasher.finalize().as_bytes())
        };

        // Queue transaction
        let pending = PendingFaucetTx {
            address: *to,
            amount,
            tx_hash,
            created_at: current_timestamp(),
            confirmed: false,
        };
        self.tx_queue.write().push(pending);

        Ok(tx_hash)
    }

    /// Verify captcha
    fn verify_captcha(&self, response: &str) -> bool {
        // In production, this would make an HTTP call to verify
        // For now, accept non-empty responses
        !response.is_empty()
    }

    /// Get pending transactions
    pub fn get_pending_txs(&self) -> Vec<H256> {
        self.tx_queue.read()
            .iter()
            .filter(|tx| !tx.confirmed)
            .map(|tx| tx.tx_hash)
            .collect()
    }

    /// Confirm transaction
    pub fn confirm_tx(&self, tx_hash: &H256) {
        let mut queue = self.tx_queue.write();
        if let Some(tx) = queue.iter_mut().find(|tx| tx.tx_hash == *tx_hash) {
            tx.confirmed = true;
        }
    }

    /// Get remaining requests for address
    pub fn remaining_requests(&self, address: &Address) -> u32 {
        let limits = self.address_limits.read();
        if let Some(entry) = limits.get(address) {
            if let Some(last_request) = entry.last_request_time() {
                let next_eligible = last_request + self.config.cooldown_secs;
                if current_timestamp() < next_eligible {
                    return 0;
                }
            }
        }
        1 // Can make one request
    }

    /// Get next eligible time for address
    pub fn next_eligible(&self, address: &Address) -> Option<u64> {
        let limits = self.address_limits.read();
        if let Some(entry) = limits.get(address) {
            if let Some(last_request) = entry.last_request_time() {
                let next = last_request + self.config.cooldown_secs;
                if current_timestamp() < next {
                    return Some(next);
                }
            }
        }
        None
    }

    /// Get statistics
    pub fn stats(&self) -> FaucetStats {
        self.stats.read().clone()
    }

    /// Clean old entries
    pub fn cleanup(&self) {
        let cutoff = current_timestamp() - 7 * 86400; // 7 days

        // Clean address limits
        {
            let mut limits = self.address_limits.write();
            limits.retain(|_, entry| {
                entry.last_request_time().map_or(false, |t| t > cutoff)
            });
        }

        // Clean IP limits
        {
            let mut limits = self.ip_limits.write();
            limits.retain(|_, entry| {
                entry.last_request_time().map_or(false, |t| t > cutoff)
            });
        }

        // Clean old confirmed txs
        {
            let mut queue = self.tx_queue.write();
            queue.retain(|tx| !tx.confirmed || tx.created_at > cutoff);
        }
    }
}

impl Default for FaucetService {
    fn default() -> Self {
        Self::new(FaucetConfig::default())
    }
}

/// Faucet statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FaucetStats {
    pub requests_fulfilled: u64,
    pub requests_failed: u64,
    pub total_dispensed: u64,
    pub unique_addresses: u64,
    pub low_balance_alerts: u64,
}

/// Faucet errors
#[derive(Debug, thiserror::Error)]
pub enum FaucetError {
    #[error("Faucet disabled")]
    Disabled,

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Invalid captcha")]
    InvalidCaptcha,

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn current_day_start() -> u64 {
    let now = current_timestamp();
    now - (now % 86400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faucet_config() {
        let config = FaucetConfig::default();
        assert!(config.enabled);
        assert_eq!(config.amount, DEFAULT_FAUCET_AMOUNT);
    }

    #[test]
    fn test_faucet_request() {
        let req = FaucetRequest::new(
            Address([1u8; 20]),
            "127.0.0.1".into(),
        );
        assert!(req.captcha_response.is_none());
    }

    #[test]
    fn test_faucet_service() {
        let mut config = FaucetConfig::default();
        config.require_captcha = false;
        
        let faucet = FaucetService::new(config);
        faucet.set_balance(1_000_000 * 100_000_000);

        let req = FaucetRequest::new(
            Address([1u8; 20]),
            "127.0.0.1".into(),
        );

        let response = faucet.request(req);
        assert!(response.success);
        assert!(response.tx_hash.is_some());
    }

    #[test]
    fn test_rate_limiting() {
        let mut config = FaucetConfig::default();
        config.require_captcha = false;
        config.cooldown_secs = 3600;
        
        let faucet = FaucetService::new(config);
        faucet.set_balance(1_000_000 * 100_000_000);

        let req = FaucetRequest::new(
            Address([1u8; 20]),
            "127.0.0.1".into(),
        );

        // First request should succeed
        let response1 = faucet.request(req.clone());
        assert!(response1.success);

        // Second request should be rate limited
        let response2 = faucet.request(req);
        assert!(!response2.success);
    }
}
