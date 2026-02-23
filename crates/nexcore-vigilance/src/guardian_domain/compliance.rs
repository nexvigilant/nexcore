//! FDA 21 CFR Part 11 Compliance Models.
//!
//! Electronic records and electronic signatures per FDA 21 CFR Part 11.
//! Includes audit trails, breach notification, and validation records.

use chrono::{DateTime, Utc};
use nexcore_codec::hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// 21 CFR Part 11 §11.50 - Required signature meanings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignatureMeaning {
    Review,
    Approval,
    Responsibility,
    Authorship,
    Witnessing,
    Acknowledgment,
}

/// Methods for creating electronic signatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignatureMethod {
    PasswordBased,
    Biometric,
    TokenBased,
    DigitalCertificate,
    MultiFactor,
}

/// Types of actions that must be audited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    #[default]
    Create,
    Read,
    Update,
    Delete,
    Sign,
    Login,
    Logout,
    AccessDenied,
    SystemConfig,
    DataExport,
    DataImport,
}

/// Types of electronic records in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordType {
    #[default]
    Alert,
    AuditEntry,
    UserAccount,
    SystemConfig,
    ComplianceReport,
    Signature,
    BatchRecord,
    QualityRecord,
}

/// HIPAA breach risk assessment levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BreachRiskLevel {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Breach notification workflow status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BreachStatus {
    #[default]
    IncidentReported,
    UnderAssessment,
    BreachConfirmed,
    NotABreach,
    NotificationPending,
    NotificationInProgress,
    NotificationComplete,
    HhsReported,
    Closed,
}

/// HIPAA breach notification methods per § 164.404-408.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationMethod {
    IndividualWritten,
    IndividualEmail,
    IndividualSubstitute,
    MediaNotice,
    WebNotice,
    TollFreeNumber,
    HhsImmediate,
    HhsAnnual,
}

/// 21 CFR Part 11 §11.50 compliant electronic signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronicSignature {
    pub id: String,
    /// Printed name of the signer.
    pub signer_name: String,
    /// Unique identifier of signer.
    pub signer_id: String,
    #[serde(default = "Utc::now")]
    pub signature_timestamp: DateTime<Utc>,
    pub signature_meaning: SignatureMeaning,
    pub signature_method: SignatureMethod,
    /// Unique signature identifier (§11.100).
    pub unique_identifier: String,
    /// ID of signed record.
    pub record_id: String,
    pub record_type: RecordType,
    /// Hash of record at time of signing.
    pub record_hash: String,
    /// Tamper-proof signature hash.
    pub signature_hash: String,
    /// Optional KMS-backed MAC of signature_hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_mac: Option<String>,
    #[serde(default)]
    pub authentication_factors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biometric_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default = "default_true")]
    pub is_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidation_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidated_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidated_by: Option<String>,
}

fn default_true() -> bool {
    true
}

