//! Epidemiology Parameters — Domain 7
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Epidemiological measures of association, impact, and survival.
//! Transfer confidence to PV: 0.95 (shared 2×2 table, same chi-square).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// 2×2 contingency table: exposed/unexposed × diseased/not-diseased
/// Identical structure to PV's (drug/no-drug × event/no-event)
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiContingencyParams {
    /// Exposed + diseased (a)
    pub a: f64,
    /// Exposed + not diseased (b)
    pub b: f64,
    /// Unexposed + diseased (c)
    pub c: f64,
    /// Unexposed + not diseased (d)
    pub d: f64,
}

/// Parameters for Relative Risk (Risk Ratio)
/// RR = [a/(a+b)] / [c/(c+d)]
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiRelativeRiskParams {
    /// Exposed + diseased (a)
    pub a: f64,
    /// Exposed + not diseased (b)
    pub b: f64,
    /// Unexposed + diseased (c)
    pub c: f64,
    /// Unexposed + not diseased (d)
    pub d: f64,
}

/// Parameters for Odds Ratio
/// OR = (a×d) / (b×c) — identical formula to PV's ROR
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiOddsRatioParams {
    /// Exposed + diseased (a)
    pub a: f64,
    /// Exposed + not diseased (b)
    pub b: f64,
    /// Unexposed + diseased (c)
    pub c: f64,
    /// Unexposed + not diseased (d)
    pub d: f64,
}

/// Parameters for Attributable Risk (Risk Difference)
/// AR = Ie - Io = a/(a+b) - c/(c+d)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiAttributableRiskParams {
    /// Exposed + diseased (a)
    pub a: f64,
    /// Exposed + not diseased (b)
    pub b: f64,
    /// Unexposed + diseased (c)
    pub c: f64,
    /// Unexposed + not diseased (d)
    pub d: f64,
}

/// Parameters for NNT (Number Needed to Treat) / NNH (Number Needed to Harm)
/// NNT = 1/ARR, NNH = 1/ARI
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiNntNnhParams {
    /// Exposed + diseased (a)
    pub a: f64,
    /// Exposed + not diseased (b)
    pub b: f64,
    /// Unexposed + diseased (c)
    pub c: f64,
    /// Unexposed + not diseased (d)
    pub d: f64,
}

/// Parameters for Attributable Fraction (among exposed)
/// AF = (RR - 1) / RR
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiAttributableFractionParams {
    /// Exposed + diseased (a)
    pub a: f64,
    /// Exposed + not diseased (b)
    pub b: f64,
    /// Unexposed + diseased (c)
    pub c: f64,
    /// Unexposed + not diseased (d)
    pub d: f64,
}

/// Parameters for Population Attributable Fraction
/// PAF = Pe(RR-1) / [1 + Pe(RR-1)]
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiPopulationAFParams {
    /// Exposed + diseased (a)
    pub a: f64,
    /// Exposed + not diseased (b)
    pub b: f64,
    /// Unexposed + diseased (c)
    pub c: f64,
    /// Unexposed + not diseased (d)
    pub d: f64,
}

/// Parameters for Incidence Rate
/// IR = events / person-time
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiIncidenceRateParams {
    /// Number of new cases (events)
    pub events: f64,
    /// Total person-time at risk (e.g., person-years)
    pub person_time: f64,
    /// Multiplier for expression (e.g., 1000 for "per 1000 person-years")
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
}

fn default_multiplier() -> f64 {
    1000.0
}

/// Parameters for Point Prevalence
/// P = cases / population
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiPrevalenceParams {
    /// Number of existing cases at a point in time
    pub cases: f64,
    /// Total population at that time
    pub population: f64,
    /// Multiplier for expression (e.g., 100 for percentage, 100000 for "per 100k")
    #[serde(default = "default_prevalence_multiplier")]
    pub multiplier: f64,
}

fn default_prevalence_multiplier() -> f64 {
    100.0
}

/// Parameters for Kaplan-Meier survival estimator
/// S(t) = Π [1 - d_i / n_i] for all i where t_i ≤ t
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiKaplanMeierParams {
    /// Time intervals: each entry is [time, events, censored]
    /// - time: the time point
    /// - events: number of events (deaths/failures) at this time
    /// - censored: number censored at this time
    pub intervals: Vec<KaplanMeierInterval>,
}

/// A single Kaplan-Meier time interval
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KaplanMeierInterval {
    /// Time point
    pub time: f64,
    /// Number of events (deaths/failures) at this time
    pub events: u32,
    /// Number censored at this time
    pub censored: u32,
}

/// Parameters for Standardized Mortality/Morbidity Ratio
/// SMR = observed / expected
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiSmrParams {
    /// Observed number of events
    pub observed: f64,
    /// Expected number of events (from reference population)
    pub expected: f64,
}

/// A single stratum for Mantel-Haenszel stratified analysis (2×2 table)
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiStratumParams {
    /// Exposed + diseased (a)
    pub a: f64,
    /// Exposed + not diseased (b)
    pub b: f64,
    /// Unexposed + diseased (c)
    pub c: f64,
    /// Unexposed + not diseased (d)
    pub d: f64,
    /// Optional stratum label (e.g. "Male", "Age 18-40")
    pub label: Option<String>,
}

/// Parameters for Mantel-Haenszel stratified analysis
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiMantelHaenszelParams {
    /// Stratified 2×2 tables — one entry per stratum
    pub strata: Vec<EpiStratumParams>,
    /// Measure to estimate: "OR" (Odds Ratio) or "RR" (Risk Ratio)
    #[serde(default = "default_mh_measure")]
    pub measure: String,
}

fn default_mh_measure() -> String {
    "OR".to_string()
}

/// Empty params for epidemiology→PV mapping catalog
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EpiPvMappingsParams {}
