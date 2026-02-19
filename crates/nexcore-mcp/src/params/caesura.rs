//! Caesura Parameters (Structural Seam Detection)
//! Tier: T3 (Boundary Logic)
//!
//! Scanning for structural seams in codebases.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for caesura scanning.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaesuraScanParams {
    /// Directory path to scan.
    pub path: String,
    /// Optional strata to scan.
    #[serde(default)]
    pub strata: Option<Vec<String>>,
    /// Sensitivity (sigma threshold).
    #[serde(default)]
    pub sensitivity: Option<f64>,
}

/// Parameters for caesura metrics on a single file.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaesuraMetricsParams {
    /// File path to analyze.
    pub file_path: String,
}

/// Parameters for caesura report generation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaesuraReportParams {
    /// Directory path to scan and report.
    pub path: String,
    /// Sensitivity (sigma threshold).
    #[serde(default)]
    pub sensitivity: Option<f64>,
}
