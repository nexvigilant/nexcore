//! User domain types: accounts, sessions, profiles, and role assignments.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

use super::enums::{CareerVertical, UserPersona, UserRole};

/// User account model.
///
/// Core authentication entity. Sensitive fields like `hashed_password` and
/// `mfa_secret` are excluded from serialization for security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier.
    pub id: NexId,

    /// User email address (unique, indexed).
    pub email: String,

    /// Full name.
    pub full_name: String,

    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Whether the account is active.
    #[serde(default = "default_true")]
    pub is_active: bool,

    /// Whether email has been verified.
    #[serde(default)]
    pub is_verified: bool,

    /// When email was verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified_at: Option<DateTime>,

    /// Last login timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_at: Option<DateTime>,

    /// Whether MFA is enabled.
    #[serde(default)]
    pub mfa_enabled: bool,

    /// Google OAuth ID (for SSO).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_id: Option<String>,

    /// Account creation timestamp.
    pub created_at: DateTime,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

fn default_true() -> bool {
    true
}

impl User {
    /// Create a new user with required fields.
    #[must_use]
    pub fn new(email: impl Into<String>, full_name: impl Into<String>) -> Self {
        Self {
            id: NexId::v4(),
            email: email.into(),
            full_name: full_name.into(),
            phone: None,
            is_active: true,
            is_verified: false,
            email_verified_at: None,
            last_login_at: None,
            mfa_enabled: false,
            google_id: None,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }
}

/// User session model for JWT token management.
///
/// PALACE CODE: Token hashes only - never store plain tokens.
/// SHA-256 hashes are 64 hex characters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Unique identifier.
    pub id: NexId,

    /// Associated user ID.
    pub user_id: NexId,

    /// SHA-256 hash of refresh token.
    pub refresh_token_hash: String,

    /// SHA-256 hash of access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_hash: Option<String>,

    /// Client IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// Client user agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Device name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,

    /// When session expires.
    pub expires_at: DateTime,

    /// Whether session is active.
    #[serde(default = "default_true")]
    pub is_active: bool,

    /// When session was revoked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime>,

    /// Session creation timestamp.
    pub created_at: DateTime,

    /// Last activity timestamp.
    pub last_activity_at: DateTime,
}

impl UserSession {
    /// Check if session is expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        DateTime::now() > self.expires_at
    }

    /// Check if session is valid (active and not expired).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired() && self.revoked_at.is_none()
    }
}

/// Extended user profile with persona and career tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique identifier.
    pub id: NexId,

    /// Associated user ID (unique).
    pub user_id: NexId,

    /// User persona classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona: Option<UserPersona>,

    /// Primary career interest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_career_vertical: Option<CareerVertical>,

    /// Secondary career interest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_career_vertical: Option<CareerVertical>,

    /// Current job title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_role: Option<String>,

    /// Years of experience.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub years_experience: Option<i32>,

    /// Education level (PharmD, PhD, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education_level: Option<String>,

    /// School or employer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub institution: Option<String>,

    /// Graduation year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graduation_year: Option<i32>,

    /// Free-text career goals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub career_goals: Option<String>,

    /// Interest tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interests: Vec<String>,

    /// User timezone.
    #[serde(default = "default_timezone")]
    pub timezone: String,

    /// Whether onboarding is complete.
    #[serde(default)]
    pub onboarding_completed: bool,

    /// When onboarding was completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onboarding_completed_at: Option<DateTime>,

    /// Whether quiz is complete.
    #[serde(default)]
    pub quiz_completed: bool,

    /// When quiz was completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quiz_completed_at: Option<DateTime>,

    /// Email notification preference.
    #[serde(default = "default_true")]
    pub email_notifications: bool,

    /// Weekly digest preference.
    #[serde(default = "default_true")]
    pub weekly_digest: bool,

    /// Community notification preference.
    #[serde(default = "default_true")]
    pub community_notifications: bool,

    /// User bio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,

    /// Avatar URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    /// LinkedIn URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin_url: Option<String>,

    /// Personal website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,

    /// Creation timestamp.
    pub created_at: DateTime,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

fn default_timezone() -> String {
    "America/New_York".to_string()
}

impl UserProfile {
    /// Create a new profile for a user.
    #[must_use]
    pub fn new(user_id: NexId) -> Self {
        Self {
            id: NexId::v4(),
            user_id,
            persona: None,
            primary_career_vertical: None,
            secondary_career_vertical: None,
            current_role: None,
            years_experience: None,
            education_level: None,
            institution: None,
            graduation_year: None,
            career_goals: None,
            interests: Vec::new(),
            timezone: default_timezone(),
            onboarding_completed: false,
            onboarding_completed_at: None,
            quiz_completed: false,
            quiz_completed_at: None,
            email_notifications: true,
            weekly_digest: true,
            community_notifications: true,
            bio: None,
            avatar_url: None,
            linkedin_url: None,
            website_url: None,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }
}

/// User-LLC relationship with role-based access control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLlcRole {
    /// Unique identifier.
    pub id: NexId,

    /// Associated user ID.
    pub user_id: NexId,

    /// Associated LLC ID.
    pub llc_id: NexId,

    /// User's role in this LLC.
    #[serde(default)]
    pub role: UserRole,

    /// Who invited this user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invited_by: Option<NexId>,

    /// When invitation was sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invited_at: Option<DateTime>,

    /// When invitation was accepted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_at: Option<DateTime>,

    /// Creation timestamp.
    pub created_at: DateTime,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

impl UserLlcRole {
    /// Create a new user-LLC role assignment.
    #[must_use]
    pub fn new(user_id: NexId, llc_id: NexId, role: UserRole) -> Self {
        Self {
            id: NexId::v4(),
            user_id,
            llc_id,
            role,
            invited_by: None,
            invited_at: None,
            accepted_at: None,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }

    /// Check if this is an owner role.
    #[must_use]
    pub fn is_owner(&self) -> bool {
        self.role == UserRole::Owner
    }

    /// Check if user has admin-level access.
    #[must_use]
    pub fn has_admin_access(&self) -> bool {
        matches!(self.role, UserRole::Owner | UserRole::Admin)
    }

    /// Check if user has write access.
    #[must_use]
    pub fn has_write_access(&self) -> bool {
        !matches!(self.role, UserRole::Viewer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let user = User::new("test@example.com", "Test User");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.full_name, "Test User");
        assert!(user.is_active);
        assert!(!user.is_verified);
    }

    #[test]
    fn test_user_profile_new() {
        let user_id = NexId::v4();
        let profile = UserProfile::new(user_id);
        assert_eq!(profile.user_id, user_id);
        assert_eq!(profile.timezone, "America/New_York");
        assert!(!profile.onboarding_completed);
    }

    #[test]
    fn test_user_llc_role_access() {
        let user_id = NexId::v4();
        let llc_id = NexId::v4();

        let owner = UserLlcRole::new(user_id, llc_id, UserRole::Owner);
        assert!(owner.is_owner());
        assert!(owner.has_admin_access());
        assert!(owner.has_write_access());

        let viewer = UserLlcRole::new(user_id, llc_id, UserRole::Viewer);
        assert!(!viewer.is_owner());
        assert!(!viewer.has_admin_access());
        assert!(!viewer.has_write_access());
    }

    #[test]
    fn test_user_serialization() {
        let user = User::new("test@example.com", "Test User");
        let json = serde_json::to_string(&user).unwrap();
        let parsed: User = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.email, user.email);
    }
}
