//! # GroundsTo implementations for skill-loader types
//!
//! Skill loading and frontmatter types grounded to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `SkillFrontmatter` -- State (varsigma) dominant as it captures parsed config.
//! - `LoadedSkill` -- Mapping (mu) dominant as it maps SKILL.md files to loaded state.
//! - `SkillLoader` -- Mapping (mu) dominant as it maps filesystem paths to skills.
//! - `FrontmatterError`, `LoaderError` -- Boundary (partial) dominant as error boundaries.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::frontmatter::{FrontmatterError, SkillFrontmatter};
use crate::loader::{LoadedSkill, LoaderError, SkillLoader};

/// SkillFrontmatter: T2-C (varsigma + mu + sigma + exists), dominant varsigma
///
/// Parsed SKILL.md frontmatter with typed fields. State-dominant as it
/// encapsulates the configuration state of a skill definition.
/// Mapping is secondary (YAML text -> structured data).
/// Sequence is tertiary (ordered triggers/requires lists).
/// Existence is quaternary (optional paired_agent, has_crate flag).
impl GroundsTo for SkillFrontmatter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- configuration state
            LexPrimitiva::Mapping,   // mu -- YAML -> struct
            LexPrimitiva::Sequence,  // sigma -- ordered triggers/requires
            LexPrimitiva::Existence, // exists -- optional fields
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// LoadedSkill: T2-P (mu + varsigma + lambda), dominant mu
///
/// A skill loaded from disk with frontmatter, prompt body, and source path.
/// Mapping-dominant because the core operation is SKILL.md file -> loaded skill.
/// State is secondary (loaded configuration state).
/// Location is tertiary (filesystem path reference).
impl GroundsTo for LoadedSkill {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- file -> loaded skill
            LexPrimitiva::State,    // varsigma -- loaded state
            LexPrimitiva::Location, // lambda -- filesystem path
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// SkillLoader: T2-C (mu + varsigma + sigma + lambda), dominant mu
///
/// Loads and manages SKILL.md files from the filesystem.
/// Mapping-dominant: maps filesystem paths to loaded skill instances.
/// State is secondary (mutable skill collection).
/// Sequence is tertiary (load order). Location is quaternary (file paths).
impl GroundsTo for SkillLoader {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- path -> loaded skill
            LexPrimitiva::State,    // varsigma -- mutable collection
            LexPrimitiva::Sequence, // sigma -- load order
            LexPrimitiva::Location, // lambda -- filesystem paths
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// FrontmatterError: T2-P (partial + kappa), dominant partial
///
/// Error boundary for frontmatter parsing. Boundary-dominant because
/// errors indicate parsing crossed a validity boundary (no frontmatter, bad YAML).
/// Comparison is secondary (discriminates error variant).
impl GroundsTo for FrontmatterError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- parsing boundary
            LexPrimitiva::Comparison, // kappa -- error variant discrimination
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// LoaderError: T2-P (partial + kappa + Sigma), dominant partial
///
/// Error boundary for skill loading. Boundary-dominant because it
/// defines the boundary between successful and failed loading operations.
/// Comparison is secondary (error type discrimination).
/// Sum is tertiary (aggregates IO and frontmatter errors).
impl GroundsTo for LoaderError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- load boundary
            LexPrimitiva::Comparison, // kappa -- error discrimination
            LexPrimitiva::Sum,        // Sigma -- aggregated error sources
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn skill_frontmatter_is_t2c() {
        assert_eq!(SkillFrontmatter::tier(), Tier::T2Composite);
    }

    #[test]
    fn loaded_skill_is_t2p() {
        assert_eq!(LoadedSkill::tier(), Tier::T2Primitive);
    }

    #[test]
    fn skill_loader_is_t2c() {
        assert_eq!(SkillLoader::tier(), Tier::T2Composite);
    }

    #[test]
    fn frontmatter_error_is_t2p() {
        assert_eq!(FrontmatterError::tier(), Tier::T2Primitive);
    }

    #[test]
    fn loader_error_is_t2p() {
        assert_eq!(LoaderError::tier(), Tier::T2Primitive);
    }

    #[test]
    fn loaded_skill_dominant_is_mapping() {
        let comp = LoadedSkill::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn frontmatter_error_dominant_is_boundary() {
        let comp = FrontmatterError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }
}
