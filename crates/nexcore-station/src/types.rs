//! Core types for Station resolution.

use nexcore_constants::Measured;
use serde::{Deserialize, Serialize};

// TODO(grounding): TrustTier grounds to ς(State) — discrete state
//   classification across the verification lifecycle.
// TODO(grounding): ResolutionResponse grounds to ε(Threshold) + κ(Comparison)
//   — confidence is threshold-crossing logic over comparative quality signals.
// TODO(grounding): Implement GroundsTo<LexPrimitiva> once nexcore-lex-primitiva
//   is added as a dependency in the integration link.

/// A request to resolve the best tool for a given domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRequest {
    /// The target domain (e.g. "dailymed.nlm.nih.gov").
    pub domain: String,
    /// Optional hint about the task (e.g. "adverse_event_search").
    pub task_hint: Option<String>,
}

/// Trust tier for a resolved config.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrustTier {
    /// Human-verified, production-ready config.
    Verified,
    /// Functional but not yet verified.
    Experimental,
    /// No config available for this domain.
    Unavailable,
}

/// Information about a coverage gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapInfo {
    /// The domain with the gap.
    pub domain: String,
    /// Priority level: "HIGH", "MED", or "LOW".
    pub priority: String,
    /// Why this gap exists.
    pub reason: String,
}

/// The result of resolving a tool for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResponse {
    /// The resolved tool name.
    pub tool_name: String,
    /// The raw config for the tool.
    pub config: serde_json::Value,
    /// Confidence score with calibration metadata.
    // CALIBRATION: confidence = weighted sum of schema_complete(0.35) +
    // selector_present(0.35) + verified(0.30). Observatory-derived boolean
    // inputs. Confidence range: [0.0, 1.0]. Source: Observatory schema audit
    // metrics (25 fields tracked per tool). Upgrade path: replace boolean inputs
    // with continuous Observatory field coverage ratios in Link 3.
    pub confidence: Measured<f64>,
    /// Trust tier classification.
    pub trust_tier: TrustTier,
    /// When this config was last verified (ISO 8601).
    pub verified_at: Option<String>,
    /// Gap information if the domain lacks full coverage.
    pub gap: Option<GapInfo>,
}
