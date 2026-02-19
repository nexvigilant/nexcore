//! Bonding Parameters (Hook-Skill Molecular Bonding)
//! Tier: T2-C (N + κ + ρ — quantity + comparison + recursion)
//!
//! Stability analysis and evolutionary reflection for hook-skill molecules.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for analyzing molecular stability.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BondingAnalyzeParams {
    /// Molecule name or JSON/YAML content
    pub molecule: String,
}

/// Parameters for evolving a molecule.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BondingEvolveParams {
    /// Original molecule name
    pub molecule: String,
    /// Reflection results or reason for evolution
    pub reflection: String,
}
