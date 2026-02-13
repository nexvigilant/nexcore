//! HIPAA/HITRUST Vendor Risk Management Models.
//!
//! Vendor risk assessment and compliance mapping.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vendor criticality levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VendorCriticality {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Vendor status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VendorStatus {
    Active,
    #[default]
    Intake,
    Suspended,
    Terminated,
}

/// HITRUST CSF domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HITRUSTDomain {
    AccessControl,
    Cryptography,
    VulnerabilityManagement,
    IncidentManagement,
    ThirdParty,
}

/// HIPAA requirement categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HIPAARequirement {
    AdministrativeSafeguards,
    PhysicalSafeguards,
    TechnicalSafeguards,
    BreachNotification,
}

/// Vendor record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub vendor_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<String>,
    #[serde(default)]
    pub criticality: VendorCriticality,
    #[serde(default)]
    pub status: VendorStatus,
    #[serde(default)]
    pub services_provided: Vec<String>,
    #[serde(default)]
    pub handling_phi: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_assessed: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

/// Control mapping to HIPAA/HITRUST requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMapping {
    pub requirement_id: String,
    /// "HIPAA" or "HITRUST".
    pub framework: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_ref: Option<String>,
    #[serde(default)]
    pub implemented: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

/// Vendor security questionnaire response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireResponse {
    pub response_id: String,
    pub vendor_id: String,
    pub answers: HashMap<String, serde_json::Value>,
    #[serde(default = "Utc::now")]
    pub submitted_at: DateTime<Utc>,
}

/// Vendor assessment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorAssessment {
    pub assessment_id: String,
    pub vendor_id: String,
    pub controls: Vec<ControlMapping>,
    /// Score (0-100).
    #[serde(default)]
    pub score: f64,
    /// Risk level: low, medium, high, critical.
    #[serde(default = "default_risk_level")]
    pub risk_level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default = "Utc::now")]
    pub assessed_at: DateTime<Utc>,
}

fn default_risk_level() -> String {
    "medium".to_string()
}

impl VendorAssessment {
    /// Clamp score to 0-100 range.
    pub fn normalized_score(&self) -> f64 {
        self.score.clamp(0.0, 100.0)
    }
}

/// Vendor risk summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorRiskSummary {
    pub vendor: Vendor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_assessment: Option<VendorAssessment>,
    pub overall_score: f64,
    pub risk_level: String,
    #[serde(default)]
    pub missing_controls: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_criticality_default() {
        assert_eq!(VendorCriticality::default(), VendorCriticality::Medium);
    }

    #[test]
    fn test_vendor_status_default() {
        assert_eq!(VendorStatus::default(), VendorStatus::Intake);
    }

    #[test]
    fn test_assessment_score_clamp() {
        let assessment = VendorAssessment {
            assessment_id: "a-1".to_string(),
            vendor_id: "v-1".to_string(),
            controls: vec![],
            score: 150.0, // Over max
            risk_level: "low".to_string(),
            notes: None,
            assessed_at: Utc::now(),
        };

        assert_eq!(assessment.normalized_score(), 100.0);
    }
}
