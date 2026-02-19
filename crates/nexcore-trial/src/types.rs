//! Core domain types for TRIAL framework
//!
//! Tier: T2 (Domain compound types)
//!
//! - `Protocol` — the immutable experiment definition (mirrors IND filing)
//! - `Arm` — a treatment or control group
//! - `Endpoint` — a measurable outcome (primary or secondary)
//! - `SafetyRule` — hard stop boundary condition
//! - `Adaptation` — pre-specified mid-trial modification
//! - `BlindingLevel` — open/single/double/triple
//! - `TrialPhase` — TARGET/REGIMENT/INTERIM/ASSAY/LIFECYCLE
//! - `TrialVerdict` — POSITIVE/NEGATIVE/INCONCLUSIVE
//! - `InterimResult`, `EndpointResult`, `SurveillanceReport`

use serde::{Deserialize, Serialize};

/// Direction of improvement for an endpoint measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EndpointDirection {
    /// Higher values are better (e.g., conversion rate)
    Higher,
    /// Lower values are better (e.g., adverse event rate)
    Lower,
}

/// A measurable outcome for the trial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// Unique name for this endpoint (e.g., "conversion_rate")
    pub name: String,
    /// The metric being measured (e.g., "proportion of users converting")
    pub metric: String,
    /// Whether higher or lower values indicate improvement
    pub direction: EndpointDirection,
    /// Minimum clinically meaningful difference threshold
    pub threshold: f64,
}

/// A treatment or control arm in the trial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arm {
    /// Name of the arm (e.g., "control", "treatment_A")
    pub name: String,
    /// Human-readable description of the arm
    pub description: String,
    /// Whether this is the control/comparator arm
    pub is_control: bool,
}

/// A safety stopping boundary rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRule {
    /// The metric being monitored (e.g., "serious_adverse_event_rate")
    pub metric: String,
    /// The threshold above which the trial must stop
    pub threshold: f64,
    /// Human-readable description of the rule
    pub description: String,
}

/// A pre-specified mid-trial adaptation rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adaptation {
    /// Type identifier (e.g., "sample_reestimate", "arm_drop")
    pub adaptation_type: String,
    /// Conditions under which this adaptation may be triggered
    pub conditions: String,
    /// What parameters may change
    pub allowed_changes: String,
}

/// Level of blinding applied to the trial.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlindingLevel {
    /// No blinding — all parties know assignments
    Open,
    /// Subject blinded only
    Single,
    /// Subject and investigator blinded
    Double,
    /// Subject, investigator, and outcome assessor blinded
    Triple,
}

/// TRIAL phase identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrialPhase {
    /// T — Target: protocol design and hypothesis
    Target,
    /// R — Regiment: randomization and blinding
    Regiment,
    /// I — Interim: interim analysis and safety monitoring
    Interim,
    /// A — Assay: final endpoint evaluation
    Assay,
    /// L — Lifecycle: report generation and adaptation
    Lifecycle,
}

/// Final verdict for the trial.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrialVerdict {
    /// Primary endpoint met with statistical significance
    Positive,
    /// Primary endpoint not met
    Negative,
    /// Results are inconclusive (boundary conditions, futility)
    Inconclusive,
}

/// The immutable experiment definition — registered once, never modified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    /// Unique trial identifier (UUID)
    pub id: String,
    /// The central hypothesis being tested
    pub hypothesis: String,
    /// Target population description
    pub population: String,
    /// Primary endpoint (only one — E9 R1 requirement)
    pub primary_endpoint: Endpoint,
    /// Secondary endpoints (tested with multiplicity adjustment)
    pub secondary_endpoints: Vec<Endpoint>,
    /// Treatment and control arms (minimum 2)
    pub arms: Vec<Arm>,
    /// Total planned sample size
    pub sample_size: u32,
    /// Statistical power (must be >= 0.80)
    pub power: f64,
    /// Type I error rate (default 0.05)
    pub alpha: f64,
    /// Planned duration in days
    pub duration_days: u32,
    /// Safety stopping rule
    pub safety_boundary: SafetyRule,
    /// Pre-specified adaptation rules
    pub adaptation_rules: Vec<Adaptation>,
    /// Blinding level
    pub blinding: BlindingLevel,
    /// ISO 8601 timestamp when protocol was registered
    pub created_at: String,
}

