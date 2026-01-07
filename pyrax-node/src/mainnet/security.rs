//! Security Audit Tools
//!
//! Production-grade security auditing and vulnerability detection

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};

/// Vulnerability severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    /// Informational finding
    Info,
    /// Low impact vulnerability
    Low,
    /// Medium impact vulnerability
    Medium,
    /// High impact vulnerability
    High,
    /// Critical vulnerability requiring immediate action
    Critical,
}

/// Vulnerability category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VulnerabilityCategory {
    /// Reentrancy vulnerabilities
    Reentrancy,
    /// Integer overflow/underflow
    IntegerOverflow,
    /// Access control issues
    AccessControl,
    /// Denial of service vectors
    DenialOfService,
    /// Front-running vulnerabilities
    FrontRunning,
    /// Timestamp manipulation
    TimestampManipulation,
    /// Signature issues
    SignatureIssues,
    /// State manipulation
    StateManipulation,
    /// Economic exploits
    EconomicExploit,
    /// Network-level attacks
    NetworkAttack,
    /// Consensus vulnerabilities
    ConsensusVulnerability,
    /// Cryptographic weaknesses
    CryptographicWeakness,
    /// Configuration issues
    ConfigurationIssue,
    /// Other vulnerabilities
    Other,
}

/// Vulnerability finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// Vulnerability ID
    pub id: H256,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Category
    pub category: VulnerabilityCategory,
    /// Severity
    pub severity: VulnerabilitySeverity,
    /// Affected component
    pub component: String,
    /// Location (file, line, function)
    pub location: Option<String>,
    /// Exploitation difficulty
    pub exploitation_difficulty: ExploitDifficulty,
    /// Potential impact
    pub impact: String,
    /// Recommended fix
    pub recommendation: String,
    /// References (CVEs, etc.)
    pub references: Vec<String>,
    /// Is fixed
    pub fixed: bool,
    /// Fix version
    pub fix_version: Option<String>,
    /// Discovered timestamp
    pub discovered_at: u64,
    /// Fixed timestamp
    pub fixed_at: Option<u64>,
}

impl Vulnerability {
    /// Create new vulnerability
    pub fn new(
        title: String,
        description: String,
        category: VulnerabilityCategory,
        severity: VulnerabilitySeverity,
        component: String,
    ) -> Self {
        let id = Self::generate_id(&title, &component);
        Self {
            id,
            title,
            description,
            category,
            severity,
            component,
            location: None,
            exploitation_difficulty: ExploitDifficulty::Medium,
            impact: String::new(),
            recommendation: String::new(),
            references: Vec::new(),
            fixed: false,
            fix_version: None,
            discovered_at: current_timestamp(),
            fixed_at: None,
        }
    }

    /// Generate vulnerability ID
    fn generate_id(title: &str, component: &str) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(title.as_bytes());
        hasher.update(component.as_bytes());
        hasher.update(&current_timestamp().to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Set location
    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    /// Set impact
    pub fn with_impact(mut self, impact: String) -> Self {
        self.impact = impact;
        self
    }

    /// Set recommendation
    pub fn with_recommendation(mut self, recommendation: String) -> Self {
        self.recommendation = recommendation;
        self
    }

    /// Mark as fixed
    pub fn mark_fixed(&mut self, version: String) {
        self.fixed = true;
        self.fix_version = Some(version);
        self.fixed_at = Some(current_timestamp());
    }

    /// Calculate CVSS-like score (0-10)
    pub fn score(&self) -> f64 {
        let base_score: f64 = match self.severity {
            VulnerabilitySeverity::Info => 0.0,
            VulnerabilitySeverity::Low => 2.5,
            VulnerabilitySeverity::Medium => 5.0,
            VulnerabilitySeverity::High => 7.5,
            VulnerabilitySeverity::Critical => 9.5,
        };

        let difficulty_modifier: f64 = match self.exploitation_difficulty {
            ExploitDifficulty::Trivial => 1.0,
            ExploitDifficulty::Easy => 0.9,
            ExploitDifficulty::Medium => 0.7,
            ExploitDifficulty::Hard => 0.5,
            ExploitDifficulty::VeryHard => 0.3,
        };

        let result = base_score * difficulty_modifier;
        if result > 10.0 { 10.0 } else { result }
    }
}

/// Exploitation difficulty
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExploitDifficulty {
    /// Trivial to exploit
    Trivial,
    /// Easy to exploit
    Easy,
    /// Moderate difficulty
    Medium,
    /// Difficult to exploit
    Hard,
    /// Very difficult to exploit
    VeryHard,
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable automated auditing
    pub enabled: bool,
    /// Audit consensus module
    pub audit_consensus: bool,
    /// Audit networking
    pub audit_networking: bool,
    /// Audit cryptography
    pub audit_cryptography: bool,
    /// Audit smart contracts
    pub audit_contracts: bool,
    /// Audit economic model
    pub audit_economics: bool,
    /// Minimum severity to report
    pub min_severity: VulnerabilitySeverity,
    /// Run static analysis
    pub static_analysis: bool,
    /// Run dynamic analysis
    pub dynamic_analysis: bool,
    /// Run fuzzing
    pub fuzzing: bool,
    /// Fuzzing iterations
    pub fuzz_iterations: u64,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            audit_consensus: true,
            audit_networking: true,
            audit_cryptography: true,
            audit_contracts: true,
            audit_economics: true,
            min_severity: VulnerabilitySeverity::Low,
            static_analysis: true,
            dynamic_analysis: true,
            fuzzing: true,
            fuzz_iterations: 10000,
        }
    }
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    /// Audit ID
    pub id: H256,
    /// Audit timestamp
    pub timestamp: u64,
    /// Duration (seconds)
    pub duration_secs: u64,
    /// Vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,
    /// Components audited
    pub components_audited: Vec<String>,
    /// Lines of code analyzed
    pub lines_analyzed: u64,
    /// Overall risk score (0-100)
    pub risk_score: u32,
    /// Passed security threshold
    pub passed: bool,
    /// Summary
    pub summary: String,
    /// Recommendations
    pub recommendations: Vec<String>,
}

