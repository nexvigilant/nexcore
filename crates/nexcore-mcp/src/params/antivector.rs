//! Parameter types for anti-vector MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Classify a harm type into its anti-vector strategy.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AntivectorClassifyParams {
    /// Harm type letter: A, B, C, D, E, F, G, H, or I.
    pub harm_type: String,
}

/// Compute a complete anti-vector for a harm vector.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AntivectorComputeParams {
    /// Drug or intervention name.
    pub drug: String,
    /// Adverse event or harm outcome.
    pub event: String,
    /// Harm type letter: A-I.
    pub harm_type: String,
    /// Signal magnitude (0.0 to 1.0).
    pub magnitude: f64,
    /// Confidence in the signal (0.0 to 1.0).
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// Optional causal pathway description.
    pub pathway: Option<String>,
    /// Bias type to assess (optional): indication, notoriety, weber, channeling, protopathic, depletion, stimulated, duplicate.
    pub bias_type: Option<String>,
    /// Bias magnitude (0.0 to 1.0, required if bias_type provided).
    pub bias_magnitude: Option<f64>,
}

fn default_confidence() -> f64 {
    0.5
}

/// Generate an annihilation report for a drug-event pair.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AntivectorReportParams {
    /// Drug or intervention name.
    pub drug: String,
    /// Adverse event or harm outcome.
    pub event: String,
    /// Harm type letter: A-I.
    pub harm_type: String,
    /// Signal magnitude (0.0 to 1.0).
    pub magnitude: f64,
    /// Confidence in the signal (0.0 to 1.0).
    #[serde(default = "default_confidence_report")]
    pub confidence: f64,
    /// Optional causal pathway description.
    pub pathway: Option<String>,
    /// Bias type to assess (optional).
    pub bias_type: Option<String>,
    /// Bias magnitude (0.0 to 1.0).
    pub bias_magnitude: Option<f64>,
    /// Mechanistic intervention description (optional).
    pub intervention: Option<String>,
    /// Expected attenuation from intervention (0.0 to 1.0).
    pub expected_attenuation: Option<f64>,
}

fn default_confidence_report() -> f64 {
    0.5
}

/// Check whether an anti-vector is already deployed in the drug label.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AntivectorLabelCheckParams {
    /// Drug name.
    pub drug: String,
    /// Adverse event to check.
    pub event: String,
    /// Text from the Adverse Reactions section (from DailyMed).
    pub adr_section: Option<String>,
    /// Text from the Warnings/Precautions section.
    pub warnings_section: Option<String>,
    /// Text from the Boxed Warning section.
    pub boxed_warning: Option<String>,
}
