//! Statistical Drift Detection Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! KS Test, Population Stability Index (PSI), and Jensen-Shannon Divergence
//! for detecting concept drift, data drift, and label shift.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for Kolmogorov-Smirnov two-sample test.
///
/// Compares two distributions to detect statistical shift.
/// Returns D-statistic and approximate p-value.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DriftKsTestParams {
    /// Reference (baseline) distribution values
    pub reference: Vec<f64>,
    /// Current (test) distribution values
    pub current: Vec<f64>,
    /// Significance level (default: 0.05)
    #[serde(default)]
    pub alpha: Option<f64>,
}

/// Parameters for Population Stability Index (PSI).
///
/// Measures population shift between reference and current distributions.
/// PSI < 0.1: no shift, 0.1-0.25: moderate, > 0.25: significant.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DriftPsiParams {
    /// Reference distribution bin proportions (must sum to ~1.0)
    pub reference: Vec<f64>,
    /// Current distribution bin proportions (must sum to ~1.0)
    pub current: Vec<f64>,
    /// Number of bins for auto-binning raw data (default: 10)
    #[serde(default)]
    pub bins: Option<usize>,
    /// Whether inputs are raw data (true) or pre-binned proportions (false, default)
    #[serde(default)]
    pub raw_data: Option<bool>,
}

/// Parameters for Jensen-Shannon Divergence.
///
/// Symmetric divergence measure between two probability distributions.
/// Range [0, ln(2)] for natural log. Returns bits if base-2.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DriftJsdParams {
    /// First probability distribution (must sum to ~1.0)
    pub p: Vec<f64>,
    /// Second probability distribution (must sum to ~1.0)
    pub q: Vec<f64>,
    /// Use base-2 logarithm (bits) instead of natural log (default: true)
    #[serde(default)]
    pub base2: Option<bool>,
}

/// Parameters for composite drift detection.
///
/// Runs KS test, PSI, and JSD simultaneously on two sample sets,
/// returning a unified drift verdict.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DriftDetectParams {
    /// Reference (baseline) sample values
    pub reference: Vec<f64>,
    /// Current sample values
    pub current: Vec<f64>,
    /// Number of histogram bins (default: 10)
    #[serde(default)]
    pub bins: Option<usize>,
    /// Significance level for KS test (default: 0.05)
    #[serde(default)]
    pub alpha: Option<f64>,
    /// PSI threshold for "significant" drift (default: 0.25)
    #[serde(default)]
    pub psi_threshold: Option<f64>,
}
