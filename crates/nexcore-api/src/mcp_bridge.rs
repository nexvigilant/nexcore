//! MCP Bridge — dispatch MCP tool calls either in-process or via Station HTTP.
//!
//! With `mcp-bridge` feature (default, local dev): in-process dispatch via `nexcore_mcp`.
//! Without (Cloud Run): forwards to NexVigilant Station at `mcp.nexvigilant.com/rpc`.

use std::time::Instant;

// ── In-process bridge (local dev, full MCP) ─────────────────────────────────

#[cfg(feature = "mcp-bridge")]
use nexcore_mcp::NexCoreMcpServer;

#[cfg(feature = "mcp-bridge")]
pub async fn call_tool(
    tool_name: &str,
    params: serde_json::Value,
    server: &NexCoreMcpServer,
) -> Result<serde_json::Value, nexcore_error::NexError> {
    use nexcore_error::Context;

    let start = Instant::now();

    let result = match nexcore_mcp::call_tool_direct(tool_name, params.clone(), server).await {
        Ok(r) => r,
        Err(_) => nexcore_mcp::unified::dispatch(tool_name, params, server)
            .await
            .map_err(|e| nexcore_error::nexerror!("MCP dispatch error: {e:?}"))?,
    };

    let value = serde_json::to_value(&result).context("failed to serialize CallToolResult")?;

    tracing::debug!(
        tool = tool_name,
        elapsed_ms = start.elapsed().as_millis() as u64,
        "mcp bridge call (in-process)"
    );

    Ok(value)
}

// ── Station HTTP bridge (Cloud Run, no nexcore-mcp) ─────────────────────────

/// Stub server type when mcp-bridge feature is disabled.
/// Terminal and project_tools use `NexCoreMcpServer` — this provides the type.
#[cfg(not(feature = "mcp-bridge"))]
pub struct NexCoreMcpServer;

#[cfg(not(feature = "mcp-bridge"))]
impl NexCoreMcpServer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "mcp-bridge"))]
pub async fn call_tool(
    tool_name: &str,
    params: serde_json::Value,
    _server: &NexCoreMcpServer,
) -> Result<serde_json::Value, nexcore_error::NexError> {
    let start = Instant::now();
    let station_url = std::env::var("NEXVIGILANT_STATION_URL")
        .unwrap_or_else(|_| "https://mcp.nexvigilant.com".to_string());

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "id": 1,
        "params": {
            "name": tool_name,
            "arguments": params,
        }
    });

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{station_url}/rpc"))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| nexcore_error::nexerror!("Station HTTP error: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(nexcore_error::nexerror!("Station HTTP {status}: {text}"));
    }

    let rpc: serde_json::Value = response
        .json()
        .await
        .map_err(|e| nexcore_error::nexerror!("Station parse error: {e}"))?;

    let result = rpc
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    tracing::debug!(
        tool = tool_name,
        elapsed_ms = start.elapsed().as_millis() as u64,
        "mcp bridge call (station HTTP)"
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    // Bridge is now a thin wrapper — integration testing covers it via route-level tests.
}
