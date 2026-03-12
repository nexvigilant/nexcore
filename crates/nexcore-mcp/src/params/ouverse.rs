//! Ouverse Chain Parameters
//!
//! MCP tool parameters for computing ouverse chains — the forward-enablement
//! direction of the conservation law. Given existence, trace what it makes
//! possible downstream.
//!
//! Source: primitives.ipynb Cell 123, entries 21-23.
//! Equation: ouverse(∃) = σ[∃₁ → π₁ → ∃₂ → π₂ → ... → ∃ₙ]

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// A single link in the ouverse chain: ∃ₙ → π(∃ₙ) → enables ∃ₙ₊₁.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OuverseLink {
    /// What exists at this level (the existence statement)
    pub existence: String,
    /// How it persists (what π holds — the mechanism that carries ∃ forward)
    pub persistence: String,
    /// T1 primitive names present at this link (e.g., ["Causality", "Boundary", "Persistence"])
    #[serde(default)]
    pub primitives: Vec<String>,
}

/// Compute an ouverse chain — trace existence forward into what it enables.
///
/// The ouverse is the opposite of inverse (Five Whys). Given an existence,
/// it asks "what does this make possible?" iteratively, producing a chain
/// of enabling conditions with the root strength at ∃₁.
///
/// Example:
/// ```json
/// {
///   "name": "Guardian Angel",
///   "chain": [
///     {
///       "existence": "Charitable mission — PV knowledge belongs to everyone",
///       "persistence": "Encoded in CLAUDE.md, rules, and team culture",
///       "primitives": ["Boundary", "Existence", "Persistence"]
///     },
///     {
///       "existence": "Domain knowledge directed outward, not hoarded",
///       "persistence": "KSB framework, micrograms, typed Rust domains",
///       "primitives": ["Mapping", "State", "Causality"]
///     }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveBrainOuverseParams {
    /// Name for this ouverse chain (e.g., "Guardian Angel", "PV Pipeline")
    pub name: String,
    /// The ordered chain of links from root strength (∃₁) to outcome (∃ₙ).
    /// First link is the root — if absent, entire chain collapses.
    pub chain: Vec<OuverseLink>,
    /// If true, persist as a brain artifact named "ouverse:{name}"
    #[serde(default = "default_true")]
    pub persist: bool,
}

fn default_true() -> bool {
    true
}
