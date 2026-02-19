//! Forge Parameters (Primitive-First Technology Construction)
//! Tier: T1-T3 (Full construction lifecycle)
//!
//! Construction, mining, prompting, and tier classification for new technologies.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Initialize a new Forge session.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeInitParams {
    /// Optional session ID
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Get primitive reference card.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeReferenceParams {}

/// Mine primitives from a concept.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeMineParams {
    /// Concept name to decompose
    pub concept: String,
    /// T1/T2 primitive symbols
    pub primitives: Vec<String>,
    /// Decomposition rationale
    pub decomposition: String,
}

/// Generate forge prompt for a task.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgePromptParams {
    /// Task name
    pub name: String,
    /// Task description
    pub description: String,
    /// Domain
    #[serde(default)]
    pub domain: Option<String>,
    /// Target tier: T1, T2-P, T2-C, or T3
    #[serde(default)]
    pub target_tier: Option<String>,
}

/// Get session summary.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeSummaryParams {}

/// Suggest the next forge action.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeSuggestParams {
    /// Number of blocking compiler errors
    #[serde(default)]
    pub blocker_count: Option<u32>,
    /// Number of clippy warnings
    #[serde(default)]
    pub warning_count: Option<u32>,
    /// Number of unmined primitives available
    #[serde(default)]
    pub primitives_available: Option<u32>,
    /// Overall generation confidence 0.0-1.0
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// Classify tier from primitive count.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeTierParams {
    /// Number of primitives
    pub count: usize,
}

/// Get Gemini system prompt for Forge mode.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeSystemPromptParams {}

// Re-export forge game/quality types from foundation for qualified access
pub use super::foundation::{
    ForgeCodeGenerateParams, ForgeNashSolveParams, ForgePayoffMatrixParams, ForgeQualityScoreParams,
};
