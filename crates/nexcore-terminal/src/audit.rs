//! Terminal audit logging — compliance-grade event recording.
//!
//! Every terminal interaction produces an audit entry. Entries are
//! append-only and include tenant+user+session context for traceability.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use vr_core::ids::{TenantId, TerminalSessionId, UserId};

/// Audit event types for terminal interactions.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    /// New terminal session created.
    SessionCreated,
    /// Terminal session ended.
    SessionTerminated,
    /// Shell command executed.
    ShellCommand,
    /// MCP tool invoked.
    McpToolCall,
    /// AI query submitted.
    AiQuery,
    /// Terminal mode switched.
    ModeSwitch,
    /// Permission check failed.
    PermissionDenied,
}

/// A single audit log entry for a terminal event.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalAuditEntry {
    /// Unique entry identifier.
    pub id: NexId,
    /// Session that produced this event.
    pub session_id: TerminalSessionId,
    /// Owning tenant.
    pub tenant_id: TenantId,
    /// Acting user.
    pub user_id: UserId,
    /// When the event occurred.
    pub timestamp: DateTime,
    /// What happened.
    pub action: AuditAction,
    /// Additional context (command text, tool name, error details).
    pub details: serde_json::Value,
}

impl TerminalAuditEntry {
    /// Create a new audit entry with the current timestamp.
    #[must_use]
    pub fn new(
        session_id: TerminalSessionId,
        tenant_id: TenantId,
        user_id: UserId,
        action: AuditAction,
        details: serde_json::Value,
    ) -> Self {
        Self {
            id: NexId::v4(),
            session_id,
            tenant_id,
            user_id,
            timestamp: DateTime::now(),
            action,
            details,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_entry_creation() {
        let entry = TerminalAuditEntry::new(
            TerminalSessionId::new(),
            TenantId::new(),
            UserId::new(),
            AuditAction::SessionCreated,
            serde_json::json!({"mode": "hybrid"}),
        );
        assert!(matches!(entry.action, AuditAction::SessionCreated));
    }

    #[test]
    fn audit_action_serde() {
        let json = serde_json::to_string(&AuditAction::McpToolCall).unwrap_or_default();
        assert_eq!(json, "\"mcp_tool_call\"");
    }
}