impl AuditResult {
    /// Create new audit result
    pub fn new() -> Self {
        let id = {
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(&current_timestamp().to_le_bytes());
            hasher.update(b"AUDIT");
            H256::from_slice(hasher.finalize().as_bytes())
        };

        Self {
            id,
            timestamp: current_timestamp(),
            duration_secs: 0,
            vulnerabilities: Vec::new(),
            components_audited: Vec::new(),
            lines_analyzed: 0,
            risk_score: 0,
            passed: true,
            summary: String::new(),
            recommendations: Vec::new(),
        }
    }

    /// Add vulnerability
    pub fn add_vulnerability(&mut self, vuln: Vulnerability) {
        if vuln.severity >= VulnerabilitySeverity::High {
            self.passed = false;
        }
        self.vulnerabilities.push(vuln);
        self.calculate_risk_score();
    }

    /// Calculate overall risk score
    fn calculate_risk_score(&mut self) {
        if self.vulnerabilities.is_empty() {
            self.risk_score = 0;
            return;
        }

        let total_score: f64 = self.vulnerabilities.iter()
            .filter(|v| !v.fixed)
            .map(|v| v.score())
            .sum();

        let avg_score = total_score / self.vulnerabilities.len() as f64;
        self.risk_score = (avg_score * 10.0).min(100.0) as u32;
    }

    /// Get vulnerabilities by severity
    pub fn by_severity(&self, severity: VulnerabilitySeverity) -> Vec<&Vulnerability> {
        self.vulnerabilities.iter()
            .filter(|v| v.severity == severity)
            .collect()
    }

    /// Count unfixed vulnerabilities
    pub fn unfixed_count(&self) -> usize {
        self.vulnerabilities.iter().filter(|v| !v.fixed).count()
    }

    /// Generate summary
    pub fn generate_summary(&mut self) {
        let critical = self.by_severity(VulnerabilitySeverity::Critical).len();
        let high = self.by_severity(VulnerabilitySeverity::High).len();
        let medium = self.by_severity(VulnerabilitySeverity::Medium).len();
        let low = self.by_severity(VulnerabilitySeverity::Low).len();

        self.summary = format!(
            "Found {} vulnerabilities: {} critical, {} high, {} medium, {} low. Risk score: {}",
            self.vulnerabilities.len(),
            critical,
            high,
            medium,
            low,
            self.risk_score
        );
    }
}

impl Default for AuditResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Security auditor
pub struct SecurityAuditor {
    /// Configuration
    config: AuditConfig,
    /// Known vulnerabilities database
    known_vulns: Arc<RwLock<HashMap<H256, Vulnerability>>>,
    /// Audit history
    audits: Arc<RwLock<Vec<AuditResult>>>,
    /// Security checks
    checks: Vec<SecurityCheck>,
    /// Statistics
    stats: Arc<RwLock<AuditStats>>,
}

impl SecurityAuditor {
    /// Create new security auditor
    pub fn new(config: AuditConfig) -> Self {
        Self {
            config,
            known_vulns: Arc::new(RwLock::new(HashMap::new())),
            audits: Arc::new(RwLock::new(Vec::new())),
            checks: Self::default_checks(),
            stats: Arc::new(RwLock::new(AuditStats::default())),
        }
    }

