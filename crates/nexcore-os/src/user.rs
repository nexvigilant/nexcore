// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! User accounts and authentication — login, sessions, and access control.
//!
//! ## Primitive Grounding
//!
//! | Component      | Primitives     | Role                        |
//! |----------------|----------------|-----------------------------|
//! | Authentication | ∂ + κ          | Boundary gate via comparison |
//! | Sessions       | ς + ν          | Stateful, time-bounded      |
//! | User records   | μ + π          | Persistent mapped identity   |
//! | Role system    | ∂ + κ + Σ      | Boundary classification      |
//! | Password hash  | ∝ + ∂          | Irreversible boundary        |
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
//! │  Shell/Login  │────▶│  UserManager  │────▶│    Vault      │
//! │  (UI prompt)  │     │  (auth gate)  │     │  (pwd store)  │
//! └──────────────┘     └──────────────┘     └──────────────┘
//!                             │
//!                             ▼
//!                      ┌──────────────┐
//!                      │   Session     │
//!                      │  (token+TTL)  │
//!                      └──────────────┘
//! ```

use core::fmt::Write as _;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Unique user identifier.
///
/// Tier: T2-P (∃ Existence — unique identity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub u64);

impl core::fmt::Display for UserId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "user-{}", self.0)
    }
}

/// User role — determines access level.
///
/// Tier: T2-P (∂ Boundary — role-based access boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UserRole {
    /// Guest: read-only, no settings, no installs.
    Guest = 0,
    /// Standard user: run apps, change own settings.
    User = 1,
    /// Administrator: install apps, manage services, manage users.
    Admin = 2,
    /// Root/owner: full system access (device owner).
    Owner = 3,
}

impl UserRole {
    /// Whether this role can install apps.
    pub fn can_install(&self) -> bool {
        matches!(self, Self::Admin | Self::Owner)
    }

    /// Whether this role can manage other users.
    pub fn can_manage_users(&self) -> bool {
        matches!(self, Self::Admin | Self::Owner)
    }

    /// Whether this role can access system settings.
    pub fn can_access_settings(&self) -> bool {
        !matches!(self, Self::Guest)
    }

    /// Whether this role can manage services.
    pub fn can_manage_services(&self) -> bool {
        matches!(self, Self::Admin | Self::Owner)
    }
}

impl core::fmt::Display for UserRole {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Guest => write!(f, "guest"),
            Self::User => write!(f, "user"),
            Self::Admin => write!(f, "admin"),
            Self::Owner => write!(f, "owner"),
        }
    }
}

/// Account status — whether the user can log in.
///
/// Tier: T2-P (ς State — account lifecycle)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    /// Account is active and can log in.
    Active,
    /// Account is locked (too many failed attempts or admin action).
    Locked,
    /// Account is disabled (soft delete).
    Disabled,
}

impl core::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Locked => write!(f, "locked"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

/// A user record — stored identity with hashed credentials.
///
/// Tier: T2-C (μ + π + ∂ + ς — mapped, persistent, bounded, stateful)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    /// Unique user ID.
    pub id: UserId,
    /// Username (unique, lowercase).
    pub username: String,
    /// Display name.
    pub display_name: String,
    /// Role-based access level.
    pub role: UserRole,
    /// Account status.
    pub status: AccountStatus,
    /// Salted password hash (SHA-256).
    password_hash: String,
    /// Salt used for password hashing.
    salt: String,
    /// When the account was created.
    pub created_at: DateTime,
    /// Last successful login.
    pub last_login: Option<DateTime>,
    /// Failed login attempt counter (resets on success).
    pub failed_attempts: u32,
}

impl UserRecord {
    /// Maximum failed login attempts before auto-lock.
    const MAX_FAILED_ATTEMPTS: u32 = 5;

    /// Check if this account should be auto-locked due to failed attempts.
    pub fn should_auto_lock(&self) -> bool {
        self.failed_attempts >= Self::MAX_FAILED_ATTEMPTS
    }
}

/// A login session — time-bounded authenticated context.
///
/// Tier: T2-C (ς + ν + ∂ — stateful, time-bounded, bounded)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session token (SHA-256 hex).
    pub token: String,
    /// User ID this session belongs to.
    pub user_id: UserId,
    /// Username (cached for display).
    pub username: String,
    /// Role at time of login.
    pub role: UserRole,
    /// When the session was created.
    pub created_at: DateTime,
    /// When the session expires.
    pub expires_at: DateTime,
    /// Whether the session is still active.
    pub active: bool,
}

