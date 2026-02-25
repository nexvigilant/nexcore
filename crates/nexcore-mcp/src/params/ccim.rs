//! CCIM MCP parameter structs (3 tools).
//!
//! Capability Compound Interest Machine: equation, assessment, projection.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{self, Deserialize};

/// Compute C(d) using the CCIM compound interest equation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcimEquationParams {
    /// Starting capability units (C₀).
    pub c0: f64,
    /// Compounding ratio (rho) in [0.0, 1.0].
    pub rho: f64,
    /// Number of directives (d).
    pub directives: u32,
    /// New tools contributed per directive (T).
    pub t_per_directive: f64,
    /// Cumulative withdrawal from depreciation (W).
    #[serde(default)]
    pub cumulative_w: f64,
    /// Number of observations for confidence calibration.
    #[serde(default = "default_observations")]
    pub observations: u32,
}

/// Assess current CCIM state: NCRR, FIRE progress, conservation check.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcimAssessParams {
    /// Opening capability units.
    pub c_opening: f64,
    /// Closing capability units.
    pub c_closing: f64,
    /// Actual compounding ratio (rho).
    pub rho: f64,
    /// Weighted average depreciation rate (delta_avg).
    #[serde(default)]
    pub delta_avg: f64,
    /// FIRE threshold (default 5000).
    #[serde(default = "default_fire_threshold")]
    pub fire_threshold: f64,
    /// Number of observations for confidence calibration.
    #[serde(default = "default_observations")]
    pub observations: u32,
}

/// Project capability trajectory over N directives with FIRE ETA.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcimProjectParams {
    /// Current capability units.
    pub current_cu: f64,
    /// Compounding ratio (rho) in [0.0, 1.0].
    pub rho: f64,
    /// Number of directives to project.
    pub n_directives: u32,
    /// New tools contributed per directive (T).
    pub t_per_directive: f64,
    /// Withdrawal per directive (W rate).
    #[serde(default)]
    pub w_per_directive: f64,
    /// FIRE threshold (default 5000).
    #[serde(default = "default_fire_threshold")]
    pub fire_threshold: f64,
    /// Number of observations for confidence calibration.
    #[serde(default = "default_observations")]
    pub observations: u32,
}

fn default_observations() -> u32 {
    1
}

fn default_fire_threshold() -> f64 {
    5000.0
}
