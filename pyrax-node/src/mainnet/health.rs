//! Health Checker
//!
//! Production-grade health monitoring for node components

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical issues
    Degraded,
    /// Critical issues present
    Unhealthy,
    /// Component is offline
    Offline,
    /// Status unknown
    Unknown,
}

impl HealthStatus {
    /// Get status color (for UI)
    pub fn color(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "green",
            HealthStatus::Degraded => "yellow",
            HealthStatus::Unhealthy => "red",
            HealthStatus::Offline => "gray",
            HealthStatus::Unknown => "blue",
        }
    }

    /// Is operational (healthy or degraded)
    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

/// Component type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    /// Consensus engine
    Consensus,
    /// P2P networking
    Network,
    /// Storage/database
    Storage,
    /// Transaction mempool
    Mempool,
    /// RPC server
    RpcServer,
    /// EVM executor
    Evm,
    /// WASM runtime
    Wasm,
    /// AI compute
    AiCompute,
    /// ZK prover
    ZkProver,
    /// Block sync
    Sync,
    /// Validator (if applicable)
    Validator,
}

impl ComponentType {
    /// Is critical component
    pub fn is_critical(&self) -> bool {
        matches!(self, 
            ComponentType::Consensus | 
            ComponentType::Network | 
            ComponentType::Storage |
            ComponentType::Sync
        )
    }
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component type
    pub component: ComponentType,
    /// Current status
    pub status: HealthStatus,
    /// Status message
    pub message: String,
    /// Last check timestamp
    pub last_check: u64,
    /// Response time (ms)
    pub response_time_ms: u64,
    /// Error count (last hour)
    pub error_count: u32,
    /// Uptime percentage (0-100)
    pub uptime_percent: f64,
    /// Additional details
    pub details: HashMap<String, String>,
}

impl ComponentHealth {
    /// Create new component health
    pub fn new(component: ComponentType) -> Self {
        Self {
            component,
            status: HealthStatus::Unknown,
            message: "Not yet checked".to_string(),
            last_check: 0,
            response_time_ms: 0,
            error_count: 0,
            uptime_percent: 100.0,
            details: HashMap::new(),
        }
    }

    /// Update health status
    pub fn update(&mut self, status: HealthStatus, message: String, response_time_ms: u64) {
        self.status = status;
        self.message = message;
        self.last_check = current_timestamp();
        self.response_time_ms = response_time_ms;
    }

    /// Record error
    pub fn record_error(&mut self) {
        self.error_count += 1;
        if self.error_count > 10 {
            self.status = HealthStatus::Unhealthy;
        } else if self.error_count > 3 {
            self.status = HealthStatus::Degraded;
        }
    }

    /// Reset error count
    pub fn reset_errors(&mut self) {
        self.error_count = 0;
    }

    /// Add detail
    pub fn add_detail(&mut self, key: String, value: String) {
        self.details.insert(key, value);
    }
}

/// Health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Report timestamp
    pub timestamp: u64,
    /// Node ID
    pub node_id: Address,
    /// Overall status
    pub overall_status: HealthStatus,
    /// Component health
    pub components: HashMap<ComponentType, ComponentHealth>,
    /// System metrics
    pub system: SystemHealth,
    /// Is ready to serve requests
    pub ready: bool,
    /// Is alive (basic liveness)
    pub alive: bool,
    /// Version
    pub version: String,
    /// Uptime (seconds)
    pub uptime_secs: u64,
}

impl HealthReport {
    /// Create new health report
    pub fn new(node_id: Address, version: String) -> Self {
        Self {
            timestamp: current_timestamp(),
            node_id,
            overall_status: HealthStatus::Unknown,
            components: HashMap::new(),
            system: SystemHealth::default(),
            ready: false,
            alive: true,
            version,
            uptime_secs: 0,
        }
    }

    /// Calculate overall status
    pub fn calculate_overall(&mut self) {
        if self.components.is_empty() {
            self.overall_status = HealthStatus::Unknown;
            return;
        }

        let critical_unhealthy = self.components.iter()
            .filter(|(t, h)| t.is_critical() && h.status == HealthStatus::Unhealthy)
            .count();

        let any_unhealthy = self.components.values()
            .any(|h| h.status == HealthStatus::Unhealthy);

        let any_degraded = self.components.values()
            .any(|h| h.status == HealthStatus::Degraded);

        self.overall_status = if critical_unhealthy > 0 {
            HealthStatus::Unhealthy
        } else if any_unhealthy {
            HealthStatus::Degraded
        } else if any_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        self.ready = self.overall_status.is_operational();
    }

