//! # Skills
//!
//! Unified crate for Claude Code skill types, loading, and implementations.
//!
//! ## Modules
//!
//! - `core` — Traits, types, registry (formerly `skill-core`)
//! - `loader` — SKILL.md file loading (formerly `skill-loader`)
//! - `transfer_confidence` — Cross-domain transfer confidence (formerly `skill-transfer-confidence`)
//! - `primitive_extractor` — Primitive extraction (formerly `skill-primitive-extractor`)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]

/// Core traits and types for skills
pub mod core;

/// Lex Primitiva grounding
pub mod grounding;

/// SKILL.md file loader
pub mod loader;

/// Cross-domain transfer confidence computation
pub mod transfer_confidence;

/// Primitive extraction skill
pub mod primitive_extractor;

/// Derive macros for skills
pub use skill_macros as macros;

use core::SkillRegistry;

/// Create registry with all built-in skills
pub fn default_registry() -> SkillRegistry {
    let mut registry = SkillRegistry::new();
    registry.register(primitive_extractor::PrimitiveExtractor::new());
    registry.register(transfer_confidence::TransferConfidenceSkill::new());
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry() {
        let registry = default_registry();
        assert!(registry.list().len() >= 2);
    }
}
