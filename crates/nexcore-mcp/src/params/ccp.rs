//! Claude Care Process (CCP) Parameters — Pharmacokinetic Engine
//! Tier: T2-C (σ + ∝ + ∂ — Sequence + Proportionality + Boundary)
//!
//! Care episode management, titration, and interaction checking.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for starting a new care episode.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpEpisodeStartParams {
    /// Unique episode identifier
    pub episode_id: String,
    /// Epoch hours when episode starts
    #[serde(default)]
    pub started_at: Option<f64>,
}

/// Parameters for computing a recommended dose.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpDoseComputeParams {
    /// Dosing strategy: "therapeutic", "loading", etc.
    pub strategy: String,
    /// Target plasma level [0, 1]
    pub target_level: f64,
    /// Current plasma level
    #[serde(default)]
    pub current_level: Option<f64>,
    /// Bioavailability (0, 1]
    #[serde(default)]
    pub bioavailability: Option<f64>,
    /// Half-life in hours
    #[serde(default)]
    pub half_life: Option<f64>,
}

/// Parameters for advancing an episode (dose + decay + phase transition).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpEpisodeAdvanceParams {
    /// Episode identifier
    pub episode_id: String,
    /// Current phase: "collect", "assess", "plan", etc.
    pub current_phase: String,
    /// Current plasma level
    #[serde(default)]
    pub current_plasma: Option<f64>,
    /// Dose to administer
    #[serde(default)]
    pub dose: Option<f64>,
    /// Bioavailability (0, 1]
    #[serde(default)]
    pub bioavailability: Option<f64>,
    /// Half-life in hours
    #[serde(default)]
    pub half_life: Option<f64>,
    /// Dosing strategy
    #[serde(default)]
    pub strategy: Option<String>,
    /// Hours of decay to apply
    #[serde(default)]
    pub decay_hours: Option<f64>,
    /// Target phase to transition to
    #[serde(default)]
    pub target_phase: Option<String>,
    /// Reason for phase transition
    #[serde(default)]
    pub reason: Option<String>,
    /// Timestamp (epoch hours)
    #[serde(default)]
    pub timestamp: Option<f64>,
}

/// Parameters for checking interaction effects.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpInteractionCheckParams {
    /// Plasma level of first intervention
    pub level_a: f64,
    /// Plasma level of second intervention
    pub level_b: f64,
    /// Interaction type: "synergistic", "antagonistic", etc.
    pub interaction_type: String,
}

/// Parameters for scoring episode quality.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpEpisodeScoreParams {
    /// Episode identifier
    pub episode_id: String,
    /// Metric to score: "adherence", "efficacy", "safety", "overall"
    pub metric: String,
}

// ============================================================================
// CCP Advanced Parameters
// ============================================================================

/// Parameters for scoring quality.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpQualityScoreParams {
    /// Current plasma level [0, 1]
    pub plasma_level: f64,
    /// Average bioavailability (default: 0.8)
    #[serde(default)]
    pub avg_bioavailability: Option<f64>,
    /// Average half-life (default: 24.0)
    #[serde(default)]
    pub avg_half_life: Option<f64>,
    /// Representative dose (default: 0.5)
    #[serde(default)]
    pub dose: Option<f64>,
}

/// Parameters for validating/executing a phase transition.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpPhaseTransitionParams {
    /// Source phase
    pub from: String,
    /// Target phase
    pub to: String,
    /// Reason for transition
    #[serde(default)]
    pub reason: Option<String>,
    /// Timestamp (epoch hours)
    #[serde(default)]
    pub timestamp: Option<f64>,
}
