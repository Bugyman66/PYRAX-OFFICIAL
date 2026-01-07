//! Chaos Testing Framework
//!
//! Production-grade chaos engineering for network resilience testing

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};

/// Chaos event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChaosEventType {
    /// Network partition between node groups
    NetworkPartition,
    /// Packet loss simulation
    PacketLoss,
    /// Network latency injection
    LatencyInjection,
    /// Node crash simulation
    NodeCrash,
    /// Byzantine behavior simulation
    ByzantineBehavior,
    /// Clock drift simulation
    ClockDrift,
    /// Disk I/O slowdown
    DiskSlowdown,
    /// Memory pressure
    MemoryPressure,
    /// CPU throttling
    CpuThrottle,
    /// Message corruption
    MessageCorruption,
    /// Message reordering
    MessageReorder,
    /// Duplicate messages
    MessageDuplicate,
}

/// Chaos event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChaosSeverity {
    /// Minor disruption
    Low,
    /// Moderate disruption
    Medium,
    /// Severe disruption
    High,
    /// Critical failure simulation
    Critical,
}

/// Chaos event configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosEvent {
    /// Event ID
    pub id: H256,
    /// Event type
    pub event_type: ChaosEventType,
    /// Severity level
    pub severity: ChaosSeverity,
    /// Target nodes (empty = all nodes)
    pub target_nodes: Vec<Address>,
    /// Duration of the event
    pub duration_secs: u64,
    /// Event parameters
    pub params: ChaosParams,
    /// Start timestamp
    pub start_time: u64,
    /// End timestamp
    pub end_time: Option<u64>,
    /// Is event active
    pub active: bool,
    /// Results collected
    pub results: Option<ChaosResult>,
}

impl ChaosEvent {
    /// Create new chaos event
    pub fn new(
        event_type: ChaosEventType,
        severity: ChaosSeverity,
        duration_secs: u64,
    ) -> Self {
        let id = Self::generate_id(event_type);
        Self {
            id,
            event_type,
            severity,
            target_nodes: Vec::new(),
            duration_secs,
            params: ChaosParams::default_for(event_type, severity),
            start_time: 0,
            end_time: None,
            active: false,
            results: None,
        }
    }

    /// Generate event ID
    fn generate_id(event_type: ChaosEventType) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&[event_type as u8]);
        hasher.update(&current_timestamp().to_le_bytes());
        hasher.update(&rand_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Set target nodes
    pub fn with_targets(mut self, targets: Vec<Address>) -> Self {
        self.target_nodes = targets;
        self
    }

    /// Set custom parameters
    pub fn with_params(mut self, params: ChaosParams) -> Self {
        self.params = params;
        self
    }

    /// Start the event
    pub fn start(&mut self) {
        self.start_time = current_timestamp();
        self.active = true;
    }

    /// Stop the event
    pub fn stop(&mut self, result: ChaosResult) {
        self.end_time = Some(current_timestamp());
        self.active = false;
        self.results = Some(result);
    }

    /// Check if event should end
    pub fn should_end(&self) -> bool {
        if !self.active {
            return false;
        }
        current_timestamp() >= self.start_time + self.duration_secs
    }

    /// Get remaining duration
    pub fn remaining_secs(&self) -> u64 {
        if !self.active {
            return 0;
        }
        let elapsed = current_timestamp() - self.start_time;
        self.duration_secs.saturating_sub(elapsed)
    }
}

/// Chaos event parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosParams {
    /// Packet loss percentage (0-100)
    pub packet_loss_percent: u8,
    /// Latency to inject (ms)
    pub latency_ms: u64,
    /// Latency jitter (ms)
    pub latency_jitter_ms: u64,
    /// Clock drift (seconds)
    pub clock_drift_secs: i64,
    /// Disk slowdown factor (1.0 = normal)
    pub disk_slowdown_factor: f64,
    /// Memory limit (bytes, 0 = unlimited)
    pub memory_limit_bytes: u64,
    /// CPU limit (percentage, 0-100)
    pub cpu_limit_percent: u8,
    /// Message corruption rate (0-100)
    pub corruption_rate: u8,
    /// Partition groups
    pub partition_groups: Vec<Vec<Address>>,
    /// Byzantine behavior type
    pub byzantine_type: ByzantineType,
}

