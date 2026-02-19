//! Primitive validation params (ICH, BioOntology, PubMed)
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveValidateParams {
    /// Term to validate
    pub term: String,
    /// Domain (e.g. "pharmacovigilance", "clinical")
    pub domain: String,
    /// Minimum validation tier (1-4)
    pub min_tier: u8,
    /// Maximum citations to return
    pub max_citations: usize,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveCiteParams {
    /// PMID or DOI identifier
    pub identifier: String,
    /// Citation format (default: vancouver)
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveValidateBatchParams {
    /// Terms to validate
    pub terms: Vec<String>,
    /// Domain
    pub domain: String,
    /// Minimum validation tier
    pub min_tier: u8,
}