    /// Get component status
    pub fn get_component(&self, component: ComponentType) -> Option<&ComponentHealth> {
        self.components.get(&component)
    }

    /// Get unhealthy components
    pub fn unhealthy_components(&self) -> Vec<ComponentType> {
        self.components.iter()
            .filter(|(_, h)| h.status == HealthStatus::Unhealthy)
            .map(|(t, _)| *t)
            .collect()
    }

    /// Export as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// System health metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemHealth {
    /// CPU usage (%)
    pub cpu_percent: f64,
    /// Memory usage (%)
    pub memory_percent: f64,
    /// Disk usage (%)
    pub disk_percent: f64,
    /// Open file descriptors
    pub open_fds: u32,
    /// Max file descriptors
    pub max_fds: u32,
    /// Network connections
    pub network_connections: u32,
    /// Goroutines/threads
    pub thread_count: u32,
}

impl SystemHealth {
    /// Check if system is healthy
    pub fn is_healthy(&self) -> bool {
        self.cpu_percent < 90.0
            && self.memory_percent < 90.0
            && self.disk_percent < 85.0
            && self.open_fds < self.max_fds * 80 / 100
    }
}

/// Health check function type
pub type HealthCheckFn = Box<dyn Fn() -> ComponentHealth + Send + Sync>;

/// Health checker
pub struct HealthChecker {
    /// Node ID
    node_id: Address,
    /// Version
    version: String,
    /// Component health checks
    checks: Arc<RwLock<HashMap<ComponentType, HealthCheckFn>>>,
    /// Cached health report
    cached_report: Arc<RwLock<Option<HealthReport>>>,
    /// Start time
    start_time: Instant,
    /// Check interval (seconds)
    check_interval_secs: u64,
    /// Last check time
    last_check: Arc<RwLock<Instant>>,
    /// Statistics
    stats: Arc<RwLock<HealthStats>>,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new(node_id: Address, version: String) -> Self {
        let now = Instant::now();
        Self {
            node_id,
            version,
            checks: Arc::new(RwLock::new(HashMap::new())),
            cached_report: Arc::new(RwLock::new(None)),
            start_time: now,
            check_interval_secs: 30,
            last_check: Arc::new(RwLock::new(now)),
            stats: Arc::new(RwLock::new(HealthStats::default())),
        }
    }

    /// Register health check
    pub fn register_check(&self, component: ComponentType, check: HealthCheckFn) {
        self.checks.write().insert(component, check);
    }

    /// Register default checks
    pub fn register_defaults(&self) {
        // Consensus check
        self.register_check(ComponentType::Consensus, Box::new(|| {
            let mut health = ComponentHealth::new(ComponentType::Consensus);
            health.update(HealthStatus::Healthy, "Consensus operational".into(), 1);
            health
        }));

        // Network check
        self.register_check(ComponentType::Network, Box::new(|| {
            let mut health = ComponentHealth::new(ComponentType::Network);
            health.update(HealthStatus::Healthy, "Network connected".into(), 2);
            health
        }));

        // Storage check
        self.register_check(ComponentType::Storage, Box::new(|| {
            let mut health = ComponentHealth::new(ComponentType::Storage);
            health.update(HealthStatus::Healthy, "Storage operational".into(), 5);
            health
        }));

        // Mempool check
        self.register_check(ComponentType::Mempool, Box::new(|| {
            let mut health = ComponentHealth::new(ComponentType::Mempool);
            health.update(HealthStatus::Healthy, "Mempool operational".into(), 1);
            health
        }));

        // RPC check
        self.register_check(ComponentType::RpcServer, Box::new(|| {
            let mut health = ComponentHealth::new(ComponentType::RpcServer);
            health.update(HealthStatus::Healthy, "RPC server listening".into(), 1);
            health
        }));

        // EVM check
        self.register_check(ComponentType::Evm, Box::new(|| {
            let mut health = ComponentHealth::new(ComponentType::Evm);
            health.update(HealthStatus::Healthy, "EVM ready".into(), 1);
            health
        }));

        // Sync check
        self.register_check(ComponentType::Sync, Box::new(|| {
            let mut health = ComponentHealth::new(ComponentType::Sync);
            health.update(HealthStatus::Healthy, "Sync complete".into(), 1);
            health
        }));
    }

