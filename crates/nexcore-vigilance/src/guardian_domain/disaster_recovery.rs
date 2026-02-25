//! Disaster Recovery Models for Healthcare Compliance.
//!
//! Data models for HIPAA/HITECH, FDA 21 CFR Part 11, ISO 27001, SOX, and GDPR
//! compliant disaster recovery operations including backup configuration,
//! recovery plans, and system health monitoring.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Recovery operation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryStatus {
    #[default]
    Active,
    BackupPending,
    BackupInProgress,
    BackupCompleted,
    BackupFailed,
    RecoveryPending,
    RecoveryInProgress,
    RecoveryCompleted,
    RecoveryFailed,
    Testing,
    Maintenance,
}

/// Types of backup operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BackupType {
    #[default]
    Full,
    Incremental,
    Differential,
    Continuous,
    Snapshot,
}

/// Healthcare data classification for recovery prioritization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataClassification {
    /// Life-critical data.
    CriticalPatientData,
    /// Regular patient records.
    StandardPatientData,
    /// Billing, insurance.
    FinancialData,
    /// Scheduling, inventory.
    OperationalData,
    /// Clinical research.
    ResearchData,
    /// HR, policies.
    AdministrativeData,
    /// Compliance logs.
    AuditData,
    /// Configuration, metadata.
    SystemData,
}

/// Supported compliance frameworks for disaster recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DRComplianceFramework {
    Hipaa,
    Hitech,
    Fda21CfrPart11,
    Iso27001,
    Sox,
    Gdpr,
    Nist,
    JointCommission,
}

/// Backup configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfiguration {
    pub backup_id: String,
    pub name: String,
    pub backup_type: BackupType,
    /// Cron expression for scheduling.
    pub schedule_cron: String,
    pub retention_days: i32,
    #[serde(default)]
    pub data_classifications: Vec<DataClassification>,
    #[serde(default)]
    pub compliance_frameworks: Vec<DRComplianceFramework>,
    #[serde(default = "default_true")]
    pub encryption_enabled: bool,
    #[serde(default = "default_true")]
    pub compression_enabled: bool,
    #[serde(default = "default_true")]
    pub verification_enabled: bool,
    #[serde(default = "default_true")]
    pub offsite_replication: bool,
    #[serde(default = "default_true")]
    pub geographic_distribution: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

fn default_true() -> bool {
    true
}

impl BackupConfiguration {
    /// Check if backup is compliant with HIPAA requirements.
    pub fn is_hipaa_compliant(&self) -> bool {
        self.encryption_enabled
            && self.offsite_replication
            && self
                .compliance_frameworks
                .contains(&DRComplianceFramework::Hipaa)
    }
}

/// Backup metadata for tracking and verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub backup_id: String,
    #[serde(default = "DateTime::now")]
    pub timestamp: DateTime,
    pub backup_type: BackupType,
    pub size_bytes: i64,
    pub file_count: i64,
    pub checksum: String,
    pub compression_ratio: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_key_id: Option<String>,
    #[serde(default)]
    pub data_classifications: Vec<DataClassification>,
    #[serde(default)]
    pub compliance_frameworks: Vec<DRComplianceFramework>,
    #[serde(default)]
    pub source_systems: Vec<String>,
    #[serde(default)]
    pub geographic_locations: Vec<String>,
    pub verification_status: String,
    pub retention_until: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl BackupMetadata {
    /// Check if backup is verified and valid.
    pub fn is_verified(&self) -> bool {
        self.verification_status == "verified" || self.verification_status == "passed"
    }

    /// Check if backup has expired.
    pub fn is_expired(&self) -> bool {
        DateTime::now() > self.retention_until
    }
}

