//! Kellnr MCP parameter structs (consolidated from kellnr-mcp server).
//!
//! 40 tools: 15 registry + 25 computation (PK, thermo, stats, graph, dtree, surveillance).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{self, Deserialize, Deserializer};

// ---------------------------------------------------------------------------
// Lenient deserializers for MCP string-encoded numerics
// ---------------------------------------------------------------------------

fn deserialize_option_f64_lenient<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde", untagged)]
    enum Val {
        Num(f64),
        Str(String),
    }

    match Option::<Val>::deserialize(deserializer)? {
        None => Ok(None),
        Some(Val::Num(n)) => Ok(Some(n)),
        Some(Val::Str(s)) if s.is_empty() => Ok(None),
        Some(Val::Str(s)) => s
            .parse::<f64>()
            .map(Some)
            .map_err(|_| serde::de::Error::custom(format!("expected f64, got: {s}"))),
    }
}

fn deserialize_option_u32_lenient<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde", untagged)]
    enum Val {
        Num(u32),
        Str(String),
    }

    match Option::<Val>::deserialize(deserializer)? {
        None => Ok(None),
        Some(Val::Num(n)) => Ok(Some(n)),
        Some(Val::Str(s)) if s.is_empty() => Ok(None),
        Some(Val::Str(s)) => s
            .parse::<u32>()
            .map(Some)
            .map_err(|_| serde::de::Error::custom(format!("expected u32, got: {s}"))),
    }
}

// ===========================================================================
// Registry params (15 tools)
// ===========================================================================

/// Search crates by name, keyword, or description.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrSearchCratesParams {
    /// Search query string
    pub query: String,
    /// Results per page (default: 20)
    #[serde(default, deserialize_with = "deserialize_option_u32_lenient")]
    pub per_page: Option<u32>,
}

/// Get crate metadata by name.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrCrateNameParams {
    /// Name of the crate
    pub crate_name: String,
}

/// Get specific version details.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrCrateVersionParams {
    /// Name of the crate
    pub crate_name: String,
    /// Version string (e.g. "1.0.0")
    pub version: String,
}

/// Add or remove a crate owner.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrCrateOwnerParams {
    /// Name of the crate
    pub crate_name: String,
    /// GitHub username to add or remove
    pub username: String,
}

/// List all crates with pagination.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrListAllCratesParams {
    /// Results per page (default: 100)
    #[serde(default, deserialize_with = "deserialize_option_u32_lenient")]
    pub per_page: Option<u32>,
}

// ===========================================================================
// Pharmacokinetics params (6 tools)
// ===========================================================================

/// AUC via trapezoidal rule.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrPkAucParams {
    /// Time points
    pub times: Vec<f64>,
    /// Concentration values at each time point
    pub concentrations: Vec<f64>,
    /// Method: "linear" or "log-linear" (default: "linear")
    pub method: Option<String>,
}

/// Time to steady state from half-life.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrPkSteadyStateParams {
    /// Elimination half-life (hours)
    pub half_life: f64,
    /// Multiplier for half-life (default: 5.0)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub multiplier: Option<f64>,
}

/// Henderson-Hasselbalch ionization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrPkIonizationParams {
    /// Acid dissociation constant (pKa)
    pub pka: f64,
    /// pH of the environment
    pub ph: f64,
    /// True for acid, false for base (default: true)
    pub is_acid: Option<bool>,
}

/// Clearance from dose and AUC.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrPkClearanceParams {
    /// Dose in mg
    pub dose: f64,
    /// Area under curve (mg*h/L)
    pub auc: f64,
    /// Bioavailability fraction 0-1 (default: 1.0)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub bioavailability: Option<f64>,
}

/// Volume of distribution from dose and initial concentration.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrPkVolumeDistributionParams {
    /// Dose in mg
    pub dose: f64,
    /// Initial plasma concentration (mg/L)
    pub initial_concentration: f64,
    /// Bioavailability fraction 0-1 (default: 1.0)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub bioavailability: Option<f64>,
}

/// Michaelis-Menten enzyme kinetics.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrPkMichaelisMentenParams {
    /// Substrate concentration
    pub substrate_concentration: f64,
    /// Maximum reaction velocity
    pub vmax: f64,
    /// Michaelis constant
    pub km: f64,
}

// ===========================================================================
// Thermodynamics params (4 tools)
// ===========================================================================

