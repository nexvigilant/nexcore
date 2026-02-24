//! MCP Bridge — In-process dispatch to nexcore-mcp tools
//!
//! Calls `nexcore_mcp::unified::dispatch()` directly, eliminating the
//! Unix socket daemon dependency. All tool execution happens in-process.

use std::time::Instant;

use nexcore_error::Context;
use nexcore_mcp::NexCoreMcpServer;

/// Call a single MCP tool via in-process dispatch.
///
/// Routes directly to `nexcore_mcp::unified::dispatch()` without any
/// socket or JSON-RPC overhead.
pub async fn call_tool(
    tool_name: &str,
    params: serde_json::Value,
    server: &NexCoreMcpServer,
) -> Result<serde_json::Value, nexcore_error::NexError> {
    let start = Instant::now();

    let result = nexcore_mcp::unified::dispatch(tool_name, params, server)
        .await
        .map_err(|e| nexcore_error::nexerror!("MCP dispatch error: {e:?}"))?;

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
