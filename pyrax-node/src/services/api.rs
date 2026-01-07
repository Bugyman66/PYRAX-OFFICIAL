//! API Gateway Service
//!
//! Production API gateway with rate limiting and authentication

use std::collections::HashMap;
use std::sync::Arc;
use std::net::IpAddr;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::MAX_REQUESTS_PER_MINUTE;

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Enable API
    pub enabled: bool,
    /// Listen address
    pub listen_addr: String,
    /// Listen port
    pub port: u16,
    /// Enable CORS
    pub cors_enabled: bool,
    /// Allowed origins
    pub cors_origins: Vec<String>,
    /// Enable rate limiting
    pub rate_limit_enabled: bool,
    /// Requests per minute (default)
    pub rate_limit_rpm: u32,
    /// Enable API keys
    pub api_keys_enabled: bool,
    /// Enable request logging
    pub request_logging: bool,
    /// Request timeout (seconds)
    pub timeout_secs: u64,
    /// Maximum request body size (bytes)
    pub max_body_size: usize,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_addr: "0.0.0.0".to_string(),
            port: 8080,
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
            rate_limit_enabled: true,
            rate_limit_rpm: MAX_REQUESTS_PER_MINUTE,
            api_keys_enabled: true,
            request_logging: true,
            timeout_secs: 30,
            max_body_size: 1024 * 1024, // 1MB
        }
    }
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    /// Success flag
    pub success: bool,
    /// Response data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    /// Request ID
    pub request_id: String,
    /// Timestamp
    pub timestamp: u64,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create success response
    pub fn success(data: T, request_id: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            request_id,
            timestamp: current_timestamp(),
        }
    }

    /// Create error response
    pub fn error(error: ApiError, request_id: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
            request_id,
            timestamp: current_timestamp(),
        }
    }
}

/// API error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: u32,
    /// Error message
    pub message: String,
    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    /// Bad request (400)
    pub fn bad_request(message: String) -> Self {
        Self { code: 400, message, details: None }
    }

    /// Unauthorized (401)
    pub fn unauthorized() -> Self {
        Self { code: 401, message: "Unauthorized".into(), details: None }
    }

    /// Forbidden (403)
    pub fn forbidden() -> Self {
        Self { code: 403, message: "Forbidden".into(), details: None }
    }

    /// Not found (404)
    pub fn not_found(resource: String) -> Self {
        Self { code: 404, message: format!("{} not found", resource), details: None }
    }

    /// Rate limited (429)
    pub fn rate_limited(retry_after: u64) -> Self {
        Self {
            code: 429,
            message: "Rate limit exceeded".into(),
            details: Some(format!("Retry after {} seconds", retry_after)),
        }
    }

    /// Internal error (500)
    pub fn internal(message: String) -> Self {
        Self { code: 500, message, details: None }
    }
}

/// API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Key ID
    pub id: String,
    /// Hashed key
    pub key_hash: String,
    /// Owner address
    pub owner: Address,
    /// Label
    pub label: String,
    /// Permissions
    pub permissions: Vec<ApiPermission>,
    /// Rate limit (requests per minute)
    pub rate_limit: u32,
    /// Is active
    pub active: bool,
    /// Created at
    pub created_at: u64,
    /// Expires at
    pub expires_at: Option<u64>,
    /// Last used
    pub last_used: Option<u64>,
    /// Total requests
    pub total_requests: u64,
}

impl ApiKey {
    /// Create new API key
    pub fn new(owner: Address, label: String, permissions: Vec<ApiPermission>) -> (Self, String) {
        let raw_key = generate_api_key();
        let key_hash = hash_api_key(&raw_key);
        let id = generate_key_id();

        let key = Self {
            id: id.clone(),
            key_hash,
            owner,
            label,
            permissions,
            rate_limit: MAX_REQUESTS_PER_MINUTE,
            active: true,
            created_at: current_timestamp(),
            expires_at: None,
            last_used: None,
            total_requests: 0,
        };

        (key, format!("pyrax_{}", raw_key))
    }

    /// Verify key
    pub fn verify(&self, raw_key: &str) -> bool {
        let hash = hash_api_key(raw_key);
        self.key_hash == hash && self.active && !self.is_expired()
    }

    /// Is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            current_timestamp() > expires
        } else {
            false
        }
    }

    /// Has permission
    pub fn has_permission(&self, permission: &ApiPermission) -> bool {
        self.permissions.contains(permission) || self.permissions.contains(&ApiPermission::Admin)
    }

    /// Record usage
    pub fn record_usage(&mut self) {
        self.last_used = Some(current_timestamp());
        self.total_requests += 1;
    }
}

