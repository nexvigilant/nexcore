//! Root Cause Diagnosis Parameters
//! Tier: T2-C (→ Causality + κ Comparison + ∅ Void + ∂ Boundary)
//!
//! Structured root cause analysis: symptoms → 5 Whys → Ishikawa → constraint.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// A symptom to diagnose.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SymptomInput {
    /// What was observed
    pub description: String,
    /// Severity: 1 (low) to 5 (critical)
    #[serde(default = "default_severity")]
    pub severity: u8,
    /// Optional: which gate, system area, or domain
    #[serde(default)]
    pub area: Option<String>,
}

fn default_severity() -> u8 {
    3
}

/// A root cause with its causal chain.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RootCauseInput {
    /// Index of the symptom this root cause traces to (0-based)
    pub symptom_index: usize,
    /// Ishikawa category: People, Process, Technology, Strategy, Environment, Measurement
    pub category: String,
    /// The root cause statement
    pub statement: String,
    /// The 5 Whys chain (list of explanations, shallowest to deepest)
    #[serde(default)]
    pub why_chain: Vec<String>,
    /// Confidence in this root cause (0.0-1.0)
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// Corrective action (immediate fix)
    #[serde(default)]
    pub corrective_action: Option<String>,
    /// Preventive action (structural fix)
    #[serde(default)]
    pub preventive_action: Option<String>,
}

fn default_confidence() -> f64 {
    0.5
}

/// Run root cause diagnosis on a set of symptoms and identified root causes.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RootCauseDiagnoseParams {
    /// Observed symptoms to analyze
    pub symptoms: Vec<SymptomInput>,
    /// Root causes identified through 5 Whys analysis
    pub root_causes: Vec<RootCauseInput>,
}
