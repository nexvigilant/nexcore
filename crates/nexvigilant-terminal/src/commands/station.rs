// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! NexVigilant Station RPC client — direct tool access without Claude.
//!
//! Connects to `mcp.nexvigilant.com` via JSON-RPC 2.0 over HTTPS.
//! Three surfaces:
//! - `GET /health` — station health + telemetry
//! - `GET /tools`  — full tool catalog (151+ tools)
//! - `POST /rpc`   — execute any tool via `tools/call`
//!
//! ## Primitive Grounding
//!
//! `μ(Mapping: JSON-RPC) + ∂(Boundary: HTTPS) + →(Causality: call→result)`

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Station endpoint base URL.
const STATION_URL: &str = "https://mcp.nexvigilant.com";

/// Request ID counter for JSON-RPC.
static RPC_ID: AtomicU64 = AtomicU64::new(1);

/// Managed state for the Station client.
pub struct StationState {
    /// HTTP client (reused across requests for connection pooling).
    client: reqwest::Client,
    /// Station base URL (configurable for local dev).
    base_url: String,
}

impl StationState {
    /// Create a new Station client pointing at production.
    #[must_use]
    pub fn new() -> Self {
        Self::with_url(STATION_URL)
    }

    /// Create a Station client with a custom base URL.
    #[must_use]
    pub fn with_url(url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            base_url: url.trim_end_matches('/').to_string(),
        }
    }
}

impl Default for StationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Station health response (subset of fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationHealth {
    /// "ok" or degraded status.
    pub status: String,
    /// Number of tools available.
    pub tools: u32,
    /// Number of configs loaded.
    pub configs: u32,
    /// Number of research courses.
    pub courses: u32,
    /// Station version.
    pub version: String,
    /// Telemetry snapshot.
    #[serde(default)]
    pub telemetry: StationTelemetry,
}

/// Station telemetry data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StationTelemetry {
    /// Uptime in seconds.
    #[serde(default)]
    pub uptime_seconds: u64,
    /// Total tool calls served.
    #[serde(default)]
    pub total_calls: u64,
    /// Calls per minute.
    #[serde(default)]
    pub calls_per_minute: f64,
    /// Error rate percentage.
    #[serde(default)]
    pub error_rate_pct: f64,
    /// SLO status.
    #[serde(default)]
    pub slo_status: String,
}

/// Tool definition from the /tools endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationTool {
    /// Tool name (e.g., "search_adverse_events").
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Input schema (JSON Schema).
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 request.
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: JsonRpcCallParams,
}

/// Parameters for a tools/call request.
#[derive(Debug, Serialize)]
struct JsonRpcCallParams {
    name: String,
    arguments: serde_json::Value,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    id: u64,
    result: Option<JsonRpcResult>,
    error: Option<JsonRpcError>,
}

/// Successful result from tools/call.
#[derive(Debug, Deserialize)]
struct JsonRpcResult {
    content: Vec<JsonRpcContent>,
}

/// Content item in a JSON-RPC result.
#[derive(Debug, Deserialize)]
struct JsonRpcContent {
    text: String,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    content_type: String,
}

/// JSON-RPC error object.
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    #[allow(dead_code)]
    code: i64,
    message: String,
}

/// Result of a station tool call — what the frontend sees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationCallResult {
    /// Whether the call succeeded.
    pub success: bool,
    /// Tool that was called.
    pub tool: String,
    /// Result text (parsed JSON if possible, raw text otherwise).
    pub result: serde_json::Value,
    /// Error message if failed.
    pub error: Option<String>,
    /// Round-trip duration in milliseconds.
    pub duration_ms: u64,
    /// JSON-RPC request ID.
    pub rpc_id: u64,
}

// ── Tauri Commands ──────────────────────────────────────────────

/// Get Station health status.
#[tauri::command]
pub async fn station_health(
    state: tauri::State<'_, StationState>,
) -> Result<StationHealth, String> {
    let url = format!("{}/health", state.base_url);
    let resp = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Station unreachable: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Station returned {}", resp.status()));
    }

    resp.json::<StationHealth>()
        .await
        .map_err(|e| format!("Failed to parse health response: {e}"))
}

/// List all available Station tools.
#[tauri::command]
pub async fn station_tools(
    state: tauri::State<'_, StationState>,
) -> Result<Vec<StationTool>, String> {
    let url = format!("{}/tools", state.base_url);
    let resp = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Station unreachable: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Station returned {}", resp.status()));
    }

    resp.json::<Vec<StationTool>>()
        .await
        .map_err(|e| format!("Failed to parse tools response: {e}"))
}

/// Call a Station tool via JSON-RPC 2.0.
///
/// This is the core capability — any of the 151+ Station tools
/// can be called directly from the terminal without Claude.
#[tauri::command]
pub async fn station_call(
    state: tauri::State<'_, StationState>,
    tool: String,
    args: serde_json::Value,
) -> Result<StationCallResult, String> {
    let rpc_id = RPC_ID.fetch_add(1, Ordering::Relaxed);
    let url = format!("{}/rpc", state.base_url);

    let request = JsonRpcRequest {
        jsonrpc: "2.0",
        id: rpc_id,
        method: "tools/call",
        params: JsonRpcCallParams {
            name: tool.clone(),
            arguments: args,
        },
    };

    let start = std::time::Instant::now();

    let resp = state
        .client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Station RPC failed: {e}"))?;

    let duration_ms = start.elapsed().as_millis() as u64;

    if !resp.status().is_success() {
        return Ok(StationCallResult {
            success: false,
            tool,
            result: serde_json::Value::Null,
            error: Some(format!("HTTP {}", resp.status())),
            duration_ms,
            rpc_id,
        });
    }

    let rpc_resp: JsonRpcResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse RPC response: {e}"))?;

    if let Some(err) = rpc_resp.error {
        return Ok(StationCallResult {
            success: false,
            tool,
            result: serde_json::Value::Null,
            error: Some(err.message),
            duration_ms,
            rpc_id,
        });
    }

    let result_text = rpc_resp
        .result
        .and_then(|r| r.content.into_iter().next())
        .map(|c| c.text)
        .unwrap_or_default();

    // Try to parse as JSON for structured display; fall back to string
    let result_value = serde_json::from_str::<serde_json::Value>(&result_text)
        .unwrap_or_else(|_| serde_json::Value::String(result_text));

    Ok(StationCallResult {
        success: true,
        tool,
        result: result_value,
        error: None,
        duration_ms,
        rpc_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn station_state_default_points_to_production() {
        let state = StationState::new();
        assert_eq!(state.base_url, "https://mcp.nexvigilant.com");
    }

    #[test]
    fn station_state_custom_url_trims_trailing_slash() {
        let state = StationState::with_url("http://localhost:8080/");
        assert_eq!(state.base_url, "http://localhost:8080");
    }

    #[test]
    fn station_call_result_serializes() {
        let result = StationCallResult {
            success: true,
            tool: "test_tool".into(),
            result: serde_json::json!({"key": "value"}),
            error: None,
            duration_ms: 42,
            rpc_id: 1,
        };
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
    }

    #[test]
    fn rpc_id_increments() {
        let a = RPC_ID.fetch_add(1, Ordering::Relaxed);
        let b = RPC_ID.fetch_add(1, Ordering::Relaxed);
        assert!(b > a);
    }
}
