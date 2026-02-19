//! Params for stoichiometry MCP tools.

use serde::Deserialize;

/// Encode a concept as a balanced primitive equation.
#[derive(Debug, Deserialize)]
pub struct StoichiometryEncodeParams {
    /// The concept name (e.g. "Drug Safety")
    pub concept: String,
    /// The authoritative definition (e.g. "monitoring and assessment of drug adverse effects")
    pub definition: String,
    /// The source identifier (e.g. "ICH E2A", "Custom: my-project")
    pub source: String,
}

/// Decode a balanced equation back to a Jeopardy-style answer.
#[derive(Debug, Deserialize)]
pub struct StoichiometryDecodeParams {
    /// JSON-serialized BalancedEquation (as returned by stoichiometry_encode)
    pub equation_json: String,
}

/// Find sister concepts with overlapping primitives.
#[derive(Debug, Deserialize)]
pub struct StoichiometrySistersParams {
    /// Concept name to find sisters for (must exist in the built-in dictionary)
    pub concept: String,
    /// Jaccard similarity threshold (0.0 - 1.0, e.g. 0.3)
    pub threshold: f64,
}

/// Compute thermodynamic mass state for a concept.
#[derive(Debug, Deserialize)]
pub struct StoichiometryMassStateParams {
    /// Concept name to analyze (must exist in the built-in dictionary)
    pub concept: String,
}

/// List or search the built-in dictionary.
#[derive(Debug, Deserialize)]
pub struct StoichiometryDictionaryParams {
    /// Action: "list" to list all terms, "search" to filter by primitive
    pub action: String,
    /// When action is "search": filter to terms containing this primitive
    /// (e.g. "Causality", "Boundary", "Existence")
    pub filter_primitive: Option<String>,
}
