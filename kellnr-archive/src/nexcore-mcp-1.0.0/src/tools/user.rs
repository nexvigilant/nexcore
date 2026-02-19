//! User Management MCP tools — authentication, sessions, and access control.
//!
//! # T1 Grounding
//! - ∂ (boundary): Authentication gate, role-based access
//! - κ (comparison): Credential verification, role ordering
//! - ς (state): Account status (Active/Locked/Disabled), session lifecycle
//! - μ (mapping): Username→UserId, token→Session
//! - π (persistence): User records, session store
//! - ∃ (existence): User creation, identity

use nexcore_os::user::{UserManager, UserRole};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    UserChangePasswordParams, UserCreateParams, UserLockParams, UserLoginParams, UserLogoutParams,
    UserUnlockParams,
};

/// Parse a user role from string input.
fn parse_role(s: &str) -> Result<UserRole, McpError> {
    match s.to_lowercase().as_str() {
        "guest" => Ok(UserRole::Guest),
        "user" => Ok(UserRole::User),
        "admin" => Ok(UserRole::Admin),
        "owner" => Ok(UserRole::Owner),
        _ => Err(McpError::invalid_params(
            format!("Unknown role: '{s}'. Valid: Guest, User, Admin, Owner"),
            None,
        )),
    }
}

// Thread-local user manager for stateful tool calls within a session.
// Each MCP server process maintains one UserManager instance,
// allowing login/logout/list to persist across tool calls.
std::thread_local! {
    static USER_MANAGER: std::cell::RefCell<UserManager> = std::cell::RefCell::new(UserManager::new());
}

/// Create a new user account.
///
/// Tier: T2-C (∃ + ∂ + μ — identity creation with auth boundary)
pub fn user_create(params: UserCreateParams) -> Result<CallToolResult, McpError> {
    let role = parse_role(&params.role)?;

    USER_MANAGER.with(|mgr| {
        let mut mgr = mgr.borrow_mut();

        // First user is always owner, subsequent use create_user with validation
        let result = if mgr.user_count() == 0 && role == UserRole::Owner {
            mgr.create_owner(&params.username, &params.display_name, &params.password)
        } else {
            mgr.create_user(
                &params.username,
                &params.display_name,
                &params.password,
                role,
            )
        };

        match result {
            Ok(id) => {
                let output = serde_json::json!({
                    "status": "created",
                    "user_id": id.0,
                    "username": params.username.to_lowercase(),
                    "display_name": params.display_name,
                    "role": format!("{role}"),
                    "total_users": mgr.user_count(),
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&output).unwrap_or_default(),
                )]))
            }
            Err(e) => Err(McpError::invalid_params(format!("{e}"), None)),
        }
    })
}

/// Authenticate a user and create a session.
///
/// Tier: T2-C (κ + ς + ν — verify credentials, create time-bounded session)
pub fn user_login(params: UserLoginParams) -> Result<CallToolResult, McpError> {
    USER_MANAGER.with(|mgr| {
        let mut mgr = mgr.borrow_mut();

        match mgr.login(&params.username, &params.password) {
            Ok(session) => {
                let output = serde_json::json!({
                    "status": "authenticated",
                    "token": session.token,
                    "user_id": session.user_id.0,
                    "username": session.username,
                    "role": format!("{}", session.role),
                    "expires_at": session.expires_at.to_rfc3339(),
                    "active_sessions": mgr.active_session_count(),
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&output).unwrap_or_default(),
                )]))
            }
            Err(e) => Err(McpError::invalid_params(format!("{e}"), None)),
        }
    })
}

/// Invalidate a session (log out).
///
/// Tier: T2-P (ς State — session lifecycle transition)
pub fn user_logout(params: UserLogoutParams) -> Result<CallToolResult, McpError> {
    USER_MANAGER.with(|mgr| {
        let mut mgr = mgr.borrow_mut();

        match mgr.logout(&params.token) {
            Ok(()) => {
                let output = serde_json::json!({
                    "status": "logged_out",
                    "active_sessions": mgr.active_session_count(),
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&output).unwrap_or_default(),
                )]))
            }
            Err(e) => Err(McpError::invalid_params(format!("{e}"), None)),
        }
    })
}

