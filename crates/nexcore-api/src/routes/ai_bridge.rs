//! AI-MCP Bridge — tool dispatch for NexChat.
//!
//! With `mcp-bridge` feature: auto-discovers from `NexCoreMcpServer::tool_router`.
//! Without: discovers tools via Station HTTP `/tools` endpoint and dispatches via `/rpc`.

use crate::mcp_bridge;
use nexcore_terminal::ai::AiTool;

#[cfg(not(feature = "mcp-bridge"))]
use crate::mcp_bridge::NexCoreMcpServer;
#[cfg(feature = "mcp-bridge")]
use nexcore_mcp::NexCoreMcpServer;

// ─────────────────────────────────────────────────────────────────────────────
// Tool Scope
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default)]
pub enum ToolScope {
    #[default]
    All,
    None,
}

// ─────────────────────────────────────────────────────────────────────────────
// AiMcpBridge
// ─────────────────────────────────────────────────────────────────────────────

pub struct AiMcpBridge<'a> {
    server: &'a NexCoreMcpServer,
    scope: ToolScope,
}

impl<'a> AiMcpBridge<'a> {
    pub fn new(server: &'a NexCoreMcpServer, scope: ToolScope) -> Self {
        Self { server, scope }
    }

    // ── In-process tool discovery (mcp-bridge feature) ──────────────────

    #[cfg(feature = "mcp-bridge")]
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

    // ── Station HTTP tool discovery (no mcp-bridge) ─────────────────────

    #[cfg(not(feature = "mcp-bridge"))]
    #[must_use]
    pub fn available_tools(&self) -> Vec<AiTool> {
        match self.scope {
            ToolScope::None => Vec::new(),
            ToolScope::All => {
                // Return a curated set of key PV tools for AI mode.
                // Full discovery via Station HTTP /tools is async — provide
                // core tools synchronously for the tool_use loop.
                vec![
                    AiTool::new(
                        "search_adverse_events",
                        "Search FDA FAERS adverse events for a drug",
                        serde_json::json!({"type":"object","properties":{"drug":{"type":"string"}},"required":["drug"]}),
                    ),
                    AiTool::new(
                        "compute_prr",
                        "Compute PRR disproportionality signal",
                        serde_json::json!({"type":"object","properties":{"a":{"type":"integer"},"b":{"type":"integer"},"c":{"type":"integer"},"d":{"type":"integer"}},"required":["a","b","c","d"]}),
                    ),
                    AiTool::new(
                        "compute_ror",
                        "Compute ROR disproportionality signal",
                        serde_json::json!({"type":"object","properties":{"a":{"type":"integer"},"b":{"type":"integer"},"c":{"type":"integer"},"d":{"type":"integer"}},"required":["a","b","c","d"]}),
                    ),
                    AiTool::new(
                        "assess_naranjo_causality",
                        "Naranjo causality assessment",
                        serde_json::json!({"type":"object","properties":{"drug":{"type":"string"},"event":{"type":"string"}},"required":["drug","event"]}),
                    ),
                    AiTool::new(
                        "search_drugs",
                        "Search DailyMed drug labels",
                        serde_json::json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
                    ),
                    AiTool::new(
                        "get_boxed_warning",
                        "Get FDA boxed warning for a drug",
                        serde_json::json!({"type":"object","properties":{"drug":{"type":"string"}},"required":["drug"]}),
                    ),
                    AiTool::new(
                        "search_articles",
                        "Search PubMed articles",
                        serde_json::json!({"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}),
                    ),
                    AiTool::new(
                        "get_interactions",
                        "Get drug-drug interactions",
                        serde_json::json!({"type":"object","properties":{"drug":{"type":"string"}},"required":["drug"]}),
                    ),
                ]
            }
        }
    }

    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.available_tools().len()
    }

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
        assert!(!tools.is_empty(), "Should have at least 1 tool");
        assert_eq!(tools.len(), bridge.tool_count());
    }

    #[test]
    fn tool_scope_default_is_all() {
        let scope = ToolScope::default();
        assert!(matches!(scope, ToolScope::All));
    }
}
