//! # Skill Loader
//!
//! Load SKILL.md files from disk and bridge to Rust skill implementations.

mod frontmatter;
#[allow(clippy::module_inception)]
mod loader;

pub use frontmatter::{SkillFrontmatter, parse_frontmatter};
pub use loader::{LoadedSkill, SkillLoader};
