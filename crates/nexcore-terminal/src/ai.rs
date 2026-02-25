//! AI backend trait — pluggable interface for AI-powered terminal mode.
//!
//! `AiBackend` is the abstraction that decouples the terminal from any
//! specific AI provider. Claude is the primary implementation, but the
//! trait allows swapping to local models (Cortex) or disabling AI entirely.
//!
//! Note: Implementations live outside this crate to avoid pulling in
//! HTTP client dependencies. `nexcore-api` provides the Claude implementation.

use serde::{Deserialize, Serialize};

/// Role in an AI conversation.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiRole {
    /// System instruction.
    System,
    /// User message.
    User,
    /// AI assistant response.
    Assistant,
    /// Tool execution result.
    ToolResult,
}

/// A message in an AI conversation.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    /// Who sent this message.
    pub role: AiRole,
    /// Message content.
    pub content: String,
}

/// A tool that the AI can invoke.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTool {
    /// Tool name (matches MCP tool name).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for the tool's input parameters.
    pub input_schema: serde_json::Value,
}

impl AiTool {
    /// Create a new tool definition.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// A tool call requested by the AI.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolCall {
    /// Unique call identifier.
    pub id: String,
    /// Tool name to invoke.
    pub name: String,
    /// Tool input parameters.
    pub input: serde_json::Value,
}

impl AiToolCall {
    /// Create a new tool call.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, input: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input,
        }
    }
}

/// Response from an AI backend after processing a message.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    /// Text content of the response.
    pub content: String,
    /// Tool calls requested by the AI (may be empty).
    pub tool_calls: Vec<AiToolCall>,
    /// Input tokens consumed.
    pub input_tokens: u64,
    /// Output tokens produced.
    pub output_tokens: u64,
}

impl AiResponse {
    /// Create a new AI response.
    #[must_use]
    pub fn new(
        content: impl Into<String>,
        tool_calls: Vec<AiToolCall>,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Self {
        Self {
            content: content.into(),
            tool_calls,
            input_tokens,
            output_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_message_serde() {
        let msg = AiMessage {
            role: AiRole::User,
            content: "What is PRR?".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap_or_default();
        assert!(json.contains("\"role\":\"user\""));
    }

    #[test]
    fn ai_tool_call_serde() {
        let call = AiToolCall {
            id: "call_1".to_string(),
            name: "pv_signal_complete".to_string(),
            input: serde_json::json!({"drug": "aspirin", "event": "hepatotoxicity"}),
        };
        let json = serde_json::to_string(&call).unwrap_or_default();
        assert!(json.contains("pv_signal_complete"));
    }

    #[test]
    fn ai_response_with_tool_calls() {
        let resp = AiResponse {
            content: String::new(),
            tool_calls: vec![AiToolCall {
                id: "call_1".to_string(),
                name: "faers_search".to_string(),
                input: serde_json::json!({"drug": "aspirin"}),
            }],
            input_tokens: 150,
            output_tokens: 0,
        };
        assert_eq!(resp.tool_calls.len(), 1);
    }
}
