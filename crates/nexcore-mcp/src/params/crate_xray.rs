//! Crate X-Ray params — deep inspection, validation trials, development goals

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Deep X-ray inspection of a nexcore crate.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CrateXrayParams {
    /// Crate name (with or without "nexcore-" prefix)
    pub crate_name: String,
    /// Include source code statistics (line counts, fn counts). Default: true
    #[serde(default = "default_true")]
    pub include_stats: Option<bool>,
}

/// Run a CTVP validation trial on a crate.
/// Phase 0 = Preclinical (compiles?), Phase 1 = Safety (denials, no panic paths),
/// Phase 2 = Efficacy (grounding, transfers), Phase 3 = Confirmation (tests, clippy),
/// Phase 4 = Surveillance (reverse deps, blast radius, adoption).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CrateXrayTrialParams {
    /// Crate name (with or without "nexcore-" prefix)
    pub crate_name: String,
    /// CTVP phase to run (0-4). Omit to run all phases.
    #[serde(default)]
    pub phase: Option<u8>,
}

/// Generate development goals for a crate based on X-ray gaps.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CrateXrayGoalsParams {
    /// Crate name (with or without "nexcore-" prefix)
    pub crate_name: String,
    /// Max goals to return. Default: 10
    #[serde(default)]
    pub max_goals: Option<usize>,
}

fn default_true() -> Option<bool> {
    Some(true)
}
