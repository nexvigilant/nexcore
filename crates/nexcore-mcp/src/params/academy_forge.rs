//! Academy Forge params — extract IR from Rust crates, validate academy content.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for `forge_extract` — extract structured IR from a Rust crate.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeExtractParams {
    /// Crate name (e.g., "nexcore-tov"). Resolves to `~/nexcore/crates/{crate_name}/`.
    pub crate_name: String,
    /// Optional domain plugin name (e.g., "vigilance") for domain-specific extraction.
    #[serde(default)]
    pub domain: Option<String>,
}

/// Parameters for `forge_validate` — validate academy content JSON against rules.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeValidateParams {
    /// Academy content JSON to validate.
    pub content: serde_json::Value,
}

/// Parameters for `forge_scaffold` — generate a pathway authoring template from domain IR.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeScaffoldParams {
    /// Crate name to extract domain IR from (e.g., "nexcore-tov").
    pub crate_name: String,
    /// Domain plugin name (e.g., "vigilance").
    pub domain: String,
    /// Pathway ID prefix (e.g., "tov-01").
    pub pathway_id: String,
    /// Pathway title (e.g., "Introduction to Theory of Vigilance").
    pub title: String,
}

/// Parameters for `forge_compile` — compile pathway JSON into Studio TypeScript files.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeCompileParams {
    /// Path to pathway JSON file (e.g., "content/pathways/tov-01.json").
    /// Resolved relative to ~/nexcore/ if not absolute.
    pub pathway_json: String,
    /// Output directory for generated TypeScript files.
    /// Resolved relative to ~/nexcore/ if not absolute.
    pub output_dir: String,
    /// Whether to overwrite existing files. Default: true.
    #[serde(default = "default_overwrite")]
    pub overwrite: bool,
}

fn default_overwrite() -> bool {
    true
}
