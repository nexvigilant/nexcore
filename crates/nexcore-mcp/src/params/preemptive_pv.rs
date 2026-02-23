//! Parameter structs for preemptive-pv MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Tier 1: Reactive signal strength from contingency table.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptiveReactiveParams {
    /// Cell a: drug+event co-reports
    pub a: f64,
    /// Cell b: drug without event
    pub b: f64,
    /// Cell c: event without drug
    pub c: f64,
    /// Cell d: neither drug nor event
    pub d: f64,
    /// Signal detection threshold (default: 2.0)
    pub threshold: Option<f64>,
}

/// Gibbs free energy signal emergence feasibility.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptiveGibbsParams {
    /// Enthalpy of mechanism (delta_h)
    pub delta_h_mechanism: f64,
    /// Exposure temperature/duration
    pub t_exposure: f64,
    /// Entropy of information (delta_s)
    pub delta_s_information: f64,
}

/// Trajectory with Hill amplification from time-series reporting data.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptiveTrajectoryParams {
    /// Time-series data points as [{time, rate}, ...]
    pub data: Vec<TrajectoryDataPoint>,
    /// Acceleration weight alpha (default: 0.5)
    pub alpha: Option<f64>,
    /// Hill coefficient n_h (default: 2.0)
    pub hill_n: Option<f64>,
    /// Half-maximal constant k_half (default: 1.0)
    pub hill_k_half: Option<f64>,
}

/// A single time-series data point.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrajectoryDataPoint {
    pub time: f64,
    pub rate: f64,
}

/// Severity/irreversibility weighting (Omega).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptiveSeverityParams {
    /// Seriousness level: "non_serious", "hospitalization", "disability", "life_threatening", "fatal"
    pub seriousness: String,
}

/// Noise floor correction (eta).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptiveNoiseParams {
    /// Stimulated reporting rate
    pub r_stimulated: f64,
    /// Baseline reporting rate
    pub r_baseline: f64,
    /// Sensitivity parameter k (default: 5.0)
    pub k: Option<f64>,
}

/// Tier 2: Predictive signal potential (Psi).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptivePredictiveParams {
    /// Gibbs parameters
    pub delta_h_mechanism: f64,
    pub t_exposure: f64,
    pub delta_s_information: f64,
    /// Time-series reporting data
    pub data: Vec<TrajectoryDataPoint>,
    /// Noise parameters
    pub r_stimulated: f64,
    pub r_baseline: f64,
    /// Optional overrides
    pub alpha: Option<f64>,
    pub hill_n: Option<f64>,
    pub hill_k_half: Option<f64>,
    pub k: Option<f64>,
}

/// Tier 3: Full preemptive evaluation (three-tier decision).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptiveEvaluateParams {
    /// Gibbs parameters
    pub delta_h_mechanism: f64,
    pub t_exposure: f64,
    pub delta_s_information: f64,
    /// Time-series reporting data
    pub data: Vec<TrajectoryDataPoint>,
    /// Noise parameters
    pub r_stimulated: f64,
    pub r_baseline: f64,
    /// Seriousness level
    pub seriousness: String,
    /// Optional config overrides
    pub intervention_cost: Option<f64>,
    pub detection_threshold: Option<f64>,
}

/// Competitive inhibition intervention effect.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptiveInterventionParams {
    /// Maximum harm velocity
    pub v_max: f64,
    /// Current substrate (exposure) level
    pub substrate: f64,
    /// Intervention strength (0=none, 10=DHPC, 25=REMS, 100=withdrawal)
    pub inhibitor: f64,
    /// Michaelis constant (default: 5.0)
    pub k_m: Option<f64>,
    /// Inhibition constant (default: 5.0)
    pub k_i: Option<f64>,
}

/// Solve for required intervention strength to achieve target reduction.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptiveRequiredStrengthParams {
    pub v_max: f64,
    pub substrate: f64,
    /// Target reduction fraction (0.0-1.0)
    pub target_reduction: f64,
    pub k_m: Option<f64>,
    pub k_i: Option<f64>,
}

/// Get the omega table for all seriousness levels.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreemptiveOmegaTableParams {}
