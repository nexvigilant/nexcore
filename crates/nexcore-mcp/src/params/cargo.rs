//! Cargo Tool Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Parameters for cargo build, check, test, clippy, fmt, and tree.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{self, Deserialize, Deserializer, Serialize};

/// Lenient bool deserializer: accepts `true`, `"true"`, `"1"`, `false`, `"false"`, `"0"`.
/// MCP clients sometimes send booleans as strings.
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

/// Parameters for cargo check (type checking without codegen)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct CargoCheckParams {
    /// Path to crate directory or workspace root. Defaults to current directory.
    pub path: Option<String>,
    /// Specific package to check (for workspace builds). Maps to -p flag.
    pub package: Option<String>,
    /// Build in release mode
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub release: bool,
    /// Extra flags to pass to cargo check (e.g. ["--lib", "--all-targets"])
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// Parameters for cargo build
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct CargoBuildParams {
    /// Path to crate directory or workspace root. Defaults to current directory.
    pub path: Option<String>,
    /// Specific package to build (for workspace builds). Maps to -p flag.
    pub package: Option<String>,
    /// Build in release mode
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub release: bool,
    /// Extra flags to pass to cargo build
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// Parameters for cargo test
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct CargoTestParams {
    /// Path to crate directory or workspace root. Defaults to current directory.
    pub path: Option<String>,
    /// Specific package to test (for workspace builds). Maps to -p flag.
    pub package: Option<String>,
    /// Run only lib tests (--lib)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub lib_only: bool,
    /// Test name filter (passed after --)
    pub test_filter: Option<String>,
    /// Tests to skip (passed as --skip after --)
    #[serde(default)]
    pub skip: Vec<String>,
    /// Extra flags to pass to cargo test
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// Parameters for cargo clippy
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct CargoClippyParams {
    /// Path to crate directory or workspace root. Defaults to current directory.
    pub path: Option<String>,
    /// Specific package to lint (for workspace builds). Maps to -p flag.
    pub package: Option<String>,
    /// Deny all warnings (adds -- -D warnings). Defaults to true.
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub deny_warnings: bool,
    /// Extra flags to pass to cargo clippy
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// Parameters for cargo fmt
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct CargoFmtParams {
    /// Path to crate directory or workspace root. Defaults to current directory.
    pub path: Option<String>,
    /// Specific package to format (for workspace builds). Maps to -p flag.
    pub package: Option<String>,
    /// Check only, don't modify files (--check)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub check_only: bool,
}

/// Parameters for cargo tree (dependency visualization)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct CargoTreeParams {
    /// Path to crate directory or workspace root. Defaults to current directory.
    pub path: Option<String>,
    /// Specific package to show tree for. Maps to -p flag.
    pub package: Option<String>,
    /// Invert the tree to show what depends on a package (--invert SPEC)
    pub invert: Option<String>,
    /// Maximum display depth
    pub depth: Option<u32>,
    /// Show duplicates only (--duplicates)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub duplicates: bool,
}

fn default_true() -> bool {
    true
}
