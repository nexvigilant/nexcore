//! Crate development framework params (scaffold + audit)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Scaffold a new nexcore crate following the gold standard pattern.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CrateDevScaffoldParams {
    /// Crate name without "nexcore-" prefix (e.g. "logistics", "genomics")
    pub name: String,
    /// Domain description for doc comments (e.g. "supply-chain", "cloud infrastructure")
    pub domain: String,
    /// Optional: brief crate description
    #[serde(default)]
    pub description: Option<String>,
    /// Number of types to scaffold (affects tier distribution). Default: 10
    #[serde(default)]
    pub type_count: Option<usize>,
}

/// Audit a nexcore crate against gold standard quality checks.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CrateDevAuditParams {
    /// Crate name (with or without "nexcore-" prefix)
    pub crate_name: String,
}
