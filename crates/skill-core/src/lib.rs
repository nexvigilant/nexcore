//! # Skill Core
//!
//! Core traits and types for Claude Code skills implemented as Rust crates.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

mod context;
mod error;
pub mod grounding;
mod output;
mod registry;
mod traits;
mod trigger;

pub use context::SkillContext;
pub use error::{SkillError, SkillResult};
pub use output::{OutputContent, SkillOutput};
pub use registry::SkillRegistry;
pub use traits::{Composable, Skill, SkillChain, SkillParallel};
pub use trigger::Trigger;

use serde::{Deserialize, Serialize};

/// Compliance level for skill quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComplianceLevel {
    /// Basic: Has SKILL.md with required frontmatter
    Bronze,
    /// Good: Has references, tests
    Silver,
    /// Excellent: Has MCP integration, paired agent
    Gold,
    /// Outstanding: Full automation, metrics
    Platinum,
}

/// Skill metadata (mirrors SKILL.md frontmatter)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Skill name
    pub name: String,
    /// Version
    pub version: String,
    /// Short description
    pub description: String,
    /// Compliance level
    pub compliance: ComplianceLevel,
    /// Required MCP tools
    pub mcp_tools: Vec<String>,
    /// Paired agent (if any)
    pub paired_agent: Option<String>,
    /// Skill dependencies
    pub requires: Vec<String>,
}