/// Gibbs free energy calculation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrThermoGibbsParams {
    /// Enthalpy change (J/mol)
    pub delta_h: f64,
    /// Entropy change (J/mol/K)
    pub delta_s: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
}

/// Dissociation constant from Gibbs free energy.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrThermoKdParams {
    /// Gibbs free energy (J/mol)
    pub delta_g: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
}

/// Binding kinetics from kon/koff.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrThermoBindingAffinityParams {
    /// Association rate constant (1/M/s)
    pub kon: f64,
    /// Dissociation rate constant (1/s)
    pub koff: f64,
}

/// Arrhenius temperature-dependent rate.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrThermoArrheniusParams {
    /// Pre-exponential factor
    pub pre_exponential: f64,
    /// Activation energy (J/mol)
    pub activation_energy: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
}

// ===========================================================================
// Statistics params (5 tools)
// ===========================================================================

/// Welch t-test for two samples.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrStatsWelchParams {
    /// First sample values
    pub sample1: Vec<f64>,
    /// Second sample values
    pub sample2: Vec<f64>,
}

/// OLS linear regression.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrStatsOlsParams {
    /// Independent variable values
    pub x: Vec<f64>,
    /// Dependent variable values
    pub y: Vec<f64>,
}

/// Poisson confidence interval.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrStatsPoissonCiParams {
    /// Observed event count
    pub count: u64,
    /// Confidence level 0-1 (default: 0.95)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub confidence_level: Option<f64>,
}

/// Bayesian posterior with Beta-Binomial.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrStatsBayesianParams {
    /// Number of successes
    pub successes: u64,
    /// Total number of trials
    pub trials: u64,
    /// Prior alpha parameter (default: 1.0 for uniform)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub prior_alpha: Option<f64>,
    /// Prior beta parameter (default: 1.0 for uniform)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub prior_beta: Option<f64>,
}

/// Shannon entropy of a probability distribution.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrStatsEntropyParams {
    /// Probability distribution (must sum to ~1.0)
    pub probabilities: Vec<f64>,
}

// ===========================================================================
// Graph Theory params (4 tools)
// ===========================================================================

/// Graph edges for betweenness, SCC, and topsort.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrGraphEdgesParams {
    /// Edges as [[from, to], ...] pairs
    pub edges: Vec<Vec<usize>>,
    /// Total number of nodes in the graph
    pub node_count: usize,
}

/// Mutual information between two discrete variables.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrGraphMutualInfoParams {
    /// First discrete variable values
    pub x: Vec<u32>,
    /// Second discrete variable values
    pub y: Vec<u32>,
}

// ===========================================================================
// Decision Tree params (3 tools)
// ===========================================================================

/// Feature importance via Gini impurity.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrDtreeFeatureImportanceParams {
    /// Feature matrix (rows = samples, cols = features)
    pub features: Vec<Vec<f64>>,
    /// Class labels for each sample
    pub labels: Vec<u32>,
}

/// Cost-complexity pruning analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrDtreePruneParams {
    /// Number of leaf nodes in the tree
    pub tree_size: usize,
    /// Training error rate
    pub training_error: f64,
    /// Complexity parameter alpha (default: 0.01)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub alpha: Option<f64>,
}

/// Convert splits to interpretable rules.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrDtreeToRulesParams {
    /// Decision tree splits to convert to rules
    pub splits: Vec<KellnrDtreeSplit>,
}

/// A single decision tree split for rule extraction.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrDtreeSplit {
    /// Feature index for the split
    pub feature_index: usize,
    /// Split threshold value
    pub threshold: f64,
    /// Optional label for the split node
    pub label: Option<String>,
}

// ===========================================================================
// Surveillance params (3 tools)
// ===========================================================================

/// Sequential Probability Ratio Test.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrSignalSprtParams {
    /// Observed event count
    pub observed: u64,
    /// Expected event count under null hypothesis
    pub expected: f64,
    /// Type I error rate (default: 0.05)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub alpha: Option<f64>,
    /// Type II error rate (default: 0.20)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub beta: Option<f64>,
}

/// CUSUM control chart.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrSignalCusumParams {
    /// Time series values to monitor
    pub values: Vec<f64>,
    /// Target/expected value
    pub target: f64,
    /// Alert threshold (default: 5.0)
    #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
    pub threshold: Option<f64>,
}

/// Weibull time-to-onset fit.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KellnrSignalWeibullTtoParams {
    /// Onset times in days
    pub onset_times: Vec<f64>,
}
