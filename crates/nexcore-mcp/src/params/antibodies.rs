//! Parameter types for antibodies MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Compute binding affinity between a paratope matcher and an epitope signature.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AntibodyAffinityParams {
    /// Paratope matcher string (detection rule).
    pub paratope_matcher: String,
    /// Epitope signature string (threat feature).
    pub epitope_signature: String,
}

/// Classify which immunoglobulin response class to use.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AntibodyClassifyResponseParams {
    /// Threat severity: "low", "medium", "high", or "critical".
    pub severity: String,
    /// Whether this is a novel (first-seen) threat.
    pub is_novel: bool,
}

/// List all immunoglobulin classes with properties.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AntibodyIgCatalogParams {}

/// Get information about an immunoglobulin class.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AntibodyIgInfoParams {
    /// Immunoglobulin class: "IgG", "IgM", "IgA", "IgD", or "IgE".
    pub class: String,
}
