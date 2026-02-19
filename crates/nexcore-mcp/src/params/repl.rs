//! Claude REPL Parameters (Agentic Integration)
//!
//! Interactive command execution and session management.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for Claude REPL interaction.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeReplParams {
    /// The prompt to send.
    pub prompt: String,
    /// Optional model (e.g., "sonnet").
    pub model: Option<String>,
    /// Optional session ID.
    pub session_id: Option<String>,
    /// Optional settings path.
    pub settings_path: Option<String>,
    /// Optional MCP config path.
    pub mcp_config_path: Option<String>,
    /// Validate MCP config boolean.
    pub strict_mcp_config: Option<bool>,
    /// Permission mode.
    pub permission_mode: Option<String>,
    /// Allowed tools list.
    pub allowed_tools: Option<Vec<String>>,
    /// Output format.
    pub output_format: Option<String>,
    /// System prompt override.
    pub system_prompt: Option<String>,
    /// System prompt append.
    pub append_system_prompt: Option<String>,
    /// Persist session boolean.
    pub persist_session: Option<bool>,
    /// Timeout in ms.
    pub timeout_ms: Option<u64>,
    /// Maximum output bytes.
    pub max_output_bytes: Option<usize>,
}