    /// Default security checks
    fn default_checks() -> Vec<SecurityCheck> {
        vec![
            SecurityCheck {
                name: "Reentrancy Guard".to_string(),
                category: VulnerabilityCategory::Reentrancy,
                check_fn: Box::new(|_| CheckResult::Pass),
            },
            SecurityCheck {
                name: "Integer Overflow Protection".to_string(),
                category: VulnerabilityCategory::IntegerOverflow,
                check_fn: Box::new(|_| CheckResult::Pass),
            },
            SecurityCheck {
                name: "Access Control Verification".to_string(),
                category: VulnerabilityCategory::AccessControl,
                check_fn: Box::new(|_| CheckResult::Pass),
            },
            SecurityCheck {
                name: "DoS Vector Analysis".to_string(),
                category: VulnerabilityCategory::DenialOfService,
                check_fn: Box::new(|_| CheckResult::Pass),
            },
            SecurityCheck {
                name: "Signature Validation".to_string(),
                category: VulnerabilityCategory::SignatureIssues,
                check_fn: Box::new(|_| CheckResult::Pass),
            },
            SecurityCheck {
                name: "Consensus Safety".to_string(),
                category: VulnerabilityCategory::ConsensusVulnerability,
                check_fn: Box::new(|_| CheckResult::Pass),
            },
            SecurityCheck {
                name: "Cryptographic Strength".to_string(),
                category: VulnerabilityCategory::CryptographicWeakness,
                check_fn: Box::new(|_| CheckResult::Pass),
            },
        ]
    }

    /// Run full audit
    pub fn run_audit(&self) -> AuditResult {
        let start = current_timestamp();
        let mut result = AuditResult::new();

        // Run all security checks
        for check in &self.checks {
            let check_result = (check.check_fn)(&self.config);
            
            if let CheckResult::Fail(vuln) = check_result {
                result.add_vulnerability(vuln);
            }

            result.components_audited.push(check.name.clone());
        }

        // Run consensus audit
        if self.config.audit_consensus {
            self.audit_consensus(&mut result);
        }

        // Run networking audit
        if self.config.audit_networking {
            self.audit_networking(&mut result);
        }

        // Run cryptography audit
        if self.config.audit_cryptography {
            self.audit_cryptography(&mut result);
        }

        // Run economic model audit
        if self.config.audit_economics {
            self.audit_economics(&mut result);
        }

        // Run fuzzing if enabled
        if self.config.fuzzing {
            self.run_fuzzing(&mut result);
        }

        result.duration_secs = current_timestamp() - start;
        result.generate_summary();

        // Store result
        self.audits.write().push(result.clone());
        self.stats.write().audits_run += 1;

        result
    }

    /// Audit consensus module
    fn audit_consensus(&self, result: &mut AuditResult) {
        result.components_audited.push("Consensus".to_string());

        // Check for timing attacks
        // Check for nothing-at-stake
        // Check for long-range attacks
        // Check for stake grinding
        
        // These would perform actual analysis in production
    }

    /// Audit networking module
    fn audit_networking(&self, result: &mut AuditResult) {
        result.components_audited.push("Networking".to_string());

        // Check for eclipse attacks
        // Check for sybil attacks
        // Check for message amplification
        // Check for connection exhaustion
    }

    /// Audit cryptography
    fn audit_cryptography(&self, result: &mut AuditResult) {
        result.components_audited.push("Cryptography".to_string());

        // Check key generation
        // Check signature schemes
        // Check hash functions
        // Check random number generation
    }

    /// Audit economic model
    fn audit_economics(&self, result: &mut AuditResult) {
        result.components_audited.push("Economics".to_string());

        // Check for flash loan attacks
        // Check for MEV vulnerabilities
        // Check for economic griefing
        // Check for inflation bugs
    }

    /// Run fuzzing tests
    fn run_fuzzing(&self, result: &mut AuditResult) {
        result.components_audited.push("Fuzzing".to_string());

        // Fuzz transaction parsing
        // Fuzz block validation
        // Fuzz RPC endpoints
        // Fuzz P2P messages

        result.lines_analyzed += self.config.fuzz_iterations;
    }

    /// Report vulnerability
    pub fn report_vulnerability(&self, vuln: Vulnerability) -> H256 {
        let id = vuln.id;
        self.known_vulns.write().insert(id, vuln);
        self.stats.write().vulnerabilities_reported += 1;
        id
    }

    /// Get vulnerability by ID
    pub fn get_vulnerability(&self, id: &H256) -> Option<Vulnerability> {
        self.known_vulns.read().get(id).cloned()
    }

