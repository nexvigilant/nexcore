//! Systemctl Tool Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Parameters for systemctl status, restart, start, list-units.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{self, Deserialize, Deserializer};

/// Lenient bool deserializer
fn deserialize_bool_lenient<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde", untagged)]
    enum BoolOrString {
        Bool(bool),
        Str(String),
    }

    match BoolOrString::deserialize(deserializer)? {
        BoolOrString::Bool(b) => Ok(b),
        BoolOrString::Str(s) => match s.to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(true),
            "false" | "0" | "no" | "" => Ok(false),
            other => Err(serde::de::Error::custom(format!(
                "expected boolean or bool-like string, got: {other}"
            ))),
        },
    }
}

fn default_true() -> bool {
    true
}

/// Parameters for systemctl status
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct SystemctlStatusParams {
    /// Unit name (e.g., "nexcore-mcp", "nginx")
    pub unit: String,
    /// Use --user scope (default: true for safety). System scope requires explicit user=false.
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub user: bool,
}

/// Parameters for systemctl restart
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct SystemctlRestartParams {
    /// Unit name to restart
    pub unit: String,
    /// Use --user scope (default: true). System-wide restart is blocked.
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub user: bool,
}

/// Parameters for systemctl start
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct SystemctlStartParams {
    /// Unit name to start
    pub unit: String,
    /// Use --user scope (default: true). System-wide start is blocked.
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub user: bool,
}

/// Parameters for systemctl list-units
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct SystemctlListParams {
    /// Use --user scope (default: true)
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub user: bool,
    /// Filter by state (e.g., "running", "failed", "active")
    #[serde(default)]
    pub state: Option<String>,
}
