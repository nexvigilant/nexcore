//! Healthcare User Management Models.
//!
//! User models for healthcare professionals with FHIR R4 alignment.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Healthcare user roles aligned with FHIR practitioner types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Doctor,
    Nurse,
    #[default]
    Admin,
    Patient,
    Technician,
    Pharmacist,
    Researcher,
    Auditor,
}

/// User account status for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    #[default]
    Pending,
}

/// Healthcare user model with FHIR R4 alignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier (UUID v4).
    pub id: String,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    #[serde(default)]
    pub status: UserStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specialization: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_number: Option<String>,
    /// National Provider Identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npi_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default = "DateTime::now")]
    pub created_at: DateTime,
    #[serde(default = "DateTime::now")]
    pub updated_at: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_login: Option<DateTime>,
    #[serde(default)]
    pub mfa_enabled: bool,
    /// Security clearance level (1-5).
    #[serde(default = "default_clearance")]
    pub security_clearance: i32,
    /// FHIR R4 Practitioner ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fhir_practitioner_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fhir_organization_id: Option<String>,
    #[serde(default)]
    pub access_permissions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_audit: Option<DateTime>,
    #[serde(default)]
    pub consent_agreements: HashMap<String, DateTime>,
}

fn default_clearance() -> i32 {
    1
}

impl User {
    /// Create a new user with a generated UUID.
    pub fn new(
        email: String,
        username: String,
        first_name: String,
        last_name: String,
        role: UserRole,
    ) -> Self {
        Self {
            id: NexId::v4().to_string(),
            email,
            username,
            first_name,
            last_name,
            role,
            status: UserStatus::Pending,
            department: None,
            specialization: None,
            license_number: None,
            npi_number: None,
            tenant_id: None,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            last_login: None,
            mfa_enabled: false,
            security_clearance: 1,
            fhir_practitioner_id: None,
            fhir_organization_id: None,
            access_permissions: vec![],
            last_audit: None,
            consent_agreements: HashMap::new(),
        }
    }

    /// Get the user's full name.
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Check if user has a specific permission.
    pub fn has_permission(&self, permission: &str) -> bool {
        self.access_permissions.iter().any(|p| p == permission)
    }

    /// Check if user has MFA enabled.
    pub fn is_mfa_required(&self) -> bool {
        self.mfa_enabled || self.security_clearance >= 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_default() {
        assert_eq!(UserRole::default(), UserRole::Admin);
    }

    #[test]
    fn test_user_status_default() {
        assert_eq!(UserStatus::default(), UserStatus::Pending);
    }

    #[test]
    fn test_user_full_name() {
        let user = User::new(
            "john@example.com".to_string(),
            "jdoe".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Doctor,
        );

        assert_eq!(user.full_name(), "John Doe");
    }

    #[test]
    fn test_user_mfa_required() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "test".to_string(),
            "Test".to_string(),
            "User".to_string(),
            UserRole::Admin,
        );

        assert!(!user.is_mfa_required());

        user.security_clearance = 3;
        assert!(user.is_mfa_required());
    }
}