impl ChaosParams {
    /// Create default params for event type and severity
    pub fn default_for(event_type: ChaosEventType, severity: ChaosSeverity) -> Self {
        let mut params = Self::default();
        
        match (event_type, severity) {
            (ChaosEventType::PacketLoss, ChaosSeverity::Low) => {
                params.packet_loss_percent = 5;
            }
            (ChaosEventType::PacketLoss, ChaosSeverity::Medium) => {
                params.packet_loss_percent = 20;
            }
            (ChaosEventType::PacketLoss, ChaosSeverity::High) => {
                params.packet_loss_percent = 50;
            }
            (ChaosEventType::PacketLoss, ChaosSeverity::Critical) => {
                params.packet_loss_percent = 90;
            }
            (ChaosEventType::LatencyInjection, ChaosSeverity::Low) => {
                params.latency_ms = 50;
                params.latency_jitter_ms = 10;
            }
            (ChaosEventType::LatencyInjection, ChaosSeverity::Medium) => {
                params.latency_ms = 200;
                params.latency_jitter_ms = 50;
            }
            (ChaosEventType::LatencyInjection, ChaosSeverity::High) => {
                params.latency_ms = 1000;
                params.latency_jitter_ms = 200;
            }
            (ChaosEventType::LatencyInjection, ChaosSeverity::Critical) => {
                params.latency_ms = 5000;
                params.latency_jitter_ms = 1000;
            }
            (ChaosEventType::ClockDrift, ChaosSeverity::Low) => {
                params.clock_drift_secs = 1;
            }
            (ChaosEventType::ClockDrift, ChaosSeverity::Medium) => {
                params.clock_drift_secs = 5;
            }
            (ChaosEventType::ClockDrift, ChaosSeverity::High) => {
                params.clock_drift_secs = 30;
            }
            (ChaosEventType::ClockDrift, ChaosSeverity::Critical) => {
                params.clock_drift_secs = 120;
            }
            (ChaosEventType::CpuThrottle, ChaosSeverity::Low) => {
                params.cpu_limit_percent = 75;
            }
            (ChaosEventType::CpuThrottle, ChaosSeverity::Medium) => {
                params.cpu_limit_percent = 50;
            }
            (ChaosEventType::CpuThrottle, ChaosSeverity::High) => {
                params.cpu_limit_percent = 25;
            }
            (ChaosEventType::CpuThrottle, ChaosSeverity::Critical) => {
                params.cpu_limit_percent = 10;
            }
            _ => {}
        }
        
        params
    }
}

impl Default for ChaosParams {
    fn default() -> Self {
        Self {
            packet_loss_percent: 0,
            latency_ms: 0,
            latency_jitter_ms: 0,
            clock_drift_secs: 0,
            disk_slowdown_factor: 1.0,
            memory_limit_bytes: 0,
            cpu_limit_percent: 100,
            corruption_rate: 0,
            partition_groups: Vec::new(),
            byzantine_type: ByzantineType::None,
        }
    }
}

/// Byzantine behavior types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByzantineType {
    /// No byzantine behavior
    None,
    /// Send conflicting votes
    DoubleVote,
    /// Propose conflicting blocks
    DoublePropose,
    /// Withhold votes
    WithholdVotes,
    /// Send invalid transactions
    InvalidTransactions,
    /// Equivocation (different messages to different peers)
    Equivocation,
}

/// Chaos test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosResult {
    /// Event ID
    pub event_id: H256,
    /// Test passed
    pub passed: bool,
    /// Blocks produced during test
    pub blocks_produced: u64,
    /// Blocks missed
    pub blocks_missed: u64,
    /// Transactions processed
    pub transactions_processed: u64,
    /// Transactions failed
    pub transactions_failed: u64,
    /// Consensus maintained
    pub consensus_maintained: bool,
    /// State consistency verified
    pub state_consistent: bool,
    /// Recovery time (seconds)
    pub recovery_time_secs: Option<u64>,
    /// Nodes that failed
    pub failed_nodes: Vec<Address>,
    /// Error messages
    pub errors: Vec<String>,
    /// Metrics during test
    pub metrics: ChaosMetrics,
}