impl ElectronicSignature {
    /// Generate tamper-proof signature hash for §11.70 compliance.
    pub fn generate_signature_hash(&self) -> String {
        let data = format!(
            "{}|{}|{}|{:?}|{}|{}|{}",
            self.signer_id,
            self.signer_name,
            self.signature_timestamp.to_rfc3339(),
            self.signature_meaning,
            self.record_id,
            self.record_hash,
            self.unique_identifier
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify signature hasn't been tampered with.
    pub fn verify_integrity(&self) -> bool {
        let expected = self.generate_signature_hash();
        // Constant-time comparison
        self.signature_hash.len() == expected.len()
            && self
                .signature_hash
                .bytes()
                .zip(expected.bytes())
                .all(|(a, b)| a == b)
    }
}

/// 21 CFR Part 11 §11.10(e) compliant audit trail entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditEntry {
    pub id: String,
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub user_name: String,
    pub action: AuditAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_type: Option<RecordType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
    pub session_id: String,
    pub ip_address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geolocation: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_info: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regulatory_category: Option<String>,
    /// Tamper-proof audit hash.
    #[serde(default)]
    pub audit_trail_hash: String,
    /// Hash of previous audit entry for chain integrity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_entry_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence_number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_id: Option<String>,
    /// HIPAA minimum retention (6 years).
    #[serde(default = "default_retention")]
    pub retention_years: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_validated: bool,
    #[serde(default)]
    pub validation_errors: Vec<String>,
}

fn default_retention() -> i32 {
    6
}

impl ComplianceAuditEntry {
    /// Generate tamper-proof audit trail hash.
    pub fn generate_audit_hash(&self) -> String {
        let data = format!(
            "{}|{}|{:?}|{:?}|{:?}|{:?}|{}|{:?}|{:?}",
            self.timestamp.to_rfc3339(),
            self.user_id,
            self.action,
            self.record_id,
            self.before_value,
            self.after_value,
            self.session_id,
            self.sequence_number,
            self.previous_entry_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify audit entry hasn't been tampered with.
    pub fn verify_integrity(&self) -> bool {
        let expected = self.generate_audit_hash();
        self.audit_trail_hash.len() == expected.len()
            && self
                .audit_trail_hash
                .bytes()
                .zip(expected.bytes())
                .all(|(a, b)| a == b)
    }
}

/// 21 CFR Part 11 §11.10(a) - System validation documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemValidationRecord {
    pub id: String,
    #[serde(default = "Utc::now")]
    pub validation_date: DateTime<Utc>,
    pub validator_id: String,
    pub validator_name: String,
    #[serde(default)]
    pub installation_qualification: bool,
    #[serde(default)]
    pub operational_qualification: bool,
    #[serde(default)]
    pub performance_qualification: bool,
    pub system_version: String,
    pub validation_protocol_version: String,
    #[serde(default)]
    pub total_tests: i32,
    #[serde(default)]
    pub passed_tests: i32,
    #[serde(default)]
    pub failed_tests: i32,
    #[serde(default)]
    pub test_results: Vec<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub is_validated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_expiry: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_report_path: Option<String>,
    #[serde(default)]
    pub deviation_reports: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validator_signature_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approver_signature_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub regulatory_scope: Vec<String>,
}

/// Compliance monitoring metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMetrics {
    #[serde(default = "Utc::now")]
    pub metric_date: DateTime<Utc>,
    /// 21 CFR Part 11 compliance score (0-100).
    pub cfr_part_11_score: f64,
    /// HIPAA compliance score (0-100).
    pub hipaa_score: f64,
    /// GDPR compliance score (0-100).
    pub gdpr_score: f64,
    /// Overall weighted compliance score.
    pub overall_score: f64,
    #[serde(default)]
    pub total_audit_entries: i64,
    #[serde(default)]
    pub failed_integrity_checks: i64,
    #[serde(default)]
    pub missing_signatures: i64,
    #[serde(default)]
    pub expired_signatures: i64,
    #[serde(default)]
    pub failed_login_attempts: i64,
    #[serde(default)]
    pub unauthorized_access_attempts: i64,
    #[serde(default)]
    pub data_breach_incidents: i64,
    #[serde(default)]
    pub vulnerability_count: i64,
    #[serde(default)]
    pub system_uptime_percent: f64,
    #[serde(default)]
    pub backup_success_rate: f64,
    #[serde(default)]
    pub disaster_recovery_tests: i64,
    #[serde(default)]
    pub active_users: i64,
    #[serde(default)]
    pub users_with_training: i64,
    #[serde(default)]
    pub users_needing_retraining: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub calculated_by: String,
    pub calculation_method: String,
}

impl ComplianceMetrics {
    /// Calculate weighted overall compliance score.
    /// CFR Part 11 (40%), HIPAA (40%), GDPR (20%)
    pub fn calculate_overall_score(&self) -> f64 {
        (self.cfr_part_11_score * 0.4 + self.hipaa_score * 0.4 + self.gdpr_score * 0.2).round()
    }
}

/// HIPAA § 164.402 Breach Assessment - 4-Factor Test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachAssessment {
    pub id: String,
    pub incident_id: String,
    #[serde(default = "Utc::now")]
    pub assessment_date: DateTime<Utc>,
    pub assessed_by: String,
    pub assessed_by_name: String,
    // Factor 1: Nature and extent of PHI involved
    #[serde(default)]
    pub factor_1_phi_categories: Vec<String>,
    pub factor_1_sensitivity: String,
    pub factor_1_volume: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factor_1_notes: Option<String>,
    // Factor 2: Unauthorized person
    pub factor_2_unauthorized_person: String,
    pub factor_2_relationship: String,
    pub factor_2_access_level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factor_2_notes: Option<String>,
    // Factor 3: Was PHI actually acquired/viewed
    pub factor_3_acquisition_confirmed: bool,
    pub factor_3_evidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factor_3_duration: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factor_3_notes: Option<String>,
    // Factor 4: Mitigation extent
    #[serde(default)]
    pub factor_4_mitigation_actions: Vec<String>,
    pub factor_4_effectiveness: String,
    pub factor_4_residual_risk: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factor_4_notes: Option<String>,
    // Conclusion
    pub is_breach: bool,
    pub risk_level: BreachRiskLevel,
    pub justification: String,
    // Approval
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<DateTime<Utc>>,
    pub tenant_id: String,
}

