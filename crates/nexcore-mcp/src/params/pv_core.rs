//! PV Core Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! IVF axiom assessment, severity classification, survival analysis.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// SURVIVAL ANALYSIS PARAMS
// ═══════════════════════════════════════════════════════════════════════════════

/// A single time-to-event observation for survival analysis.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SurvivalObservationParam {
    /// Time to event or censoring
    pub time: f64,
    /// true = event occurred, false = censored
    pub event: bool,
}

/// Parameters for Kaplan-Meier survival estimation with Measured<T> confidence.
///
/// Takes individual observation-level data (time, event/censored).
/// Returns full survival curve with Greenwood SE, CI, and confidence scores.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvCoreKaplanMeierParams {
    /// Individual time-to-event observations
    pub observations: Vec<SurvivalObservationParam>,
}

/// Parameters for log-rank test comparing two survival groups.
///
/// Returns chi-squared statistic, p-value, hazard ratio, and Measured confidence.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvCoreLogRankParams {
    /// Group 0 (e.g. treatment) observations
    pub group0: Vec<SurvivalObservationParam>,
    /// Group 1 (e.g. control/placebo) observations
    pub group1: Vec<SurvivalObservationParam>,
}

/// Parameters for cumulative incidence estimation: CI(t) = 1 - S(t).
///
/// Complementary perspective to Kaplan-Meier: probability that event
/// has occurred by time t, rather than probability of survival.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvCoreCumulativeIncidenceParams {
    /// Individual time-to-event observations
    pub observations: Vec<SurvivalObservationParam>,
}

/// A single Cox regression observation with covariates.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CoxObservationParam {
    /// Time to event or censoring
    pub time: f64,
    /// true = event occurred, false = censored
    pub event: bool,
    /// Covariate values (one per predictor)
    pub covariates: Vec<f64>,
}

/// Parameters for Cox proportional hazards regression with Measured<T> confidence.
///
/// Fits a Cox PH model and returns hazard ratios with confidence intervals
/// and Measured confidence scores per coefficient.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvCoreCoxParams {
    /// Observations with covariates
    pub observations: Vec<CoxObservationParam>,
    /// Maximum Newton-Raphson iterations (default: 25)
    #[serde(default = "default_cox_max_iter")]
    pub max_iterations: usize,
    /// Convergence tolerance (default: 1e-6)
    #[serde(default = "default_cox_tolerance")]
    pub tolerance: f64,
}

fn default_cox_max_iter() -> usize {
    25
}

fn default_cox_tolerance() -> f64 {
    1e-6
}

/// Parameters for quick two-group hazard ratio with Measured<T> confidence.
///
/// Simplified interface: provide treatment and control times/events directly.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvCoreHazardRatioParams {
    /// Treatment group event times
    pub treatment_times: Vec<f64>,
    /// Treatment group event indicators (true=event, false=censored)
    pub treatment_events: Vec<bool>,
    /// Control group event times
    pub control_times: Vec<f64>,
    /// Control group event indicators (true=event, false=censored)
    pub control_events: Vec<bool>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// FDR / MULTIPLE TESTING CORRECTION PARAMS
// ═══════════════════════════════════════════════════════════════════════════════

/// Parameters for adjusting p-values for multiple comparisons.
///
/// Supports Benjamini-Hochberg (FDR), Bonferroni, Holm, and Šidák (FWER).
/// Use for any batch of p-values from statistical tests.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvFdrAdjustParams {
    /// Raw p-values from multiple hypothesis tests
    pub p_values: Vec<f64>,
    /// Correction method: "bh" (default), "bonferroni", "holm", "sidak"
    #[serde(default = "default_fdr_method")]
    pub method: String,
    /// FDR/FWER level (default: 0.05)
    #[serde(default = "default_fdr_level")]
    pub fdr_level: f64,
}

fn default_fdr_method() -> String {
    "bh".to_string()
}

fn default_fdr_level() -> f64 {
    0.05
}

// ═══════════════════════════════════════════════════════════════════════════════
// BAYESIAN UPDATE PARAMS
// ═══════════════════════════════════════════════════════════════════════════════

/// Parameters for Beta-Binomial conjugate Bayesian update.
///
/// Models binary outcome data (e.g. ADR present/absent) using a Beta prior
/// conjugated with binomial evidence. Posterior is Beta(alpha + successes,
/// beta + failures).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvCoreBetaBinomialParams {
    /// Beta prior alpha (default: 0.5 = Jeffreys non-informative)
    #[serde(default = "default_jeffreys")]
    pub prior_alpha: f64,
    /// Beta prior beta (default: 0.5 = Jeffreys non-informative)
    #[serde(default = "default_jeffreys")]
    pub prior_beta: f64,
    /// Number of successes (events observed)
    pub successes: u64,
    /// Number of failures (non-events)
    pub failures: u64,
}

fn default_jeffreys() -> f64 {
    0.5
}

/// Parameters for Gamma-Poisson conjugate Bayesian update.
///
/// Models count/rate data (e.g. ADR count per exposure time) using a Gamma
/// prior conjugated with Poisson evidence. Posterior is
/// Gamma(shape + count, rate + exposure).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvCoreGammaPoissonParams {
    /// Gamma prior shape (default: 0.5 = weak)
    #[serde(default = "default_jeffreys")]
    pub prior_shape: f64,
    /// Gamma prior rate (default: 0.5 = weak)
    #[serde(default = "default_jeffreys")]
    pub prior_rate: f64,
    /// Observed event count
    pub count: u64,
    /// Exposure time (person-years or equivalent)
    pub exposure: f64,
}

/// Parameters for sequential Bayesian update with multiple evidence batches.
///
/// Processes a sequence of evidence updates against the same prior,
/// demonstrating that order doesn't matter for conjugate models.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvCoreSequentialBetaBinomialParams {
    /// Beta prior alpha (default: 0.5 = Jeffreys)
    #[serde(default = "default_jeffreys")]
    pub prior_alpha: f64,
    /// Beta prior beta (default: 0.5 = Jeffreys)
    #[serde(default = "default_jeffreys")]
    pub prior_beta: f64,
    /// Sequence of evidence batches: each is [successes, failures]
    pub evidence_sequence: Vec<[u64; 2]>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// IVF / SEVERITY PARAMS
// ═══════════════════════════════════════════════════════════════════════════════

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