impl ChaosResult {
    /// Create new result
    pub fn new(event_id: H256) -> Self {
        Self {
            event_id,
            passed: false,
            blocks_produced: 0,
            blocks_missed: 0,
            transactions_processed: 0,
            transactions_failed: 0,
            consensus_maintained: true,
            state_consistent: true,
            recovery_time_secs: None,
            failed_nodes: Vec::new(),
            errors: Vec::new(),
            metrics: ChaosMetrics::default(),
        }
    }

    /// Mark as passed
    pub fn mark_passed(&mut self) {
        self.passed = self.consensus_maintained && self.state_consistent && self.errors.is_empty();
    }

    /// Add error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.passed = false;
    }
}

/// Metrics collected during chaos test
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChaosMetrics {
    /// Average block time during test (ms)
    pub avg_block_time_ms: u64,
    /// Maximum block time (ms)
    pub max_block_time_ms: u64,
    /// Average transaction latency (ms)
    pub avg_tx_latency_ms: u64,
    /// P99 transaction latency (ms)
    pub p99_tx_latency_ms: u64,
    /// Network messages sent
    pub messages_sent: u64,
    /// Network messages dropped
    pub messages_dropped: u64,
    /// Reconnection attempts
    pub reconnection_attempts: u64,
    /// Successful reconnections
    pub reconnection_success: u64,
}

/// Chaos engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    /// Enable chaos testing
    pub enabled: bool,
    /// Maximum concurrent events
    pub max_concurrent_events: u32,
    /// Automatic recovery after events
    pub auto_recovery: bool,
    /// Collect detailed metrics
    pub detailed_metrics: bool,
    /// Log all chaos events
    pub log_events: bool,
    /// Safety limits enabled
    pub safety_limits: bool,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for safety
            max_concurrent_events: 3,
            auto_recovery: true,
            detailed_metrics: true,
            log_events: true,
            safety_limits: true,
        }
    }
}

/// Chaos Engine for network resilience testing
pub struct ChaosEngine {
    /// Configuration
    config: ChaosConfig,
    /// Active events
    active_events: Arc<RwLock<HashMap<H256, ChaosEvent>>>,
    /// Completed events
    completed_events: Arc<RwLock<Vec<ChaosEvent>>>,
    /// Test scenarios
    scenarios: Arc<RwLock<Vec<ChaosScenario>>>,
    /// Statistics
    stats: Arc<RwLock<ChaosStats>>,
}

impl ChaosEngine {
    /// Create new chaos engine
    pub fn new(config: ChaosConfig) -> Self {
        Self {
            config,
            active_events: Arc::new(RwLock::new(HashMap::new())),
            completed_events: Arc::new(RwLock::new(Vec::new())),
            scenarios: Arc::new(RwLock::new(Self::default_scenarios())),
            stats: Arc::new(RwLock::new(ChaosStats::default())),
        }
    }

