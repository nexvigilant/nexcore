//! Grounded Parameters (Epistemological Substrate)
//! Tier: T1 (Confidence, Uncertainty, Evidence — universal primitives)
//!
//! Uncertainty tracking, evidence chain propagation, and confidence gating.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Create an uncertain value with confidence.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GroundedUncertainParams {
    /// The value (as JSON — string, number, object, etc.)
    pub value: serde_json::Value,
    /// Confidence in [0.0, 1.0]
    pub confidence: f64,
    /// Optional provenance description (where this value came from)
    #[serde(default)]
    pub provenance: Option<String>,
}

/// Gate a value on minimum confidence threshold.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GroundedRequireParams {
    /// The value (as JSON)
    pub value: serde_json::Value,
    /// Current confidence in [0.0, 1.0]
    pub confidence: f64,
    /// Minimum confidence required to proceed
    pub min_confidence: f64,
    /// Optional provenance
    #[serde(default)]
    pub provenance: Option<String>,
}

/// Compose two confidence values (multiplicative).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GroundedComposeParams {
    /// First confidence in [0.0, 1.0]
    pub confidence_a: f64,
    /// Second confidence in [0.0, 1.0]
    pub confidence_b: f64,
    /// Optional label for the composition
    #[serde(default)]
    pub label: Option<String>,
}

/// Start a new evidence chain for a claim.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GroundedEvidenceNewParams {
    /// The claim this evidence chain supports
    pub claim: String,
    /// Initial prior confidence in [0.0, 1.0]
    pub initial_confidence: f64,
}

/// Add a step to an evidence chain (strengthen or weaken).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GroundedEvidenceStepParams {
    /// Chain ID (from grounded_evidence_new)
    pub chain_id: String,
    /// Description of this evidence step
    pub description: String,
    /// Factor magnitude in [0.0, 1.0]
    pub factor: f64,
    /// Direction: "strengthen" or "weaken"
    pub direction: String,
}

/// Get the full evidence chain with all steps and current confidence.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GroundedEvidenceGetParams {
    /// Chain ID (from grounded_evidence_new)
    pub chain_id: String,
}

/// Run a grounded skill assessment against a skill directory.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GroundedSkillAssessParams {
    /// Path to the skill directory to assess
    pub skill_path: String,
}
