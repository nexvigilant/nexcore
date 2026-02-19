//! HTTP Request Parameters
//!
//! Primitives: → Causality + λ Location + μ Mapping + ∂ Boundary

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for making an HTTP request (curl equivalent).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HttpRequestParams {
    /// Full URL to request (e.g., "http://localhost:9080/health")
    pub url: String,

    /// HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD). Defaults to GET.
    #[serde(default = "default_method")]
    pub method: String,

    /// Optional request body (for POST/PUT/PATCH).
    #[serde(default)]
    pub body: Option<String>,

    /// Optional headers as key=value pairs (e.g., ["Content-Type=application/json"]).
    #[serde(default)]
    pub headers: Vec<String>,

    /// Request timeout in seconds. Defaults to 10.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// If true, return only the response body. If false, include status and headers.
    #[serde(default)]
    pub body_only: bool,
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_timeout() -> u64 {
    10
}
