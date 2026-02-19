//! TRIAL Framework Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! 10 param structs for the universal experimentation framework.
//! Derived from FDA clinical trial methodology (E9 R1, E20, E3).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ── trial_protocol_register ───────────────────────────────────────────────────

/// Parameters for registering a new trial protocol.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialProtocolRegisterParams {
    /// The central hypothesis being tested (required, non-empty)
    pub hypothesis: String,
    /// Target population description
    pub population: String,
    /// Primary endpoint as JSON: {name, metric, direction ("Higher"|"Lower"), threshold}
    pub primary_endpoint_json: String,
    /// Arms as JSON array: [{name, description, is_control}] (min 2, one must have is_control=true)
    pub arms_json: String,
    /// Total planned sample size
    pub sample_size: u32,
    /// Required statistical power (must be >= 0.80)
    pub power: f64,
    /// Type I error rate (default 0.05)
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    /// Planned duration in days (default 90)
    #[serde(default = "default_duration")]
    pub duration_days: u32,
    /// Safety boundary as JSON: {metric, threshold, description}
    pub safety_boundary_json: String,
    /// Adaptation rules as JSON array (may be empty)
    #[serde(default)]
    pub adaptation_rules_json: Option<String>,
    /// Blinding level: "Open" | "Single" | "Double" | "Triple" (default "Double")
    #[serde(default = "default_blinding")]
    pub blinding: String,
}

fn default_alpha() -> f64 { 0.05 }
fn default_duration() -> u32 { 90 }
fn default_blinding() -> String { "Double".into() }

// ── trial_power_analysis ──────────────────────────────────────────────────────

/// Parameters for sample size / power analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialPowerAnalysisParams {
    /// Test type: "two_proportion" | "two_mean" | "survival"
    pub test_type: String,
    /// Proportion in arm 1 — for two_proportion
    #[serde(default)]
    pub p1: Option<f64>,
    /// Proportion in arm 2 — for two_proportion
    #[serde(default)]
    pub p2: Option<f64>,
    /// Cohen's d effect size — for two_mean
    #[serde(default)]
    pub effect_size: Option<f64>,
    /// Hazard ratio — for survival
    #[serde(default)]
    pub hazard_ratio: Option<f64>,
    /// Expected event probability — for survival
    #[serde(default)]
    pub event_prob: Option<f64>,
    /// Two-sided type I error rate (default 0.05)
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    /// Required statistical power (default 0.80)
    #[serde(default = "default_power")]
    pub power: f64,
}

fn default_power() -> f64 { 0.80 }

// ── trial_randomize ───────────────────────────────────────────────────────────

/// Parameters for subject randomization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialRandomizeParams {
    /// Number of subjects to assign
    pub n: u32,
    /// Number of arms (must be >= 2)
    pub arms: usize,
    /// Randomization method: "simple" | "block" | "stratified"
    #[serde(default = "default_method")]
    pub method: String,
    /// Block size for block/stratified (must be multiple of arms, default = arms * 2)
    #[serde(default)]
    pub block_size: Option<usize>,
    /// Optional random seed for reproducibility
    #[serde(default)]
    pub seed: Option<u64>,
    /// Strata as JSON array [{id, n}] — required for stratified method
    #[serde(default)]
    pub strata_json: Option<String>,
}

fn default_method() -> String { "block".into() }

// ── trial_blind_verify ────────────────────────────────────────────────────────

/// Parameters for blinding integrity verification.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialBlindVerifyParams {
    /// Assignments as JSON array from trial_randomize output
    pub assignments_json: String,
    /// Protocol as JSON (from trial_protocol_register output)
    pub protocol_json: String,
}

// ── trial_interim_analyze ─────────────────────────────────────────────────────

