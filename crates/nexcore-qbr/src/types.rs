//! Core types for Quantitative Benefit-Risk computation.
//!
//! ## Type Hierarchy
//!
//! - `BenefitRiskInput` — What goes in (contingency tables, weights, Hill params)
//! - `QBR` — What comes out (all four forms, with method details)
//! - `HillCurveParams` — Sigmoid curve shape (k_half, n_hill)
//! - `IntegrationBounds` — Dose range for therapeutic window integration
//! - `QbrSignalMethod` — Which signal detection algorithm to use
//!
//! Tier: T3-D (Domain Composite)
//! Grounding: →(Causality) + N(Quantity) + κ(Comparison) + ∂(Boundary)

use nexcore_constants::Measured;
use nexcore_pv_core::signals::ContingencyTable;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// SIGNAL METHOD SELECTION
// ═══════════════════════════════════════════════════════════════════════════

/// Which signal detection algorithm to use for QBR computation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QbrSignalMethod {
    /// Proportional Reporting Ratio
    Prr,
    /// Reporting Odds Ratio
    Ror,
    /// Information Component (Bayesian)
    Ic,
    /// Empirical Bayes Geometric Mean — default, enables Form 2
    #[default]
    Ebgm,
}

// ═══════════════════════════════════════════════════════════════════════════
// HILL CURVE PARAMETERS (shape only — pharmacological model)
// ═══════════════════════════════════════════════════════════════════════════

/// Hill curve shape parameters.
///
/// Describes a sigmoid dose-response relationship:
/// `Y = dose^n_hill / (k_half^n_hill + dose^n_hill)`
///
/// Separating shape from integration bounds allows reusing the same
/// curve parameters across different dose range queries.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HillCurveParams {
    /// Half-saturation constant (EC50 for efficacy, TC50 for toxicity).
    /// Must be positive.
    pub k_half: f64,
    /// Hill coefficient (cooperativity factor).
    /// Must be positive. nH > 1 = positive cooperativity.
    pub n_hill: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// INTEGRATION BOUNDS (the question asked of the curves)
// ═══════════════════════════════════════════════════════════════════════════

/// Dose range for therapeutic window integration.
///
/// Defines the definite integral bounds: `∫[dose_min → dose_max]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IntegrationBounds {
    /// Lower dose bound (inclusive). Must be non-negative.
    pub dose_min: f64,
    /// Upper dose bound (inclusive). Must be > dose_min.
    pub dose_max: f64,
    /// Number of Simpson's rule intervals (must be even, default 1000).
    /// Higher = more accurate.
    #[serde(default = "default_intervals")]
    pub intervals: usize,
}

fn default_intervals() -> usize {
    1000
}

impl Default for IntegrationBounds {
    fn default() -> Self {
        Self {
            dose_min: 0.0,
            dose_max: 100.0,
            intervals: 1000,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INPUT
// ═══════════════════════════════════════════════════════════════════════════

/// Input for QBR computation.
///
/// At minimum, requires one benefit table and one risk table.
/// Optional weights enable Form 3 (composite). Optional Hill parameters
/// enable Form 4 (therapeutic window).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenefitRiskInput {
    /// Contingency tables for benefit outcomes (exposure × beneficial outcome).
    pub benefit_tables: Vec<ContingencyTable>,
    /// Contingency tables for risk outcomes (exposure × adverse outcome).
    pub risk_tables: Vec<ContingencyTable>,
    /// Clinical significance weights for benefit outcomes.
    /// Must match `benefit_tables.len()` if provided.
    #[serde(default)]
    pub benefit_weights: Option<Vec<Measured<f64>>>,
    /// Clinical significance weights for risk outcomes.
    /// Must match `risk_tables.len()` if provided.
    #[serde(default)]
    pub risk_weights: Option<Vec<Measured<f64>>>,
    /// Efficacy Hill curve (for therapeutic window).
    #[serde(default)]
    pub hill_efficacy: Option<HillCurveParams>,
    /// Toxicity Hill curve (for therapeutic window).
    #[serde(default)]
    pub hill_toxicity: Option<HillCurveParams>,
    /// Integration bounds for therapeutic window.
    #[serde(default)]
    pub integration_bounds: Option<IntegrationBounds>,
    /// Signal detection method to use.
    #[serde(default)]
    pub method: QbrSignalMethod,
}

// ═══════════════════════════════════════════════════════════════════════════
// OUTPUT
// ═══════════════════════════════════════════════════════════════════════════

/// Per-method intermediate results for audit transparency.
///
/// In a regulated environment, you need to show your work.
/// An auditor can inspect each component independently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QbrMethodDetails {
    /// Signal strength computed from primary benefit table.
    pub benefit_signal: Measured<f64>,
    /// Signal strength computed from primary risk table.
    pub risk_signal: Measured<f64>,
    /// EBGM lower bound (EB05) for benefit — only with EBGM method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_eb05: Option<f64>,
    /// EBGM upper bound (EB95) for risk — only with EBGM method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_eb95: Option<f64>,
    /// Worst-case Bayesian QBR: EB05_benefit / EB95_risk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worst_case_bayesian: Option<Measured<f64>>,
    /// Signal method used.
    pub method: QbrSignalMethod,
}

/// Full QBR computation result — all applicable forms.
///
/// Every output carries propagated `Measured<T>` confidence.
/// Optional forms are `None` when their required inputs are absent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QBR {
    /// Form 1: Simple ratio — signal_strength(benefit) / signal_strength(risk).
    pub simple: Measured<f64>,
    /// Form 2: Bayesian — EBGM_benefit / EBGM_risk (only with EBGM method).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bayesian: Option<Measured<f64>>,
    /// Form 3: Composite weighted — Σ(w_i × signal(benefit_i)) / Σ(w_j × signal(risk_j)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite: Option<Measured<f64>>,
    /// Form 4: Therapeutic window — ∫(Hill_efficacy - Hill_toxicity) dd.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub therapeutic_window: Option<Measured<f64>>,
    /// Intermediate computation details for audit trail.
    pub details: QbrMethodDetails,
}
