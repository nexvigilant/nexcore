//! Privacy Compliance Models (GDPR, CCPA, HIPAA, PIPEDA, LGPD, PDPA).
//!
//! Data models for privacy regulations and data subject rights management.
//! Supports comprehensive privacy compliance and data subject rights tracking.

use nexcore_chrono::{DateTime, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported privacy regulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrivacyRegulation {
    /// EU General Data Protection Regulation.
    Gdpr,
    /// California Consumer Privacy Act.
    Ccpa,
    /// Health Insurance Portability and Accountability Act.
    Hipaa,
    /// Personal Information Protection and Electronic Documents Act (Canada).
    Pipeda,
    /// Lei Geral de Proteção de Dados (Brazil).
    Lgpd,
    /// Personal Data Protection Act (Singapore).
    Pdpa,
}

/// Privacy rights available to data subjects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataSubjectRight {
    /// Right to access personal data.
    Access,
    /// Right to correct inaccurate data.
    Rectification,
    /// Right to be forgotten.
    Erasure,
    /// Right to data portability.
    Portability,
    /// Right to restrict processing.
    Restriction,
    /// Right to object to processing.
    Objection,
    /// Right to withdraw consent.
    WithdrawConsent,
    /// Right to non-discriminatory treatment (CCPA).
    NonDiscrimination,
}

/// Privacy request processing status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestStatus {
    #[default]
    Submitted,
    UnderReview,
    InProgress,
    Completed,
    Rejected,
    Cancelled,
}

/// Categories of personal data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataCategory {
    /// Name, address, email, phone.
    BasicIdentity,
    /// Medical records, health information.
    HealthData,
    /// Payment information, insurance.
    FinancialData,
    /// Fingerprints, facial recognition.
    BiometricData,
    /// Website usage, preferences.
    BehavioralData,
    /// GPS coordinates, IP addresses.
    LocationData,
    /// Device IDs, browser information.
    DeviceData,
    /// Emails, messages, call logs.
    CommunicationData,
}

/// GDPR lawful basis for processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessingLawfulBasis {
    #[default]
    Consent,
    Contract,
    LegalObligation,
    VitalInterests,
    PublicTask,
    LegitimateInterests,
}

/// Consent tracking status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConsentStatus {
    Given,
    Withdrawn,
    Expired,
    #[default]
    Pending,
}

/// Data subject information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubject {
    pub subject_id: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<PrivacyRegulation>,
    #[serde(default = "default_language")]
    pub preferred_language: String,
    #[serde(default = "DateTime::now")]
    pub created_at: DateTime,
    #[serde(default = "DateTime::now")]
    pub last_updated: DateTime,
}

fn default_language() -> String {
    "en".to_string()
}

impl DataSubject {
    /// Create a new data subject with required fields.
    pub fn new(subject_id: impl Into<String>, email: impl Into<String>) -> Self {
        let now = DateTime::now();
        Self {
            subject_id: subject_id.into(),
            email: email.into(),
            first_name: None,
            last_name: None,
            phone: None,
            address: None,
            country: None,
            jurisdiction: None,
            preferred_language: default_language(),
            created_at: now,
            last_updated: now,
        }
    }
}

/// Consent tracking record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub consent_id: String,
    pub subject_id: String,
    pub purpose: String,
    #[serde(default)]
    pub data_categories: Vec<DataCategory>,
    pub status: ConsentStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub granted_at: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub withdrawn_at: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime>,
    #[serde(default)]
    pub lawful_basis: ProcessingLawfulBasis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consent_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub withdrawal_reason: Option<String>,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl ConsentRecord {
    /// Check if consent is currently valid (given and not expired).
    pub fn is_valid(&self) -> bool {
        if self.status != ConsentStatus::Given {
            return false;
        }
        if let Some(expires) = self.expires_at {
            return DateTime::now() < expires;
        }
        true
    }
}

/// Data subject privacy request (DSAR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyRequest {
    pub request_id: String,
    pub subject_id: String,
    pub request_type: DataSubjectRight,
    #[serde(default)]
    pub status: RequestStatus,
    #[serde(default = "DateTime::now")]
    pub submitted_at: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_verification_status")]
    pub verification_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    #[serde(default)]
    pub applicable_regulations: Vec<PrivacyRegulation>,
    #[serde(default)]
    pub data_categories_requested: Vec<DataCategory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_data: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub internal_notes: Vec<String>,
    #[serde(default)]
    pub communication_log: Vec<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

