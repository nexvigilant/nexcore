//! Domain-specific identifier types.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{HookError, HookResult};

/// Valid Claude Code tool name.
///
/// # Invariant
/// Matches one of:
/// - Standard tool: `^[A-Z][a-zA-Z]+` (e.g., "Bash", "Write", "Edit")
/// - MCP tool: `^mcp__[a-z][a-z0-9_]*__[a-z][a-z0-9_]*`
///
/// # Examples
/// - Valid: "Bash", "WebFetch", "mcp__memory__create"
/// - Invalid: "bash", "BASH", "mcp_memory_create", ""
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ToolName(String);

impl ToolName {
    /// Validate and create tool name.
    pub fn new(s: impl Into<String>) -> HookResult<Self> {
        let s = s.into();
        if Self::is_valid(&s) {
            Ok(Self(s))
        } else {
            Err(HookError::ValidationFailed(format!(
                "invalid tool name: {}",
                s
            )))
        }
    }

    /// Check if string is valid tool name.
    #[must_use]
    pub fn is_valid(s: &str) -> bool {
        Self::is_standard_tool(s) || Self::is_mcp_tool(s)
    }

    /// Check if standard tool pattern (PascalCase).
    /// Must start with uppercase, contain at least one lowercase, all alphabetic.
    fn is_standard_tool(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        let mut chars = s.chars();
        // First char must be uppercase A-Z
        match chars.next() {
            Some(c) if c.is_ascii_uppercase() => {}
            _ => return false,
        }
        // Rest must be ascii alphabetic with at least one lowercase (PascalCase)
        let rest: String = chars.collect();
        !rest.is_empty()
            && rest.chars().all(|c| c.is_ascii_alphabetic())
            && rest.chars().any(|c| c.is_ascii_lowercase())
    }

    /// Check if MCP tool pattern.
    fn is_mcp_tool(s: &str) -> bool {
        let parts: Vec<&str> = s.split("__").collect();

        // Must be exactly 3 parts: "mcp", server, tool
        let [prefix, server, tool] = parts.as_slice() else {
            return false;
        };

        if *prefix != "mcp" {
            return false;
        }

        // Server and tool names: lowercase alphanumeric + underscore, starting with lowercase
        let valid_part = |p: &str| -> bool {
            !p.is_empty()
                && p.chars().next().is_some_and(|c| c.is_ascii_lowercase())
                && p.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        };

        valid_part(server) && valid_part(tool)
    }

    /// Check if this is an MCP tool.
    #[inline]
    #[must_use]
    pub fn is_mcp(&self) -> bool {
        self.0.starts_with("mcp__")
    }

    /// For MCP tools, extract server name.
    #[must_use]
    pub fn mcp_server(&self) -> Option<&str> {
        if self.is_mcp() {
            self.0.split("__").nth(1)
        } else {
            None
        }
    }

    /// For MCP tools, extract tool name.
    #[must_use]
    pub fn mcp_tool(&self) -> Option<&str> {
        if self.is_mcp() {
            self.0.split("__").nth(2)
        } else {
            None
        }
    }

    /// Access inner string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for ToolName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ToolName::new(s).map_err(serde::de::Error::custom)
    }
}

/// Valid hook event name.
///
/// # Invariant
/// Is one of the 13 defined Claude Code hook events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEventName {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionRequest,
    UserPromptSubmit,
    Stop,
    SubagentStop,
    SubagentStart,
    SessionStart,
    SessionEnd,
    Setup,
    Notification,
    PreCompact,
}

impl HookEventName {
    /// Parse from string.
    pub fn parse(s: &str) -> HookResult<Self> {
        match s {
            "PreToolUse" => Ok(Self::PreToolUse),
            "PostToolUse" => Ok(Self::PostToolUse),
            "PostToolUseFailure" => Ok(Self::PostToolUseFailure),
            "PermissionRequest" => Ok(Self::PermissionRequest),
            "UserPromptSubmit" => Ok(Self::UserPromptSubmit),
            "Stop" => Ok(Self::Stop),
            "SubagentStop" => Ok(Self::SubagentStop),
            "SubagentStart" => Ok(Self::SubagentStart),
            "SessionStart" => Ok(Self::SessionStart),
            "SessionEnd" => Ok(Self::SessionEnd),
            "Setup" => Ok(Self::Setup),
            "Notification" => Ok(Self::Notification),
            "PreCompact" => Ok(Self::PreCompact),
            _ => Err(HookError::UnknownEvent(s.into())),
        }
    }

