//! Station Bridge — HTTP dispatch to NexVigilant Station (mcp.nexvigilant.com).
//!
//! Sends MCP `tools/call` requests to the Station's REST endpoint and returns
//! the result as a JSON value. Uses the `/rpc` surface for single-tool calls.
//!
//! ## Grounding
//!
//! `∂(Boundary: terminal → Station HTTP) + μ(Mapping: tool_name → endpoint) +
//!  →(Causality: request → response)`

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Default Station endpoint.
pub const DEFAULT_STATION_URL: &str = "https://mcp.nexvigilant.com";

/// Station HTTP client configuration with shared connection pool.
#[derive(Debug, Clone)]
pub struct StationConfig {
    /// Base URL for the Station (e.g., "https://mcp.nexvigilant.com").
    pub base_url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Shared HTTP client — reuses connection pool across calls.
    client: reqwest::Client,
}

impl StationConfig {
    /// Create a new config with the given base URL and timeout.
    ///
    /// # Errors
    ///
    /// Returns `StationError::Http` if the HTTP client cannot be built.
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> Result<Self, StationError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(4)
            .build()
            .map_err(|e| StationError::Http(e.to_string()))?;
        Ok(Self {
            base_url: base_url.into(),
            timeout,
            client,
        })
    }
}

impl Default for StationConfig {
    fn default() -> Self {
        let base_url = std::env::var("NEXVIGILANT_STATION_URL")
            .unwrap_or_else(|_| DEFAULT_STATION_URL.to_string());
        let timeout = Duration::from_secs(30);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(4)
            .build()
            .unwrap_or_default();
        Self {
            base_url,
            timeout,
            client,
        }
    }
}

/// JSON-RPC request body for MCP `tools/call`.
#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    method: &'static str,
    id: u64,
    params: RpcCallParams,
}

/// Parameters for an MCP `tools/call` request.
#[derive(Debug, Serialize)]
struct RpcCallParams {
    name: String,
    arguments: serde_json::Value,
}

/// JSON-RPC response envelope.
#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<serde_json::Value>,
    error: Option<RpcError>,
}

/// JSON-RPC error object.
#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

/// Result of a Station tool call.
#[derive(Debug)]
pub struct StationResult {
    /// The tool result payload.
    pub value: serde_json::Value,
    /// Round-trip latency.
    pub elapsed: Duration,
}

/// Call a tool on NexVigilant Station via HTTP.
///
/// Sends a JSON-RPC `tools/call` to `{base_url}/rpc` and returns the result.
pub async fn call_station_tool(
    tool_name: &str,
    params: serde_json::Value,
    config: &StationConfig,
) -> Result<StationResult, StationError> {
    let start = Instant::now();

    let url = format!("{}/rpc", config.base_url);
    let body = RpcRequest {
        jsonrpc: "2.0",
        method: "tools/call",
        id: 1,
        params: RpcCallParams {
            name: tool_name.to_string(),
            arguments: params,
        },
    };

    let json_body = serde_json::to_string(&body).map_err(|e| StationError::Parse(e.to_string()))?;

    let response = config
        .client
        .post(&url)
        .header("content-type", "application/json")
        .body(json_body)
        .send()
        .await
        .map_err(|e| StationError::Http(e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let body_text = response.text().await.unwrap_or_default();
        return Err(StationError::HttpStatus {
            status: status.as_u16(),
            body: body_text,
        });
    }

    let rpc_response: RpcResponse = response
        .json()
        .await
        .map_err(|e| StationError::Parse(e.to_string()))?;

    if let Some(err) = rpc_response.error {
        return Err(StationError::Rpc {
            code: err.code,
            message: err.message,
        });
    }

    let value = rpc_response.result.unwrap_or(serde_json::Value::Null);

    let elapsed = start.elapsed();
    tracing::debug!(
        tool = tool_name,
        elapsed_ms = elapsed.as_millis() as u64,
        "station bridge call (HTTP)"
    );

    Ok(StationResult { value, elapsed })
}

/// Errors from Station HTTP dispatch.
#[derive(Debug)]
pub enum StationError {
    /// HTTP transport error.
    Http(String),
    /// Non-2xx HTTP status.
    HttpStatus {
        /// HTTP status code.
        status: u16,
        /// Response body.
        body: String,
    },
    /// JSON-RPC error from Station.
    Rpc {
        /// Error code.
        code: i64,
        /// Error message.
        message: String,
    },
    /// Failed to parse response.
    Parse(String),
}

impl std::fmt::Display for StationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "Station HTTP error: {e}"),
            Self::HttpStatus { status, body } => {
                write!(f, "Station returned HTTP {status}: {body}")
            }
            Self::Rpc { code, message } => {
                write!(f, "Station RPC error ({code}): {message}")
            }
            Self::Parse(e) => write!(f, "Station response parse error: {e}"),
        }
    }
}

impl std::error::Error for StationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_production_url() {
        // Verify the production URL constant. Env var override is checked by
        // StationConfig::default() but is not deterministic in tests.
        assert_eq!(DEFAULT_STATION_URL, "https://mcp.nexvigilant.com");
    }

    #[test]
    fn rpc_request_serializes_correctly() {
        let req = RpcRequest {
            jsonrpc: "2.0",
            method: "tools/call",
            id: 1,
            params: RpcCallParams {
                name: "search_adverse_events".to_string(),
                arguments: serde_json::json!({"drug": "metformin"}),
            },
        };
        let json = serde_json::to_value(&req);
        assert!(json.is_ok());
        let json = json.unwrap_or_default();
        assert_eq!(json["method"], "tools/call");
        assert_eq!(json["params"]["name"], "search_adverse_events");
        assert_eq!(json["params"]["arguments"]["drug"], "metformin");
    }

    #[test]
    fn station_error_display() {
        let err = StationError::Rpc {
            code: -32601,
            message: "Method not found".to_string(),
        };
        let display = format!("{err}");
        assert!(display.contains("-32601"));
        assert!(display.contains("Method not found"));
    }
}
