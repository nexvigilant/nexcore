//! Parameter types for cybercinetics feedback controller tools.

use schemars::JsonSchema;

use rmcp::serde::{Deserialize, Serialize};

/// Parameters for feedback_controller_tick.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FeedbackControllerTickParams {
    /// Current measured frequency (Hz or iterations/sec)
    pub nu_rate: f64,
    /// Minimum acceptable frequency before decay verdict
    pub nu_floor: f64,
    /// Maximum recursion depth (observation layers)
    #[serde(default = "default_rho_ceiling")]
    pub rho_ceiling: u8,
    /// Minimum fidelity threshold for the causal chain
    #[serde(default = "default_f_min")]
    pub f_min: f64,
    /// Current recursion depth (0 = fresh)
    #[serde(default)]
    pub rho_depth: u8,
    /// Causal chain links: array of {cause, effect, fidelity}
    #[serde(default)]
    pub causal_links: Vec<CausalLinkInput>,
}

/// A single causal link for the arrow chain.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CausalLinkInput {
    pub cause: String,
    pub effect: String,
    pub fidelity: f64,
}

fn default_rho_ceiling() -> u8 {
    3
}

fn default_f_min() -> f64 {
    0.80
}

/// Parameters for feedback_registry_status.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FeedbackRegistryStatusParams {
    /// Fidelity threshold below which bindings are flagged as degraded
    #[serde(default = "default_degraded_threshold")]
    pub degraded_threshold: f64,
}

fn default_degraded_threshold() -> f64 {
    0.80
}
