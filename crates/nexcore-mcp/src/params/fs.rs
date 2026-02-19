//! Filesystem Tool Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Parameters for mkdir, copy, move, chmod operations.

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

/// Parameters for fs_mkdir
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct FsMkdirParams {
    /// Directory path to create
    pub path: String,
    /// Create parent directories as needed (-p flag)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub parents: bool,
}

/// Parameters for fs_copy
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct FsCopyParams {
    /// Source file or directory
    pub source: String,
    /// Destination file or directory
    pub dest: String,
    /// Copy directories recursively (-r flag)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub recursive: bool,
}

/// Parameters for fs_move
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct FsMoveParams {
    /// Source file or directory
    pub source: String,
    /// Destination file or directory
    pub dest: String,
}

/// Parameters for fs_chmod
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct FsChmodParams {
    /// File or directory path
    pub path: String,
    /// Permission mode (e.g., "755", "644", "+x"). Mode "777" is blocked.
    pub mode: String,
    /// Apply recursively (-R flag)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub recursive: bool,
}