fn default_verification_status() -> String {
    "pending".to_string()
}

impl PrivacyRequest {
    /// Calculate default due date based on regulation (GDPR: 30 days, CCPA: 45 days).
    pub fn calculate_due_date(&self) -> DateTime {
        let days = if self
            .applicable_regulations
            .contains(&PrivacyRegulation::Gdpr)
        {
            30
        } else if self
            .applicable_regulations
            .contains(&PrivacyRegulation::Ccpa)
        {
            45
        } else {
            30 // Default to 30 days
        };
        self.submitted_at + Duration::days(days)
    }

    /// Check if request is overdue.
    pub fn is_overdue(&self) -> bool {
        if self.status == RequestStatus::Completed || self.status == RequestStatus::Cancelled {
            return false;
        }
        if let Some(due) = self.due_date {
            return DateTime::now() > due;
        }
        DateTime::now() > self.calculate_due_date()
    }
}

/// Record of data processing activities (ROPA).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessingRecord {
    pub processing_id: String,
    pub purpose: String,
    pub lawful_basis: ProcessingLawfulBasis,
    #[serde(default)]
    pub data_categories: Vec<DataCategory>,
    #[serde(default)]
    pub data_sources: Vec<String>,
    /// Retention period in days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_days: Option<i64>,
    #[serde(default)]
    pub recipients: Vec<String>,
    #[serde(default)]
    pub international_transfers: Vec<String>,
    #[serde(default)]
    pub safeguards: Vec<String>,
    #[serde(default = "DateTime::now")]
    pub created_at: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_reviewed: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_assessment: Option<String>,
    #[serde(default)]
    pub technical_measures: Vec<String>,
    #[serde(default)]
    pub organizational_measures: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

/// Data breach incident record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyDataBreach {
    pub breach_id: String,
    pub incident_date: DateTime,
    pub discovered_date: DateTime,
    /// Type: unauthorized_access, data_loss, system_compromise, etc.
    pub breach_type: String,
    pub affected_subjects_count: i64,
    #[serde(default)]
    pub data_categories_affected: Vec<DataCategory>,
    pub cause: String,
    pub impact_assessment: String,
    #[serde(default)]
    pub containment_measures: Vec<String>,
    #[serde(default)]
    pub notification_authorities: Vec<String>,
    #[serde(default)]
    pub notification_subjects: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_sent_at: Option<DateTime>,
    #[serde(default)]
    pub remediation_steps: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lessons_learned: Option<String>,
    /// Status: open, investigating, contained, resolved.
    #[serde(default = "default_breach_status")]
    pub status: String,
    #[serde(default = "default_true")]
    pub regulatory_reporting_required: bool,
    /// GDPR requires notification within 72 hours.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reporting_deadline: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reported_at: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

fn default_breach_status() -> String {
    "open".to_string()
}

fn default_true() -> bool {
    true
}

impl PrivacyDataBreach {
    /// Calculate GDPR reporting deadline (72 hours from discovery).
    pub fn gdpr_reporting_deadline(&self) -> DateTime {
        self.discovered_date + Duration::hours(72)
    }

    /// Check if breach notification is overdue.
    pub fn is_notification_overdue(&self) -> bool {
        if self.notification_sent_at.is_some() {
            return false;
        }
        if let Some(deadline) = self.reporting_deadline {
            return DateTime::now() > deadline;
        }
        DateTime::now() > self.gdpr_reporting_deadline()
    }
}

/// Data inventory item for privacy impact assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataInventoryItem {
    pub item_id: String,
    pub system_name: String,
    #[serde(default)]
    pub data_categories: Vec<DataCategory>,
    #[serde(default)]
    pub processing_purposes: Vec<String>,
    #[serde(default)]
    pub lawful_bases: Vec<ProcessingLawfulBasis>,
    #[serde(default)]
    pub data_sources: Vec<String>,
    #[serde(default)]
    pub data_recipients: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_period: Option<String>,
    #[serde(default)]
    pub geographic_locations: Vec<String>,
    #[serde(default)]
    pub security_measures: Vec<String>,
    /// Risk level: low, medium, high, critical.
    #[serde(default = "default_risk_level")]
    pub risk_level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_assessed: Option<DateTime>,
    /// Status: compliant, non_compliant, under_review.
    #[serde(default = "default_compliance_status")]
    pub compliance_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

fn default_risk_level() -> String {
    "medium".to_string()
}

