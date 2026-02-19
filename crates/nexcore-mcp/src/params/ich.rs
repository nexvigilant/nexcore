//! ICH glossary params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IchLookupParams {
    /// Term to look up in ICH glossary
    pub term: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IchSearchParams {
    /// Search query
    pub query: String,
    /// Max results
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IchGuidelineParams {
    /// Guideline identifier (e.g. "E2B", "E2A")
    pub guideline_id: String,
}