/// API permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiPermission {
    /// Read blockchain data
    Read,
    /// Submit transactions
    Write,
    /// Access admin endpoints
    Admin,
    /// Access faucet
    Faucet,
    /// Access explorer
    Explorer,
    /// Access metrics
    Metrics,
    /// Access websocket
    Websocket,
}

/// Rate limiter
pub struct RateLimiter {
    /// Requests by IP
    ip_requests: Arc<RwLock<HashMap<String, RequestWindow>>>,
    /// Requests by API key
    key_requests: Arc<RwLock<HashMap<String, RequestWindow>>>,
    /// Default limit
    default_limit: u32,
    /// Window size (seconds)
    window_secs: u64,
}

/// Request window
#[derive(Debug, Clone)]
struct RequestWindow {
    /// Window start
    window_start: u64,
    /// Request count
    count: u32,
    /// Limit
    limit: u32,
}

impl RequestWindow {
    fn new(limit: u32) -> Self {
        Self {
            window_start: current_timestamp(),
            count: 0,
            limit,
        }
    }

    fn check_and_increment(&mut self, window_secs: u64) -> bool {
        let now = current_timestamp();
        
        // Reset if window expired
        if now >= self.window_start + window_secs {
            self.window_start = now;
            self.count = 0;
        }

        if self.count >= self.limit {
            return false;
        }

        self.count += 1;
        true
    }

    fn remaining(&self, window_secs: u64) -> u32 {
        let now = current_timestamp();
        if now >= self.window_start + window_secs {
            self.limit
        } else {
            self.limit.saturating_sub(self.count)
        }
    }

    fn reset_time(&self, window_secs: u64) -> u64 {
        self.window_start + window_secs
    }
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(default_limit: u32, window_secs: u64) -> Self {
        Self {
            ip_requests: Arc::new(RwLock::new(HashMap::new())),
            key_requests: Arc::new(RwLock::new(HashMap::new())),
            default_limit,
            window_secs,
        }
    }

    /// Check IP rate limit
    pub fn check_ip(&self, ip: &str) -> RateLimitResult {
        let mut requests = self.ip_requests.write();
        let window = requests.entry(ip.to_string())
            .or_insert_with(|| RequestWindow::new(self.default_limit));

        if window.check_and_increment(self.window_secs) {
            RateLimitResult::Allowed {
                remaining: window.remaining(self.window_secs),
                reset: window.reset_time(self.window_secs),
            }
        } else {
            RateLimitResult::Limited {
                retry_after: window.reset_time(self.window_secs) - current_timestamp(),
            }
        }
    }

    /// Check API key rate limit
    pub fn check_key(&self, key_id: &str, limit: u32) -> RateLimitResult {
        let mut requests = self.key_requests.write();
        let window = requests.entry(key_id.to_string())
            .or_insert_with(|| RequestWindow::new(limit));
        window.limit = limit; // Update limit in case it changed

        if window.check_and_increment(self.window_secs) {
            RateLimitResult::Allowed {
                remaining: window.remaining(self.window_secs),
                reset: window.reset_time(self.window_secs),
            }
        } else {
            RateLimitResult::Limited {
                retry_after: window.reset_time(self.window_secs) - current_timestamp(),
            }
        }
    }

    /// Clean expired entries
    pub fn cleanup(&self) {
        let cutoff = current_timestamp() - self.window_secs * 2;
        
        self.ip_requests.write().retain(|_, w| w.window_start > cutoff);
        self.key_requests.write().retain(|_, w| w.window_start > cutoff);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(MAX_REQUESTS_PER_MINUTE, 60)
    }
}

/// Rate limit result
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    /// Request allowed
    Allowed {
        /// Remaining requests
        remaining: u32,
        /// Reset timestamp
        reset: u64,
    },
    /// Request limited
    Limited {
        /// Retry after (seconds)
        retry_after: u64,
    },
}

/// API Gateway
pub struct ApiGateway {
    /// Configuration
    config: ApiConfig,
    /// Rate limiter
    rate_limiter: RateLimiter,
    /// API keys
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// Request log
    request_log: Arc<RwLock<Vec<RequestLog>>>,
    /// Statistics
    stats: Arc<RwLock<ApiStats>>,
}

/// Request log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    /// Request ID
    pub request_id: String,
    /// Timestamp
    pub timestamp: u64,
    /// IP address
    pub ip: String,
    /// Method
    pub method: String,
    /// Path
    pub path: String,
    /// Status code
    pub status: u16,
    /// Duration (ms)
    pub duration_ms: u64,
    /// API key ID (if used)
    pub api_key_id: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
}

