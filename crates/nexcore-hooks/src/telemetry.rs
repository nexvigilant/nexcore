//! Telemetry emission for hooks
//!
//! Provides consistent telemetry logging to the watchtower telemetry file
//! at `~/.claude/logs/hook_telemetry.jsonl`.

use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Get the path to the hook telemetry log file
pub fn telemetry_log_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".claude/logs/hook_telemetry.jsonl"))
        .unwrap_or_else(|| PathBuf::from("/tmp/hook_telemetry.jsonl"))
}

/// Telemetry entry for subagent events
#[derive(Debug, Serialize)]
pub struct SubagentTelemetry {
    pub timestamp: String,
    pub hook: String,
    pub event: String,
    pub agent_id: String,
    pub agent_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exceeded_timeout: Option<bool>,
}

impl SubagentTelemetry {
    /// Create a new telemetry entry for SubagentStart
    pub fn start(hook: &str, agent_id: &str, agent_type: &str, session_id: Option<&str>) -> Self {
        Self {
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            hook: hook.to_string(),
            event: "SubagentStart".to_string(),
            agent_id: agent_id.to_string(),
            agent_type: agent_type.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            duration_ms: None,
            description: None,
            exceeded_timeout: None,
        }
    }

    /// Create a new telemetry entry for SubagentStop
    pub fn stop(
        hook: &str,
        agent_id: &str,
        agent_type: &str,
        session_id: Option<&str>,
        duration_ms: u64,
        description: Option<&str>,
        exceeded_timeout: bool,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            hook: hook.to_string(),
            event: "SubagentStop".to_string(),
            agent_id: agent_id.to_string(),
            agent_type: agent_type.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            duration_ms: Some(duration_ms),
            description: description.map(|s| s.to_string()),
            exceeded_timeout: Some(exceeded_timeout),
        }
    }

    /// Emit this telemetry entry to the log file
    pub fn emit(&self) {
        emit_telemetry(self);
    }
}

/// Generic telemetry entry for any hook
#[derive(Debug, Serialize)]
pub struct HookTelemetry {
    pub timestamp: String,
    pub hook: String,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl HookTelemetry {
    /// Create a new generic telemetry entry
    pub fn new(hook: &str, event: &str) -> Self {
        Self {
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            hook: hook.to_string(),
            event: event.to_string(),
            tool: None,
            session: None,
            duration_ms: None,
            result: None,
            extra: serde_json::json!({}),
        }
    }

    /// Set the tool name
    pub fn with_tool(mut self, tool: &str) -> Self {
        self.tool = Some(tool.to_string());
        self
    }

    /// Set the session ID
    pub fn with_session(mut self, session: &str) -> Self {
        self.session = Some(session.to_string());
        self
    }

    /// Set the duration in milliseconds
    pub fn with_duration_ms(mut self, duration: u64) -> Self {
        self.duration_ms = Some(duration);
        self
    }

    /// Set the result message
    pub fn with_result(mut self, result: &str) -> Self {
        self.result = Some(result.to_string());
        self
    }

    /// Add extra fields
    pub fn with_extra(mut self, extra: serde_json::Value) -> Self {
        self.extra = extra;
        self
    }

    /// Emit this telemetry entry to the log file
    pub fn emit(&self) {
        emit_telemetry(self);
    }
}

/// Emit a telemetry entry to the log file (best-effort, non-blocking)
pub fn emit_telemetry<T: Serialize>(entry: &T) {
    let log_path = telemetry_log_path();

    // Ensure parent directory exists
    if let Some(parent) = log_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Warning: Could not create telemetry dir: {}", e);
            return;
        }
    }

    // Serialize entry
    let json = match serde_json::to_string(entry) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Warning: Could not serialize telemetry: {}", e);
            return;
        }
    };

    // Append to log file
    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut f| writeln!(f, "{}", json));

    if let Err(e) = result {
        eprintln!("Warning: Could not write telemetry: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_start_telemetry() {
        let entry = SubagentTelemetry::start(
            "subagent_timeout_tracker",
            "agent-123",
            "Explore",
            Some("session-456"),
        );
        assert_eq!(entry.event, "SubagentStart");
        assert_eq!(entry.agent_type, "Explore");
        assert!(entry.duration_ms.is_none());
    }

    #[test]
    fn test_subagent_stop_telemetry() {
        let entry = SubagentTelemetry::stop(
            "subagentstop_timeout_checker",
            "agent-123",
            "Explore",
            Some("session-456"),
            5000,
            Some("Completed successfully"),
            false,
        );
        assert_eq!(entry.event, "SubagentStop");
        assert_eq!(entry.duration_ms, Some(5000));
        assert_eq!(entry.exceeded_timeout, Some(false));
    }

    #[test]
    fn test_hook_telemetry_builder() {
        let entry = HookTelemetry::new("my_hook", "PreToolUse")
            .with_tool("Write")
            .with_session("session-789")
            .with_duration_ms(100)
            .with_result("allowed");

        assert_eq!(entry.hook, "my_hook");
        assert_eq!(entry.tool, Some("Write".to_string()));
        assert_eq!(entry.duration_ms, Some(100));
    }
}
