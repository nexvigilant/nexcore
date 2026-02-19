//! Composite types for nexcore-mcp
//!
//! T2-C: Composed from T1 primitives, used for system operations.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Unified command for the NexCore dispatcher.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UnifiedCommand {
    /// Command name
    pub command: String,
    /// Command parameters
    pub params: serde_json::Value,
}

/// Status of an MCP server connection.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpServerStatus {
    /// Server name
    pub name: String,
    /// Whether the server is connected
    pub connected: bool,
    /// Tool count
    pub tools: usize,
}

/// Lock held by an agent on a resource.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AgentLock {
    /// Agent identifier
    pub agent_id: String,
    /// Resource path
    pub path: String,
    /// Expiry timestamp (seconds)
    pub expires_at: u64,
}
