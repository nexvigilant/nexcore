#[tool(
    description = "Health check for NexCore MCP server (Probe). Returns version, tool count, and status."
)]
pub async fn nexcore_health_probe(&self) -> Result<CallToolResult, McpError> {
    let tool_count = self.tool_router.list_all().len();
    let health =
        serde_json::json!({
        "status": "healthy",
        "server": "nexcore-mcp",
        "version": env!("CARGO_PKG_VERSION"),
        "tool_count": tool_count,
        "probe": true
    });
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&health).unwrap_or_default()
                )
            ]
        )
    )
}
