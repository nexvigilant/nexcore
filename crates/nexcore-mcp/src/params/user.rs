//! User Management Parameters
//! Tier: T2-C (∃ + ∂ + μ — Identity Creation with Boundary)
//!
//! Account creation, login, logout, locking, and password management.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for creating a new user account.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserCreateParams {
    /// Username
    pub username: String,
    /// Display name
    pub display_name: String,
    /// Password
    pub password: String,
    /// Role: "Guest", "User", "Admin", "Owner"
    #[serde(default = "default_user_role")]
    pub role: String,
}

fn default_user_role() -> String {
    "User".to_string()
}

/// Parameters for user login.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserLoginParams {
    /// Username to authenticate.
    pub username: String,
    /// Password to verify.
    pub password: String,
}

/// Parameters for user logout.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserLogoutParams {
    /// Session token to invalidate.
    pub token: String,
}

/// Parameters for locking a user account.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserLockParams {
    /// Username to lock.
    pub username: String,
}

/// Parameters for unlocking a user account.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserUnlockParams {
    /// Username to unlock.
    pub username: String,
}

/// Parameters for changing a user's password.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserChangePasswordParams {
    /// Username whose password to change.
    pub username: String,
    /// Current password.
    pub old_password: String,
    /// New password.
    pub new_password: String,
}
