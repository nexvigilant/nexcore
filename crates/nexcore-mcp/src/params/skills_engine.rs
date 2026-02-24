//! Skills Engine Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Advanced skill analysis: SQI, maturity, KSB verify, ecosystem, dependency graph,
//! gap analysis, evolution tracking.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for computing Skill Quality Index
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillQualityIndexParams {
    /// Path to skill directory (e.g., "~/.claude/skills/rust-dev")
    pub path: String,
}

/// Parameters for computing skill maturity
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillMaturityParams {
    /// Path to skill directory
    pub path: String,
}

/// Parameters for KSB compliance verification
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillKsbVerifyParams {
    /// Path to skill directory
    pub path: String,
}

/// Parameters for ecosystem-level scoring
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillEcosystemScoreParams {
    /// Path to skills root directory (e.g., "~/.claude/skills")
    pub directory: String,
}

/// Parameters for skill dependency graph traversal
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillDependencyGraphParams {
    /// Root skill name to start traversal from
    pub root_skill: String,
    /// Maximum traversal depth (default: 2)
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

fn default_max_depth() -> usize {
    2
}

/// Parameters for gap analysis against target compliance
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillGapAnalysisParams {
    /// Path to skill directory
    pub path: String,
    /// Target compliance level: "diamond", "platinum", "gold", "silver", "bronze"
    #[serde(default = "default_compliance")]
    pub target_compliance: String,
}

fn default_compliance() -> String {
    "gold".to_string()
}

/// Parameters for tracking skill evolution over time
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillEvolutionTrackParams {
    /// Skill name as registered in the skill registry
    pub skill_name: String,
}
