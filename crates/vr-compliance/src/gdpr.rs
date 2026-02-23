//! GDPR compliance — data subject requests, consent management, and deletion manifests.
//!
//! Implements core requirements from the General Data Protection Regulation:
//! - **Article 12(3)**: 30-day response deadline for data subject requests
//! - **Article 15**: Right of access (data export)
//! - **Article 17**: Right to erasure (data deletion)
//! - **Article 16**: Right to rectification
//! - **Article 20**: Right to data portability
//! - **Article 18**: Right to restriction of processing
//!
//! Consent records track granular consent per processing purpose,
//! supporting the "freely given, specific, informed, unambiguous" standard
//! from Article 4(11).

use chrono::{DateTime, Duration, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use vr_core::TenantId;

// ============================================================================
// Data Subject Request Types
// ============================================================================

/// The type of data subject request per GDPR Articles 15-20.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestType {
    /// Article 15 — Right of access: provide a copy of all personal data.
    Access,
    /// Article 17 — Right to erasure ("right to be forgotten").
    Erasure,
    /// Article 16 — Right to rectification: correct inaccurate data.
    Rectification,
    /// Article 20 — Right to data portability: export in machine-readable format.
    Portability,
    /// Article 18 — Right to restriction of processing.
    Restriction,
}

/// Processing status of a data subject request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    /// Request received, not yet started.
    Received,
    /// Request is being processed (identity verified, data gathering).
    Processing,
    /// Request fulfilled and response sent to data subject.
    Completed,
    /// Request denied (e.g., identity not verified, legal exemption applies).
    Denied,
}

// ============================================================================
// Data Subject Request
// ============================================================================

/// A formal request from a data subject exercising their GDPR rights.
///
/// The platform must respond within 30 calendar days per Article 12(3).
/// Extensions of up to 60 additional days are possible for complex requests,
/// but the data subject must be informed within the initial 30-day period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    /// Unique identifier for tracking this request.
    pub id: NexId,
    /// Tenant whose data is subject to the request.
    pub tenant_id: TenantId,
    /// Type of GDPR right being exercised.
    pub request_type: RequestType,
    /// Email of the person making the request.
    pub requester_email: String,
    /// Current processing status.
    pub status: RequestStatus,
    /// When the request was received.
    pub requested_at: DateTime<Utc>,
    /// Legal deadline: 30 calendar days from receipt per Article 12(3).
    pub deadline_at: DateTime<Utc>,
    /// When the request was completed (if applicable).
    pub completed_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Consent Management
// ============================================================================

/// Categories of data processing that require explicit consent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentType {
    /// Consent to process personal data for the core service.
    DataProcessing,
    /// Consent to contribute anonymized data to shared datasets.
    DataContribution,
    /// Consent to share compound data on the marketplace.
    MarketplaceSharing,
    /// Consent to collect and process usage analytics.
    Analytics,
}

/// A record of consent granted (or revoked) by a data subject.
///
/// Consent must be "freely given, specific, informed and unambiguous"
/// per GDPR Article 4(11). Each consent type is tracked independently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// Tenant this consent applies to.
    pub tenant_id: TenantId,
    /// The specific processing purpose.
    pub consent_type: ConsentType,
    /// Whether consent is currently granted.
    pub granted: bool,
    /// When consent was granted.
    pub granted_at: DateTime<Utc>,
    /// When consent was revoked (None if still active).
    pub revoked_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Deletion Manifest
// ============================================================================

/// A manifest describing all data that must be deleted for a GDPR erasure request.
///
/// This enumerates every storage location where tenant data resides,
/// ensuring complete deletion across all systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionManifest {
    /// Database tables containing tenant data.
    pub tables: Vec<String>,
    /// Object storage prefixes (S3/GCS paths) containing tenant files.
    pub object_storage_prefixes: Vec<String>,
    /// Estimated total number of records to be deleted.
    pub estimated_records: u64,
    /// Whether a backup should be created before deletion (for legal hold).
    pub requires_backup: bool,
}

