//! Params for T1 primitive coverage analysis tools.

use serde::Deserialize;

/// Analyze T1 primitive coverage in a Rust source file or crate.
#[derive(Debug, Deserialize)]
pub struct PrimitiveCoverageCheckParams {
    /// Source code content OR file path to analyze
    pub source: String,
    /// If true, treat `source` as a file path and read it
    pub is_path: Option<bool>,
}

/// Get the T1 primitive detection rules.
#[derive(Debug, Deserialize)]
pub struct PrimitiveCoverageRulesParams {}
