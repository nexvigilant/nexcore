//! # Claude Code urinary system types
//!
//! Specific implementations for Claude Code's log management, telemetry pruning,
//! session expiry, and retention policies.

use serde::{Deserialize, Serialize};

/// Telemetry data pruning operation.
///
/// Biological mapping: Excretion of metabolic waste products.
///
/// Type tier: T2-C (∝ irreversibility + N quantity + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryPruning {
    /// Source of telemetry (e.g., "watchtower", "guardian")
    pub source: String,
    /// Count before pruning
    pub before_count: usize,
    /// Count after pruning
    pub after_count: usize,
    /// Bytes freed by pruning
    pub bytes_freed: u64,
}

impl TelemetryPruning {
    /// Create a new telemetry pruning record.
    pub fn new(source: String, before_count: usize, after_count: usize, bytes_freed: u64) -> Self {
        Self {
            source,
            before_count,
            after_count,
            bytes_freed,
        }
    }

    /// Get pruning rate (fraction removed, 0.0 to 1.0).
    pub fn pruning_rate(&self) -> f64 {
        if self.before_count == 0 {
            0.0
        } else {
            (self.before_count - self.after_count) as f64 / self.before_count as f64
        }
    }
}

/// Session expiry and garbage collection.
///
/// Biological mapping: Excretion of aged cellular components (autophagy).
///
/// Type tier: T2-C (∝ irreversibility + ν frequency + ∃ existence)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionExpiry {
    /// Brain session identifier
    pub session_id: String,
    /// Age of session in days
    pub age_days: u32,
    /// Whether session has expired
    pub expired: bool,
    /// Number of artifacts preserved from this session
    pub artifacts_preserved: usize,
}

impl SessionExpiry {
    /// Create a new session expiry record.
    pub fn new(
        session_id: String,
        age_days: u32,
        expired: bool,
        artifacts_preserved: usize,
    ) -> Self {
        Self {
            session_id,
            age_days,
            expired,
            artifacts_preserved,
        }
    }

    /// Check if session should be expired based on age threshold.
    pub fn should_expire(&self, threshold_days: u32) -> bool {
        self.age_days > threshold_days
    }
}

/// Artifact retention lifecycle management.
///
/// Biological mapping: Selective reabsorption of valuable substances.
///
/// Type tier: T2-C (π persistence + ν frequency + κ comparison)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactRetention {
    /// Artifact identifier
    pub artifact_id: String,
    /// Retention classification (e.g., "ephemeral", "standard", "archival")
    pub retention_class: String,
    /// Age of artifact in days
    pub age_days: u32,
    /// Whether artifact has been resolved/completed
    pub resolved: bool,
}

impl ArtifactRetention {
    /// Create a new artifact retention record.
    pub fn new(
        artifact_id: String,
        retention_class: String,
        age_days: u32,
        resolved: bool,
    ) -> Self {
        Self {
            artifact_id,
            retention_class,
            age_days,
            resolved,
        }
    }

    /// Check if artifact should be retained based on class and resolution status.
    pub fn should_retain(&self, max_age_days: u32) -> bool {
        if self.retention_class == "archival" {
            true
        } else if self.retention_class == "ephemeral" {
            self.age_days <= 7
        } else if self.resolved {
            self.age_days <= 30
        } else {
            self.age_days <= max_age_days
        }
    }
}

/// Log rotation management.
///
/// Biological mapping: Bladder filling and periodic emptying.
///
/// Type tier: T2-C (σ sequence + N quantity + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogRotation {
    /// Path to log file
    pub log_path: String,
    /// Maximum size before rotation (MB)
    pub max_size_mb: u64,
    /// Current size (MB)
    pub current_size_mb: u64,
    /// Number of rotations performed
    pub rotations: u32,
}

impl LogRotation {
    /// Create a new log rotation record.
    pub fn new(log_path: String, max_size_mb: u64, current_size_mb: u64, rotations: u32) -> Self {
        Self {
            log_path,
            max_size_mb,
            current_size_mb,
            rotations,
        }
    }

    /// Check if log should be rotated.
    pub fn should_rotate(&self) -> bool {
        self.current_size_mb >= self.max_size_mb
    }

    /// Get utilization as fraction of max size (0.0 to 1.0+).
    pub fn utilization(&self) -> f64 {
        self.current_size_mb as f64 / self.max_size_mb as f64
    }
}

