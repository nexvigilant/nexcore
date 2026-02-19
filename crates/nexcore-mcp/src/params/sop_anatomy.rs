//! SOP-Anatomy-Code parameters
//!
//! Params for triple mapping, cross-domain bridge, codebase audit, and coverage tools.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for sop_anatomy_map — look up triple mapping.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SopAnatomyMapParams {
    /// Section number (1-18). Omit for all 18 sections.
    #[serde(default)]
    pub section: Option<u8>,
}

/// Parameters for sop_anatomy_bridge — cross-domain transfer.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SopAnatomyBridgeParams {
    /// Source domain: "sop", "anatomy", or "code".
    pub source_domain: String,
    /// Concept to transfer (e.g. "Skeleton", "Cargo.toml", "Document Control").
    pub concept: String,
    /// Target domain: "sop", "anatomy", or "code".
    pub target_domain: String,
}

/// Parameters for sop_anatomy_audit — audit codebase against governance.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SopAnatomyAuditParams {
    /// Path to the project/crate root directory to audit.
    pub path: String,
}

/// Parameters for sop_anatomy_coverage — no params needed.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SopAnatomyCoverageParams {}
