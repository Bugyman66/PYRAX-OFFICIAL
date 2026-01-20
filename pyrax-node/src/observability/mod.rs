//! Observability Stack for PYRAX Node
//!
//! Comprehensive monitoring and tracing:
//! - OpenTelemetry distributed tracing
//! - Prometheus metrics exposition
//! - Structured logging with log aggregation support
//! - Alerting integration (PagerDuty, Slack)

pub mod metrics;
pub mod tracing;
pub mod alerts;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use metrics::{MetricsRegistry, NodeMetrics, P2PMetrics, ConsensusMetrics, MiningMetrics};
pub use tracing::{TracingConfig, init_tracing};
pub use alerts::{AlertManager, AlertRule, AlertSeverity};

/// Observability configuration
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// Enable Prometheus metrics
    pub prometheus_enabled: bool,
    /// Prometheus metrics port
    pub prometheus_port: u16,
    /// Enable OpenTelemetry tracing
    pub otel_enabled: bool,
    /// OpenTelemetry collector endpoint
    pub otel_endpoint: Option<String>,
    /// Enable structured JSON logging
    pub json_logs: bool,
    /// Log level
    pub log_level: String,
    /// Enable alerting
    pub alerts_enabled: bool,
    /// PagerDuty integration key
    pub pagerduty_key: Option<String>,
    /// Slack webhook URL
    pub slack_webhook: Option<String>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            prometheus_enabled: true,
            prometheus_port: 9090,
            otel_enabled: false,
            otel_endpoint: None,
            json_logs: false,
            log_level: "info".to_string(),
            alerts_enabled: false,
            pagerduty_key: None,
            slack_webhook: None,
        }
    }
}

/// Main observability manager
pub struct ObservabilityManager {
    config: ObservabilityConfig,
    metrics: Arc<RwLock<MetricsRegistry>>,
    alerts: Option<AlertManager>,
}

impl ObservabilityManager {
    pub fn new(config: ObservabilityConfig) -> Self {
        let alerts = if config.alerts_enabled {
            Some(AlertManager::new(
                config.pagerduty_key.clone(),
                config.slack_webhook.clone(),
            ))
        } else {
            None
        };
        
        Self {
            config,
            metrics: Arc::new(RwLock::new(MetricsRegistry::new())),
            alerts,
        }
    }
    
    pub fn metrics(&self) -> Arc<RwLock<MetricsRegistry>> {
        Arc::clone(&self.metrics)
    }
    
    pub async fn start_prometheus_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.prometheus_enabled {
            return Ok(());
        }
        
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.prometheus_port).parse()?;
        let metrics = Arc::clone(&self.metrics);
        
        tokio::spawn(async move {
            metrics::serve_prometheus(addr, metrics).await;
        });
        
        Ok(())
    }
    
    pub async fn check_alerts(&self) {
        if let Some(alert_mgr) = &self.alerts {
            let metrics = self.metrics.read().await;
            alert_mgr.check_metrics(&metrics).await;
        }
    }
}
