//! Telemetry MCP Tools - Self-reporting for MCP call metrics
//!
//! Provides tools to query telemetry data collected during MCP tool invocations:
//! - `telemetry_summary`: Aggregate statistics across all tools
//! - `telemetry_by_tool`: Per-tool breakdown with percentiles
//! - `telemetry_slow_calls`: Identify calls exceeding a duration threshold
//!
//! Tier: T3 (Domain-specific MCP telemetry tools)
//! Grounds to: T2-C telemetry types via query functions

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

use crate::params::{
    AuditTrailParams, TelemetryByToolParams, TelemetrySlowCallsParams, TelemetrySummaryParams,
};
use crate::telemetry;

// ============================================================================
// Helper Functions (Primitive Extraction)
// ============================================================================

fn format_summary(summary: &telemetry::TelemetrySummary) -> serde_json::Value {
    let success_rate = compute_success_rate(summary.success_count, summary.total_calls);
    json!({
        "total_calls": summary.total_calls,
        "success_count": summary.success_count,
        "failure_count": summary.failure_count,
        "success_rate": success_rate,
        "duration": format_duration_stats(
            summary.total_duration_ms,
            summary.avg_duration_ms,
            summary.min_duration_ms,
            summary.max_duration_ms
        ),
        "bytes": format_byte_stats(summary.total_input_bytes, summary.total_output_bytes),
        "time_range": format_time_range(&summary.first_call, &summary.last_call)
    })
}

fn compute_success_rate(success_count: u64, total_calls: u64) -> f64 {
    if total_calls > 0 {
        success_count as f64 / total_calls as f64
    } else {
        0.0
    }
}

fn format_duration_stats(
    total_ms: u64,
    avg_ms: f64,
    min_ms: u64,
    max_ms: u64,
) -> serde_json::Value {
    json!({
        "total_ms": total_ms,
        "avg_ms": avg_ms,
        "min_ms": min_ms,
        "max_ms": max_ms
    })
}

fn format_byte_stats(total_input: u64, total_output: u64) -> serde_json::Value {
    json!({
        "total_input": total_input,
        "total_output": total_output
    })
}

fn format_time_range(
    first_call: &Option<chrono::DateTime<chrono::Utc>>,
    last_call: &Option<chrono::DateTime<chrono::Utc>>,
) -> serde_json::Value {
    json!({
        "first_call": first_call,
        "last_call": last_call
    })
}

fn format_tool_stats(stats: &telemetry::ToolStats) -> serde_json::Value {
    json!({
        "tool": stats.tool,
        "call_count": stats.call_count,
        "success_rate": stats.success_rate,
        "duration": format_duration_with_percentiles(stats),
        "bytes": format_byte_stats(stats.total_input_bytes, stats.total_output_bytes)
    })
}

fn format_duration_with_percentiles(stats: &telemetry::ToolStats) -> serde_json::Value {
    json!({
        "avg_ms": stats.avg_duration_ms,
        "min_ms": stats.min_duration_ms,
        "max_ms": stats.max_duration_ms,
        "p50_ms": stats.p50_duration_ms,
        "p95_ms": stats.p95_duration_ms
    })
}

fn format_slow_call(call: &telemetry::SlowCall) -> serde_json::Value {
    json!({
        "timestamp": call.timestamp,
        "tool": call.tool,
        "duration_ms": call.duration_ms,
        "success": call.success
    })
}

fn json_to_result(value: serde_json::Value) -> CallToolResult {
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string());
    CallToolResult::success(vec![Content::text(text)])
}

// ============================================================================
// Tool Implementations
// ============================================================================

/// Get aggregate telemetry statistics.
///
/// Returns total calls, success/failure counts, duration stats, and byte totals.
pub async fn telemetry_summary(
    _params: TelemetrySummaryParams,
) -> Result<CallToolResult, McpError> {
    let summary = telemetry::summary().await;
    let result = format_summary(&summary);
    Ok(json_to_result(result))
}

/// Get statistics for a specific tool.
///
/// Returns call count, success rate, duration percentiles, and byte totals.
pub async fn telemetry_by_tool(params: TelemetryByToolParams) -> Result<CallToolResult, McpError> {
    let stats_opt = telemetry::by_tool(&params.tool_name).await;

    let result = match stats_opt {
        Some(stats) => format_tool_stats(&stats),
        None => json!({
            "error": "not_found",
            "message": format!("No telemetry data found for tool: {}", params.tool_name)
        }),
    };

    Ok(json_to_result(result))
}

/// Get calls that exceeded a duration threshold.
///
/// Returns a list of slow calls sorted by duration (descending).
pub async fn telemetry_slow_calls(
    params: TelemetrySlowCallsParams,
) -> Result<CallToolResult, McpError> {
    let mut slow_calls = telemetry::slow_calls(params.threshold_ms).await;

    // Sort by duration descending
    slow_calls.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));

    // Limit results if specified
    if let Some(limit) = params.limit {
        slow_calls.truncate(limit);
    }

    let formatted_calls: Vec<_> = slow_calls.iter().map(format_slow_call).collect();

    let result = json!({
        "threshold_ms": params.threshold_ms,
        "count": slow_calls.len(),
        "calls": formatted_calls
    });

    Ok(json_to_result(result))
}

/// Query the permanent audit trail (full params + response content).
///
/// Returns matching audit records with tool name, input params, output, and timing.
/// Filters: tool_name (exact match), since (ISO-8601 datetime), success_only, limit.
pub async fn audit_trail(params: AuditTrailParams) -> Result<CallToolResult, McpError> {
    let since_dt = params.since.as_ref().and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    });

    let limit = params.limit.unwrap_or(50);

    let records = telemetry::query_audit_trail(
        params.tool_name.as_deref(),
        since_dt,
        params.success_only,
        Some(limit),
    )
    .await;

    let formatted: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            json!({
                "timestamp": r.timestamp,
                "tool": r.tool,
                "input_json": r.input_json,
                "output_json": r.output_json,
                "duration_ms": r.duration_ms,
                "success": r.success,
                "error_msg": r.error_msg,
            })
        })
        .collect();

    let result = json!({
        "count": formatted.len(),
        "filters": {
            "tool_name": params.tool_name,
            "since": params.since,
            "success_only": params.success_only,
            "limit": limit,
        },
        "records": formatted,
    });

    Ok(json_to_result(result))
}
