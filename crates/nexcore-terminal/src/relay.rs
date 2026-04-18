//! Relay Bridge — HTTP dispatch to nexwatch-relay for AI + voice.
//!
//! Connects to the nexwatch-relay server's `/query` endpoint for text-based
//! AI queries. The relay handles Claude Agent SDK, Station MCP routing,
//! and conversation context.
//!
//! ## Architecture
//!
//! ```text
//! Terminal (AI mode) → relay bridge (HTTP) → nexwatch-relay /query
//!                                                  ↓
//!                                            Agent SDK + MCP
//!                                                  ↓
//!                                            mcp.nexvigilant.com
//! ```
//!
//! ## Grounding
//!
//! `∂(Boundary: terminal → relay HTTP) + μ(Mapping: message → agent) +
//!  →(Causality: prompt → tools → response)`

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Default relay URL.
pub const DEFAULT_RELAY_URL: &str = "http://localhost:8080";

/// Relay connection configuration with shared HTTP client.
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// Base URL for the relay server (e.g., "http://localhost:8080").
    pub base_url: String,
    /// Request timeout for the full round-trip.
    pub timeout: Duration,
    /// Shared HTTP client — reuses connection pool across calls.
    client: reqwest::Client,
}

impl RelayConfig {
    /// Create a new config with the given base URL and timeout.
    ///
    /// # Errors
    ///
    /// Returns `RelayError::Http` if the HTTP client cannot be built.
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> Result<Self, RelayError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(4)
            .build()
            .map_err(|e| RelayError::Http(e.to_string()))?;
        Ok(Self {
            base_url: base_url.into(),
            timeout,
            client,
        })
    }
}

impl Default for RelayConfig {
    fn default() -> Self {
        let base_url =
            std::env::var("NEXWATCH_RELAY_URL").unwrap_or_else(|_| DEFAULT_RELAY_URL.to_string());
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

/// Request body for relay /query endpoint.
#[derive(Debug, Serialize)]
struct QueryRequest {
    text: String,
}

/// Response from relay /query endpoint.
#[derive(Debug, Deserialize)]
struct QueryResponse {
    response: Option<String>,
    elapsed_ms: Option<u64>,
    error: Option<String>,
}

/// Result from a relay query.
#[derive(Debug)]
pub struct RelayResult {
    /// The agent's text response.
    pub text: String,
    /// Round-trip time (relay-reported).
    pub relay_elapsed_ms: u64,
    /// Total round-trip time (including network).
    pub total_elapsed_ms: u64,
}

/// Query the relay with a text message via HTTP POST.
pub async fn query_relay(message: &str, config: &RelayConfig) -> Result<RelayResult, RelayError> {
    let start = Instant::now();
    let url = format!("{}/query", config.base_url);

    let body = QueryRequest {
        text: message.to_string(),
    };

    let response = config
        .client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| RelayError::Http(e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|e| format!("(failed to read response body: {e})"));
        return Err(RelayError::HttpStatus {
            status: status.as_u16(),
            body: body_text,
        });
    }

    let query_response: QueryResponse = response
        .json()
        .await
        .map_err(|e| RelayError::Parse(e.to_string()))?;

    if let Some(error) = query_response.error {
        return Err(RelayError::Relay(error));
    }

    let total_elapsed_ms = start.elapsed().as_millis() as u64;

    let text = query_response
        .response
        .ok_or_else(|| RelayError::Parse("response field missing".to_string()))?;

    tracing::debug!(
        total_ms = total_elapsed_ms,
        relay_ms = query_response.elapsed_ms.unwrap_or(0),
        "relay bridge query"
    );

    Ok(RelayResult {
        text,
        relay_elapsed_ms: query_response.elapsed_ms.unwrap_or(0),
        total_elapsed_ms,
    })
}

/// Errors from relay dispatch.
#[derive(Debug)]
pub enum RelayError {
    /// HTTP transport error.
    Http(String),
    /// Non-2xx HTTP status.
    HttpStatus {
        /// HTTP status code.
        status: u16,
        /// Response body.
        body: String,
    },
    /// Error returned by relay.
    Relay(String),
    /// Failed to parse response.
    Parse(String),
}

impl std::fmt::Display for RelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "Relay HTTP error: {e}"),
            Self::HttpStatus { status, body } => {
                write!(f, "Relay returned HTTP {status}: {body}")
            }
            Self::Relay(e) => write!(f, "Relay error: {e}"),
            Self::Parse(e) => write!(f, "Relay response parse error: {e}"),
        }
    }
}

impl std::error::Error for RelayError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_localhost() {
        let config = RelayConfig::default();
        assert!(config.base_url.contains("localhost"));
    }

    #[test]
    fn query_request_serializes() {
        let req = QueryRequest {
            text: "what signals for semaglutide?".to_string(),
        };
        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
        assert!(json.unwrap_or_default().contains("semaglutide"));
    }

    #[test]
    fn relay_error_display() {
        let err = RelayError::Relay("agent timeout".to_string());
        assert_eq!(format!("{err}"), "Relay error: agent timeout");
    }
}
