//! Declension Parameters (Architectural Linguistics)
//! Tier: T3 (Logical Inflection)
//!
//! Classify, inflect, agree, and pro-drop potential analysis for crates/tools.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for crate classification.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DeclensionClassifyParams {
    /// Crate name.
    pub crate_name: String,
}

/// Parameters for tool family inflection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DeclensionInflectParams {
    /// List of tool names.
    pub tool_names: Vec<String>,
}

/// Parameters for checking agreement.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DeclensionAgreeParams {
    /// Dependent crate.
    pub from_crate: String,
    /// Dependency crate.
    pub to_crate: String,
}

/// Parameters for pro-drop potential analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DeclensionProdropParams {
    /// Tool name.
    pub tool_name: String,
    /// Parameter names.
    pub param_names: Vec<String>,
    /// Optional CWD.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Optional last tool.
    #[serde(default)]
    pub last_tool: Option<String>,
}