impl Session {
    /// Default session duration: 24 hours.
    const DEFAULT_TTL_HOURS: i64 = 24;

    /// Check if the session has expired.
    pub fn is_expired(&self) -> bool {
        DateTime::now() > self.expires_at
    }

    /// Check if the session is valid (active and not expired).
    pub fn is_valid(&self) -> bool {
        self.active && !self.is_expired()
    }
}

/// User authentication error.
///
/// Tier: T2-P (∂ Boundary — authentication failure boundary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// User not found.
    UserNotFound(String),
    /// Invalid password.
    InvalidPassword,
    /// Account is locked.
    AccountLocked(String),
    /// Account is disabled.
    AccountDisabled(String),
    /// Session expired or invalid.
    InvalidSession,
    /// Username already exists.
    UserAlreadyExists(String),
    /// Insufficient permissions for operation.
    InsufficientRole {
        required: UserRole,
        actual: UserRole,
    },
    /// Password does not meet requirements.
    WeakPassword(String),
}

impl core::fmt::Display for AuthError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UserNotFound(u) => write!(f, "user not found: {u}"),
            Self::InvalidPassword => write!(f, "invalid password"),
            Self::AccountLocked(u) => write!(f, "account locked: {u}"),
            Self::AccountDisabled(u) => write!(f, "account disabled: {u}"),
            Self::InvalidSession => write!(f, "invalid or expired session"),
            Self::UserAlreadyExists(u) => write!(f, "user already exists: {u}"),
            Self::InsufficientRole { required, actual } => {
                write!(f, "requires {required}, have {actual}")
            }
            Self::WeakPassword(reason) => write!(f, "weak password: {reason}"),
        }
    }
}

/// User and session manager — the OS authentication subsystem.
///
/// Tier: T3 (μ + π + ∂ + ς + κ + ∃ — full identity management)
pub struct UserManager {
    /// All user records, keyed by user ID.
    users: HashMap<UserId, UserRecord>,
    /// Username → user ID index.
    username_index: HashMap<String, UserId>,
    /// Active sessions, keyed by token.
    sessions: HashMap<String, Session>,
    /// Next user ID counter.
    next_id: u64,
    /// Session counter (for unique token generation).
    session_counter: u64,
}