/// Data retention policy specification.
///
/// Biological mapping: Tubular reabsorption selectivity rules.
///
/// Type tier: T2-C (κ comparison + ∂ boundary + ν frequency)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Category of data (e.g., "sessions", "telemetry", "logs")
    pub category: String,
    /// Maximum age in days before disposal
    pub max_age_days: u32,
    /// Maximum count to retain
    pub max_count: usize,
    /// Auto-prune when limits exceeded
    pub auto_prune: bool,
}

impl RetentionPolicy {
    /// Create a new retention policy.
    pub fn new(category: String, max_age_days: u32, max_count: usize, auto_prune: bool) -> Self {
        Self {
            category,
            max_age_days,
            max_count,
            auto_prune,
        }
    }

    /// Check if an item should be retained based on age and count.
    pub fn should_retain(&self, age_days: u32, current_count: usize) -> bool {
        age_days <= self.max_age_days && current_count < self.max_count
    }
}

/// Decision audit log cleanup operation.
///
/// Biological mapping: Excretion of metabolic byproducts.
///
/// Type tier: T2-C (∝ irreversibility + N quantity + κ comparison)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionAuditCleanup {
    /// Total audit entries before cleanup
    pub total_entries: usize,
    /// Entries pruned
    pub pruned_entries: usize,
    /// Entries kept
    pub kept_entries: usize,
    /// Age in days of oldest kept entry
    pub oldest_kept_days: u32,
}

impl DecisionAuditCleanup {
    /// Create a new decision audit cleanup record.
    pub fn new(total_entries: usize, pruned_entries: usize, oldest_kept_days: u32) -> Self {
        Self {
            total_entries,
            pruned_entries,
            kept_entries: total_entries.saturating_sub(pruned_entries),
            oldest_kept_days,
        }
    }

    /// Get retention rate (fraction kept, 0.0 to 1.0).
    pub fn retention_rate(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            self.kept_entries as f64 / self.total_entries as f64
        }
    }
}

/// Categories of waste in the system.
///
/// Biological mapping: Types of waste products excreted by kidneys.
///
/// Type tier: T2-P (Σ sum, enumeration over waste domains)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasteCategory {
    /// Expired session data
    StaleSessions,
    /// Resolved or expired artifacts
    ExpiredArtifacts,
    /// Old telemetry data
    OldTelemetry,
    /// Rotated log files
    RotatedLogs,
    /// Temporary files
    TempFiles,
}

/// Methods for disposing of waste.
///
/// Biological mapping: Routes of excretion (renal, fecal, respiratory).
///
/// Type tier: T2-P (Σ sum + ∝ irreversibility, enumeration over disposal methods)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisposalMethod {
    /// Permanent deletion
    Delete,
    /// Move to archive storage
    Archive,
    /// Compress to reduce size
    Compress,
    /// Strip identifying information
    Anonymize,
}

/// Silent failure risk assessment for urinary system.
///
/// Biological mapping: Acute kidney injury — sudden decline in kidney function
/// without obvious symptoms until severe.
///
/// Type tier: T2-C (∅ void + κ comparison + N quantity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentFailureRisk {
    /// Timestamp of last health check
    pub last_check_timestamp: String,
    /// Number of nephrons actively filtering
    pub nephrons_active: usize,
    /// File paths not being monitored for cleanup
    pub unmonitored_paths: Vec<String>,
    /// Risk level (0.0 = no risk, 1.0 = imminent failure)
    pub risk_level: f64,
}

impl SilentFailureRisk {
    /// Create a new silent failure risk assessment.
    pub fn new(
        last_check_timestamp: String,
        nephrons_active: usize,
        unmonitored_paths: Vec<String>,
    ) -> Self {
        let base_risk = if nephrons_active == 0 { 0.8 } else { 0.0 };
        let path_risk = (unmonitored_paths.len() as f64 * 0.05).min(0.5);
        let risk_level = (base_risk + path_risk).min(1.0);

        Self {
            last_check_timestamp,
            nephrons_active,
            unmonitored_paths,
            risk_level,
        }
    }

    /// Check if risk level is critical (> 0.7).
    pub fn is_critical(&self) -> bool {
        self.risk_level > 0.7
    }
}

