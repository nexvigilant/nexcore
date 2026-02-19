//! NPM Tool Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Parameters for npm run, install, list, outdated.

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

fn default_false() -> bool {
    false
}

/// Parameters for npm run
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct NpmRunParams {
    /// Script name to run (from package.json scripts)
    pub script: String,
    /// Path to the project directory. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
}

/// Parameters for npm install
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct NpmInstallParams {
    /// Packages to install. If empty, installs from package.json.
    #[serde(default)]
    pub packages: Vec<String>,
    /// Install as devDependencies (--save-dev)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub dev: bool,
    /// Path to the project directory. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
}

/// Parameters for npm list
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct NpmListParams {
    /// Dependency tree depth (default: 0, top-level only)
    #[serde(default)]
    pub depth: u32,
    /// Path to the project directory. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
}

/// Parameters for npm outdated
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct NpmOutdatedParams {
    /// Path to the project directory. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
}