/// Parameters for interim analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialInterimAnalyzeParams {
    /// Information fraction accumulated (0 < t ≤ 1)
    pub information_fraction: f64,
    /// Successes observed in treatment arm
    pub treatment_successes: u32,
    /// Total subjects in treatment arm
    pub treatment_n: u32,
    /// Successes observed in control arm
    pub control_successes: u32,
    /// Total subjects in control arm
    pub control_n: u32,
    /// Count of serious safety events
    #[serde(default)]
    pub safety_events: u32,
    /// Protocol as JSON (for alpha, safety boundary, and adaptation rules)
    pub protocol_json: String,
    /// Statistical method: "obf" (default) | "pocock"
    #[serde(default = "default_interim_method")]
    pub method: String,
}

fn default_interim_method() -> String { "obf".into() }

// ── trial_safety_check ────────────────────────────────────────────────────────

/// Parameters for a safety boundary check.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialSafetyCheckParams {
    /// The metric being monitored (e.g., "serious_adverse_event_rate")
    pub metric: String,
    /// Current observed value
    pub observed_value: f64,
    /// The stopping threshold
    pub threshold: f64,
    /// Human-readable description of the rule
    #[serde(default)]
    pub description: Option<String>,
}

// ── trial_endpoint_evaluate ───────────────────────────────────────────────────

/// Parameters for final endpoint evaluation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialEndpointEvaluateParams {
    /// Test type: "two_proportion" | "two_mean"
    pub test_type: String,
    /// Endpoint name (for labeling)
    #[serde(default)]
    pub endpoint_name: Option<String>,
    /// Successes in group 1 — for two_proportion
    #[serde(default)]
    pub s1: Option<u32>,
    /// Total in group 1
    #[serde(default)]
    pub n1: Option<u32>,
    /// Successes in group 2 — for two_proportion
    #[serde(default)]
    pub s2: Option<u32>,
    /// Total in group 2
    #[serde(default)]
    pub n2: Option<u32>,
    /// Mean in group 1 — for two_mean
    #[serde(default)]
    pub mean1: Option<f64>,
    /// Standard deviation in group 1 — for two_mean
    #[serde(default)]
    pub sd1: Option<f64>,
    /// Mean in group 2 — for two_mean
    #[serde(default)]
    pub mean2: Option<f64>,
    /// Standard deviation in group 2 — for two_mean
    #[serde(default)]
    pub sd2: Option<f64>,
    /// Significance level (default 0.05)
    #[serde(default = "default_alpha")]
    pub alpha: f64,
}

// ── trial_multiplicity_adjust ─────────────────────────────────────────────────

/// Parameters for multiplicity adjustment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialMultiplicityAdjustParams {
    /// Raw p-values as comma-separated string (e.g., "0.01,0.03,0.04")
    pub p_values: String,
    /// Family-wise error rate (default 0.05)
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    /// Correction method: "bonferroni" | "holm" | "hochberg" | "bh" (default "holm")
    #[serde(default = "default_multiplicity_method")]
    pub method: String,
}

fn default_multiplicity_method() -> String { "holm".into() }

// ── trial_adapt_decide ────────────────────────────────────────────────────────

/// Parameters for adaptation decision evaluation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialAdaptDecideParams {
    /// Protocol as JSON (must contain the pre-specified adaptation_rules)
    pub protocol_json: String,
    /// The adaptation type to evaluate (must be pre-specified)
    pub adaptation_type: String,
    /// Interim data as JSON {information_fraction, treatment_successes, treatment_n, ...}
    pub interim_data_json: String,
}

// ── trial_report_generate ─────────────────────────────────────────────────────

/// Parameters for CONSORT-style report generation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrialReportGenerateParams {
    /// Protocol as JSON (from trial_protocol_register)
    pub protocol_json: String,
    /// Endpoint results as JSON array (from trial_endpoint_evaluate calls)
    pub results_json: String,
}

// ── Serialize derives for output types ───────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(crate = "rmcp::serde")]
pub struct PowerAnalysisOutput {
    pub test_type: String,
    pub sample_size_per_arm: u32,
    pub alpha: f64,
    pub power: f64,
}
