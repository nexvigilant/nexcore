//! Primitive Brain Parameters
//!
//! MCP tool parameters for the Primitive Brain — making T1 Lex Primitiva
//! first-class citizens in working memory with decomposition, querying,
//! distance measurement, conservation checking, and composition.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Decompose a concept into its T1 primitive composition and persist as brain artifact.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveBrainDecomposeParams {
    /// Name of the concept to decompose (e.g., "CausalityAssessment", "SignalDetection")
    pub concept: String,
    /// The T1 primitive names composing this concept (e.g., ["Causality", "Comparison", "Boundary"])
    pub primitives: Vec<String>,
    /// The dominant primitive (if known). Auto-detected if omitted.
    #[serde(default)]
    pub dominant: Option<String>,
    /// Confidence in the decomposition (0.0-1.0). Defaults to 0.8.
    #[serde(default)]
    pub confidence: Option<f64>,
    /// Optional domain tag (e.g., "pharmacovigilance", "software", "biology")
    #[serde(default)]
    pub domain: Option<String>,
    /// If true, persist as a brain artifact named "decomposition:{concept}"
    #[serde(default = "default_true")]
    pub persist: bool,
}

fn default_true() -> bool {
    true
}

/// Query brain state by T1 primitive — find beliefs, patterns, and artifacts grounded to a primitive.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveBrainQueryParams {
    /// T1 primitive name to query by (e.g., "Causality", "Boundary")
    pub primitive: String,
    /// What to search: "beliefs", "patterns", "artifacts", or "all" (default)
    #[serde(default)]
    pub scope: Option<String>,
    /// Maximum results per category
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Compute symmetric difference distance |A△B| between two primitive compositions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveBrainDistanceParams {
    /// First concept name or list of primitive names
    pub a: Vec<String>,
    /// Second concept name or list of primitive names
    pub b: Vec<String>,
    /// If true, treat inputs as concept names and look up their decompositions
    #[serde(default)]
    pub lookup: bool,
}

/// Check the conservation law ∃ = ∂(×(ς, ∅)) against a primitive composition.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveBrainConserveParams {
    /// Concept name or list of primitives to check
    pub primitives: Vec<String>,
    /// Concept name for display
    #[serde(default)]
    pub concept: Option<String>,
}

/// Compose primitives into a named structure with tier classification and suggestions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveBrainComposeParams {
    /// T1 primitive names to compose
    pub primitives: Vec<String>,
    /// Optional target name for the composition
    #[serde(default)]
    pub name: Option<String>,
    /// If true, suggest known types that match this composition
    #[serde(default)]
    pub suggest_matches: bool,
}