impl UserManager {
    /// Create a new user manager.
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            username_index: HashMap::new(),
            sessions: HashMap::new(),
            next_id: 1,
            session_counter: 0,
        }
    }

    /// Create the initial owner account (device setup).
    ///
    /// This is called during first boot to create the device owner.
    /// The owner has full system access and cannot be deleted.
    pub fn create_owner(
        &mut self,
        username: &str,
        display_name: &str,
        password: &str,
    ) -> Result<UserId, AuthError> {
        self.create_user_internal(username, display_name, password, UserRole::Owner)
    }

    /// Create a new user account.
    ///
    /// Requires the caller to have Admin or Owner role (enforced at kernel level).
    pub fn create_user(
        &mut self,
        username: &str,
        display_name: &str,
        password: &str,
        role: UserRole,
    ) -> Result<UserId, AuthError> {
        // Validate password strength
        Self::validate_password(password)?;
        self.create_user_internal(username, display_name, password, role)
    }

    /// Internal user creation (bypasses password validation for owner setup).
    fn create_user_internal(
        &mut self,
        username: &str,
        display_name: &str,
        password: &str,
        role: UserRole,
    ) -> Result<UserId, AuthError> {
        let normalized = username.to_lowercase();

        if self.username_index.contains_key(&normalized) {
            return Err(AuthError::UserAlreadyExists(normalized));
        }

        let id = UserId(self.next_id);
        self.next_id += 1;

        let salt = Self::generate_salt(&normalized);
        let password_hash = Self::hash_password(password, &salt);

        let record = UserRecord {
            id,
            username: normalized.clone(),
            display_name: display_name.to_string(),
            role,
            status: AccountStatus::Active,
            password_hash,
            salt,
            created_at: DateTime::now(),
            last_login: None,
            failed_attempts: 0,
        };

        self.users.insert(id, record);
        self.username_index.insert(normalized, id);

        Ok(id)
    }

    /// Validate password meets minimum requirements.
    fn validate_password(password: &str) -> Result<(), AuthError> {
        if password.len() < 8 {
            return Err(AuthError::WeakPassword("minimum 8 characters".to_string()));
        }

        let has_upper = password.chars().any(char::is_uppercase);
        let has_lower = password.chars().any(char::is_lowercase);
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        if !has_upper || !has_lower || !has_digit {
            return Err(AuthError::WeakPassword(
                "must contain uppercase, lowercase, and digit".to_string(),
            ));
        }

        Ok(())
    }

    /// Authenticate a user — verify credentials and create a session.
    pub fn login(&mut self, username: &str, password: &str) -> Result<Session, AuthError> {
        let normalized = username.to_lowercase();

        let user_id = self
            .username_index
            .get(&normalized)
            .copied()
            .ok_or_else(|| AuthError::UserNotFound(normalized.clone()))?;

        let user = self
            .users
            .get_mut(&user_id)
            .ok_or_else(|| AuthError::UserNotFound(normalized.clone()))?;

        // Check account status
        match user.status {
            AccountStatus::Active => {}
            AccountStatus::Locked => return Err(AuthError::AccountLocked(normalized)),
            AccountStatus::Disabled => return Err(AuthError::AccountDisabled(normalized)),
        }

        // Verify password
        let hash = Self::hash_password(password, &user.salt);
        if hash != user.password_hash {
            user.failed_attempts += 1;

            // Auto-lock after too many failures
            if user.should_auto_lock() {
                user.status = AccountStatus::Locked;
                return Err(AuthError::AccountLocked(normalized));
            }

            return Err(AuthError::InvalidPassword);
        }

        // Successful login — reset failure counter
        user.failed_attempts = 0;
        user.last_login = Some(DateTime::now());

        let role = user.role;
        let display = user.display_name.clone();

        // Create session
        let session = self.create_session(user_id, &normalized, role);

        // Emit would happen at kernel level
        let _ = display;

        Ok(session)
    }

    /// Log out — invalidate a session.
    pub fn logout(&mut self, token: &str) -> Result<(), AuthError> {
        let session = self
            .sessions
            .get_mut(token)
            .ok_or(AuthError::InvalidSession)?;

        session.active = false;
        Ok(())
    }

    /// Validate a session token — returns the session if valid.
    pub fn validate_session(&self, token: &str) -> Result<&Session, AuthError> {
        let session = self.sessions.get(token).ok_or(AuthError::InvalidSession)?;

        if !session.is_valid() {
            return Err(AuthError::InvalidSession);
        }

        Ok(session)
    }

    /// Change a user's password.
    pub fn change_password(
        &mut self,
        username: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), AuthError> {
        let normalized = username.to_lowercase();

        Self::validate_password(new_password)?;

        let user_id = self
            .username_index
            .get(&normalized)
            .copied()
            .ok_or_else(|| AuthError::UserNotFound(normalized.clone()))?;

        let user = self
            .users
            .get_mut(&user_id)
            .ok_or_else(|| AuthError::UserNotFound(normalized.clone()))?;

        // Verify old password
        let old_hash = Self::hash_password(old_password, &user.salt);
        if old_hash != user.password_hash {
            return Err(AuthError::InvalidPassword);
        }

        // Set new password
        let new_salt = Self::generate_salt(&normalized);
        user.password_hash = Self::hash_password(new_password, &new_salt);
        user.salt = new_salt;

        Ok(())
    }

    /// Lock a user account (admin action).
    pub fn lock_user(&mut self, username: &str) -> Result<(), AuthError> {
        let user = self.get_user_mut_by_name(username)?;
        user.status = AccountStatus::Locked;
        user.failed_attempts = 0;

        // Invalidate all sessions for this user
        let user_id = user.id;
        self.invalidate_user_sessions(user_id);

        Ok(())
    }

    /// Unlock a user account (admin action).
    pub fn unlock_user(&mut self, username: &str) -> Result<(), AuthError> {
        let user = self.get_user_mut_by_name(username)?;
        user.status = AccountStatus::Active;
        user.failed_attempts = 0;
        Ok(())
    }

    /// Disable a user account (soft delete).
    pub fn disable_user(&mut self, username: &str) -> Result<(), AuthError> {
        let user = self.get_user_mut_by_name(username)?;
        user.status = AccountStatus::Disabled;

        let user_id = user.id;
        self.invalidate_user_sessions(user_id);

        Ok(())
    }

    /// Get a user record by username.
    pub fn get_user(&self, username: &str) -> Option<&UserRecord> {
        let normalized = username.to_lowercase();
        let id = self.username_index.get(&normalized)?;
        self.users.get(id)
    }

    /// Get a user record by ID.
    pub fn get_user_by_id(&self, id: UserId) -> Option<&UserRecord> {
        self.users.get(&id)
    }

    /// List all users (summary info).
    pub fn list_users(&self) -> Vec<UserSummary> {
        let mut users: Vec<_> = self
            .users
            .values()
            .map(|u| UserSummary {
                id: u.id,
                username: u.username.clone(),
                display_name: u.display_name.clone(),
                role: u.role,
                status: u.status,
                last_login: u.last_login,
            })
            .collect();
        users.sort_by_key(|u| u.id.0);
        users
    }

    /// Count total users.
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Count active sessions.
    pub fn active_session_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_valid()).count()
    }

    /// Get all active sessions.
    pub fn active_sessions(&self) -> Vec<&Session> {
        self.sessions.values().filter(|s| s.is_valid()).collect()
    }

    /// Expire old sessions (housekeeping).
    pub fn cleanup_expired_sessions(&mut self) -> usize {
        let expired: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(token, _)| token.clone())
            .collect();

        let count = expired.len();
        for token in &expired {
            self.sessions.remove(token);
        }
        count
    }

    // ── Internal helpers ─────────────────────────────────────────

    /// Generate a deterministic salt from username.
    ///
    /// In production, use a CSPRNG. For the OS layer we derive from
    /// username + pepper to avoid needing a `rand` dependency.
    fn generate_salt(username: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"nexcore-os-salt-pepper-v1:");
        hasher.update(username.as_bytes());
        hasher.update(b":salt");
        let result = hasher.finalize();

        result[..16]
            .iter()
            .fold(String::with_capacity(32), |mut s, b| {
                let _ = write!(s, "{b:02x}");
                s
            })
    }

    /// Hash a password with the given salt.
    ///
    /// Uses SHA-256(salt || password || salt) — double-salt sandwich.
    fn hash_password(password: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(password.as_bytes());
        hasher.update(salt.as_bytes());
        let result = hasher.finalize();

        result.iter().fold(String::with_capacity(64), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
    }

    /// Create a new session for a user.
    fn create_session(&mut self, user_id: UserId, username: &str, role: UserRole) -> Session {
        self.session_counter += 1;
        let now = DateTime::now();

        // Generate session token: SHA-256(user_id || counter || timestamp)
        let mut hasher = Sha256::new();
        hasher.update(user_id.0.to_le_bytes());
        hasher.update(self.session_counter.to_le_bytes());
        hasher.update(now.timestamp().to_le_bytes());
        hasher.update(b"nexcore-session");
        let result = hasher.finalize();

        let token = result.iter().fold(String::with_capacity(64), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        });

        let session = Session {
            token: token.clone(),
            user_id,
            username: username.to_string(),
            role,
            created_at: now,
            expires_at: now + nexcore_chrono::Duration::hours(Session::DEFAULT_TTL_HOURS),
            active: true,
        };

        self.sessions.insert(token, session.clone());
        session
    }

    /// Get a mutable user record by username.
    fn get_user_mut_by_name(&mut self, username: &str) -> Result<&mut UserRecord, AuthError> {
        let normalized = username.to_lowercase();
        let id = self
            .username_index
            .get(&normalized)
            .copied()
            .ok_or(AuthError::UserNotFound(normalized))?;
        self.users
            .get_mut(&id)
            .ok_or_else(|| AuthError::UserNotFound(username.to_string()))
    }

    /// Invalidate all sessions for a user.
    fn invalidate_user_sessions(&mut self, user_id: UserId) {
        for session in self.sessions.values_mut() {
            if session.user_id == user_id {
                session.active = false;
            }
        }
    }
}