/// Request to register a new protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRequest {
    /// The central hypothesis being tested
    pub hypothesis: String,
    /// Target population description
    pub population: String,
    /// Primary endpoint
    pub primary_endpoint: Endpoint,
    /// Secondary endpoints (may be empty)
    #[serde(default)]
    pub secondary_endpoints: Vec<Endpoint>,
    /// Arms (minimum 2 required)
    pub arms: Vec<Arm>,
    /// Total planned sample size
    pub sample_size: u32,
    /// Required statistical power (must be >= 0.80)
    pub power: f64,
    /// Type I error rate (must be in (0, 1))
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    /// Planned duration in days
    #[serde(default = "default_duration")]
    pub duration_days: u32,
    /// Safety stopping rule
    pub safety_boundary: SafetyRule,
    /// Pre-specified adaptation rules (may be empty)
    #[serde(default)]
    pub adaptation_rules: Vec<Adaptation>,
    /// Blinding level
    #[serde(default = "default_blinding")]
    pub blinding: BlindingLevel,
}

fn default_alpha() -> f64 {
    0.05
}

fn default_duration() -> u32 {
    90
}

fn default_blinding() -> BlindingLevel {
    BlindingLevel::Double
}

/// Interim analysis data snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterimData {
    /// Fraction of planned information accumulated (0.0 to 1.0)
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
    pub safety_events: u32,
}

/// Decision from an interim analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterimDecision {
    /// Continue as planned
    Continue,
    /// Stop early for efficacy (overwhelming evidence of benefit)
    StopEfficacy,
    /// Stop early for futility (insufficient evidence of benefit)
    StopFutility,
    /// Stop early for safety concerns
    StopSafety,
    /// Adapt sample size per pre-specified rule
    AdaptSampleSize,
}

/// Result from an interim analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterimResult {
    /// What to do next
    pub decision: InterimDecision,
    /// Critical boundary value used
    pub boundary: f64,
    /// Observed test statistic
    pub test_statistic: f64,
    /// Bayesian posterior probability of superiority
    pub posterior_prob: f64,
    /// Rationale for the decision
    pub rationale: String,
}

/// Result from evaluating a single endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointResult {
    /// Endpoint name
    pub name: String,
    /// Observed test statistic (Z or t)
    pub test_statistic: f64,
    /// Two-sided p-value
    pub p_value: f64,
    /// Whether the result is statistically significant
    pub significant: bool,
    /// Observed effect size (e.g., difference in proportions)
    pub effect_size: f64,
    /// Lower bound of 95% confidence interval
    pub ci_lower: f64,
    /// Upper bound of 95% confidence interval
    pub ci_upper: f64,
    /// Number Needed to Treat (for binary endpoints)
    pub nnt: Option<f64>,
}

/// Report on blinding integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindingReport {
    /// Blinding level applied
    pub level: BlindingLevel,
    /// Score from 0.0 (completely unblinded) to 1.0 (perfect blinding)
    pub integrity_score: f64,
    /// List of detected blinding violations
    pub violations: Vec<String>,
}

/// Result from checking a safety boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheckResult {
    /// Whether the safety boundary has NOT been crossed (safe = true means continue)
    pub is_safe: bool,
    /// The metric that was checked
    pub metric: String,
    /// The observed value
    pub observed: f64,
    /// The threshold being enforced
    pub threshold: f64,
    /// Margin (threshold - observed); negative means boundary crossed
    pub margin: f64,
}

/// An individual randomization assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmAssignment {
    /// Subject identifier
    pub subject_id: u32,
    /// Index into protocol.arms
    pub arm_index: usize,
    /// Optional stratum identifier
    pub stratum: Option<String>,
    /// Block identifier for block randomization
    pub block_id: Option<u32>,
}

/// A stratum for stratified randomization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stratum {
    /// Stratum identifier (e.g., "age_under_50")
    pub id: String,
    /// Number of subjects in this stratum
    pub n: u32,
}

/// Multiplicity-adjusted result for a single hypothesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustedResult {
    /// Original uncorrected p-value
    pub original_p: f64,
    /// Adjusted significance threshold after correction
    pub adjusted_threshold: f64,
    /// Whether this result is significant after adjustment
    pub significant: bool,
    /// The adjustment method applied
    pub method: String,
}

/// Decision from an adaptation evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationDecision {
    /// Whether the adaptation was approved
    pub approved: bool,
    /// Explanation for the decision
    pub rationale: String,
    /// New parameters if adaptation was approved
    pub new_parameters: Option<serde_json::Value>,
}

/// Alpha spending function for group sequential methods.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpendingFunction {
    /// O'Brien-Fleming (very conservative early, liberal late)
    OBrienFleming,
    /// Pocock (constant spending across looks)
    Pocock,
    /// Kim-DeMets power family (gamma parameter)
    KimDeMets { gamma: f64 },
}
