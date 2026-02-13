//! Setup Wizard Session Models.
//!
//! Wizard progression, validation, and state management for healthcare compliance setup.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Healthcare organization types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrganizationType {
    #[default]
    Hospital,
    Clinic,
    Outpatient,
    Biotech,
    Pharmaceutical,
    MedicalDevice,
    HealthSystem,
    SpecialtyPractice,
}

/// Organization size categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrganizationSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// Compliance level requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceLevel {
    Basic,
    #[default]
    Standard,
    High,
    Critical,
}

/// Validation error for a wizard step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardStepValidationError {
    pub field_name: String,
    pub error_message: String,
    pub error_code: String,
    #[serde(default)]
    pub suggestions: Vec<String>,
}

/// Organization profile data (Step 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationProfile {
    pub organization_name: String,
    pub organization_type: OrganizationType,
    pub organization_size: OrganizationSize,
    pub contact_email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(default = "default_true")]
    pub handles_phi: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patient_volume_monthly: Option<i64>,
    #[serde(default)]
    pub specialty_areas: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Complete wizard session data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardSessionData {
    pub session_id: String,
    pub user_email: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    #[serde(default)]
    pub completion_percentage: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_1_organization: Option<OrganizationProfile>,
    #[serde(default)]
    pub validation_errors: HashMap<String, Vec<WizardStepValidationError>>,
    #[serde(default)]
    pub time_spent_per_step: HashMap<String, i64>,
    #[serde(default)]
    pub help_requests: Vec<String>,
    #[serde(default)]
    pub abandoned_attempts: i32,
}

impl WizardSessionData {
    /// Get recommended compliance level based on organization profile.
    pub fn get_recommended_compliance_level(&self) -> ComplianceLevel {
        let Some(ref org) = self.step_1_organization else {
            return ComplianceLevel::Standard;
        };

        // High-risk specialties require higher compliance
        let high_risk = ["oncology", "pathology", "psychiatry", "pediatrics"];
        if org
            .specialty_areas
            .iter()
            .any(|s| high_risk.contains(&s.to_lowercase().as_str()))
        {
            return ComplianceLevel::High;
        }

        // Large organizations need higher compliance
        if org.organization_size == OrganizationSize::Large {
            return ComplianceLevel::High;
        }

        // High patient volume requires higher compliance
        if let Some(volume) = org.patient_volume_monthly {
            if volume > 10000 {
                return ComplianceLevel::High;
            }
        }

        ComplianceLevel::Standard
    }

    /// Update completion percentage based on completed steps.
    pub fn update_completion_percentage(&mut self) {
        let completed = if self.step_1_organization.is_some() {
            1
        } else {
            0
        };
        // Assuming 7 total steps like the Python version
        self.completion_percentage = (completed as f64 / 7.0) * 100.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_type_default() {
        assert_eq!(OrganizationType::default(), OrganizationType::Hospital);
    }

    #[test]
    fn test_organization_size_default() {
        assert_eq!(OrganizationSize::default(), OrganizationSize::Medium);
    }

    #[test]
    fn test_compliance_level_default() {
        assert_eq!(ComplianceLevel::default(), ComplianceLevel::Standard);
    }

    #[test]
    fn test_recommended_compliance_large_org() {
        let session = WizardSessionData {
            session_id: "sess-1".to_string(),
            user_email: "test@example.com".to_string(),
            created_at: Utc::now(),
            last_modified: Utc::now(),
            completion_percentage: 0.0,
            step_1_organization: Some(OrganizationProfile {
                organization_name: "Large Hospital".to_string(),
                organization_type: OrganizationType::Hospital,
                organization_size: OrganizationSize::Large,
                contact_email: "admin@hospital.com".to_string(),
                address: None,
                phone: None,
                website: None,
                handles_phi: true,
                patient_volume_monthly: None,
                specialty_areas: vec![],
            }),
            validation_errors: HashMap::new(),
            time_spent_per_step: HashMap::new(),
            help_requests: vec![],
            abandoned_attempts: 0,
        };

        assert_eq!(
            session.get_recommended_compliance_level(),
            ComplianceLevel::High
        );
    }
}
