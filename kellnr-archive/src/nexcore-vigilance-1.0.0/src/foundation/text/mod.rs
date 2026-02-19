//! # Text Processing Module
//!
//! SKILL.md parsing, SMST extraction, and text manipulation.
//!
//! ## Submodules
//!
//! - **generic** - General text processing utilities
//! - **skill_metadata** - SKILL.md frontmatter parsing
//! - **machine_spec** - SMST (Skill Machine Specification) extraction

pub mod generic;
pub mod machine_spec;
pub mod skill_metadata;

pub use generic::{extract_code_blocks, normalize_whitespace, word_count};
pub use machine_spec::{SmstResult, extract_smst};
pub use skill_metadata::{SkillMetadata, parse_frontmatter};
