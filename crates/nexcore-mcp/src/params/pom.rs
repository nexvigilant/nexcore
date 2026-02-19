//! Proof of Meaning (PoM) Parameters (Semantic Equivalence)
//! Tier: T2-T3 (Chemistry-Inspired Semantics)
//!
//! Distillation, chromatography, titration, and equivalence proofing.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for distilling expressions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomDistillParams {
    /// The expression to distill.
    pub expression: String,
}

/// Parameters for hierarchy classification via chromatography.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomChromatographParams {
    /// The expression to separate.
    pub expression: String,
}

/// Parameters for titration against canonical standards.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomTitrateParams {
    /// The expression to titrate.
    pub expression: String,
}

/// Parameters for semantic equivalence proofing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomProveEquivalenceParams {
    /// First expression.
    pub expression_a: String,
    /// Second expression.
    pub expression_b: String,
}

/// Parameters for registry statistics (dispatch wrapper).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomRegistryStatsParams {}
