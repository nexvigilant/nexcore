//! NexChat Tools — AI chat management and tool discovery
//!
//! # T1 Grounding
//! - ς (state): Chat session configuration
//! - μ (mapping): Tool discovery and listing
//! - ∂ (boundary): API key validation

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

use crate::NexCoreMcpServer;
use crate::params::nexchat::{NexchatConfigParams, NexchatStatusParams, NexchatToolsParams};

/// Check NexChat readiness — API key configured, model available.
pub fn nexchat_status(_params: NexchatStatusParams) -> Result<CallToolResult, McpError> {
    let api_key_set = std::env::var("ANTHROPIC_API_KEY")
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    let status = json!({
        "nexchat": {
            "status": if api_key_set { "ready" } else { "api_key_missing" },
            "api_key_configured": api_key_set,
            "default_model": "claude-sonnet-4-6",
            "default_max_tokens": 4096,
            "features": {
                "streaming": true,
                "tool_use": true,
                "conversation_memory": true,
                "auto_tool_discovery": true,
            }
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&status).unwrap_or_default(),
    )]))
}

/// Get or display NexChat configuration.
pub fn nexchat_config(params: NexchatConfigParams) -> Result<CallToolResult, McpError> {
    let model = params
        .model
        .unwrap_or_else(|| "claude-sonnet-4-6".to_string());
    let max_tokens = params.max_tokens.unwrap_or(4096);

    let config = json!({
        "nexchat_config": {
            "model": model,
            "max_tokens": max_tokens,
            "api_base": "https://api.anthropic.com/v1/messages",
            "api_version": "2023-06-01",
            "streaming": true,
            "tool_scope": "all",
            "max_tool_rounds": 5,
            "context_window": 200_000,
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&config).unwrap_or_default(),
    )]))
}

/// List MCP tools available to NexChat for Claude tool_use.
pub fn nexchat_tools(
    params: NexchatToolsParams,
    server: &NexCoreMcpServer,
) -> Result<CallToolResult, McpError> {
    let all_tools = server.tool_router.list_all();
    let limit = params.limit.unwrap_or(50) as usize;

    let filtered: Vec<serde_json::Value> = all_tools
        .into_iter()
        .filter(|t| {
            if let Some(ref prefix) = params.filter {
                t.name.as_ref().starts_with(prefix.as_str())
            } else {
                true
            }
        })
        .take(limit)
        .map(|t| {
            json!({
                "name": t.name.as_ref(),
                "description": t.description.as_deref().unwrap_or(""),
            })
        })
        .collect();

    let result = json!({
        "nexchat_tools": {
            "count": filtered.len(),
            "filter": params.filter,
            "tools": filtered,
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