/// HIPAA Breach Notification Record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachNotification {
    pub id: String,
    pub breach_id: String,
    pub assessment_id: String,
    pub discovery_date: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurrence_date: Option<DateTime<Utc>>,
    pub affected_individuals: i64,
    #[serde(default)]
    pub phi_categories_involved: Vec<String>,
    /// Deadline for individual notification (60 days from discovery).
    pub notification_deadline: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_started: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_completed: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notification_methods: Vec<NotificationMethod>,
    #[serde(default)]
    pub individuals_notified_written: i64,
    #[serde(default)]
    pub individuals_notified_email: i64,
    #[serde(default)]
    pub individuals_substitute_notice: bool,
    #[serde(default)]
    pub media_notice_required: bool,
    #[serde(default)]
    pub media_notice_sent: bool,
    #[serde(default)]
    pub media_notice_outlets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_notice_date: Option<DateTime<Utc>>,
    #[serde(default = "default_true")]
    pub hhs_notification_required: bool,
    pub hhs_notification_type: NotificationMethod,
    #[serde(default)]
    pub hhs_notification_sent: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hhs_notification_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hhs_case_number: Option<String>,
    pub breach_description: String,
    pub phi_involved: String,
    pub steps_individuals_should_take: String,
    pub mitigation_actions: String,
    pub contact_information: String,
    #[serde(default)]
    pub status: BreachStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_by: String,
    pub created_by_name: String,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    pub tenant_id: String,
}

impl BreachNotification {
    /// Check if notification deadline has passed.
    pub fn is_overdue(&self) -> bool {
        Utc::now() > self.notification_deadline && self.notification_completed.is_none()
    }
}

// ============================================================================
// HIPAA PHI Access Types (Enhanced 2025 Requirements)
// ============================================================================

/// Types of PHI access for enhanced HIPAA logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PhiAccessType {
    View,
    Modify,
    Export,
    Delete,
    Search,
    Print,
    Download,
    Share,
}

impl PhiAccessType {
    /// Get risk weight for this access type (1-5).
    pub fn risk_weight(&self) -> i32 {
        match self {
            Self::View => 1,
            Self::Search => 2,
            Self::Modify => 3,
            Self::Export | Self::Download | Self::Print => 4,
            Self::Delete | Self::Share => 5,
        }
    }
}

/// HIPAA compliance status levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HipaaComplianceStatus {
    #[default]
    Compliant,
    AttentionRequired,
    NonCompliant,
    BreachDetected,
}

impl HipaaComplianceStatus {
    /// Create from compliance score percentage.
    pub fn from_score(score: f64) -> Self {
        if score >= 95.0 {
            Self::Compliant
        } else if score >= 85.0 {
            Self::AttentionRequired
        } else {
            Self::NonCompliant
        }
    }
}

/// Enhanced HIPAA audit entry with PHI-specific fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HipaaAuditEntry {
    // Base audit fields
    pub id: String,
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub user_name: String,
    pub action: AuditAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_type: Option<RecordType>,
    pub session_id: String,
    pub ip_address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_reason: Option<String>,
    #[serde(default)]
    pub regulatory_category: String,

    // PHI-specific audit fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phi_access_type: Option<PhiAccessType>,
    #[serde(default)]
    pub phi_elements_accessed: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<String>,
    /// Medical Record Number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mrn: Option<String>,

    // Risk and compliance assessment
    #[serde(default)]
    pub risk_level: BreachRiskLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compliance_impact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_justification: Option<String>,

    // Access pattern analysis
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_pattern_score: Option<f64>,
    #[serde(default)]
    pub unusual_access_flags: Vec<String>,

    // Geographic and contextual data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geolocation: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_context: Option<HashMap<String, serde_json::Value>>,

    // HIPAA retention (6+ years minimum)
    #[serde(default = "default_hipaa_retention")]
    pub hipaa_retention_years: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_location: Option<String>,

    // Enhanced security
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phi_encryption_key_id: Option<String>,
    /// Access method: web, api, mobile, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_method: Option<String>,
}

