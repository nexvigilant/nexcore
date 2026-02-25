//! Terminal session lifecycle — creation, tracking, and cleanup.
//!
//! Every terminal connection creates a `TerminalSession` scoped to a
//! tenant+user pair. The `SessionRegistry` tracks active sessions and
//! enforces per-tenant concurrency limits.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use vr_core::ids::{TenantId, TerminalSessionId, UserId};

/// Terminal operating mode — determines how input is routed.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalMode {
    /// Raw shell commands routed to PTY.
    Shell,
    /// MCP tool dispatch for regulatory queries.
    Regulatory,
    /// Natural language routed to AI backend.
    Ai,
    /// Auto-routes based on input analysis.
    Hybrid,
}

/// Terminal session lifecycle status.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Sandbox is being provisioned.
    Creating,
    /// PTY connected and accepting input.
    Active,
    /// No input received for longer than idle timeout.
    Idle,
    /// Process stopped (SIGSTOP), can be resumed.
    Suspended,
    /// Session ended, resources released.
    Terminated,
}

/// Per-session metadata tracked alongside the session.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Terminal width in columns.
    pub cols: u16,
    /// Terminal height in rows.
    pub rows: u16,
    /// Shell binary path (e.g., "/bin/bash").
    pub shell: String,
    /// Current working directory.
    pub working_dir: String,
    /// AI conversation identifier (if AI mode has been used).
    pub ai_conversation_id: Option<String>,
    /// Cumulative AI tokens consumed in this session.
    pub ai_tokens_used: u64,
    /// Cumulative MCP tool calls made in this session.
    pub mcp_calls_made: u64,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            shell: "/bin/bash".to_string(),
            working_dir: "/workspace".to_string(),
            ai_conversation_id: None,
            ai_tokens_used: 0,
            mcp_calls_made: 0,
        }
    }
}

/// A terminal session owned by a specific tenant and user.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    /// Unique session identifier.
    pub id: TerminalSessionId,
    /// Owning tenant.
    pub tenant_id: TenantId,
    /// Owning user within the tenant.
    pub user_id: UserId,
    /// Current operating mode.
    pub mode: TerminalMode,
    /// Lifecycle status.
    pub status: SessionStatus,
    /// When the session was created.
    pub created_at: DateTime,
    /// Last input or output activity.
    pub last_activity: DateTime,
    /// Session metadata (terminal size, usage counters).
    pub metadata: SessionMetadata,
}

impl TerminalSession {
    /// Create a new terminal session in `Creating` status.
    #[must_use]
    pub fn new(tenant_id: TenantId, user_id: UserId, mode: TerminalMode) -> Self {
        let now = DateTime::now();
        Self {
            id: TerminalSessionId::new(),
            tenant_id,
            user_id,
            mode,
            status: SessionStatus::Creating,
            created_at: now,
            last_activity: now,
            metadata: SessionMetadata::default(),
        }
    }

    /// Mark the session as active (PTY connected).
    pub fn activate(&mut self) {
        self.status = SessionStatus::Active;
        self.last_activity = DateTime::now();
    }

    /// Mark the session as idle.
    pub fn mark_idle(&mut self) {
        self.status = SessionStatus::Idle;
    }

    /// Mark the session as terminated.
    pub fn terminate(&mut self) {
        self.status = SessionStatus::Terminated;
    }

    /// Record activity (updates last_activity timestamp).
    pub fn touch(&mut self) {
        self.last_activity = DateTime::now();
        if self.status == SessionStatus::Idle {
            self.status = SessionStatus::Active;
        }
    }

    /// Whether the session is in a state that accepts input.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        matches!(
            self.status,
            SessionStatus::Active | SessionStatus::Idle | SessionStatus::Creating
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_starts_creating() {
        let session = TerminalSession::new(TenantId::new(), UserId::new(), TerminalMode::Hybrid);
        assert_eq!(session.status, SessionStatus::Creating);
        assert!(session.is_alive());
    }

    #[test]
    fn activate_sets_active_status() {
        let mut session = TerminalSession::new(TenantId::new(), UserId::new(), TerminalMode::Shell);
        session.activate();
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn touch_reactivates_idle_session() {
        let mut session = TerminalSession::new(TenantId::new(), UserId::new(), TerminalMode::Shell);
        session.activate();
        session.mark_idle();
        assert_eq!(session.status, SessionStatus::Idle);
        session.touch();
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn terminated_session_is_not_alive() {
        let mut session = TerminalSession::new(TenantId::new(), UserId::new(), TerminalMode::Shell);
        session.terminate();
        assert!(!session.is_alive());
    }

    #[test]
    fn default_metadata_dimensions() {
        let meta = SessionMetadata::default();
        assert_eq!(meta.cols, 80);
        assert_eq!(meta.rows, 24);
        assert_eq!(meta.shell, "/bin/bash");
    }

    #[test]
    fn terminal_mode_serde_roundtrip() {
        let mode = TerminalMode::Regulatory;
        let json = serde_json::to_string(&mode).unwrap_or_default();
        assert_eq!(json, "\"regulatory\"");
    }

    #[test]
    fn session_status_serde_roundtrip() {
        let status = SessionStatus::Suspended;
        let json = serde_json::to_string(&status).unwrap_or_default();
        assert_eq!(json, "\"suspended\"");
    }
}
