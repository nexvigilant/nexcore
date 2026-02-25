//! NexChat Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Parameters for AI chat status, configuration, and tool discovery.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for nexchat_status — no input needed.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NexchatStatusParams {}

/// Parameters for nexchat_config — optional model override.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NexchatConfigParams {
    /// Model identifier (e.g., "claude-sonnet-4-6").
    #[serde(default)]
    pub model: Option<String>,
    /// Max output tokens per response.
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

/// Parameters for nexchat_tools — optional filter.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NexchatToolsParams {
    /// Filter tools by name prefix (e.g., "pv_" for PV tools).
    #[serde(default)]
    pub filter: Option<String>,
    /// Maximum number of tools to return (default: 50).
    #[serde(default)]
    pub limit: Option<u32>,
}