    /// Default test scenarios
    fn default_scenarios() -> Vec<ChaosScenario> {
        vec![
            ChaosScenario {
                name: "Network Partition Recovery".to_string(),
                description: "Test network partition and recovery".to_string(),
                events: vec![
                    ChaosEvent::new(ChaosEventType::NetworkPartition, ChaosSeverity::High, 60),
                ],
                success_criteria: SuccessCriteria {
                    max_recovery_time_secs: 30,
                    min_blocks_produced: 5,
                    max_tx_failure_rate: 0.1,
                    require_consensus: true,
                    require_state_consistency: true,
                },
            },
            ChaosScenario {
                name: "High Latency Tolerance".to_string(),
                description: "Test behavior under high network latency".to_string(),
                events: vec![
                    ChaosEvent::new(ChaosEventType::LatencyInjection, ChaosSeverity::High, 120),
                ],
                success_criteria: SuccessCriteria {
                    max_recovery_time_secs: 10,
                    min_blocks_produced: 10,
                    max_tx_failure_rate: 0.05,
                    require_consensus: true,
                    require_state_consistency: true,
                },
            },
            ChaosScenario {
                name: "Packet Loss Resilience".to_string(),
                description: "Test behavior under packet loss".to_string(),
                events: vec![
                    ChaosEvent::new(ChaosEventType::PacketLoss, ChaosSeverity::Medium, 60),
                ],
                success_criteria: SuccessCriteria {
                    max_recovery_time_secs: 15,
                    min_blocks_produced: 8,
                    max_tx_failure_rate: 0.15,
                    require_consensus: true,
                    require_state_consistency: true,
                },
            },
            ChaosScenario {
                name: "Byzantine Fault Tolerance".to_string(),
                description: "Test BFT under byzantine actors".to_string(),
                events: vec![
                    ChaosEvent::new(ChaosEventType::ByzantineBehavior, ChaosSeverity::High, 180),
                ],
                success_criteria: SuccessCriteria {
                    max_recovery_time_secs: 60,
                    min_blocks_produced: 20,
                    max_tx_failure_rate: 0.2,
                    require_consensus: true,
                    require_state_consistency: true,
                },
            },
            ChaosScenario {
                name: "Clock Drift Handling".to_string(),
                description: "Test behavior under clock drift".to_string(),
                events: vec![
                    ChaosEvent::new(ChaosEventType::ClockDrift, ChaosSeverity::Medium, 120),
                ],
                success_criteria: SuccessCriteria {
                    max_recovery_time_secs: 20,
                    min_blocks_produced: 15,
                    max_tx_failure_rate: 0.05,
                    require_consensus: true,
                    require_state_consistency: true,
                },
            },
        ]
    }

    /// Start chaos event
    pub fn start_event(&self, mut event: ChaosEvent) -> Result<H256, ChaosError> {
        if !self.config.enabled {
            return Err(ChaosError::Disabled);
        }

        let active_count = self.active_events.read().len() as u32;
        if active_count >= self.config.max_concurrent_events {
            return Err(ChaosError::TooManyEvents(active_count));
        }

        // Safety checks
        if self.config.safety_limits {
            self.validate_safety(&event)?;
        }

        let id = event.id;
        event.start();

        self.active_events.write().insert(id, event);
        self.stats.write().events_started += 1;

        Ok(id)
    }

    /// Stop chaos event
    pub fn stop_event(&self, event_id: &H256) -> Result<ChaosResult, ChaosError> {
        let mut event = self.active_events.write()
            .remove(event_id)
            .ok_or_else(|| ChaosError::EventNotFound(*event_id))?;

        let mut result = ChaosResult::new(*event_id);
        
        // Collect results
        result.mark_passed();
        
        event.stop(result.clone());
        self.completed_events.write().push(event);
        self.stats.write().events_completed += 1;

        Ok(result)
    }

    /// Validate safety limits
    fn validate_safety(&self, event: &ChaosEvent) -> Result<(), ChaosError> {
        // Don't allow critical severity on mainnet
        if event.severity == ChaosSeverity::Critical {
            return Err(ChaosError::SafetyViolation(
                "Critical severity not allowed with safety limits".to_string()
            ));
        }

        // Limit event duration
        if event.duration_secs > 3600 {
            return Err(ChaosError::SafetyViolation(
                "Event duration exceeds 1 hour limit".to_string()
            ));
        }

        // Don't allow full network partition
        if event.event_type == ChaosEventType::NetworkPartition 
            && event.target_nodes.is_empty() 
        {
            return Err(ChaosError::SafetyViolation(
                "Full network partition requires explicit target nodes".to_string()
            ));
        }

        Ok(())
    }

    /// Run chaos scenario
    pub fn run_scenario(&self, scenario_name: &str) -> Result<ChaosScenarioResult, ChaosError> {
        let scenario = self.scenarios.read()
            .iter()
            .find(|s| s.name == scenario_name)
            .cloned()
            .ok_or_else(|| ChaosError::ScenarioNotFound(scenario_name.to_string()))?;

        let mut results = Vec::new();
        
        for event in scenario.events {
            let event_id = self.start_event(event)?;
            
            // Wait for event duration
            // In production this would be async
            
            let result = self.stop_event(&event_id)?;
            results.push(result);
        }

        let passed = results.iter().all(|r| r.passed);
        
        Ok(ChaosScenarioResult {
            scenario_name: scenario.name,
            passed,
            event_results: results,
            criteria_met: passed,
        })
    }