/// List all user accounts (summary info, no secrets).
///
/// Tier: T2-P (μ Mapping — user record projection)
pub fn user_list() -> CallToolResult {
    USER_MANAGER.with(|mgr| {
        let mgr = mgr.borrow();
        let users = mgr.list_users();

        let user_list: Vec<serde_json::Value> = users
            .iter()
            .map(|u| {
                serde_json::json!({
                    "id": u.id.0,
                    "username": u.username,
                    "display_name": u.display_name,
                    "role": format!("{}", u.role),
                    "status": format!("{}", u.status),
                    "last_login": u.last_login.map(|t| t.to_rfc3339()),
                })
            })
            .collect();

        let output = serde_json::json!({
            "total_users": mgr.user_count(),
            "active_sessions": mgr.active_session_count(),
            "users": user_list,
        });
        CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&output).unwrap_or_default(),
        )])
    })
}

/// Lock a user account (admin action).
///
/// Tier: T2-P (ς State — account state: Active → Locked)
pub fn user_lock(params: UserLockParams) -> Result<CallToolResult, McpError> {
    USER_MANAGER.with(|mgr| {
        let mut mgr = mgr.borrow_mut();

        match mgr.lock_user(&params.username) {
            Ok(()) => {
                let output = serde_json::json!({
                    "status": "locked",
                    "username": params.username.to_lowercase(),
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&output).unwrap_or_default(),
                )]))
            }
            Err(e) => Err(McpError::invalid_params(format!("{e}"), None)),
        }
    })
}

/// Unlock a user account (admin action).
///
/// Tier: T2-P (ς State — account state: Locked → Active)
pub fn user_unlock(params: UserUnlockParams) -> Result<CallToolResult, McpError> {
    USER_MANAGER.with(|mgr| {
        let mut mgr = mgr.borrow_mut();

        match mgr.unlock_user(&params.username) {
            Ok(()) => {
                let output = serde_json::json!({
                    "status": "unlocked",
                    "username": params.username.to_lowercase(),
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&output).unwrap_or_default(),
                )]))
            }
            Err(e) => Err(McpError::invalid_params(format!("{e}"), None)),
        }
    })
}

/// Get user management status overview.
///
/// Tier: T2-C (Σ + ς — aggregated system state)
pub fn user_status() -> CallToolResult {
    USER_MANAGER.with(|mgr| {
        let mgr = mgr.borrow();

        let sessions = mgr.active_sessions();
        let session_info: Vec<serde_json::Value> = sessions
            .iter()
            .map(|s| {
                serde_json::json!({
                    "username": s.username,
                    "role": format!("{}", s.role),
                    "created_at": s.created_at.to_rfc3339(),
                    "expires_at": s.expires_at.to_rfc3339(),
                })
            })
            .collect();

        let output = serde_json::json!({
            "total_users": mgr.user_count(),
            "active_sessions": mgr.active_session_count(),
            "sessions": session_info,
            "subsystem": "user-auth",
            "primitive_grounding": "∂ + κ + ς + π + μ",
        });
        CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&output).unwrap_or_default(),
        )])
    })
}

/// Change a user's password.
///
/// Tier: T2-P (∝ Irreversibility — credential rotation)
pub fn user_change_password(params: UserChangePasswordParams) -> Result<CallToolResult, McpError> {
    USER_MANAGER.with(|mgr| {
        let mut mgr = mgr.borrow_mut();

        match mgr.change_password(&params.username, &params.old_password, &params.new_password) {
            Ok(()) => {
                let output = serde_json::json!({
                    "status": "password_changed",
                    "username": params.username.to_lowercase(),
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&output).unwrap_or_default(),
                )]))
            }
            Err(e) => Err(McpError::invalid_params(format!("{e}"), None)),
        }
    })
}
