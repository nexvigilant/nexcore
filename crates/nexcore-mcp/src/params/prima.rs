//! Prima language params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimaParseParams {
    /// Prima source code
    pub source: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimaEvalParams {
    /// Prima source code to evaluate
    pub source: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimaCodegenParams {
    /// Prima source code
    pub source: String,
    /// Target language (default: rust)
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimaPrimitivesParams {
    /// Prima source code
    pub source: String,
}
