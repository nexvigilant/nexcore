//! User types — profiles, roles, auth state

use serde::{Deserialize, Serialize};
use crate::common::Timestamp;

/// User role in the platform
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    #[default]
    User,
    Student,
    Professional,
    Moderator,
    Admin,
}

/// Authentication state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    /// Initial loading state
    Loading,
    /// Authenticated with a valid token
    Authenticated,
    /// Not authenticated
    Unauthenticated,
}

/// User profile stored in Firestore
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    #[serde(default)]
    pub uid: String,
    #[serde(default)]
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    pub role: UserRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Timestamp>,
}

/// Membership tier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MembershipTier {
    Free,
    Student,
    Professional,
    Enterprise,
}
