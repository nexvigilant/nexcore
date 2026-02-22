//! Skills Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Skill registry, validation, and execution parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for skill scan
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillScanParams {
    /// Directory to scan for skills
    pub directory: String,
}

/// Parameters for skill get
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillGetParams {
    /// Skill name
    pub name: String,
}

/// Parameters for skill validation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillValidateParams {
    /// Path to skill directory
    pub path: String,
}

/// Parameters for skill search by tag
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillSearchByTagParams {
    /// Tag to search for
    pub tag: String,
}

/// Parameters for nexcore_assist intent-based skill search
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AssistParams {
    /// Natural language intent describing what you want to do
    pub intent: String,
    /// Optional context to narrow search (tag filter)
    pub context: Option<String>,
    /// Maximum number of results (default: 5)
    #[serde(default = "default_assist_limit")]
    pub limit: usize,
}

fn default_assist_limit() -> usize {
    5
}

/// Parameters for listing nested skills
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillListNestedParams {
    /// Parent skill name
    pub parent: String,
}

/// Parameters for skill execution
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillExecuteParams {
    /// Name of the skill to execute
    pub name: String,
    /// Parameters to pass to the skill (JSON object)
    #[serde(default)]
    pub parameters: serde_json::Value,
    /// Timeout in seconds (default: 60)
    #[serde(default = "default_skill_timeout")]
    pub timeout_seconds: u64,
}

fn default_skill_timeout() -> u64 {
    60
}

/// Parameters for getting a skill's input/output schema.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillSchemaParams {
    /// Name of the skill
    pub name: String,
}

/// Parameters for taxonomy query
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TaxonomyQueryParams {
    /// Taxonomy type: compliance, smst, category, or node
    pub taxonomy_type: String,
    /// Key to look up
    pub key: String,
}

/// Parameters for taxonomy list
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TaxonomyListParams {
    /// Taxonomy type: compliance, smst, category, or node
    pub taxonomy_type: String,
}

/// Parameters for compilation of compound skills
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillCompileParams {
    /// Skill names to compose
    pub skills: Vec<String>,
    /// Composition strategy: sequential, parallel, feedback_loop
    #[serde(default = "default_strategy")]
    pub strategy: String,
    /// Name for the compound skill
    pub name: String,
    /// Whether to build the binary (default: false, just generates source)
    #[serde(default)]
    pub build: bool,
    /// If true, use optimized graph traversal
    #[serde(default)]
    pub fast_path: bool,
}

// ============================================================================
// SQI (Skill Quality Index) Parameters
// ============================================================================

/// Parameters for scoring a single skill's SQI.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SqiScoreParams {
    /// SKILL.md content to score
    pub content: String,
}

/// Parameters for ecosystem-level SQI scoring.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SqiEcosystemParams {
    /// Tool counts per skill/server
    pub tool_counts: Vec<usize>,
    /// Optional SKILL.md contents to score
    #[serde(default)]
    pub skill_contents: Vec<String>,
}

fn default_strategy() -> String {
    "sequential".into()
}

/// Parameters for checking skill compilation compatibility (dry run)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillCompileCheckParams {
    /// Skill names to check
    pub skills: Vec<String>,
    /// Composition strategy: sequential, parallel, feedback_loop
    #[serde(default = "default_strategy")]
    pub strategy: String,
}

/// Parameters for skill token analysis
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillTokenAnalyzeParams {
    /// Path to skill directory (e.g., "~/.claude/skills/forge")
    pub path: String,
}

/// Parameters for skill orchestration analysis
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillOrchestrationAnalyzeParams {
    /// Skill path or glob pattern
    pub path_or_pattern: String,
    /// Include recommendations for frontmatter additions
    #[serde(default = "default_include_recommendations")]
    pub include_recommendations: bool,
}

fn default_include_recommendations() -> bool {
    true
}

/// Parameters for skill VDAG route resolution
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillRouteParams {
    /// Skill name or natural-language intent to route
    pub query: String,
    /// Maximum chain depth to resolve (default: 3)
    #[serde(default = "default_chain_depth")]
    pub depth: usize,
}

fn default_chain_depth() -> usize {
    3
}

// Re-export skill-related types from knowledge module for qualified access
pub use super::knowledge::{
    PrimitiveSkillLookupParams, SkillChainLookupParams, VocabSkillLookupParams,
};
