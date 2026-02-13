//! Space PV domain models
//!
//! Consolidated from ~/projects/space-pv/nexvigilant-api/src/models/

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Organization Models
// ============================================================================

/// Organization types in the Space PV ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum OrgType {
    /// Drug manufacturer
    Manufacturer,
    /// Regulatory body (FDA, EMA, etc.)
    Regulator,
    /// NexVigilant internal staff
    Internal,
    /// Research institution
    Research,
}

/// An organization registered with NexVigilant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub org_type: OrgType,
    pub country: String,
    pub registration_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrganization {
    pub name: String,
    pub org_type: OrgType,
    pub country: String,
    pub registration_number: Option<String>,
}

// ============================================================================
// Drug Models
// ============================================================================

/// A drug registered in the Space PV system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Drug {
    pub id: String,
    pub name: String,
    pub generic_name: String,
    pub ndc_code: String,
    pub manufacturer_id: String,
    pub therapeutic_class: String,
    pub route_of_administration: String,
    pub dosage_form: String,
    pub strength: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to register a new drug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDrug {
    pub name: String,
    pub generic_name: String,
    pub ndc_code: String,
    pub manufacturer_id: String,
    pub therapeutic_class: String,
    pub route_of_administration: String,
    pub dosage_form: String,
    pub strength: String,
    pub description: Option<String>,
}

/// Request to update a drug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDrug {
    pub name: Option<String>,
    pub description: Option<String>,
    pub therapeutic_class: Option<String>,
}

/// Query parameters for listing drugs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListDrugsQuery {
    pub manufacturer_id: Option<String>,
    pub therapeutic_class: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// Clearance Models
// ============================================================================

/// Space clearance application status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClearanceStatus {
    /// Initial draft, not yet submitted
    Draft,
    /// Submitted and awaiting review
    Submitted,
    /// Under active review
    UnderReview,
    /// Requires stability testing data
    StabilityRequired,
    /// Requires pharmacokinetic validation
    PkValidation,
    /// Approved for space use
    Approved,
    /// Approved with conditions
    ConditionallyApproved,
    /// Application denied
    Denied,
    /// Application withdrawn
    Withdrawn,
}

/// A space clearance application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceClearance {
    pub id: String,
    pub drug_id: String,
    pub applicant_id: String,
    pub status: ClearanceStatus,
    pub mission_type: String,
    pub duration_days: i32,
    pub crew_size: i32,
    pub justification: String,
    pub stability_data: Option<serde_json::Value>,
    pub pk_data: Option<serde_json::Value>,
    pub reviewer_notes: Option<String>,
    pub decision_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a clearance application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClearance {
    pub drug_id: String,
    pub mission_type: String,
    pub duration_days: i32,
    pub crew_size: i32,
    pub justification: String,
}

/// Request to submit a clearance for review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitClearance {
    pub stability_data: Option<serde_json::Value>,
    pub pk_data: Option<serde_json::Value>,
}

/// Regulator decision on a clearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearanceDecision {
    pub status: ClearanceStatus,
    pub notes: Option<String>,
    pub expiry_days: Option<i32>,
}

/// Query parameters for listing clearances
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListClearancesQuery {
    pub drug_id: Option<String>,
    pub applicant_id: Option<String>,
    pub status: Option<ClearanceStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// Authentication
// ============================================================================

/// Authenticated organization (extracted from request)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthenticatedOrg {
    pub id: String,
    pub name: String,
    pub org_type: OrgType,
}
