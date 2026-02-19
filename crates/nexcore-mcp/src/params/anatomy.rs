//! Anatomy Parameters (Workspace Structural Analysis)
//! Tier: T3 (Structural Logic)
//!
//! Blast radius and Chomsky classification for crates.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for blast radius query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyBlastRadiusParams {
    /// Crate name.
    pub crate_name: String,
}

/// Parameters for Chomsky classification query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyChomskyParams {
    /// Crate name to classify.
    pub crate_name: Option<String>,
}
