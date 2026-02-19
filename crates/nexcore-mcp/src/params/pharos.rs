//! PHAROS surveillance pipeline params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PharosRunParams {
    /// FAERS data directory path
    pub faers_dir: String,
    /// Output directory for results (default: ./output/pharos)
    #[serde(default)]
    pub output_dir: Option<String>,
    /// Minimum case count threshold (default: 3)
    #[serde(default)]
    pub min_cases: Option<i64>,
    /// Include all drug roles (default: false, primary suspect only)
    #[serde(default)]
    pub include_all_roles: Option<bool>,
    /// Threshold mode: "default", "strict", or "sensitive"
    #[serde(default)]
    pub threshold_mode: Option<String>,
    /// Top N signals to include in report (default: 50)
    #[serde(default)]
    pub top_n: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PharosStatusParams {
    /// Output directory to check for reports (default: ./output/pharos)
    #[serde(default)]
    pub output_dir: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PharosReportParams {
    /// Run ID to retrieve (if not specified, returns latest)
    #[serde(default)]
    pub run_id: Option<String>,
    /// Output directory (default: ./output/pharos)
    #[serde(default)]
    pub output_dir: Option<String>,
}