/// Disaster recovery plan definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub plan_id: String,
    pub name: String,
    pub description: String,
    /// Recovery Time Objective in hours.
    pub rto_hours: i32,
    /// Recovery Point Objective in minutes.
    pub rpo_minutes: i32,
    #[serde(default)]
    pub priority_order: Vec<DataClassification>,
    #[serde(default)]
    pub critical_systems: Vec<String>,
    #[serde(default)]
    pub recovery_procedures: Vec<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub compliance_requirements: Vec<DRComplianceFramework>,
    #[serde(default)]
    pub contact_information: Vec<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_tested: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_results: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl RecoveryPlan {
    /// Check if plan needs testing (> 90 days since last test).
    pub fn needs_testing(&self) -> bool {
        match self.last_tested {
            Some(last) => {
                let days_since = (DateTime::now() - last).num_days();
                days_since > 90
            }
            None => true,
        }
    }

    /// Get RTO as a duration description.
    pub fn rto_description(&self) -> String {
        if self.rto_hours < 24 {
            format!("{} hours", self.rto_hours)
        } else {
            format!("{} days", self.rto_hours / 24)
        }
    }

    /// Get RPO as a duration description.
    pub fn rpo_description(&self) -> String {
        if self.rpo_minutes < 60 {
            format!("{} minutes", self.rpo_minutes)
        } else {
            format!("{} hours", self.rpo_minutes / 60)
        }
    }
}

/// Active recovery operation tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryOperation {
    pub operation_id: String,
    pub plan_id: String,
    pub initiated_by: String,
    #[serde(default = "DateTime::now")]
    pub initiated_at: DateTime,
    #[serde(default)]
    pub status: RecoveryStatus,
    #[serde(default)]
    pub target_systems: Vec<String>,
    #[serde(default)]
    pub data_classifications: Vec<DataClassification>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_completion: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_completion: Option<DateTime>,
    pub recovery_point: DateTime,
    #[serde(default)]
    pub progress_percentage: f64,
    #[serde(default)]
    pub status_message: String,
    #[serde(default)]
    pub compliance_validated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl RecoveryOperation {
    /// Check if operation is complete.
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            RecoveryStatus::RecoveryCompleted | RecoveryStatus::BackupCompleted
        )
    }

    /// Check if operation has failed.
    pub fn has_failed(&self) -> bool {
        matches!(
            self.status,
            RecoveryStatus::RecoveryFailed | RecoveryStatus::BackupFailed
        )
    }

    /// Check if operation is in progress.
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self.status,
            RecoveryStatus::RecoveryInProgress
                | RecoveryStatus::BackupInProgress
                | RecoveryStatus::Testing
        )
    }
}

/// System health monitoring for disaster recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    #[serde(default = "DateTime::now")]
    pub timestamp: DateTime,
    /// CPU usage percentage (0-100).
    pub cpu_usage: f64,
    /// Memory usage percentage (0-100).
    pub memory_usage: f64,
    /// Disk usage percentage (0-100).
    pub disk_usage: f64,
    /// Network latency in milliseconds.
    pub network_latency: f64,
    pub database_connections: i32,
    /// Available backup storage in GB.
    pub backup_storage_available: f64,
    /// Replication lag in seconds.
    pub replication_lag_seconds: f64,
    #[serde(default)]
    pub alerts: Vec<String>,
    /// Status: healthy, degraded, critical.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl SystemHealth {
    /// Determine overall health status based on metrics.
    pub fn calculate_status(&self) -> &'static str {
        if self.cpu_usage > 90.0
            || self.memory_usage > 90.0
            || self.disk_usage > 95.0
            || self.replication_lag_seconds > 300.0
        {
            "critical"
        } else if self.cpu_usage > 75.0
            || self.memory_usage > 75.0
            || self.disk_usage > 85.0
            || self.replication_lag_seconds > 60.0
        {
            "degraded"
        } else {
            "healthy"
        }
    }

    /// Check if system is healthy.
    pub fn is_healthy(&self) -> bool {
        self.status == "healthy" || self.calculate_status() == "healthy"
    }
}

