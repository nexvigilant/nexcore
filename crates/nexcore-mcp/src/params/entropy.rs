//! Entropy computation parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Shannon entropy, cross-entropy, KL divergence, mutual information,
//! normalized and conditional entropy with configurable log base.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for the unified entropy_compute MCP tool.
///
/// Computes information-theoretic quantities from probability distributions
/// or raw counts, with configurable logarithm base.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EntropyComputeParams {
    /// Computation mode
    /// - "shannon": Shannon entropy H(X) from distribution P
    /// - "cross": Cross-entropy H(P,Q) from two distributions
    /// - "kl": KL divergence D_KL(P||Q)
    /// - "mutual": Mutual information I(X;Y) from joint distribution
    /// - "normalized": Normalized entropy H(X)/H_max in [0,1]
    /// - "conditional": Conditional entropy H(Y|X) from joint distribution
    pub mode: String,

    /// Primary probability distribution P (required for all modes).
    /// For "mutual" and "conditional", this is a flattened joint distribution
    /// matrix in row-major order.
    pub distribution_p: Vec<f64>,

    /// Secondary distribution Q (required for "cross" and "kl" modes).
    #[serde(default)]
    pub distribution_q: Option<Vec<f64>>,

    /// Number of rows for joint distribution matrix.
    /// Required for "mutual" and "conditional" modes.
    #[serde(default)]
    pub joint_rows: Option<usize>,

    /// Logarithm base: "bits" (default), "nats", "hartleys"
    #[serde(default = "default_base")]
    pub base: String,

    /// If true, treat distribution_p as raw counts instead of probabilities
    #[serde(default)]
    pub from_counts: bool,
}

fn default_base() -> String {
    "bits".to_string()
}