    /// Get canonical string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PreToolUse => "PreToolUse",
            Self::PostToolUse => "PostToolUse",
            Self::PostToolUseFailure => "PostToolUseFailure",
            Self::PermissionRequest => "PermissionRequest",
            Self::UserPromptSubmit => "UserPromptSubmit",
            Self::Stop => "Stop",
            Self::SubagentStop => "SubagentStop",
            Self::SubagentStart => "SubagentStart",
            Self::SessionStart => "SessionStart",
            Self::SessionEnd => "SessionEnd",
            Self::Setup => "Setup",
            Self::Notification => "Notification",
            Self::PreCompact => "PreCompact",
        }
    }

    /// All event names.
    pub const ALL: &'static [HookEventName] = &[
        Self::PreToolUse,
        Self::PostToolUse,
        Self::PostToolUseFailure,
        Self::PermissionRequest,
        Self::UserPromptSubmit,
        Self::Stop,
        Self::SubagentStop,
        Self::SubagentStart,
        Self::SessionStart,
        Self::SessionEnd,
        Self::Setup,
        Self::Notification,
        Self::PreCompact,
    ];
}

impl fmt::Display for HookEventName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Session identifier.
///
/// # Invariant
/// Non-empty string, typically UUID format.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    /// Create new session ID.
    pub fn new(s: impl Into<String>) -> HookResult<Self> {
        let s = s.into();
        if s.is_empty() {
            Err(HookError::ValidationFailed(
                "session_id cannot be empty".into(),
            ))
        } else {
            Ok(Self(s))
        }
    }

    /// Access inner string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for SessionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SessionId::new(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn tool_name_standard_valid() {
        assert!(ToolName::new("Bash").is_ok());
        assert!(ToolName::new("WebFetch").is_ok());
        assert!(ToolName::new("Read").is_ok());
        assert!(ToolName::new("NotebookEdit").is_ok());
    }

    #[test]
    fn tool_name_standard_invalid() {
        assert!(ToolName::new("bash").is_err()); // lowercase start
        assert!(ToolName::new("BASH").is_err()); // all caps after first
        assert!(ToolName::new("Web_Fetch").is_err()); // underscore
        assert!(ToolName::new("").is_err()); // empty
        assert!(ToolName::new("Web123").is_err()); // digits
    }

    #[test]
    fn tool_name_mcp_valid() {
        assert!(ToolName::new("mcp__memory__create").is_ok());
        assert!(ToolName::new("mcp__github__search_repos").is_ok());
        assert!(ToolName::new("mcp__nexcore__foundation_sha256").is_ok());
    }

    #[test]
    fn tool_name_mcp_invalid() {
        assert!(ToolName::new("mcp_memory_create").is_err()); // single underscore
        assert!(ToolName::new("mcp__Memory__create").is_err()); // uppercase
        assert!(ToolName::new("MCP__memory__create").is_err()); // MCP uppercase
        assert!(ToolName::new("mcp__memory").is_err()); // missing third part
    }

    #[test]
    fn tool_name_mcp_extraction() {
        let tool = ToolName::new("mcp__nexcore__foundation_sha256").unwrap();
        assert!(tool.is_mcp());
        assert_eq!(tool.mcp_server(), Some("nexcore"));
        assert_eq!(tool.mcp_tool(), Some("foundation_sha256"));
    }

    #[test]
    fn tool_name_standard_no_mcp_parts() {
        let tool = ToolName::new("Bash").unwrap();
        assert!(!tool.is_mcp());
        assert_eq!(tool.mcp_server(), None);
        assert_eq!(tool.mcp_tool(), None);
    }

    #[test]
    fn hook_event_parse_all() {
        for event in HookEventName::ALL {
            let parsed = HookEventName::parse(event.as_str()).unwrap();
            assert_eq!(&parsed, event);
        }
    }

    #[test]
    fn hook_event_parse_invalid() {
        assert!(HookEventName::parse("InvalidEvent").is_err());
        assert!(HookEventName::parse("").is_err());
    }

    #[test]
    fn hook_event_count() {
        assert_eq!(HookEventName::ALL.len(), 13);
    }

    #[test]
    fn session_id_rejects_empty() {
        assert!(SessionId::new("").is_err());
    }

    #[test]
    fn session_id_accepts_uuid() {
        let id = SessionId::new("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(id.as_str(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn session_id_serde_roundtrip() {
        let original = SessionId::new("test-session-123").unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: SessionId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn session_id_serde_rejects_empty() {
        let json = r#""""#;
        let result: Result<SessionId, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "should reject empty session_id on deserialization"
        );
    }
}
