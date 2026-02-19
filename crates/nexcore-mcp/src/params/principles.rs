//! Principles knowledge base params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrinciplesListParams {}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrinciplesGetParams {
    /// Principle name (e.g. "dalio-principles", "kiss", "first-principles")
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrinciplesSearchParams {
    /// Search query
    pub query: String,
    /// Max results
    #[serde(default)]
    pub limit: Option<usize>,
}
