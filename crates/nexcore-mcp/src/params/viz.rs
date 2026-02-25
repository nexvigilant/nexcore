//! Visualization Parameters (nexcore-viz)
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Renders SVG diagrams for taxonomy, composition, loops, and confidence chains.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for STEM taxonomy sunburst visualization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizTaxonomyParams {
    /// Title for the diagram
    #[serde(default)]
    pub title: Option<String>,
}

/// Parameters for type composition visualization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizCompositionParams {
    /// Type name
    pub type_name: String,
    /// Tier classification (T1, T2-P, T2-C, T3)
    pub tier: String,
    /// Comma-separated list of T1 primitives
    pub primitives: String,
    /// Dominant primitive name
    #[serde(default)]
    pub dominant: Option<String>,
    /// Confidence in grounding
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// Parameters for science loop visualization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizLoopParams {
    /// Which loop to render: "science", "chemistry", or "math"
    pub domain: String,
}

/// Parameters for confidence chain visualization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizConfidenceParams {
    /// JSON array of claims
    pub claims: String,
    /// Title for the diagram
    #[serde(default)]
    pub title: Option<String>,
}

/// Parameters for bounds visualization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizBoundsParams {
    /// The value to visualize
    pub value: f64,
    /// Lower bound
    #[serde(default)]
    pub lower: Option<f64>,
    /// Upper bound
    #[serde(default)]
    pub upper: Option<f64>,
    /// Label for the value
    #[serde(default)]
    pub label: Option<String>,
}

/// Parameters for DAG topology visualization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizDagParams {
    /// JSON array of edges
    pub edges: String,
    /// Title for the diagram
    #[serde(default)]
    pub title: Option<String>,
}

/// Parameters for node confidence computation.
///
/// Computes `Measured<f64>` confidence for Observatory graph nodes.
/// Accepts a JSON-encoded `ConfidenceSource` or individual fields.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizNodeConfidenceParams {
    /// Confidence source type: "chi_squared", "signal_strength", "severity",
    /// "relevance_score", or "structural_certainty"
    pub source: String,
    /// Chi-squared statistic (for chi_squared source)
    #[serde(default)]
    pub chi_squared: Option<f64>,
    /// Whether FDR-rejected (for chi_squared source, default: true)
    #[serde(default)]
    pub fdr_rejected: Option<bool>,
    /// PRR value (for signal_strength source)
    #[serde(default)]
    pub prr: Option<f64>,
    /// Whether signal detected (for signal_strength source)
    #[serde(default)]
    pub signal_detected: Option<bool>,
    /// Severity score (for severity source)
    #[serde(default)]
    pub severity: Option<f64>,
    /// Relevance score (for relevance_score source)
    #[serde(default)]
    pub score: Option<f64>,
    /// Whether API-derived (for structural_certainty source)
    #[serde(default)]
    pub api_derived: Option<bool>,
}
