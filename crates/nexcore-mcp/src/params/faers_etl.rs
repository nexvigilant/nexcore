//! FAERS ETL pipeline params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlRunParams {
    /// FAERS data directory path
    pub faers_dir: String,
    /// Include all drug roles (default: false, primary suspect only)
    #[serde(default)]
    pub include_all_roles: Option<bool>,
    /// Minimum case count threshold
    #[serde(default)]
    pub min_cases: Option<i64>,
    /// Top N results
    #[serde(default)]
    pub top_n: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlSignalsParams {
    /// FAERS data directory path
    pub faers_dir: String,
    /// Drug name filter
    #[serde(default)]
    pub drug: Option<String>,
    /// Event name filter
    #[serde(default)]
    pub event: Option<String>,
    /// Include all drug roles
    #[serde(default)]
    pub include_all_roles: Option<bool>,
    /// Minimum case count
    #[serde(default)]
    pub min_cases: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlDrugEventPair {
    /// Drug name
    pub drug: String,
    /// Event name
    pub event: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlKnownPairsParams {
    /// FAERS data directory path
    pub faers_dir: String,
    /// Known drug-event pairs to check
    pub pairs: Vec<FaersEtlDrugEventPair>,
    /// Minimum case count
    #[serde(default)]
    pub min_cases: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlStatusParams {
    /// Output directory to check
    #[serde(default)]
    pub output_dir: Option<String>,
}