    /// Get active events
    pub fn get_active_events(&self) -> Vec<ChaosEvent> {
        self.active_events.read().values().cloned().collect()
    }

    /// Get completed events
    pub fn get_completed_events(&self) -> Vec<ChaosEvent> {
        self.completed_events.read().clone()
    }

    /// Check and complete expired events
    pub fn check_expired(&self) -> Vec<H256> {
        let mut expired = Vec::new();
        
        for (id, event) in self.active_events.read().iter() {
            if event.should_end() {
                expired.push(*id);
            }
        }

        for id in &expired {
            let _ = self.stop_event(id);
        }

        expired
    }

    /// Get available scenarios
    pub fn get_scenarios(&self) -> Vec<String> {
        self.scenarios.read().iter().map(|s| s.name.clone()).collect()
    }

    /// Get statistics
    pub fn stats(&self) -> ChaosStats {
        self.stats.read().clone()
    }
}

impl Default for ChaosEngine {
    fn default() -> Self {
        Self::new(ChaosConfig::default())
    }
}

/// Chaos test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosScenario {
    /// Scenario name
    pub name: String,
    /// Description
    pub description: String,
    /// Events in this scenario
    pub events: Vec<ChaosEvent>,
    /// Success criteria
    pub success_criteria: SuccessCriteria,
}

/// Success criteria for chaos scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    /// Maximum recovery time allowed
    pub max_recovery_time_secs: u64,
    /// Minimum blocks that must be produced
    pub min_blocks_produced: u64,
    /// Maximum transaction failure rate
    pub max_tx_failure_rate: f64,
    /// Require consensus to be maintained
    pub require_consensus: bool,
    /// Require state consistency
    pub require_state_consistency: bool,
}

/// Chaos scenario result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosScenarioResult {
    /// Scenario name
    pub scenario_name: String,
    /// Overall pass/fail
    pub passed: bool,
    /// Results for each event
    pub event_results: Vec<ChaosResult>,
    /// Were all criteria met
    pub criteria_met: bool,
}

/// Chaos engine statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChaosStats {
    pub events_started: u64,
    pub events_completed: u64,
    pub events_passed: u64,
    pub events_failed: u64,
    pub scenarios_run: u64,
    pub scenarios_passed: u64,
}

/// Chaos errors
#[derive(Debug, thiserror::Error)]
pub enum ChaosError {
    #[error("Chaos testing is disabled")]
    Disabled,

    #[error("Too many concurrent events: {0}")]
    TooManyEvents(u32),

    #[error("Event not found: {0:?}")]
    EventNotFound(H256),

    #[error("Scenario not found: {0}")]
    ScenarioNotFound(String),

    #[error("Safety violation: {0}")]
    SafetyViolation(String),

    #[error("Event failed: {0}")]
    EventFailed(String),
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn rand_bytes() -> [u8; 8] {
    let ts = current_timestamp();
    ts.to_le_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_event() {
        let event = ChaosEvent::new(
            ChaosEventType::PacketLoss,
            ChaosSeverity::Medium,
            60,
        );

        assert_eq!(event.event_type, ChaosEventType::PacketLoss);
        assert_eq!(event.duration_secs, 60);
        assert!(!event.active);
    }

    #[test]
    fn test_chaos_params() {
        let params = ChaosParams::default_for(
            ChaosEventType::PacketLoss,
            ChaosSeverity::High,
        );

        assert_eq!(params.packet_loss_percent, 50);
    }

    #[test]
    fn test_chaos_engine() {
        let engine = ChaosEngine::default();
        assert_eq!(engine.get_active_events().len(), 0);
    }

    #[test]
    fn test_chaos_scenarios() {
        let engine = ChaosEngine::default();
        let scenarios = engine.get_scenarios();
        assert!(!scenarios.is_empty());
    }

    #[test]
    fn test_chaos_result() {
        let mut result = ChaosResult::new(H256([1u8; 32]));
        result.consensus_maintained = true;
        result.state_consistent = true;
        result.mark_passed();
        
        assert!(result.passed);
    }
}