impl ApiGateway {
    /// Create new API gateway
    pub fn new(config: ApiConfig) -> Self {
        let rate_limiter = RateLimiter::new(config.rate_limit_rpm, 60);

        Self {
            config,
            rate_limiter,
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            request_log: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(ApiStats::default())),
        }
    }

    /// Generate new API key
    pub fn create_api_key(
        &self,
        owner: Address,
        label: String,
        permissions: Vec<ApiPermission>,
    ) -> (String, String) {
        let (key, raw) = ApiKey::new(owner, label, permissions);
        let id = key.id.clone();
        self.api_keys.write().insert(id.clone(), key);
        (id, raw)
    }

    /// Validate API key
    pub fn validate_key(&self, raw_key: &str) -> Option<ApiKey> {
        // Strip prefix if present
        let key_part = raw_key.strip_prefix("pyrax_").unwrap_or(raw_key);
        
        for key in self.api_keys.read().values() {
            if key.verify(key_part) {
                return Some(key.clone());
            }
        }
        None
    }

    /// Check rate limit for request
    pub fn check_rate_limit(&self, ip: &str, api_key: Option<&ApiKey>) -> RateLimitResult {
        if !self.config.rate_limit_enabled {
            return RateLimitResult::Allowed { remaining: u32::MAX, reset: 0 };
        }

        if let Some(key) = api_key {
            self.rate_limiter.check_key(&key.id, key.rate_limit)
        } else {
            self.rate_limiter.check_ip(ip)
        }
    }

    /// Log request
    pub fn log_request(&self, log: RequestLog) {
        if !self.config.request_logging {
            return;
        }

        let mut logs = self.request_log.write();
        logs.push(log);

        // Keep last 10000 logs
        if logs.len() > 10000 {
            logs.remove(0);
        }

        // Update stats
        let mut stats = self.stats.write();
        stats.total_requests += 1;
    }

    /// Get API key by ID
    pub fn get_key(&self, id: &str) -> Option<ApiKey> {
        self.api_keys.read().get(id).cloned()
    }

    /// Revoke API key
    pub fn revoke_key(&self, id: &str) -> bool {
        if let Some(key) = self.api_keys.write().get_mut(id) {
            key.active = false;
            true
        } else {
            false
        }
    }

    /// Get recent requests
    pub fn get_recent_requests(&self, limit: usize) -> Vec<RequestLog> {
        let logs = self.request_log.read();
        logs.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> ApiStats {
        self.stats.read().clone()
    }

    /// Get listen address
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.config.listen_addr, self.config.port)
    }
}

impl Default for ApiGateway {
    fn default() -> Self {
        Self::new(ApiConfig::default())
    }
}

/// API statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limited: u64,
    pub avg_response_time_ms: u64,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn generate_api_key() -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(&current_timestamp().to_le_bytes());
    hasher.update(b"api_key");
    hasher.update(&rand::random::<[u8; 16]>());
    hex::encode(&hasher.finalize().as_bytes()[..24])
}

fn generate_key_id() -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(&current_timestamp().to_le_bytes());
    hasher.update(b"key_id");
    hex::encode(&hasher.finalize().as_bytes()[..8])
}

fn hash_api_key(key: &str) -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize().as_bytes())
}

mod rand {
    pub fn random<T: Default + AsMut<[u8]>>() -> T {
        let mut value = T::default();
        let timestamp = super::current_timestamp();
        for (i, byte) in value.as_mut().iter_mut().enumerate() {
            *byte = ((timestamp >> (i % 8)) & 0xFF) as u8;
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response() {
        let response = ApiResponse::success("test", "req-123".into());
        assert!(response.success);
        assert_eq!(response.data, Some("test"));
    }

    #[test]
    fn test_api_error() {
        let error = ApiError::not_found("Block".into());
        assert_eq!(error.code, 404);
    }

    #[test]
    fn test_api_key() {
        let (key, raw) = ApiKey::new(
            Address([1u8; 20]),
            "Test Key".into(),
            vec![ApiPermission::Read],
        );

        assert!(key.active);
        assert!(raw.starts_with("pyrax_"));
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, 60);

        for _ in 0..5 {
            match limiter.check_ip("127.0.0.1") {
                RateLimitResult::Allowed { .. } => {}
                RateLimitResult::Limited { .. } => panic!("Should not be limited"),
            }
        }

        // 6th request should be limited
        match limiter.check_ip("127.0.0.1") {
            RateLimitResult::Allowed { .. } => panic!("Should be limited"),
            RateLimitResult::Limited { .. } => {}
        }
    }

    #[test]
    fn test_api_gateway() {
        let gateway = ApiGateway::default();
        
        let (id, raw) = gateway.create_api_key(
            Address([1u8; 20]),
            "Test".into(),
            vec![ApiPermission::Read],
        );

        assert!(gateway.validate_key(&raw).is_some());
        assert!(gateway.get_key(&id).is_some());
    }
}
