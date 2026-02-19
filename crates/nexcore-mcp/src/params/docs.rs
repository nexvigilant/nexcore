//! Documentation Generation Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Autonomous generation of CLAUDE.md and specialized documentation for LLM agents.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for autonomous CLAUDE.md generation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DocsGenerateClaudeMdParams {
    /// Path to codebase root
    pub path: Option<String>,
    /// Include architecture section (default: true)
    pub include_architecture: Option<bool>,
    /// Include command reference (default: true)
    pub include_commands: Option<bool>,
    /// Include key directories (default: true)
    pub include_directories: Option<bool>,
}
