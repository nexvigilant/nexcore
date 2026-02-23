//! Parameter types for harm taxonomy MCP tools (ToV §9).

use schemars::JsonSchema;
use serde::Deserialize;

/// Classify a harm event from its three binary characteristics.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HarmClassifyParams {
    /// Perturbation multiplicity: "single" or "multiple".
    pub multiplicity: String,
    /// Temporal profile: "acute", "chronic", or "any".
    pub temporal: String,
    /// Response determinism: "deterministic" or "stochastic".
    pub determinism: String,
}

/// Get the full definition for a harm type by letter (A-I).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HarmDefinitionParams {
    /// Harm type letter: A, B, C, D, E, F, G, H, or I.
    pub harm_type: String,
}

/// Get the harm-axiom connection for a specific type.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HarmAxiomParams {
    /// Harm type letter: A, B, C, D, E, F, G, H, or I.
    pub harm_type: String,
}

/// List all harm type definitions (A-I catalog).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HarmCatalogParams {}

/// Verify Theorem 9.0.1 (exhaustiveness).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HarmExhaustivenessParams {}

/// List all harm-axiom connections.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HarmAxiomCatalogParams {}

/// List common harm type combinations.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HarmCombinationsParams {}

/// Derive manifestation level from propagation probabilities.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HarmManifestationDeriveParams {
    /// Harm type letter: A through I.
    pub harm_type: String,
    /// Propagation probabilities at each hierarchy level.
    pub propagation_probs: Vec<f64>,
    /// Detection threshold (0.0-1.0).
    pub detection_threshold: f64,
    /// Probability threshold for manifestation (0.0-1.0).
    pub p_thresh: f64,
}
