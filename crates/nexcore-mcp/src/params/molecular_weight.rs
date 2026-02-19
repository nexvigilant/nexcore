//! Molecular weight (information-theoretic) params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MwComputeParams {
    /// Optional concept name
    #[serde(default)]
    pub name: Option<String>,
    /// List of primitive symbols
    pub primitives: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MwCompareParams {
    /// First concept name
    #[serde(default)]
    pub name_a: Option<String>,
    /// First concept primitives
    pub primitives_a: Vec<String>,
    /// Second concept name
    #[serde(default)]
    pub name_b: Option<String>,
    /// Second concept primitives
    pub primitives_b: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MwPredictTransferParams {
    /// Primitives to predict transfer for
    pub primitives: Vec<String>,
}
