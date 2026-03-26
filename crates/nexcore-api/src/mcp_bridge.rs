//! MCP Bridge — In-process dispatch to nexcore-mcp tools
//!
//! Tries the first-class tool router (540+ tools) via `nexcore_mcp::call_tool_direct`,
//! then falls back to the unified dispatcher for legacy short-name commands.

use std::time::Instant;

use nexcore_error::Context;
use nexcore_mcp::NexCoreMcpServer;

/// Call a single MCP tool via in-process dispatch.
///
/// Tries `nexcore_mcp::call_tool_direct` first (all `#[tool]` methods),
/// then falls back to `unified::dispatch` for legacy short-name commands.
pub async fn call_tool(
    tool_name: &str,
    params: serde_json::Value,
    server: &NexCoreMcpServer,
) -> Result<serde_json::Value, nexcore_error::NexError> {
    let start = Instant::now();

    // Try first-class tool router, fallback to unified
    let result = match nexcore_mcp::call_tool_direct(tool_name, params.clone(), server).await {
        Ok(r) => r,
        Err(_) => {
            // Fallback: unified dispatcher (legacy short-name commands)
            nexcore_mcp::unified::dispatch(tool_name, params, server)
                .await
                .map_err(|e| nexcore_error::nexerror!("MCP dispatch error: {e:?}"))?
        }
    };

    let value = serde_json::to_value(&result).context("failed to serialize CallToolResult")?;

    let elapsed = start.elapsed();
    tracing::debug!(
        tool = tool_name,
        elapsed_ms = elapsed.as_millis() as u64,
        "mcp bridge call (in-process)"
    );

    Ok(value)
}

#[cfg(test)]
mod tests {
    // Bridge is now a thin wrapper over unified::dispatch — integration
    // testing covers it via the route-level tests.
}
