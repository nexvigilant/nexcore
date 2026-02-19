//! Reproductive Parameters (Genetic Maintenance)
//! Tier: T1 (Persistence and Repair)
//!
//! Mutation guarding, specialization, and mitosis repair.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for checking lethal mutation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ReproductiveGuardMutationParams {
    /// List of T1 primitives present.
    pub primitives: Vec<String>,
    /// Whether the change uses unsafe code.
    #[serde(default)]
    pub uses_unsafe: bool,
}

/// Parameters for somatic tissue specialization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ReproductiveSpecializeAgentParams {
    /// Tissue type: 'Nervous', 'Immune', etc.
    pub phenotype: String,
}

/// Parameters for mitotic repair.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ReproductiveStartMitosisParams {
    /// Name of the failing crate.
    pub crate_name: String,
    /// Type of failure.
    pub error_type: String,
    /// Failure severity (0.0 - 1.0).
    #[serde(default = "default_reproductive_severity")]
    pub severity: f64,
}

fn default_reproductive_severity() -> f64 {
    0.5
}
