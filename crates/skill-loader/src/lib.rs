//! # Skill Loader
//!
//! Load SKILL.md files from disk and bridge to Rust skill implementations.

#![forbid(unsafe_code)]

mod frontmatter;
pub mod grounding;
mod loader;

pub use frontmatter::{SkillFrontmatter, parse_frontmatter};
pub use loader::{LoadedSkill, SkillLoader};
