//! User domain types.
//!
//! Defines user accounts, authentication methods, and session tracking.
//! Migrated from Python Ormar User model.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

/// User account in the quiz platform.
///
/// Supports multiple authentication methods: local password, Google, GitHub,
/// or custom OpenID Connect providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier.
    pub id: NexId,

    /// User's email address (unique).
    pub email: String,

    /// Display username (unique).
    pub username: String,

    /// Argon2 password hash (None for OAuth-only users).
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,

    /// Whether email has been verified.
    pub verified: bool,

    /// Email verification key (random token).
    #[serde(skip_serializing)]
    pub verify_key: Option<String>,

    /// Account creation timestamp.
    pub created_at: DateTime,

    /// Authentication method used.
    pub auth_type: UserAuthType,

    /// Google OAuth UID (if using Google auth).
    pub google_uid: Option<String>,

    /// GitHub user ID (if using GitHub auth).
    pub github_user_id: Option<i64>,

    /// Avatar image data (base64 encoded, max 25KB).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub avatar: Vec<u8>,

    /// Whether password is required for login.
    pub require_password: bool,

    /// Backup code for 2FA recovery (64 hex chars).
    #[serde(skip_serializing)]
    pub backup_code: String,

    /// TOTP secret for 2FA (32 chars, base32).
    #[serde(skip_serializing)]
    pub totp_secret: Option<String>,

    /// Total storage used by this user in bytes.
    pub storage_used: i64,
}

impl User {
    /// Create a new local user with email/password authentication.
    pub fn new_local(email: String, username: String, password_hash: String) -> Self {
        Self {
            id: NexId::v4(),
            email,
            username,
            password_hash: Some(password_hash),
            verified: false,
            verify_key: Some(generate_verify_key()),
            created_at: DateTime::now(),
            auth_type: UserAuthType::Local,
            google_uid: None,
            github_user_id: None,
            avatar: Vec::new(),
            require_password: true,
            backup_code: generate_backup_code(),
            totp_secret: None,
            storage_used: 0,
        }
    }

    /// Create a new OAuth user (Google/GitHub/Custom).
    pub fn new_oauth(
        email: String,
        username: String,
        auth_type: UserAuthType,
        provider_uid: Option<String>,
    ) -> Self {
        Self {
            id: NexId::v4(),
            email,
            username,
            password_hash: None,
            verified: true, // OAuth users are pre-verified
            verify_key: None,
            created_at: DateTime::now(),
            auth_type,
            google_uid: if auth_type == UserAuthType::Google {
                provider_uid.clone()
            } else {
                None
            },
            github_user_id: None,
            avatar: Vec::new(),
            require_password: false,
            backup_code: generate_backup_code(),
            totp_secret: None,
            storage_used: 0,
        }
    }

    /// Check if user has 2FA enabled.
    pub fn has_2fa(&self) -> bool {
        self.totp_secret.is_some()
    }

    /// Check if user is a moderator.
    pub fn is_mod(&self, mod_list: &[String]) -> bool {
        mod_list.contains(&self.username)
    }
}

/// Authentication method used for a user account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserAuthType {
    /// Local email/password authentication.
    #[default]
    Local,
    /// Google OAuth.
    Google,
    /// GitHub OAuth.
    #[serde(rename = "GITHUB")]
    GitHub,
    /// Custom OpenID Connect provider.
    Custom,
}

/// WebAuthn/FIDO2 credential for passwordless authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidoCredential {
    /// Internal primary key.
    pub pk: i32,

    /// Credential ID (from WebAuthn).
    pub id: Vec<u8>,

    /// Public key for verification.
    pub public_key: Vec<u8>,

    /// Signature counter for replay protection.
    pub sign_count: u32,

    /// User this credential belongs to.
    pub user_id: NexId,
}

/// API key for programmatic access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// The API key value (48 characters).
    pub key: String,

    /// User this key belongs to.
    pub user_id: NexId,
}

impl ApiKey {
    /// Generate a new random API key.
    pub fn generate(user_id: NexId) -> Self {
        Self {
            key: generate_api_key(),
            user_id,
        }
    }
}

/// User login session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Session identifier.
    pub id: NexId,

    /// User this session belongs to.
    pub user_id: NexId,

    /// Session key (random token).
    pub session_key: String,

    /// Session creation time.
    pub created_at: DateTime,

    /// IP address of the client.
    pub ip_address: Option<String>,

    /// User agent string.
    pub user_agent: Option<String>,

    /// Last activity timestamp.
    pub last_seen: DateTime,
}

impl UserSession {
    /// Create a new session for a user.
    pub fn new(user_id: NexId, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        let now = DateTime::now();
        Self {
            id: NexId::v4(),
            user_id,
            session_key: generate_session_key(),
            created_at: now,
            ip_address,
            user_agent,
            last_seen: now,
        }
    }
}

/// Public user response (safe for API responses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    /// User ID.
    pub id: NexId,

    /// Username.
    pub username: String,
}

impl From<&User> for PublicUser {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
        }
    }
}

// === Helper functions ===

fn generate_verify_key() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..32)
        .map(|_| format!("{:02x}", rng.random::<u8>()))
        .collect()
}

fn generate_backup_code() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..32)
        .map(|_| format!("{:02x}", rng.random::<u8>()))
        .collect()
}

fn generate_session_key() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..32)
        .map(|_| format!("{:02x}", rng.random::<u8>()))
        .collect()
}

fn generate_api_key() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..24)
        .map(|_| format!("{:02x}", rng.random::<u8>()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_local_user() {
        let user = User::new_local(
            "test@example.com".into(),
            "testuser".into(),
            "hashed_password".into(),
        );

        assert_eq!(user.auth_type, UserAuthType::Local);
        assert!(!user.verified);
        assert!(user.verify_key.is_some());
        assert!(user.password_hash.is_some());
        assert!(!user.has_2fa());
    }

    #[test]
    fn test_new_oauth_user() {
        let user = User::new_oauth(
            "test@example.com".into(),
            "testuser".into(),
            UserAuthType::Google,
            Some("google_uid_123".into()),
        );

        assert_eq!(user.auth_type, UserAuthType::Google);
        assert!(user.verified);
        assert!(user.password_hash.is_none());
        assert_eq!(user.google_uid, Some("google_uid_123".into()));
    }

    #[test]
    fn test_api_key_generation() {
        let user_id = NexId::v4();
        let key = ApiKey::generate(user_id);

        assert_eq!(key.key.len(), 48);
        assert_eq!(key.user_id, user_id);
    }

    #[test]
    fn test_backup_code_length() {
        let code = generate_backup_code();
        assert_eq!(code.len(), 64);
    }
}
