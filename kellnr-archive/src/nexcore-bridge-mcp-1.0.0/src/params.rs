//! NexCore Bridge MCP Server Parameter Definitions

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for Levenshtein distance calculation
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LevenshteinParams {
    /// Source string
    pub source: String,
    /// Target string
    pub target: String,
}

/// Parameters for SHA-256 hashing
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct Sha256Params {
    /// Input string to hash
    pub input: String,
}

/// Parameters for YAML parsing
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct YamlParseParams {
    /// YAML content to parse
    pub content: String,
}

/// Parameters for complete signal analysis
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalCompleteParams {
    /// Count of drug+event
    pub a: u32,
    /// Count of drug+no_event
    pub b: u32,
    /// Count of other_drug+event
    pub c: u32,
    /// Count of other_drug+no_event
    pub d: u32,
}

/// Parameters for skill schema extraction
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillSchemaParams {
    /// Skill name
    pub name: String,
}
