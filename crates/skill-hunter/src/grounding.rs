//! # GroundsTo implementations for skill-hunter types
//!
//! Gamified skill validator types grounded to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `SkillFrontmatter` -- State (varsigma) dominant as it captures parsed configuration.
//! - `Severity` -- Comparison (kappa) dominant as it classifies issue severity.
//! - `Issue` -- Boundary (partial) dominant as it marks validation boundaries.
//! - `SkillResult` -- Mapping (mu) dominant as it maps skill scan to scored results.
//! - `GameState` -- Sum (Sigma) dominant as it aggregates all scan statistics.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{DiagnosticLevel, GameState, Issue, SkillFrontmatter, SkillResult};

/// SkillFrontmatter: T2-C (varsigma + mu + exists + sigma), dominant varsigma
///
/// Parsed SKILL.md metadata with optional fields. State-dominant because it
/// captures the configuration state of a skill definition.
/// Mapping is secondary (YAML -> struct). Existence is tertiary (all fields are Option).
/// Sequence is quaternary (ordered tags/triggers lists).
impl GroundsTo for SkillFrontmatter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- configuration state
            LexPrimitiva::Mapping,   // mu -- YAML -> struct
            LexPrimitiva::Existence, // exists -- all Optional fields
            LexPrimitiva::Sequence,  // sigma -- ordered tags/triggers
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// DiagnosticLevel: T1-Universal (kappa), dominant kappa
///
/// Three-level severity classification: Critical, Warning, Info.
/// Pure comparison -- discriminates issue importance.
impl GroundsTo for DiagnosticLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- severity level comparison
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// Issue: T2-P (partial + kappa + mu), dominant partial
///
/// A detected issue with severity and fix hint. Boundary-dominant because
/// issues represent violations of expected constraints.
/// Comparison is secondary (severity classification).
/// Mapping is tertiary (issue -> fix hint).
impl GroundsTo for Issue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- constraint violation
            LexPrimitiva::Comparison, // kappa -- severity classification
            LexPrimitiva::Mapping,    // mu -- issue -> fix hint
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// SkillResult: T2-P (mu + N + sigma), dominant mu
///
/// Result of scanning a single skill: name, issues list, computed score.
/// Mapping-dominant because it transforms a skill scan into a scored result.
/// Quantity is secondary (numeric score). Sequence is tertiary (ordered issue list).
impl GroundsTo for SkillResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- scan -> scored result
            LexPrimitiva::Quantity, // N -- numeric score
            LexPrimitiva::Sequence, // sigma -- ordered issues
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// GameState: T2-C (Sigma + N + kappa + mu), dominant Sigma
///
/// Aggregated statistics across all scanned skills. Sum-dominant because it
/// accumulates counts and scores from all individual results.
/// Quantity is secondary (numeric counters). Comparison is tertiary (critical/warning counts).
/// Mapping is quaternary (results -> aggregated state).
impl GroundsTo for GameState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- multi-skill aggregation
            LexPrimitiva::Quantity,   // N -- numeric counters
            LexPrimitiva::Comparison, // kappa -- critical/warning discrimination
            LexPrimitiva::Mapping,    // mu -- results -> aggregate
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
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
    fn severity_is_t1() {
        assert_eq!(DiagnosticLevel::tier(), Tier::T1Universal);
    }

    #[test]
    fn issue_is_t2p() {
        assert_eq!(Issue::tier(), Tier::T2Primitive);
    }

    #[test]
    fn skill_result_is_t2p() {
        assert_eq!(SkillResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn game_state_is_t2c() {
        assert_eq!(GameState::tier(), Tier::T2Composite);
    }

    #[test]
    fn severity_dominant_is_comparison() {
        let comp = DiagnosticLevel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn issue_dominant_is_boundary() {
        let comp = Issue::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn game_state_dominant_is_sum() {
        let comp = GameState::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }
}