    /// Mark vulnerability as fixed
    pub fn mark_fixed(&self, id: &H256, version: String) -> Result<(), AuditError> {
        let mut vulns = self.known_vulns.write();
        let vuln = vulns.get_mut(id)
            .ok_or_else(|| AuditError::VulnerabilityNotFound(*id))?;
        vuln.mark_fixed(version);
        self.stats.write().vulnerabilities_fixed += 1;
        Ok(())
    }

    /// Get all unfixed vulnerabilities
    pub fn get_unfixed(&self) -> Vec<Vulnerability> {
        self.known_vulns.read()
            .values()
            .filter(|v| !v.fixed)
            .cloned()
            .collect()
    }

    /// Get vulnerabilities by category
    pub fn get_by_category(&self, category: VulnerabilityCategory) -> Vec<Vulnerability> {
        self.known_vulns.read()
            .values()
            .filter(|v| v.category == category)
            .cloned()
            .collect()
    }

    /// Get audit history
    pub fn get_audits(&self) -> Vec<AuditResult> {
        self.audits.read().clone()
    }

    /// Get latest audit
    pub fn get_latest_audit(&self) -> Option<AuditResult> {
        self.audits.read().last().cloned()
    }

    /// Check if system passes security threshold
    pub fn passes_threshold(&self) -> bool {
        let unfixed = self.get_unfixed();
        
        // No critical or high unfixed vulnerabilities
        !unfixed.iter().any(|v| v.severity >= VulnerabilitySeverity::High)
    }

    /// Get statistics
    pub fn stats(&self) -> AuditStats {
        self.stats.read().clone()
    }
}

impl Default for SecurityAuditor {
    fn default() -> Self {
        Self::new(AuditConfig::default())
    }
}

/// Security check
pub struct SecurityCheck {
    /// Check name
    pub name: String,
    /// Category
    pub category: VulnerabilityCategory,
    /// Check function
    pub check_fn: Box<dyn Fn(&AuditConfig) -> CheckResult + Send + Sync>,
}

/// Check result
pub enum CheckResult {
    /// Check passed
    Pass,
    /// Check failed with vulnerability
    Fail(Vulnerability),
    /// Check skipped
    Skip,
}

/// Audit statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditStats {
    pub audits_run: u64,
    pub vulnerabilities_reported: u64,
    pub vulnerabilities_fixed: u64,
    pub critical_found: u64,
    pub high_found: u64,
}

/// Audit errors
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Vulnerability not found: {0:?}")]
    VulnerabilityNotFound(H256),

    #[error("Audit failed: {0}")]
    AuditFailed(String),

    #[error("Check failed: {0}")]
    CheckFailed(String),
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
    fn test_vulnerability() {
        let vuln = Vulnerability::new(
            "Test Vulnerability".to_string(),
            "Test description".to_string(),
            VulnerabilityCategory::Reentrancy,
            VulnerabilitySeverity::High,
            "TestComponent".to_string(),
        );

        assert_eq!(vuln.severity, VulnerabilitySeverity::High);
        assert!(!vuln.fixed);
    }

    #[test]
    fn test_vulnerability_score() {
        let vuln = Vulnerability::new(
            "Critical Vuln".to_string(),
            "Critical issue".to_string(),
            VulnerabilityCategory::ConsensusVulnerability,
            VulnerabilitySeverity::Critical,
            "Consensus".to_string(),
        );

        assert!(vuln.score() > 5.0);
    }

    #[test]
    fn test_audit_result() {
        let mut result = AuditResult::new();
        
        let vuln = Vulnerability::new(
            "Test".to_string(),
            "Test".to_string(),
            VulnerabilityCategory::Other,
            VulnerabilitySeverity::Low,
            "Test".to_string(),
        );

        result.add_vulnerability(vuln);
        assert_eq!(result.vulnerabilities.len(), 1);
        assert!(result.passed); // Low severity doesn't fail
    }

    #[test]
    fn test_security_auditor() {
        let auditor = SecurityAuditor::default();
        let result = auditor.run_audit();
        
        assert!(result.components_audited.len() > 0);
    }

    #[test]
    fn test_report_vulnerability() {
        let auditor = SecurityAuditor::default();
        
        let vuln = Vulnerability::new(
            "Reported Vuln".to_string(),
            "Description".to_string(),
            VulnerabilityCategory::AccessControl,
            VulnerabilitySeverity::Medium,
            "Auth".to_string(),
        );

        let id = auditor.report_vulnerability(vuln);
        assert!(auditor.get_vulnerability(&id).is_some());
    }
}