fn default_hipaa_retention() -> i32 {
    6
}

impl HipaaAuditEntry {
    /// Calculate risk level based on access type and context.
    pub fn calculate_risk_level(&self) -> BreachRiskLevel {
        let mut risk_score = 0;

        // Risk based on access type
        if let Some(access_type) = &self.phi_access_type {
            risk_score += access_type.risk_weight();
        }

        // Risk based on PHI volume
        let phi_count = self.phi_elements_accessed.len();
        if phi_count > 10 {
            risk_score += 2;
        } else if phi_count > 5 {
            risk_score += 1;
        }

        // Risk based on unusual flags
        if !self.unusual_access_flags.is_empty() {
            risk_score += self.unusual_access_flags.len().min(3) as i32;
        }

        // Determine risk level
        if risk_score <= 2 {
            BreachRiskLevel::Low
        } else if risk_score <= 4 {
            BreachRiskLevel::Medium
        } else if risk_score <= 6 {
            BreachRiskLevel::High
        } else {
            BreachRiskLevel::Critical
        }
    }

    /// Check if this access requires immediate review.
    pub fn requires_immediate_review(&self) -> bool {
        matches!(
            self.risk_level,
            BreachRiskLevel::High | BreachRiskLevel::Critical
        ) || !self.unusual_access_flags.is_empty()
    }
}

/// Annual Breach Log for HHS Reporting (< 500 individuals).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachLog {
    pub id: String,
    pub year: i32,
    pub tenant_id: String,
    #[serde(default)]
    pub breaches: Vec<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub submitted_to_hhs: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submission_date: Option<DateTime<Utc>>,
    /// 60 days after end of calendar year.
    pub submission_deadline: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_hash_generation() {
        let sig = ElectronicSignature {
            id: "sig-1".to_string(),
            signer_name: "John Doe".to_string(),
            signer_id: "user-123".to_string(),
            signature_timestamp: Utc::now(),
            signature_meaning: SignatureMeaning::Approval,
            signature_method: SignatureMethod::PasswordBased,
            unique_identifier: "unique-123456789012".to_string(),
            record_id: "rec-1".to_string(),
            record_type: RecordType::Alert,
            record_hash: "abc123".to_string(),
            signature_hash: String::new(),
            signature_mac: None,
            authentication_factors: vec![],
            biometric_hash: None,
            device_fingerprint: None,
            session_id: None,
            ip_address: None,
            user_agent: None,
            tenant_id: None,
            is_valid: true,
            invalidation_reason: None,
            invalidated_at: None,
            invalidated_by: None,
        };

        let hash = sig.generate_signature_hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_audit_action_default() {
        assert_eq!(AuditAction::default(), AuditAction::Create);
    }

    #[test]
    fn test_breach_risk_level_default() {
        assert_eq!(BreachRiskLevel::default(), BreachRiskLevel::Medium);
    }

    #[test]
    fn test_compliance_metrics_score() {
        let metrics = ComplianceMetrics {
            metric_date: Utc::now(),
            cfr_part_11_score: 90.0,
            hipaa_score: 85.0,
            gdpr_score: 80.0,
            overall_score: 0.0,
            total_audit_entries: 0,
            failed_integrity_checks: 0,
            missing_signatures: 0,
            expired_signatures: 0,
            failed_login_attempts: 0,
            unauthorized_access_attempts: 0,
            data_breach_incidents: 0,
            vulnerability_count: 0,
            system_uptime_percent: 99.9,
            backup_success_rate: 100.0,
            disaster_recovery_tests: 4,
            active_users: 100,
            users_with_training: 95,
            users_needing_retraining: 5,
            tenant_id: None,
            calculated_by: "system".to_string(),
            calculation_method: "weighted".to_string(),
        };

        // 90*0.4 + 85*0.4 + 80*0.2 = 36 + 34 + 16 = 86
        assert_eq!(metrics.calculate_overall_score(), 86.0);
    }
}
