//! Alerting System for PYRAX Node

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use super::metrics::MetricsRegistry;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl AlertSeverity {
    pub fn emoji(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Critical => "🔴",
            AlertSeverity::Emergency => "🚨",
        }
    }
}

/// Alert rule definition
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub condition: AlertCondition,
    pub cooldown: Duration,
    pub last_fired: Option<Instant>,
}

/// Condition types for alerts
#[derive(Debug, Clone)]
pub enum AlertCondition {
    PeerCountBelow(u64),
    PeerCountAbove(u64),
    BlockHeightStale(Duration),
    SyncProgressBelow(f64),
    MemoryUsageAbove(u64),
    DiskUsageAbove(u64),
    GpuTempAbove(f64),
    HashrateDrop(f64),
    Custom(String),
}

/// Active alert instance
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: String,
    pub resolved: bool,
}

/// Alert manager
pub struct AlertManager {
    rules: Vec<AlertRule>,
    active_alerts: HashMap<String, Alert>,
    pagerduty_key: Option<String>,
    slack_webhook: Option<String>,
    last_check: Instant,
}

impl AlertManager {
    pub fn new(pagerduty_key: Option<String>, slack_webhook: Option<String>) -> Self {
        let mut mgr = Self {
            rules: Vec::new(),
            active_alerts: HashMap::new(),
            pagerduty_key,
            slack_webhook,
            last_check: Instant::now(),
        };
        mgr.add_default_rules();
        mgr
    }
    