impl Default for UserManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of a user account (safe to expose via MCP).
///
/// Tier: T2-P (μ Mapping — projection of user record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    /// User ID.
    pub id: UserId,
    /// Username.
    pub username: String,
    /// Display name.
    pub display_name: String,
    /// Role.
    pub role: UserRole,
    /// Account status.
    pub status: AccountStatus,
    /// Last login time.
    pub last_login: Option<DateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_owner() {
        let mut mgr = UserManager::new();
        let result = mgr.create_owner("matthew", "Matthew Campion", "Nexcore2026!");
        assert!(result.is_ok());

        let id = result.unwrap_or(UserId(0));
        assert_eq!(id, UserId(1));
        assert_eq!(mgr.user_count(), 1);
    }

    #[test]
    fn create_user_validates_password() {
        let mut mgr = UserManager::new();

        // Too short
        let r = mgr.create_user("alice", "Alice", "short", UserRole::User);
        assert!(r.is_err());

        // No uppercase
        let r = mgr.create_user("alice", "Alice", "alllowercase1", UserRole::User);
        assert!(r.is_err());

        // No digit
        let r = mgr.create_user("alice", "Alice", "NoDigitHere", UserRole::User);
        assert!(r.is_err());

        // Valid
        let r = mgr.create_user("alice", "Alice", "ValidPass1", UserRole::User);
        assert!(r.is_ok());
    }

    #[test]
    fn duplicate_username_rejected() {
        let mut mgr = UserManager::new();
        let r1 = mgr.create_owner("matthew", "Matthew", "Pass1234");
        assert!(r1.is_ok());

        let r2 = mgr.create_user("Matthew", "Matt", "Pass1234", UserRole::User);
        assert!(r2.is_err());
        if let Err(AuthError::UserAlreadyExists(name)) = r2 {
            assert_eq!(name, "matthew"); // Normalized to lowercase
        }
    }

    #[test]
    fn login_success() {
        let mut mgr = UserManager::new();
        let _id = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");
        let session = mgr.login("matthew", "Nexcore2026!");
        assert!(session.is_ok());

        if let Ok(s) = session {
            assert!(s.is_valid());
            assert_eq!(s.username, "matthew");
            assert_eq!(s.role, UserRole::Owner);
            assert!(!s.token.is_empty());
        }
    }

    #[test]
    fn login_wrong_password() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");
        let result = mgr.login("matthew", "WrongPassword1");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(AuthError::InvalidPassword));
    }

    #[test]
    fn login_user_not_found() {
        let mut mgr = UserManager::new();
        let result = mgr.login("nobody", "Pass1234");
        assert!(result.is_err());
    }

    #[test]
    fn auto_lock_after_failed_attempts() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");

        // Fail 5 times
        for _ in 0..5 {
            let _ = mgr.login("matthew", "WrongPass1");
        }

        // Account should now be locked
        let result = mgr.login("matthew", "Nexcore2026!");
        assert_eq!(
            result.err(),
            Some(AuthError::AccountLocked("matthew".to_string()))
        );
    }

    #[test]
    fn login_resets_failed_count() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");

        // Fail 3 times
        for _ in 0..3 {
            let _ = mgr.login("matthew", "WrongPass1");
        }

        // Successful login
        let result = mgr.login("matthew", "Nexcore2026!");
        assert!(result.is_ok());

        // Failed attempts reset
        let user = mgr.get_user("matthew");
        assert_eq!(user.map(|u| u.failed_attempts), Some(0));
    }

    #[test]
    fn logout_invalidates_session() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");
        let session = mgr.login("matthew", "Nexcore2026!");
        assert!(session.is_ok());

        if let Ok(s) = session {
            let token = s.token.clone();

            // Session valid before logout
            assert!(mgr.validate_session(&token).is_ok());

            // Logout
            let r = mgr.logout(&token);
            assert!(r.is_ok());

            // Session invalid after logout
            assert!(mgr.validate_session(&token).is_err());
        }
    }

    #[test]
    fn validate_session_rejects_invalid_token() {
        let mgr = UserManager::new();
        let result = mgr.validate_session("nonexistent-token");
        assert!(result.is_err());
    }

    #[test]
    fn change_password() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");

        // Change with wrong old password fails
        let r = mgr.change_password("matthew", "WrongOld1", "NewPass123");
        assert!(r.is_err());

        // Change with correct old password
        let r = mgr.change_password("matthew", "Nexcore2026!", "NewPass123");
        assert!(r.is_ok());

        // Old password no longer works
        let login = mgr.login("matthew", "Nexcore2026!");
        assert!(login.is_err());

        // New password works
        let login = mgr.login("matthew", "NewPass123");
        assert!(login.is_ok());
    }

    #[test]
    fn lock_unlock_user() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");
        let _ = mgr.create_user("alice", "Alice", "AlicePass1", UserRole::User);

        // Lock alice
        let r = mgr.lock_user("alice");
        assert!(r.is_ok());

        // Login fails
        let login = mgr.login("alice", "AlicePass1");
        assert_eq!(
            login.err(),
            Some(AuthError::AccountLocked("alice".to_string()))
        );

        // Unlock alice
        let r = mgr.unlock_user("alice");
        assert!(r.is_ok());

        // Login succeeds
        let login = mgr.login("alice", "AlicePass1");
        assert!(login.is_ok());
    }

    #[test]
    fn disable_user_invalidates_sessions() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");
        let _ = mgr.create_user("bob", "Bob", "BobPass123", UserRole::User);

        // Bob logs in
        let session = mgr.login("bob", "BobPass123");
        assert!(session.is_ok());
        let token = session.map(|s| s.token.clone()).unwrap_or_default();

        // Disable bob
        let r = mgr.disable_user("bob");
        assert!(r.is_ok());

        // Session invalidated
        assert!(mgr.validate_session(&token).is_err());

        // Login fails
        let login = mgr.login("bob", "BobPass123");
        assert_eq!(
            login.err(),
            Some(AuthError::AccountDisabled("bob".to_string()))
        );
    }

    #[test]
    fn list_users_sorted() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("charlie", "Charlie", "Charlie123");
        let _ = mgr.create_user("alice", "Alice", "AlicePass1", UserRole::User);
        let _ = mgr.create_user("bob", "Bob", "BobPass123", UserRole::Admin);

        let users = mgr.list_users();
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].username, "charlie");
        assert_eq!(users[1].username, "alice");
        assert_eq!(users[2].username, "bob");
    }

    #[test]
    fn active_session_count() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");
        let _ = mgr.create_user("alice", "Alice", "AlicePass1", UserRole::User);

        assert_eq!(mgr.active_session_count(), 0);

        let s1 = mgr.login("matthew", "Nexcore2026!");
        assert!(s1.is_ok());
        assert_eq!(mgr.active_session_count(), 1);

        let s2 = mgr.login("alice", "AlicePass1");
        assert!(s2.is_ok());
        assert_eq!(mgr.active_session_count(), 2);

        // Logout matthew
        if let Ok(s) = s1 {
            let _ = mgr.logout(&s.token);
        }
        assert_eq!(mgr.active_session_count(), 1);
    }

    #[test]
    fn password_hash_deterministic() {
        let salt = UserManager::generate_salt("testuser");
        let h1 = UserManager::hash_password("password", &salt);
        let h2 = UserManager::hash_password("password", &salt);
        let h3 = UserManager::hash_password("different", &salt);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn salt_unique_per_user() {
        let s1 = UserManager::generate_salt("alice");
        let s2 = UserManager::generate_salt("bob");
        assert_ne!(s1, s2);
    }

    #[test]
    fn case_insensitive_username() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("Matthew", "Matthew", "Nexcore2026!");

        // Login with different case
        let result = mgr.login("MATTHEW", "Nexcore2026!");
        assert!(result.is_ok());

        let result = mgr.login("matthew", "Nexcore2026!");
        assert!(result.is_ok());
    }

    #[test]
    fn role_permissions() {
        assert!(!UserRole::Guest.can_install());
        assert!(!UserRole::Guest.can_manage_users());
        assert!(!UserRole::Guest.can_access_settings());

        assert!(!UserRole::User.can_install());
        assert!(!UserRole::User.can_manage_users());
        assert!(UserRole::User.can_access_settings());

        assert!(UserRole::Admin.can_install());
        assert!(UserRole::Admin.can_manage_users());
        assert!(UserRole::Admin.can_manage_services());

        assert!(UserRole::Owner.can_install());
        assert!(UserRole::Owner.can_manage_users());
        assert!(UserRole::Owner.can_manage_services());
    }

    #[test]
    fn session_token_unique() {
        let mut mgr = UserManager::new();
        let _ = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");

        let s1 = mgr.login("matthew", "Nexcore2026!");
        let s2 = mgr.login("matthew", "Nexcore2026!");

        assert!(s1.is_ok());
        assert!(s2.is_ok());

        if let (Ok(a), Ok(b)) = (s1, s2) {
            assert_ne!(a.token, b.token);
        }
    }

    #[test]
    fn get_user_by_id() {
        let mut mgr = UserManager::new();
        let id = mgr.create_owner("matthew", "Matthew", "Nexcore2026!");
        assert!(id.is_ok());

        if let Ok(uid) = id {
            let user = mgr.get_user_by_id(uid);
            assert!(user.is_some());
            assert_eq!(user.map(|u| u.username.as_str()), Some("matthew"));
        }
    }

    #[test]
    fn default_creates_empty() {
        let mgr = UserManager::default();
        assert_eq!(mgr.user_count(), 0);
        assert_eq!(mgr.active_session_count(), 0);
    }
}
