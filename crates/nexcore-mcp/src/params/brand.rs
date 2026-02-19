//! Brand Semantics & Primitive Reasoning Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! NEXCORE brand decomposition and primitive-first reasoning validation.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for getting a brand decomposition
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrandDecompositionGetParams {
    /// Brand name to look up (e.g., "nexvigilant")
    pub name: String,
}

/// Parameters for testing if a term is primitive
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrandPrimitiveTestParams {
    /// Term to test
    pub term: String,
    /// Definition of the term
    pub definition: String,
    /// Domain-specific terms found in the definition
    #[serde(default)]
    pub domain_terms_in_definition: Option<Vec<String>>,
    /// External concepts the term grounds to
    #[serde(default)]
    pub external_grounding: Option<Vec<String>>,
    /// Whether the term is merely a synonym
    #[serde(default)]
    pub is_synonym: Option<bool>,
    /// Analysis of synonym status
    #[serde(default)]
    pub synonym_analysis: Option<String>,
    /// Number of domains the term appears in
    #[serde(default)]
    pub domain_count: Option<u32>,
}
