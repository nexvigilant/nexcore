//! Insight System Compositor Parameters
//! Tier: T2-T3 (Multi-domain Composite Insight)
//!
//! Status, ingestion, registration, and reset for the unified compositor.

use crate::params::insight::InsightObservationInput;
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for system status query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightSystemStatusParams {
    /// Placeholder for schema compatibility.
    #[serde(default)]
    pub _placeholder: Option<bool>,
}

/// Parameters for system-level ingestion.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightSystemIngestParams {
    /// Domain name (e.g., "guardian").
    pub domain: String,
    /// Observations to ingest.
    pub observations: Vec<InsightObservationInput>,
}

/// Parameters for domain registration.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightSystemRegisterParams {
    /// Domain name to register.
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Parameters for system reset.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightSystemResetParams {
    /// Confirmation flag.
    #[serde(default)]
    pub confirm: Option<bool>,
}