/// Overall urinary system health for Claude Code.
///
/// Biological mapping: Comprehensive renal function panel.
///
/// Type tier: T2-C (ς state + κ comparison + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrinarySystemHealth {
    /// Telemetry pruning active
    pub telemetry_pruned: bool,
    /// Session garbage collection active
    pub sessions_gc_active: bool,
    /// Log rotation configured
    pub log_rotation_active: bool,
    /// Silent failure risk score (0.0-1.0)
    pub silent_failure_risk: f64,
    /// Number of active retention policies
    pub retention_policies_count: usize,
}

impl UrinarySystemHealth {
    /// Create a new urinary system health record.
    pub fn new(
        telemetry_pruned: bool,
        sessions_gc_active: bool,
        log_rotation_active: bool,
        silent_failure_risk: f64,
        retention_policies_count: usize,
    ) -> Self {
        Self {
            telemetry_pruned,
            sessions_gc_active,
            log_rotation_active,
            silent_failure_risk,
            retention_policies_count,
        }
    }

    /// Check if system is healthy overall.
    pub fn is_healthy(&self) -> bool {
        self.telemetry_pruned
            && self.sessions_gc_active
            && self.log_rotation_active
            && self.silent_failure_risk < 0.5
            && self.retention_policies_count > 0
    }

    /// Get health score (0.0 to 1.0).
    pub fn health_score(&self) -> f64 {
        let mut score = 0.0;

        if self.telemetry_pruned {
            score += 0.2;
        }
        if self.sessions_gc_active {
            score += 0.2;
        }
        if self.log_rotation_active {
            score += 0.2;
        }
        if self.silent_failure_risk < 0.5 {
            score += 0.2;
        }
        if self.retention_policies_count > 0 {
            score += 0.2;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_pruning_rate() {
        let pruning = TelemetryPruning::new("watchtower".to_string(), 100, 25, 75000);
        assert!((pruning.pruning_rate() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_session_expiry_should_expire() {
        let expiry = SessionExpiry::new("sess_123".to_string(), 45, false, 3);
        assert!(expiry.should_expire(30));
        assert!(!expiry.should_expire(60));
    }

    #[test]
    fn test_artifact_retention_archival() {
        let artifact =
            ArtifactRetention::new("art_123".to_string(), "archival".to_string(), 365, true);
        assert!(artifact.should_retain(90));
    }

    #[test]
    fn test_artifact_retention_ephemeral() {
        let artifact =
            ArtifactRetention::new("art_456".to_string(), "ephemeral".to_string(), 10, false);
        assert!(!artifact.should_retain(30));
    }

    #[test]
    fn test_log_rotation_should_rotate() {
        let log = LogRotation::new("/var/log/app.log".to_string(), 100, 105, 5);
        assert!(log.should_rotate());
    }

    #[test]
    fn test_log_rotation_utilization() {
        let log = LogRotation::new("/var/log/app.log".to_string(), 100, 80, 3);
        assert!((log.utilization() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_retention_policy_should_retain() {
        let policy = RetentionPolicy::new("sessions".to_string(), 30, 100, true);
        assert!(policy.should_retain(20, 50));
        assert!(!policy.should_retain(40, 50));
        assert!(!policy.should_retain(20, 150));
    }

    #[test]
    fn test_decision_audit_cleanup_retention_rate() {
        let cleanup = DecisionAuditCleanup::new(200, 150, 30);
        assert!((cleanup.retention_rate() - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_silent_failure_risk_no_nephrons() {
        let risk = SilentFailureRisk::new("2026-02-10T12:00:00Z".to_string(), 0, vec![]);
        assert!(risk.is_critical());
    }

    #[test]
    fn test_silent_failure_risk_many_unmonitored() {
        let paths = vec![
            "/path1".to_string(),
            "/path2".to_string(),
            "/path3".to_string(),
            "/path4".to_string(),
            "/path5".to_string(),
        ];
        let risk = SilentFailureRisk::new("2026-02-10T12:00:00Z".to_string(), 5, paths);
        assert!(risk.risk_level > 0.2);
    }

    #[test]
    fn test_urinary_system_health_is_healthy() {
        let health = UrinarySystemHealth::new(true, true, true, 0.2, 5);
        assert!(health.is_healthy());
    }

    #[test]
    fn test_urinary_system_health_unhealthy_high_risk() {
        let health = UrinarySystemHealth::new(true, true, true, 0.8, 5);
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_urinary_system_health_score() {
        let health = UrinarySystemHealth::new(true, true, true, 0.3, 5);
        assert!((health.health_score() - 1.0).abs() < 0.01);
    }
}
