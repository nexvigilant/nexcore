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

/// Parameters for `forge_scaffold_from_guidance` — scaffold a pathway from FDA guidance metadata.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeGuidanceScaffoldParams {
    /// FDA guidance document slug or partial title. Used to look up the guidance
    /// document in the embedded index (2,794+ docs).
    pub guidance_id: String,
    /// Pathway ID (e.g., "fda-01"). Must match pattern `^[a-z]+-\d{2}(-\d{2})?(-[a-z-]+)?$`.
    pub pathway_id: String,
    /// Pathway title (e.g., "Safety Reporting Fundamentals").
    pub title: String,
    /// Domain name. Default: "pharmacovigilance".
    #[serde(default = "default_domain")]
    pub domain: String,
    /// Optional section titles to structure stages around. If empty, stages
    /// are auto-generated from the guidance document's topics.
    #[serde(default)]
    pub sections: Vec<String>,
}

/// Parameters for `forge_atomize` — decompose a pathway into Atomic Learning Objects.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeAtomizeParams {
    /// Path to pathway JSON file (e.g., "content/pathways/tov-01.json").
    /// Resolved relative to ~/nexcore/ if not absolute.
    pub pathway_json: String,
}

/// Parameters for `forge_graph` — build a cross-pathway Learning Graph.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeGraphParams {
    /// List of pathway JSON file paths to include in the graph.
    /// Resolved relative to ~/nexcore/ if not absolute.
    pub pathway_files: Vec<String>,
    /// Enable fuzzy overlap detection via Jaccard word-level similarity.
    /// Default: false (only exact KSB overlap).
    #[serde(default)]
    pub include_fuzzy: bool,
    /// Similarity threshold for fuzzy overlap (0.0–1.0). Default: 0.6.
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,
}

/// Parameters for `forge_shortest_path` — find shortest path to a target ALO or KSB.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeShortestPathParams {
    /// List of pathway JSON file paths to build the graph from.
    pub pathway_files: Vec<String>,
    /// Target ALO ID (e.g., "tov-01-04-c01"). Mutually exclusive with `target_ksb`.
    #[serde(default)]
    pub target_alo_id: Option<String>,
    /// Target KSB reference (e.g., "K1"). Mutually exclusive with `target_alo_id`.
    #[serde(default)]
    pub target_ksb: Option<String>,
    /// Set of already-completed ALO IDs to skip in path computation.
    #[serde(default)]
    pub completed: Vec<String>,
}

fn default_domain() -> String {
    "pharmacovigilance".to_string()
}

fn default_overwrite() -> bool {
    true
}

fn default_similarity_threshold() -> f32 {
    0.6
}
