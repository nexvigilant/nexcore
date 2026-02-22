//! Pharmacokinetic (PK) MCP Tool Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Typed param structs for 6 PV pharmacokinetic tools.
//! Each struct mirrors the formula inputs for its corresponding PK calculation.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

// ============================================================================
// AUC — Area Under the Curve (trapezoidal rule)
// ============================================================================

/// Parameters for `pv_pk_auc` — area under the plasma concentration-time curve.
///
/// Uses linear or log-linear trapezoidal rule to compute AUC from discrete
/// time-concentration data. AUC is the primary measure of total drug exposure.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvPkAucParams {
    /// Time points (hours). Must be monotonically increasing.
    pub times: Vec<f64>,
    /// Plasma concentration values at each time point (e.g. mg/L).
    /// Must have the same length as `times`.
    pub concentrations: Vec<f64>,
    /// Trapezoidal method: "linear" (default) or "log-linear".
    /// Log-linear is preferred during the elimination phase.
    #[serde(default = "default_method")]
    pub method: String,
}

fn default_method() -> String {
    "linear".to_string()
}

// ============================================================================
// Clearance — CL = F * Dose / AUC
// ============================================================================

/// Parameters for `pv_pk_clearance` — systemic clearance calculation.
///
/// Clearance (CL) represents the volume of plasma cleared of drug per unit time.
/// CL = (F × Dose) / AUC, where F is bioavailability.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvPkClearanceParams {
    /// Administered dose (mg)
    pub dose: f64,
    /// Area under the curve (mg·h/L)
    pub auc: f64,
    /// Bioavailability fraction (0.0–1.0). Default: 1.0 (IV administration).
    #[serde(default = "default_bioavailability")]
    pub bioavailability: f64,
}

fn default_bioavailability() -> f64 {
    1.0
}

// ============================================================================
// Half-Life — t½ = 0.693 × Vd / CL
// ============================================================================

/// Parameters for `pv_pk_half_life` — elimination half-life calculation.
///
/// t½ = (0.693 × Vd) / CL. The time for plasma concentration to decrease by 50%.
/// Clinically determines dosing interval and time to steady state.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvPkHalfLifeParams {
    /// Volume of distribution (L)
    pub volume_distribution: f64,
    /// Systemic clearance (L/h)
    pub clearance: f64,
}

// ============================================================================
// Steady-State Concentration — Css = F × Dose / (CL × τ)
// ============================================================================

/// Parameters for `pv_pk_steady_state` — average steady-state concentration.
///
/// Css_avg = (F × Dose) / (CL × τ), where τ is the dosing interval.
/// Reached after approximately 4–5 half-lives of repeated dosing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvPkSteadyStateParams {
    /// Administered dose per interval (mg)
    pub dose: f64,
    /// Systemic clearance (L/h)
    pub clearance: f64,
    /// Dosing interval τ (hours)
    pub tau: f64,
    /// Bioavailability fraction (0.0–1.0). Default: 1.0.
    #[serde(default = "default_bioavailability")]
    pub bioavailability: f64,
}

// ============================================================================
// Ionization — Henderson-Hasselbalch equation
// ============================================================================

/// Parameters for `pv_pk_ionization` — ionization fraction via Henderson-Hasselbalch.
///
/// For acids: ratio = 10^(pH − pKa). For bases: ratio = 10^(pKa − pH).
/// Ionized fraction determines membrane permeability, absorption, and distribution.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvPkIonizationParams {
    /// Acid dissociation constant (pKa) of the drug
    pub pka: f64,
    /// pH of the biological environment (e.g. stomach 1.5, plasma 7.4)
    pub ph: f64,
    /// True if the drug is a weak acid, false if a weak base. Default: true.
    #[serde(default = "default_is_acid")]
    pub is_acid: bool,
}

fn default_is_acid() -> bool {
    true
}

// ============================================================================
// Michaelis-Menten — v = Vmax × [S] / (Km + [S])
// ============================================================================

/// Parameters for `pv_pk_michaelis_menten` — enzyme kinetics rate calculation.
///
/// Models saturable drug metabolism. When [S] >> Km, rate approaches Vmax
/// (zero-order). When [S] << Km, rate is approximately (Vmax/Km)×[S] (first-order).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvPkMichaelisMentenParams {
    /// Substrate (drug) concentration [S] (e.g. mg/L)
    pub substrate_concentration: f64,
    /// Maximum metabolic velocity Vmax (e.g. mg/h)
    pub vmax: f64,
    /// Michaelis constant Km — concentration at half-Vmax (e.g. mg/L)
    pub km: f64,
}