fn default_compliance_status() -> String {
    "compliant".to_string()
}

/// Privacy compliance report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyComplianceReport {
    pub report_id: String,
    /// Type: monthly, quarterly, annual, incident, audit.
    pub report_type: String,
    pub period_start: DateTime,
    pub period_end: DateTime,
    #[serde(default = "DateTime::now")]
    pub generated_at: DateTime,
    #[serde(default)]
    pub regulations_covered: Vec<PrivacyRegulation>,
    #[serde(default)]
    pub summary: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub metrics: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub findings: Vec<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub recommendations: Vec<String>,
    #[serde(default)]
    pub action_items: Vec<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub attachments: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_subject_creation() {
        let subject = DataSubject::new("sub-123", "test@example.com");
        assert_eq!(subject.subject_id, "sub-123");
        assert_eq!(subject.email, "test@example.com");
        assert_eq!(subject.preferred_language, "en");
    }

    #[test]
    fn test_consent_validity() {
        let consent = ConsentRecord {
            consent_id: "consent-1".to_string(),
            subject_id: "sub-123".to_string(),
            purpose: "marketing".to_string(),
            data_categories: vec![DataCategory::BasicIdentity],
            status: ConsentStatus::Given,
            granted_at: Some(DateTime::now()),
            withdrawn_at: None,
            expires_at: Some(DateTime::now() + Duration::days(365)),
            lawful_basis: ProcessingLawfulBasis::Consent,
            consent_text: None,
            withdrawal_reason: None,
            version: "1.0".to_string(),
            metadata: HashMap::new(),
        };
        assert!(consent.is_valid());
    }

    #[test]
    fn test_consent_expired() {
        let consent = ConsentRecord {
            consent_id: "consent-2".to_string(),
            subject_id: "sub-123".to_string(),
            purpose: "marketing".to_string(),
            data_categories: vec![],
            status: ConsentStatus::Given,
            granted_at: Some(DateTime::now() - Duration::days(400)),
            withdrawn_at: None,
            expires_at: Some(DateTime::now() - Duration::days(1)), // Expired yesterday
            lawful_basis: ProcessingLawfulBasis::Consent,
            consent_text: None,
            withdrawal_reason: None,
            version: "1.0".to_string(),
            metadata: HashMap::new(),
        };
        assert!(!consent.is_valid());
    }

    #[test]
    fn test_privacy_request_due_date() {
        let request = PrivacyRequest {
            request_id: "req-1".to_string(),
            subject_id: "sub-123".to_string(),
            request_type: DataSubjectRight::Access,
            status: RequestStatus::Submitted,
            submitted_at: DateTime::now(),
            description: None,
            verification_status: "pending".to_string(),
            assigned_to: None,
            due_date: None,
            completed_at: None,
            rejection_reason: None,
            applicable_regulations: vec![PrivacyRegulation::Gdpr],
            data_categories_requested: vec![],
            response_data: None,
            internal_notes: vec![],
            communication_log: vec![],
            tenant_id: None,
        };

        let due = request.calculate_due_date();
        let expected = request.submitted_at + Duration::days(30);
        assert!((due - expected).num_seconds().abs() < 1);
    }

    #[test]
    fn test_breach_notification_deadline() {
        let breach = PrivacyDataBreach {
            breach_id: "breach-1".to_string(),
            incident_date: DateTime::now() - Duration::hours(48),
            discovered_date: DateTime::now(),
            breach_type: "unauthorized_access".to_string(),
            affected_subjects_count: 100,
            data_categories_affected: vec![DataCategory::HealthData],
            cause: "Phishing attack".to_string(),
            impact_assessment: "Moderate".to_string(),
            containment_measures: vec!["Password reset".to_string()],
            notification_authorities: vec![],
            notification_subjects: false,
            notification_sent_at: None,
            remediation_steps: vec![],
            lessons_learned: None,
            status: "open".to_string(),
            regulatory_reporting_required: true,
            reporting_deadline: None,
            reported_at: None,
            tenant_id: None,
        };

        let deadline = breach.gdpr_reporting_deadline();
        assert!(deadline > DateTime::now());
        assert!(!breach.is_notification_overdue());
    }

    #[test]
    fn test_request_status_default() {
        assert_eq!(RequestStatus::default(), RequestStatus::Submitted);
    }

    #[test]
    fn test_consent_status_default() {
        assert_eq!(ConsentStatus::default(), ConsentStatus::Pending);
    }
}