    fn add_default_rules(&mut self) {
        self.rules.push(AlertRule {
            name: "low_peer_count".to_string(),
            description: "Peer count below minimum threshold".to_string(),
            severity: AlertSeverity::Warning,
            condition: AlertCondition::PeerCountBelow(5),
            cooldown: Duration::from_secs(300),
            last_fired: None,
        });
        
        self.rules.push(AlertRule {
            name: "no_peers".to_string(),
            description: "No peers connected".to_string(),
            severity: AlertSeverity::Critical,
            condition: AlertCondition::PeerCountBelow(1),
            cooldown: Duration::from_secs(60),
            last_fired: None,
        });
        
        self.rules.push(AlertRule {
            name: "sync_stalled".to_string(),
            description: "Sync progress stalled".to_string(),
            severity: AlertSeverity::Warning,
            condition: AlertCondition::SyncProgressBelow(0.01),
            cooldown: Duration::from_secs(600),
            last_fired: None,
        });
        
        self.rules.push(AlertRule {
            name: "gpu_overheat".to_string(),
            description: "GPU temperature critical".to_string(),
            severity: AlertSeverity::Critical,
            condition: AlertCondition::GpuTempAbove(85.0),
            cooldown: Duration::from_secs(60),
            last_fired: None,
        });
        
        self.rules.push(AlertRule {
            name: "hashrate_drop".to_string(),
            description: "Mining hashrate dropped significantly".to_string(),
            severity: AlertSeverity::Warning,
            condition: AlertCondition::HashrateDrop(0.5),
            cooldown: Duration::from_secs(300),
            last_fired: None,
        });
    }
    
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }
    
    pub async fn check_metrics(&self, metrics: &MetricsRegistry) {
        for rule in &self.rules {
            if self.should_skip_rule(rule) {
                continue;
            }
            
            let triggered = match &rule.condition {
                AlertCondition::PeerCountBelow(threshold) => {
                    metrics.p2p.peer_count < *threshold
                }
                AlertCondition::PeerCountAbove(threshold) => {
                    metrics.p2p.peer_count > *threshold
                }
                AlertCondition::SyncProgressBelow(threshold) => {
                    metrics.consensus.sync_progress < *threshold && metrics.consensus.sync_progress > 0.0
                }
                AlertCondition::GpuTempAbove(threshold) => {
                    metrics.mining.gpu_temperature_celsius > *threshold
                }
                AlertCondition::HashrateDrop(threshold) => {
                    metrics.mining.hashrate < *threshold && metrics.mining.hashrate > 0.0
                }
                AlertCondition::MemoryUsageAbove(threshold) => {
                    metrics.node.memory_usage_bytes > *threshold
                }
                AlertCondition::DiskUsageAbove(threshold) => {
                    metrics.node.db_size_bytes > *threshold
                }
                _ => false,
            };
            
            if triggered {
                self.fire_alert(rule, metrics).await;
            }
        }
    }
    
    fn should_skip_rule(&self, rule: &AlertRule) -> bool {
        if let Some(last) = rule.last_fired {
            last.elapsed() < rule.cooldown
        } else {
            false
        }
    }
    
    async fn fire_alert(&self, rule: &AlertRule, metrics: &MetricsRegistry) {
        let message = self.format_alert_message(rule, metrics);
        
        warn!("{} Alert: {} - {}", rule.severity.emoji(), rule.name, message);
        
        // Send to PagerDuty
        if let Some(key) = &self.pagerduty_key {
            if rule.severity == AlertSeverity::Critical || rule.severity == AlertSeverity::Emergency {
                self.send_pagerduty(key, rule, &message).await;
            }
        }
        
        // Send to Slack
        if let Some(webhook) = &self.slack_webhook {
            self.send_slack(webhook, rule, &message).await;
        }
    }
    
    fn format_alert_message(&self, rule: &AlertRule, metrics: &MetricsRegistry) -> String {
        match &rule.condition {
            AlertCondition::PeerCountBelow(t) => {
                format!("Peer count {} is below threshold {}", metrics.p2p.peer_count, t)
            }
            AlertCondition::GpuTempAbove(t) => {
                format!("GPU temperature {:.1}°C exceeds threshold {:.1}°C", 
                    metrics.mining.gpu_temperature_celsius, t)
            }
            AlertCondition::SyncProgressBelow(_) => {
                format!("Sync progress stalled at {:.1}%", metrics.consensus.sync_progress * 100.0)
            }
            _ => rule.description.clone(),
        }
    }
    
    async fn send_pagerduty(&self, key: &str, rule: &AlertRule, message: &str) {
        let payload = serde_json::json!({
            "routing_key": key,
            "event_action": "trigger",
            "payload": {
                "summary": format!("PYRAX Node: {}", rule.name),
                "severity": match rule.severity {
                    AlertSeverity::Critical | AlertSeverity::Emergency => "critical",
                    AlertSeverity::Warning => "warning",
                    AlertSeverity::Info => "info",
                },
                "source": "pyrax-node",
                "custom_details": {
                    "message": message,
                }
            }
        });
        
        if let Ok(client) = reqwest::Client::builder().timeout(Duration::from_secs(5)).build() {
            let _ = client.post("https://events.pagerduty.com/v2/enqueue")
                .json(&payload)
                .send()
                .await;
        }
    }
    
    async fn send_slack(&self, webhook: &str, rule: &AlertRule, message: &str) {
        let color = match rule.severity {
            AlertSeverity::Info => "#36a64f",
            AlertSeverity::Warning => "#ffcc00",
            AlertSeverity::Critical => "#ff0000",
            AlertSeverity::Emergency => "#8b0000",
        };
        
        let payload = serde_json::json!({
            "attachments": [{
                "color": color,
                "title": format!("{} PYRAX Node Alert: {}", rule.severity.emoji(), rule.name),
                "text": message,
                "footer": "PYRAX Monitoring"
            }]
        });
        
        if let Ok(client) = reqwest::Client::builder().timeout(Duration::from_secs(5)).build() {
            let _ = client.post(webhook).json(&payload).send().await;
        }
    }
    
    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        self.active_alerts.values().filter(|a| !a.resolved).collect()
    }
    
    pub fn resolve_alert(&mut self, rule_name: &str) {
        if let Some(alert) = self.active_alerts.get_mut(rule_name) {
            alert.resolved = true;
            info!("Alert resolved: {}", rule_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_alert_manager_creation() {
        let mgr = AlertManager::new(None, None);
        assert!(!mgr.rules.is_empty());
    }
    
    #[test]
    fn test_alert_severity() {
        assert_eq!(AlertSeverity::Critical.emoji(), "🔴");
        assert_eq!(AlertSeverity::Warning.emoji(), "⚠️");
    }
}
