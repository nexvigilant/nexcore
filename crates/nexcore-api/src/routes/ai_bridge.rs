//! AI-MCP Bridge — zero-wiring tool dispatch for NexChat.
//!
//! Auto-discovers MCP tools from `NexCoreMcpServer::tool_router` and
//! converts them to Anthropic API tool format. Adding a tool to
//! nexcore-mcp automatically makes it available to Claude — no extra
//! wiring required.
//!
//! ## Primitive Grounding
//!
//! `μ(Mapping) + →(Causality) + ∂(Boundary) + ν(Frequency)`

use nexcore_mcp::NexCoreMcpServer;
use nexcore_terminal::ai::AiTool;

use crate::mcp_bridge;

// ─────────────────────────────────────────────────────────────────────────────
// Tool Scope
// ─────────────────────────────────────────────────────────────────────────────

/// Controls which tools are exposed to Claude.
///
/// GroundsTo: T1 — ∂(Boundary)
#[derive(Debug, Clone, Copy, Default)]
pub enum ToolScope {
    /// All MCP tools from the tool_router.
    #[default]
    All,
    /// No tools — text-only mode.
    None,
}

// ─────────────────────────────────────────────────────────────────────────────
// AiMcpBridge
// ─────────────────────────────────────────────────────────────────────────────

/// Bridge between Claude tool_use and MCP in-process dispatch.
///
/// Zero-wiring: auto-discovers tools from `NexCoreMcpServer::tool_router`.
///
/// GroundsTo: T2-C — μ(Mapping) 0.35 + →(Causality) 0.30 + ∂(Boundary) 0.20 + ν(Frequency) 0.15
pub struct AiMcpBridge<'a> {
    server: &'a NexCoreMcpServer,
    scope: ToolScope,
}

impl<'a> AiMcpBridge<'a> {
    /// Create a bridge bound to a server instance.
    pub fn new(server: &'a NexCoreMcpServer, scope: ToolScope) -> Self {
        Self { server, scope }
    }

    /// List all available tools in Anthropic API format.
    ///
    /// Auto-discovers from `tool_router.list_all()` — adding a tool to
    /// nexcore-mcp automatically makes it available here.
    #[must_use]
    pub fn available_tools(&self) -> Vec<AiTool> {
        match self.scope {
            ToolScope::None => Vec::new(),
            ToolScope::All => {
                let all = self.server.tool_router.list_all();
                all.into_iter()
                    .map(|t| {
                        let schema = serde_json::to_value(&t.input_schema)
                            .unwrap_or_else(|_| serde_json::json!({"type": "object"}));
                        AiTool::new(
                            t.name.as_ref(),
                            t.description.as_deref().unwrap_or(""),
                            schema,
                        )
                    })
                    .collect()
            }
        }
    }

    /// Count available tools.
    #[must_use]
    pub fn tool_count(&self) -> usize {
        match self.scope {
            ToolScope::None => 0,
            ToolScope::All => self.server.tool_router.list_all().len(),
        }
    }

    /// Execute a tool call from Claude's tool_use block.
    ///
    /// Dispatches to `mcp_bridge::call_tool` for in-process execution.
    ///
    /// # Errors
    ///
    /// Returns `NexChatError::ToolDispatchFailed` if MCP dispatch fails.
    pub async fn execute_tool_call(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<String, super::ai_client::NexChatError> {
        let result = mcp_bridge::call_tool(tool_name, input, self.server)
            .await
            .map_err(|e| super::ai_client::NexChatError::ToolDispatchFailed {
                tool_name: tool_name.to_string(),
                message: e.to_string(),
            })?;

        // Extract the content string from the result
        // MCP returns JSON with a "content" array; flatten to text
        if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
            let texts: Vec<&str> = content
                .iter()
                .filter_map(|block| block.get("text").and_then(|t| t.as_str()))
                .collect();
            if texts.is_empty() {
                Ok(result.to_string())
            } else {
                Ok(texts.join("\n"))
            }
        } else {
            Ok(result.to_string())
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_scope_none_returns_empty() {
        let server = NexCoreMcpServer::new();
        let bridge = AiMcpBridge::new(&server, ToolScope::None);
        assert!(bridge.available_tools().is_empty());
        assert_eq!(bridge.tool_count(), 0);
    }

    #[test]
    fn tool_scope_all_returns_tools() {
        let server = NexCoreMcpServer::new();
        let bridge = AiMcpBridge::new(&server, ToolScope::All);
        let tools = bridge.available_tools();
        // Server registers at least the unified dispatcher
        assert!(!tools.is_empty(), "Should have at least 1 tool");
        assert_eq!(tools.len(), bridge.tool_count());
    }

    #[test]
    fn tools_have_valid_schema() {
        let server = NexCoreMcpServer::new();
        let bridge = AiMcpBridge::new(&server, ToolScope::All);
        let tools = bridge.available_tools();
        for tool in &tools {
            assert!(!tool.name.is_empty(), "Tool name must not be empty");
            // Schema must be a JSON object
            assert!(
                tool.input_schema.is_object(),
                "Tool {} schema must be object, got: {}",
                tool.name,
                tool.input_schema
            );
        }
    }

    #[test]
    fn tool_scope_default_is_all() {
        let scope = ToolScope::default();
        assert!(matches!(scope, ToolScope::All));
    }

    #[tokio::test]
    async fn execute_tool_call_dispatches() {
        let server = NexCoreMcpServer::new();
        let bridge = AiMcpBridge::new(&server, ToolScope::All);

        // Use "nexcore_health" — a valid unified dispatch command
        let result = bridge
            .execute_tool_call("nexcore_health", serde_json::json!({}))
            .await;

        assert!(
            result.is_ok(),
            "nexcore_health should succeed: {:?}",
            result.err()
        );
        let text = result.unwrap_or_default();
        assert!(
            text.contains("nexcore") || text.contains("status") || text.contains("tool"),
            "Should contain health info: {text}"
        );
    }

    #[tokio::test]
    async fn execute_tool_call_unknown_returns_error() {
        let server = NexCoreMcpServer::new();
        let bridge = AiMcpBridge::new(&server, ToolScope::All);

        let result = bridge
            .execute_tool_call("nonexistent_tool_xyz", serde_json::json!({}))
            .await;

        assert!(result.is_err(), "Unknown tool should fail");
    }
}