/// Disaster recovery test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DRTestResult {
    pub test_id: String,
    pub plan_id: String,
    #[serde(default = "DateTime::now")]
    pub test_date: DateTime,
    pub test_type: String, // full, tabletop, partial
    pub conducted_by: String,
    /// Actual RTO achieved in hours.
    pub actual_rto_hours: f64,
    /// Actual RPO achieved in minutes.
    pub actual_rpo_minutes: f64,
    pub rto_met: bool,
    pub rpo_met: bool,
    #[serde(default)]
    pub systems_tested: Vec<String>,
    #[serde(default)]
    pub issues_found: Vec<String>,
    #[serde(default)]
    pub recommendations: Vec<String>,
    pub overall_result: String, // passed, failed, partial
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl DRTestResult {
    /// Check if test was successful.
    pub fn is_successful(&self) -> bool {
        self.overall_result == "passed" && self.rto_met && self.rpo_met
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_config_hipaa_compliance() {
        let config = BackupConfiguration {
            backup_id: "backup-1".to_string(),
            name: "Daily Full Backup".to_string(),
            backup_type: BackupType::Full,
            schedule_cron: "0 2 * * *".to_string(),
            retention_days: 365,
            data_classifications: vec![DataClassification::CriticalPatientData],
            compliance_frameworks: vec![DRComplianceFramework::Hipaa],
            encryption_enabled: true,
            compression_enabled: true,
            verification_enabled: true,
            offsite_replication: true,
            geographic_distribution: true,
            tenant_id: None,
        };
        assert!(config.is_hipaa_compliant());
    }

    #[test]
    fn test_backup_config_not_hipaa_compliant() {
        let config = BackupConfiguration {
            backup_id: "backup-2".to_string(),
            name: "Local Backup".to_string(),
            backup_type: BackupType::Incremental,
            schedule_cron: "0 * * * *".to_string(),
            retention_days: 30,
            data_classifications: vec![],
            compliance_frameworks: vec![],
            encryption_enabled: false,
            compression_enabled: true,
            verification_enabled: true,
            offsite_replication: false,
            geographic_distribution: false,
            tenant_id: None,
        };
        assert!(!config.is_hipaa_compliant());
    }

    #[test]
    fn test_recovery_plan_rto_description() {
        let plan = RecoveryPlan {
            plan_id: "plan-1".to_string(),
            name: "Critical Systems Recovery".to_string(),
            description: "Recovery plan for critical systems".to_string(),
            rto_hours: 4,
            rpo_minutes: 15,
            priority_order: vec![DataClassification::CriticalPatientData],
            critical_systems: vec!["EHR".to_string()],
            recovery_procedures: vec![],
            compliance_requirements: vec![DRComplianceFramework::Hipaa],
            contact_information: vec![],
            last_tested: None,
            test_results: None,
            tenant_id: None,
        };
        assert_eq!(plan.rto_description(), "4 hours");
        assert_eq!(plan.rpo_description(), "15 minutes");
        assert!(plan.needs_testing());
    }

    #[test]
    fn test_recovery_operation_status() {
        let op = RecoveryOperation {
            operation_id: "op-1".to_string(),
            plan_id: "plan-1".to_string(),
            initiated_by: "user-123".to_string(),
            initiated_at: DateTime::now(),
            status: RecoveryStatus::RecoveryInProgress,
            target_systems: vec!["EHR".to_string()],
            data_classifications: vec![DataClassification::CriticalPatientData],
            estimated_completion: None,
            actual_completion: None,
            recovery_point: DateTime::now(),
            progress_percentage: 50.0,
            status_message: "Restoring database".to_string(),
            compliance_validated: false,
            tenant_id: None,
        };
        assert!(op.is_in_progress());
        assert!(!op.is_complete());
        assert!(!op.has_failed());
    }

    #[test]
    fn test_system_health_status() {
        let health = SystemHealth {
            timestamp: DateTime::now(),
            cpu_usage: 45.0,
            memory_usage: 60.0,
            disk_usage: 70.0,
            network_latency: 10.0,
            database_connections: 50,
            backup_storage_available: 500.0,
            replication_lag_seconds: 5.0,
            alerts: vec![],
            status: "healthy".to_string(),
            tenant_id: None,
        };
        assert!(health.is_healthy());
        assert_eq!(health.calculate_status(), "healthy");
    }

    #[test]
    fn test_system_health_critical() {
        let health = SystemHealth {
            timestamp: DateTime::now(),
            cpu_usage: 95.0,
            memory_usage: 92.0,
            disk_usage: 98.0,
            network_latency: 500.0,
            database_connections: 100,
            backup_storage_available: 10.0,
            replication_lag_seconds: 600.0,
            alerts: vec!["High CPU".to_string()],
            status: "critical".to_string(),
            tenant_id: None,
        };
        assert!(!health.is_healthy());
        assert_eq!(health.calculate_status(), "critical");
    }

    #[test]
    fn test_recovery_status_default() {
        assert_eq!(RecoveryStatus::default(), RecoveryStatus::Active);
    }

    #[test]
    fn test_backup_type_default() {
        assert_eq!(BackupType::default(), BackupType::Full);
    }
}
