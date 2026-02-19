//! Theory of Vigilance (ToV) Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Signal strength (S = U × R × T), stability shells, epistemic trust.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for calculating signal strength S = U × R × T
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TovSignalStrengthParams {
    /// Uniqueness U: rarity measure -log2 P(C|H0) in bits
    pub uniqueness_bits: f64,
    /// Recognition R: detection sensitivity × accuracy (0.0-1.0)
    pub recognition: f64,
    /// Temporal T: decaying relevance factor (0.0-1.0)
    pub temporal: f64,
}

/// Parameters for checking architectural stability shell
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TovStabilityShellParams {
    /// Complexity count (number of elements/modules/dependencies)
    pub complexity: u64,
}

/// Parameters for scoring epistemic trust
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TovEpistemicTrustParams {
    /// ToV hierarchy levels covered (1-8: Molecular to Regulatory)
    pub levels_covered: Vec<u8>,
    /// Number of independent evidence sources
    pub sources: usize,
}
