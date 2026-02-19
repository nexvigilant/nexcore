//! Sentinel (Security & Boundary) Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! IP whitelisting, auth log parsing, and security configuration.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for checking if an IP is whitelisted by sentinel.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SentinelCheckIpParams {
    /// IP address to check
    pub ip: String,
    /// Optional CIDR ranges to check against
    #[serde(default)]
    pub whitelist_cidrs: Vec<String>,
}

/// Parameters for parsing an auth log line.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SentinelParseLineParams {
    /// A syslog auth line
    pub line: String,
}

/// Parameters for getting sentinel config defaults.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SentinelConfigDefaultsParams {
    /// Output format: "json" or "toml"
    #[serde(default)]
    pub format: Option<String>,
}