// ============================================================================
// GDPR Functions
// ============================================================================

/// Calculate the GDPR response deadline: 30 calendar days from the request date.
///
/// Per GDPR Article 12(3): "The controller shall provide information on action
/// taken on a request [...] without undue delay and in any event within one
/// month of receipt of the request."
#[must_use]
pub fn calculate_deadline(requested_at: DateTime<Utc>) -> DateTime<Utc> {
    requested_at + Duration::days(30)
}

/// Check whether a data subject request has passed its legal deadline.
///
/// A request is overdue if:
/// - It is not yet completed (status is Received or Processing), AND
/// - The current time is past the 30-day deadline
#[must_use]
pub fn is_overdue(request: &DataSubjectRequest) -> bool {
    match request.status {
        RequestStatus::Completed | RequestStatus::Denied => false,
        RequestStatus::Received | RequestStatus::Processing => Utc::now() > request.deadline_at,
    }
}

/// Calculate the number of days remaining until the GDPR deadline.
///
/// Returns a positive number if the deadline is in the future,
/// zero or negative if the deadline has passed.
#[must_use]
pub fn days_remaining(request: &DataSubjectRequest) -> i64 {
    let remaining = request.deadline_at - Utc::now();
    remaining.num_days()
}

/// Build a deletion manifest for a tenant, enumerating all storage locations.
///
/// This returns a comprehensive manifest of all database tables and
/// object storage prefixes that contain data for the given tenant.
/// The manifest is used to drive the actual deletion process.
#[must_use]
pub fn build_deletion_manifest(tenant_id: &TenantId) -> DeletionManifest {
    let tenant_prefix = tenant_id.to_string();

    // All database tables that contain tenant-scoped data
    let tables = vec![
        "tenants".to_string(),
        "users".to_string(),
        "programs".to_string(),
        "compounds".to_string(),
        "assays".to_string(),
        "assay_results".to_string(),
        "orders".to_string(),
        "deals".to_string(),
        "assets".to_string(),
        "invoices".to_string(),
        "api_keys".to_string(),
        "team_invitations".to_string(),
        "consent_records".to_string(),
        "notifications".to_string(),
        "usage_metrics".to_string(),
    ];

    // Object storage prefixes containing tenant files
    let object_storage_prefixes = vec![
        format!("tenants/{tenant_prefix}/compounds/"),
        format!("tenants/{tenant_prefix}/assays/"),
        format!("tenants/{tenant_prefix}/reports/"),
        format!("tenants/{tenant_prefix}/exports/"),
        format!("tenants/{tenant_prefix}/uploads/"),
    ];

    // Estimate: roughly 500 records per table for an active tenant
    let estimated_records = tables.len() as u64 * 500;

    DeletionManifest {
        tables,
        object_storage_prefixes,
        estimated_records,
        // Always require backup for erasure — legal hold may apply
        requires_backup: true,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_request(status: RequestStatus, days_ago: i64) -> DataSubjectRequest {
        let requested_at = Utc::now() - Duration::days(days_ago);
        DataSubjectRequest {
            id: NexId::v4(),
            tenant_id: TenantId::new(),
            request_type: RequestType::Access,
            requester_email: "subject@example.com".to_string(),
            status,
            requested_at,
            deadline_at: calculate_deadline(requested_at),
            completed_at: None,
        }
    }

    #[test]
    fn deadline_is_30_days_from_request() {
        let now = Utc::now();
        let deadline = calculate_deadline(now);
        let diff = deadline - now;
        assert_eq!(diff.num_days(), 30);
    }

    #[test]
    fn is_overdue_returns_false_for_completed_requests() {
        // Even if deadline has passed, completed requests are not overdue
        let mut request = make_request(RequestStatus::Completed, 60);
        request.completed_at = Some(Utc::now() - Duration::days(25));
        assert!(!is_overdue(&request));
    }

    #[test]
    fn is_overdue_returns_false_for_denied_requests() {
        let request = make_request(RequestStatus::Denied, 60);
        assert!(!is_overdue(&request));
    }

    #[test]
    fn is_overdue_returns_true_for_stale_received() {
        // Request received 35 days ago, still in Received status
        let request = make_request(RequestStatus::Received, 35);
        assert!(is_overdue(&request));
    }

    #[test]
    fn is_overdue_returns_true_for_stale_processing() {
        // Request processing for 31 days — overdue
        let request = make_request(RequestStatus::Processing, 31);
        assert!(is_overdue(&request));
    }

    #[test]
    fn is_overdue_returns_false_within_deadline() {
        // Request received 10 days ago — well within deadline
        let request = make_request(RequestStatus::Received, 10);
        assert!(!is_overdue(&request));
    }

    #[test]
    fn days_remaining_positive_for_future_deadline() {
        let request = make_request(RequestStatus::Received, 5);
        let remaining = days_remaining(&request);
        // 30 - 5 = 25 days remaining (may be 24 due to time-of-day)
        assert!(remaining >= 24 && remaining <= 25);
    }

    #[test]
    fn days_remaining_negative_for_past_deadline() {
        let request = make_request(RequestStatus::Processing, 35);
        let remaining = days_remaining(&request);
        // 30 - 35 = -5 days remaining
        assert!(remaining >= -6 && remaining <= -4);
    }

    #[test]
    fn deletion_manifest_contains_all_tables() {
        let tenant_id = TenantId::new();
        let manifest = build_deletion_manifest(&tenant_id);

        assert!(!manifest.tables.is_empty());
        assert!(manifest.tables.contains(&"tenants".to_string()));
        assert!(manifest.tables.contains(&"compounds".to_string()));
        assert!(manifest.tables.contains(&"users".to_string()));
        assert!(manifest.tables.contains(&"consent_records".to_string()));
    }

    #[test]
    fn deletion_manifest_has_storage_prefixes() {
        let tenant_id = TenantId::new();
        let manifest = build_deletion_manifest(&tenant_id);

        assert!(!manifest.object_storage_prefixes.is_empty());
        // All prefixes should contain the tenant ID
        let tenant_str = tenant_id.to_string();
        for prefix in &manifest.object_storage_prefixes {
            assert!(prefix.contains(&tenant_str));
        }
    }

    #[test]
    fn deletion_manifest_requires_backup() {
        let manifest = build_deletion_manifest(&TenantId::new());
        assert!(manifest.requires_backup);
    }

    #[test]
    fn deletion_manifest_estimates_records() {
        let manifest = build_deletion_manifest(&TenantId::new());
        assert!(manifest.estimated_records > 0);
        // Should be tables * 500
        assert_eq!(
            manifest.estimated_records,
            manifest.tables.len() as u64 * 500
        );
    }

    #[test]
    fn consent_record_serialization_roundtrip() {
        let record = ConsentRecord {
            tenant_id: TenantId::new(),
            consent_type: ConsentType::DataProcessing,
            granted: true,
            granted_at: Utc::now(),
            revoked_at: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: ConsentRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.consent_type, ConsentType::DataProcessing);
        assert!(deserialized.granted);
        assert!(deserialized.revoked_at.is_none());
    }

    #[test]
    fn data_subject_request_serialization_roundtrip() {
        let request = make_request(RequestStatus::Received, 5);
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: DataSubjectRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, request.id);
        assert_eq!(deserialized.request_type, RequestType::Access);
        assert_eq!(deserialized.status, RequestStatus::Received);
    }

    #[test]
    fn all_request_types_exist() {
        let types = vec![
            RequestType::Access,
            RequestType::Erasure,
            RequestType::Rectification,
            RequestType::Portability,
            RequestType::Restriction,
        ];
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn all_consent_types_exist() {
        let types = vec![
            ConsentType::DataProcessing,
            ConsentType::DataContribution,
            ConsentType::MarketplaceSharing,
            ConsentType::Analytics,
        ];
        assert_eq!(types.len(), 4);
    }
}