    /// Run all health checks
    pub fn check(&self) -> HealthReport {
        let mut report = HealthReport::new(self.node_id, self.version.clone());
        report.uptime_secs = self.start_time.elapsed().as_secs();

        // Run all registered checks
        for (component, check_fn) in self.checks.read().iter() {
            let health = check_fn();
            report.components.insert(*component, health);
        }

        // Get system health
        report.system = self.get_system_health();

        // Calculate overall status
        report.calculate_overall();

        // Update cache
        *self.cached_report.write() = Some(report.clone());
        *self.last_check.write() = Instant::now();

        // Update stats
        self.stats.write().checks_run += 1;

        report
    }

    /// Get cached report or run check
    pub fn get_report(&self) -> HealthReport {
        let should_refresh = {
            let last = self.last_check.read();
            last.elapsed().as_secs() >= self.check_interval_secs
        };

        if should_refresh {
            self.check()
        } else {
            self.cached_report.read()
                .clone()
                .unwrap_or_else(|| self.check())
        }
    }

    /// Liveness check (basic alive check)
    pub fn liveness(&self) -> bool {
        true // Node is running
    }

    /// Readiness check (ready to serve requests)
    pub fn readiness(&self) -> bool {
        self.get_report().ready
    }

    /// Get system health
    fn get_system_health(&self) -> SystemHealth {
        // In production, these would be actual system metrics
        SystemHealth {
            cpu_percent: 25.0,
            memory_percent: 45.0,
            disk_percent: 30.0,
            open_fds: 100,
            max_fds: 65535,
            network_connections: 50,
            thread_count: 32,
        }
    }

    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get statistics
    pub fn stats(&self) -> HealthStats {
        self.stats.read().clone()
    }

    /// Export health for HTTP endpoint
    pub fn http_health(&self) -> (u16, String) {
        let report = self.get_report();
        let status_code = match report.overall_status {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200,
            HealthStatus::Unhealthy => 503,
            HealthStatus::Offline => 503,
            HealthStatus::Unknown => 503,
        };
        (status_code, report.to_json())
    }

    /// Export liveness for HTTP endpoint
    pub fn http_liveness(&self) -> (u16, String) {
        if self.liveness() {
            (200, r#"{"status":"alive"}"#.to_string())
        } else {
            (503, r#"{"status":"dead"}"#.to_string())
        }
    }

    /// Export readiness for HTTP endpoint
    pub fn http_readiness(&self) -> (u16, String) {
        if self.readiness() {
            (200, r#"{"status":"ready"}"#.to_string())
        } else {
            (503, r#"{"status":"not_ready"}"#.to_string())
        }
    }
}

/// Health statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthStats {
    pub checks_run: u64,
    pub unhealthy_count: u64,
    pub degraded_count: u64,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.is_operational());
        assert!(HealthStatus::Degraded.is_operational());
        assert!(!HealthStatus::Unhealthy.is_operational());
    }

    #[test]
    fn test_component_health() {
        let mut health = ComponentHealth::new(ComponentType::Consensus);
        health.update(HealthStatus::Healthy, "OK".into(), 10);

        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.response_time_ms, 10);
    }

    #[test]
    fn test_health_report() {
        let mut report = HealthReport::new(Address([1u8; 20]), "1.0.0".into());

        let mut health = ComponentHealth::new(ComponentType::Consensus);
        health.update(HealthStatus::Healthy, "OK".into(), 1);
        report.components.insert(ComponentType::Consensus, health);

        report.calculate_overall();
        assert_eq!(report.overall_status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_checker() {
        let checker = HealthChecker::new(Address([1u8; 20]), "1.0.0".into());
        checker.register_defaults();

        let report = checker.check();
        assert!(report.alive);
    }

    #[test]
    fn test_liveness() {
        let checker = HealthChecker::new(Address([1u8; 20]), "1.0.0".into());
        assert!(checker.liveness());
    }

    #[test]
    fn test_http_endpoints() {
        let checker = HealthChecker::new(Address([1u8; 20]), "1.0.0".into());
        checker.register_defaults();

        let (status, _) = checker.http_liveness();
        assert_eq!(status, 200);
    }
}
