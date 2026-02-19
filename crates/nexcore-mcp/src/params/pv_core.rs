//! PV Core Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! IVF axiom assessment, severity classification.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for IVF axiom assessment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IvfAssessParams {
    /// Potency/power of intervention (0.0-1.0)
    pub potency: f64,
    /// Uncertainty about emergence patterns (0.0-1.0)
    pub emergence_uncertainty: f64,
    /// Exposure to vulnerable populations (0.0-1.0)
    pub vulnerability_exposure: f64,
    /// Deployment scale (0.0-1.0)
    #[serde(default = "default_half")]
    pub deployment_scale: f64,
    /// Completeness of pre-deployment testing (0.0-1.0)
    #[serde(default = "default_half")]
    pub testing_completeness: f64,
}

fn default_half() -> f64 {
    0.5
}

/// Parameters for listing IVF axioms (no params needed)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IvfAxiomsParams {}

/// Parameters for Hartwig-Siegel severity assessment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SeverityAssessParams {
    /// Was treatment changed (drug held, discontinued, or switched)?
    #[serde(default)]
    pub treatment_changed: bool,
    /// Was an antidote or specific treatment required?
    #[serde(default)]
    pub antidote_required: bool,
    /// Was hospital admission required (or stay prolonged)?
    #[serde(default)]
    pub hospitalization_required: bool,
    /// Was ICU admission required?
    #[serde(default)]
    pub icu_required: bool,
    /// Did permanent disability or harm result?
    #[serde(default)]
    pub permanent_harm: bool,
    /// Did the patient die?
    #[serde(default)]
    pub death: bool,
}
