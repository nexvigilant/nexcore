//! System & Management Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Hook registry, MCP server management, and system-level diagnostics.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ============================================================================
// Unified Dispatcher Parameters
// ============================================================================

/// Parameters for the unified `nexcore` dispatcher tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UnifiedParams {
    /// Command name (e.g. "foundation_levenshtein", "pv_signal_complete").
    pub command: String,
    /// Command-specific parameters as JSON object.
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Parameters for listing configured MCP servers
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpServersListParams {
    /// Include project-specific servers
    #[serde(default)]
    pub include_projects: bool,
}

/// Parameters for getting a specific MCP server configuration
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpServerGetParams {
    /// MCP server name
    pub name: String,
}

/// Parameters for adding a new MCP server
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpServerAddParams {
    /// MCP server name (identifier)
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

/// Parameters for removing an MCP server
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpServerRemoveParams {
    /// MCP server name to remove
    pub name: String,
}

// ============================================================================
// MCP Lock Parameters
// ============================================================================

/// Parameters for acquiring an agent lock on the MCP server
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpLockParams {
    /// Agent identifier requesting the lock
    pub agent_id: String,
    /// Resource or state path to lock
    pub path: String,
    /// Lock duration in seconds (default: 3600)
    #[serde(default = "default_mcp_lock_ttl")]
    pub ttl_seconds: u64,
}

fn default_mcp_lock_ttl() -> u64 {
    3600
}

/// Parameters for releasing an agent lock on the MCP server
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpUnlockParams {
    /// Agent identifier releasing the lock
    pub agent_id: String,
    /// Resource path to unlock
    pub path: String,
}

// ============================================================================
// Toolbox Parameters
// ============================================================================

/// Parameters for searching the tool catalog.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ToolboxParams {
    /// Keyword to search across category and tool names
    #[serde(default)]
    pub query: Option<String>,
    /// Category name to list all tools in that category
    #[serde(default)]
    pub category: Option<String>,
}
