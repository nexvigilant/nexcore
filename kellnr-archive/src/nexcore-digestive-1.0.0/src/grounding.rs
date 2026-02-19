//! # GroundsTo implementations for nexcore-digestive types
//!
//! Connects the digestive system model to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `Quality` -- Sum (Sigma) dominates: 4-variant quality classification
//! - `DataKind` -- Sum (Sigma) dominates: 6-variant data type classification
//! - `Fragment` -- Sequence (sigma) dominates: ordered piece of broken-down input
//! - `Taste` -- Comparison (kappa) dominates: quality assessment of input
//! - `Nutrients` -- Mapping (mu) dominates: maps raw data to structured components
//! - `Absorbed` -- Boundary (partial) dominates: triage at absorption boundary
//! - `Metabolized` -- Causality (arrow) dominates: transformation from raw to processed
//! - `DigestiveError` -- Boundary (partial) dominates: error = boundary violation

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::claude_code::{
    ContextMode, DigestiveHealth, EnzymeSubstitution, EnzymeType, Microbiome, SkillArguments,
    SkillExecution, SkillFrontmatter, SkillLoad, SkillResult, SkillTrigger, Sphincter,
};
use crate::{Absorbed, DataKind, DigestiveError, Fragment, Metabolized, Nutrients, Quality, Taste};

// ---------------------------------------------------------------------------
// Quality -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------

/// Quality: T2-P (Sigma + kappa), dominant Sigma
///
/// A 4-variant enum classifying input data quality (Poor, Empty, Rich, Normal).
/// Sum-dominant: exhaustive classification of quality states.
/// Comparison secondary: quality levels imply ordering.
impl GroundsTo for Quality {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- 4-variant quality classification
            LexPrimitiva::Comparison, // kappa -- quality implies ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// DataKind -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------

/// DataKind: T2-P (Sigma + mu), dominant Sigma
///
/// A 6-variant enum classifying data types for processing dispatch.
/// Sum-dominant: type classification drives the digestion pathway.
/// Mapping secondary: each kind maps to a processing strategy.
impl GroundsTo for DataKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,     // Sigma -- 6-variant type classification
            LexPrimitiva::Mapping, // mu -- kind -> processing strategy
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Fragment -- Sequence dominant (T2-P)
// ---------------------------------------------------------------------------

/// Fragment: T2-P (sigma + partial), dominant sigma
///
/// An ordered piece of broken-down input data, produced by Mouth::chew.
/// Sequence-dominant: fragments have positional order from the source.
/// Boundary secondary: each fragment has defined start/end limits.
impl GroundsTo for Fragment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered position in source
            LexPrimitiva::Boundary, // partial -- fragment has defined limits
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Taste -- Comparison dominant (T2-P)
// ---------------------------------------------------------------------------

/// Taste: T2-P (kappa + mu + N), dominant kappa
///
/// Quick assessment result from Mouth::taste: quality + kind + size.
/// Comparison-dominant: the purpose is to evaluate/compare input quality.
/// Mapping secondary: transforms raw input to quality assessment.
/// Quantity secondary: measures input size.
impl GroundsTo for Taste {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- quality evaluation
            LexPrimitiva::Mapping,    // mu -- input -> assessment
            LexPrimitiva::Quantity,   // N -- size measurement
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.70)
    }
}

// ---------------------------------------------------------------------------
// Nutrients -- Mapping dominant (T2-C)
// ---------------------------------------------------------------------------

/// Nutrients: T2-C (mu + times + sigma + N), dominant mu
///
/// Structured decomposition of digested input: keys, numbers, values, metadata.
/// Mapping-dominant: the entire purpose is transforming raw data into categorized
/// components (proteins=keys, carbs=numbers, fats=values, vitamins=metadata).
/// Product for composite struct. Sequence for ordered extraction. Quantity for counts.
impl GroundsTo for Nutrients {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- raw data -> categorized components
            LexPrimitiva::Product,  // times -- composite struct of 4 nutrient fields
            LexPrimitiva::Sequence, // sigma -- ordered extraction from source
            LexPrimitiva::Quantity, // N -- counts of extracted items
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.65)
    }
}

// ---------------------------------------------------------------------------
// Absorbed -- Boundary dominant (T2-C)
// ---------------------------------------------------------------------------

/// Absorbed: T2-C (partial + arrow + sigma + varsigma), dominant partial
///
/// Triage result from SmallIntestine::absorb: immediate, stored, waste.
/// Boundary-dominant: the absorption barrier separates useful from waste.
/// Causality for routing decisions (immediate vs stored vs waste).
/// Sequence for ordered processing. State for tracking absorption progress.
impl GroundsTo for Absorbed {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // partial -- absorption barrier
            LexPrimitiva::Causality, // arrow -- routes to immediate/stored/waste
            LexPrimitiva::Sequence,  // sigma -- ordered processing
            LexPrimitiva::State,     // varsigma -- tracks absorption progress
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.60)
    }
}

// ---------------------------------------------------------------------------
// Metabolized -- Causality dominant (T2-P)
// ---------------------------------------------------------------------------

/// Metabolized: T2-P (arrow + N + pi), dominant arrow
///
/// Result of liver processing: original input transformed to processed output
/// with extracted energy value. Causality-dominant: the entire operation is
/// a cause-to-effect transformation (raw → processed).
/// Quantity for energy measurement. Persistence for storage.
impl GroundsTo for Metabolized {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,   // arrow -- raw -> processed transformation
            LexPrimitiva::Quantity,    // N -- energy value
            LexPrimitiva::Persistence, // pi -- stored in liver
        ])
        .with_dominant(LexPrimitiva::Causality, 0.75)
    }
}

// ---------------------------------------------------------------------------
// DigestiveError -- Boundary dominant (T2-P)
// ---------------------------------------------------------------------------

/// DigestiveError: T2-P (partial + Sigma), dominant partial
///
/// Error type for digestion failures: empty input, malformed data,
/// capacity exceeded. Boundary-dominant: errors are boundary violations.
impl GroundsTo for DigestiveError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error = boundary violation
            LexPrimitiva::Sum,      // Sigma -- 3-variant error enum
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// SkillTrigger -- Comparison dominant (T2-C)
// ---------------------------------------------------------------------------

/// SkillTrigger: T2-C (kappa + partial), dominant kappa
///
/// The Mouth: detects skill invocations via pattern matching.
/// Comparison-dominant: the entire purpose is matching patterns against input.
/// Boundary secondary: trigger pattern defines the invocation boundary.
impl GroundsTo for SkillTrigger {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- pattern matching/comparison
            LexPrimitiva::Boundary,   // partial -- trigger boundary
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ---------------------------------------------------------------------------
// SkillLoad -- Sequence dominant (T2-C)
// ---------------------------------------------------------------------------

/// SkillLoad: T2-C (sigma + pi), dominant sigma
///
/// The Esophagus: loads and transports skill files.
/// Sequence-dominant: ordered peristaltic transport from detection to parsing.
/// Persistence secondary: records load metadata.
impl GroundsTo for SkillLoad {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,    // sigma -- ordered transport
            LexPrimitiva::Persistence, // pi -- load record
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ---------------------------------------------------------------------------
// ContextMode -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------

/// ContextMode: T2-P (Sigma + rho), dominant Sigma
///
/// Binary choice: Fork (separate context) vs Inherit (shared state).
/// Sum-dominant: 2-variant enum classification.
/// Recursion secondary: Inherit mode creates recursive context references.
impl GroundsTo for ContextMode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Sigma -- binary choice
            LexPrimitiva::Recursion, // rho -- Inherit is recursive reference
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// SkillFrontmatter -- Mapping dominant (T2-C)
// ---------------------------------------------------------------------------

/// SkillFrontmatter: T2-C (mu + pi + partial + Sigma), dominant mu
///
/// The Stomach: parses SKILL.md frontmatter into structured config.
/// Mapping-dominant: transforms raw markdown to typed configuration.
/// Persistence for config storage, Boundary for gate controls, Sum for context mode enum.
impl GroundsTo for SkillFrontmatter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,     // mu -- raw text -> config
            LexPrimitiva::Persistence, // pi -- config storage
            LexPrimitiva::Boundary,    // partial -- sphincter gates
            LexPrimitiva::Sum,         // Sigma -- contains ContextMode enum
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.65)
    }
}

// ---------------------------------------------------------------------------
// Sphincter -- Boundary dominant (T2-P)
// ---------------------------------------------------------------------------

/// Sphincter: T2-P (Sigma + partial), dominant partial
///
/// Gate control: Open vs Closed.
/// Boundary-dominant: sphincters are gate/barrier primitives.
/// Sum secondary: 2-variant enum (Open/Closed).
impl GroundsTo for Sphincter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- gate/barrier
            LexPrimitiva::Sum,      // Sigma -- Open/Closed binary
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// SkillArguments -- Sequence dominant (T2-C)
// ---------------------------------------------------------------------------

/// SkillArguments: T2-C (sigma + mu), dominant sigma
///
/// The Chyme: broken-down user input flowing through the pipeline.
/// Sequence-dominant: ordered flow of arguments through stages.
/// Mapping secondary: raw_args -> parsed tokens.
impl GroundsTo for SkillArguments {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered flow
            LexPrimitiva::Mapping,  // mu -- raw -> parsed
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ---------------------------------------------------------------------------
// EnzymeType -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------

/// EnzymeType: T2-P (Sigma + mu), dominant Sigma
///
/// 3-variant enzyme classification: Amylase, Pepsin, Lipase.
/// Sum-dominant: type classification drives substitution strategy.
/// Mapping secondary: each enzyme type maps to a substitution mechanism.
impl GroundsTo for EnzymeType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,     // Sigma -- 3-variant classification
            LexPrimitiva::Mapping, // mu -- type -> substitution strategy
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// EnzymeSubstitution -- Mapping dominant (T2-C)
// ---------------------------------------------------------------------------

/// EnzymeSubstitution: T2-C (mu + arrow), dominant mu
///
/// Pattern-to-replacement transformation.
/// Mapping-dominant: pattern -> replacement is a mapping operation.
/// Causality secondary: substitution is a cause-to-effect transformation.
impl GroundsTo for EnzymeSubstitution {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- pattern -> replacement
            LexPrimitiva::Causality, // arrow -- cause-to-effect transform
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Microbiome -- Sequence dominant (T2-C)
// ---------------------------------------------------------------------------

/// Microbiome: T2-C (sigma + exists), dominant sigma
///
/// Collection of shell commands producing external context.
/// Sequence-dominant: ordered collection of command invocations.
/// Existence secondary: tracks presence/absence of symbiotic processes.
impl GroundsTo for Microbiome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // sigma -- ordered commands
            LexPrimitiva::Existence, // exists -- presence/absence
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

// ---------------------------------------------------------------------------
// SkillResult -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------

/// SkillResult: T2-P (Sigma + partial), dominant Sigma
///
/// Binary outcome: Success vs Failure (forward failure).
/// Sum-dominant: 2-variant result classification.
/// Boundary secondary: separates success/failure boundary.
impl GroundsTo for SkillResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- binary outcome
            LexPrimitiva::Boundary, // partial -- success/failure boundary
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// SkillExecution -- Sequence dominant (T2-C)
// ---------------------------------------------------------------------------

/// SkillExecution: T2-C (sigma + N + mu + Sigma), dominant sigma
///
/// The Small Intestine: WHERE 90% OF VALUE IS EXTRACTED.
/// Sequence-dominant: ordered execution pipeline stage.
/// Quantity for tokens_consumed and value_extracted metrics.
/// Mapping for arguments -> result transformation. Sum for SkillResult enum.
impl GroundsTo for SkillExecution {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- execution pipeline
            LexPrimitiva::Quantity, // N -- tokens + value metrics
            LexPrimitiva::Mapping,  // mu -- arguments -> result
            LexPrimitiva::Sum,      // Sigma -- contains SkillResult enum
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.65)
    }
}

// ---------------------------------------------------------------------------
// DigestiveHealth -- State dominant (T2-C)
// ---------------------------------------------------------------------------

/// DigestiveHealth: T2-C (varsigma + kappa + partial + N), dominant varsigma
///
/// Holistic diagnostic state of the skill pipeline.
/// State-dominant: captures current health status of the entire system.
/// Comparison for health checks against baselines.
/// Boundary for sphincter/flow correctness checks. Quantity for skill_count metric.
impl GroundsTo for DigestiveHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- system health state
            LexPrimitiva::Comparison, // kappa -- baseline comparisons
            LexPrimitiva::Boundary,   // partial -- gate correctness
            LexPrimitiva::Quantity,   // N -- skill_count measurement
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn quality_grounds_to_sum() {
        let comp = Quality::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn quality_is_t2_primitive() {
        assert_eq!(Quality::tier(), Tier::T2Primitive);
    }

    #[test]
    fn data_kind_grounds_to_sum() {
        let comp = DataKind::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn fragment_grounds_to_sequence() {
        let comp = Fragment::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn taste_grounds_to_comparison() {
        let comp = Taste::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn nutrients_grounds_to_mapping() {
        let comp = Nutrients::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn nutrients_is_t2_composite() {
        assert_eq!(Nutrients::tier(), Tier::T2Composite);
    }

    #[test]
    fn absorbed_grounds_to_boundary() {
        let comp = Absorbed::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn absorbed_is_t2_composite() {
        assert_eq!(Absorbed::tier(), Tier::T2Composite);
    }

    #[test]
    fn metabolized_grounds_to_causality() {
        let comp = Metabolized::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn digestive_error_grounds_to_boundary() {
        let comp = DigestiveError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn all_types_have_dominant() {
        assert!(Quality::dominant_primitive().is_some());
        assert!(DataKind::dominant_primitive().is_some());
        assert!(Fragment::dominant_primitive().is_some());
        assert!(Taste::dominant_primitive().is_some());
        assert!(Nutrients::dominant_primitive().is_some());
        assert!(Absorbed::dominant_primitive().is_some());
        assert!(Metabolized::dominant_primitive().is_some());
        assert!(DigestiveError::dominant_primitive().is_some());
    }

    #[test]
    fn dominant_primitives_are_mostly_unique() {
        // Quality and DataKind share Sum (both are classifications)
        // DigestiveError shares Boundary with Absorbed
        // But the main processing types should be distinct
        let processing_dominants = vec![
            Fragment::dominant_primitive(),    // Sequence
            Taste::dominant_primitive(),       // Comparison
            Nutrients::dominant_primitive(),   // Mapping
            Metabolized::dominant_primitive(), // Causality
        ];

        let mut seen = std::collections::HashSet::new();
        for d in &processing_dominants {
            assert!(
                seen.insert(d),
                "Duplicate dominant in processing types: {d:?}"
            );
        }
    }

    #[test]
    fn tier_distribution() {
        let tiers = vec![
            Quality::tier(),
            DataKind::tier(),
            Fragment::tier(),
            Taste::tier(),
            Nutrients::tier(),
            Absorbed::tier(),
            Metabolized::tier(),
            DigestiveError::tier(),
        ];

        let t2p = tiers.iter().filter(|t| **t == Tier::T2Primitive).count();
        let t2c = tiers.iter().filter(|t| **t == Tier::T2Composite).count();

        assert_eq!(t2p, 6, "Expected 6 T2-P types");
        assert_eq!(t2c, 2, "Expected 2 T2-C types (Nutrients, Absorbed)");
    }

    // --- SkillTrigger tests ---

    #[test]
    fn skill_trigger_grounds_to_comparison() {
        let comp = SkillTrigger::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn skill_trigger_is_t2_primitive() {
        assert_eq!(SkillTrigger::tier(), Tier::T2Primitive);
    }

    // --- SkillLoad tests ---

    #[test]
    fn skill_load_grounds_to_sequence() {
        let comp = SkillLoad::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn skill_load_is_t2_primitive() {
        assert_eq!(SkillLoad::tier(), Tier::T2Primitive);
    }

    // --- ContextMode tests ---

    #[test]
    fn context_mode_grounds_to_sum() {
        let comp = ContextMode::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn context_mode_is_t2_primitive() {
        assert_eq!(ContextMode::tier(), Tier::T2Primitive);
    }

    // --- SkillFrontmatter tests ---

    #[test]
    fn skill_frontmatter_grounds_to_mapping() {
        let comp = SkillFrontmatter::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn skill_frontmatter_is_t2_composite() {
        assert_eq!(SkillFrontmatter::tier(), Tier::T2Composite);
    }

    // --- Sphincter tests ---

    #[test]
    fn sphincter_grounds_to_boundary() {
        let comp = Sphincter::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn sphincter_is_t2_primitive() {
        assert_eq!(Sphincter::tier(), Tier::T2Primitive);
    }

    // --- SkillArguments tests ---

    #[test]
    fn skill_arguments_grounds_to_sequence() {
        let comp = SkillArguments::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn skill_arguments_is_t2_primitive() {
        assert_eq!(SkillArguments::tier(), Tier::T2Primitive);
    }

    // --- EnzymeType tests ---

    #[test]
    fn enzyme_type_grounds_to_sum() {
        let comp = EnzymeType::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn enzyme_type_is_t2_primitive() {
        assert_eq!(EnzymeType::tier(), Tier::T2Primitive);
    }

    // --- EnzymeSubstitution tests ---

    #[test]
    fn enzyme_substitution_grounds_to_mapping() {
        let comp = EnzymeSubstitution::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn enzyme_substitution_is_t2_primitive() {
        assert_eq!(EnzymeSubstitution::tier(), Tier::T2Primitive);
    }

    // --- Microbiome tests ---

    #[test]
    fn microbiome_grounds_to_sequence() {
        let comp = Microbiome::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn microbiome_is_t2_primitive() {
        assert_eq!(Microbiome::tier(), Tier::T2Primitive);
    }

    // --- SkillResult tests ---

    #[test]
    fn skill_result_grounds_to_sum() {
        let comp = SkillResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn skill_result_is_t2_primitive() {
        assert_eq!(SkillResult::tier(), Tier::T2Primitive);
    }

    // --- SkillExecution tests ---

    #[test]
    fn skill_execution_grounds_to_sequence() {
        let comp = SkillExecution::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn skill_execution_is_t2_composite() {
        assert_eq!(SkillExecution::tier(), Tier::T2Composite);
    }

    // --- DigestiveHealth tests ---

    #[test]
    fn digestive_health_grounds_to_state() {
        let comp = DigestiveHealth::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn digestive_health_is_t2_composite() {
        assert_eq!(DigestiveHealth::tier(), Tier::T2Composite);
    }

    // --- Claude Code types all have dominants ---

    #[test]
    fn claude_code_types_all_have_dominant() {
        assert!(SkillTrigger::dominant_primitive().is_some());
        assert!(SkillLoad::dominant_primitive().is_some());
        assert!(ContextMode::dominant_primitive().is_some());
        assert!(SkillFrontmatter::dominant_primitive().is_some());
        assert!(Sphincter::dominant_primitive().is_some());
        assert!(SkillArguments::dominant_primitive().is_some());
        assert!(EnzymeType::dominant_primitive().is_some());
        assert!(EnzymeSubstitution::dominant_primitive().is_some());
        assert!(Microbiome::dominant_primitive().is_some());
        assert!(SkillResult::dominant_primitive().is_some());
        assert!(SkillExecution::dominant_primitive().is_some());
        assert!(DigestiveHealth::dominant_primitive().is_some());
    }

    // --- Dominant primitive uniqueness across pipeline stages ---

    #[test]
    fn claude_code_pipeline_stages_unique_dominants() {
        // The 5 main pipeline stages should have distinct dominants
        let stage_dominants = vec![
            SkillTrigger::dominant_primitive(), // kappa -- comparison (Mouth)
            SkillLoad::dominant_primitive(),    // sigma -- sequence (Esophagus)
            SkillFrontmatter::dominant_primitive(), // mu -- mapping (Stomach)
            SkillExecution::dominant_primitive(), // sigma -- sequence (Small Intestine)
            DigestiveHealth::dominant_primitive(), // varsigma -- state (Diagnostics)
        ];

        // SkillLoad and SkillExecution both use sigma (both are sequence operations)
        // but the other 3 should be unique
        assert_eq!(
            SkillTrigger::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(
            SkillFrontmatter::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(
            DigestiveHealth::dominant_primitive(),
            Some(LexPrimitiva::State)
        );

        // SkillLoad and SkillExecution share Sequence (both transport/flow)
        assert_eq!(
            SkillLoad::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            SkillExecution::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    // --- Tier distribution including claude_code types ---

    #[test]
    fn tier_distribution_with_claude_code() {
        let all_tiers = vec![
            // Original 8
            Quality::tier(),
            DataKind::tier(),
            Fragment::tier(),
            Taste::tier(),
            Nutrients::tier(),
            Absorbed::tier(),
            Metabolized::tier(),
            DigestiveError::tier(),
            // Claude Code 12
            SkillTrigger::tier(),
            SkillLoad::tier(),
            ContextMode::tier(),
            SkillFrontmatter::tier(),
            Sphincter::tier(),
            SkillArguments::tier(),
            EnzymeType::tier(),
            EnzymeSubstitution::tier(),
            Microbiome::tier(),
            SkillResult::tier(),
            SkillExecution::tier(),
            DigestiveHealth::tier(),
        ];

        let t2p = all_tiers
            .iter()
            .filter(|t| **t == Tier::T2Primitive)
            .count();
        let t2c = all_tiers
            .iter()
            .filter(|t| **t == Tier::T2Composite)
            .count();

        // Original 8: 6 T2-P (Quality, DataKind, Fragment, Taste, Metabolized, DigestiveError)
        //             + 2 T2-C (Nutrients, Absorbed)
        // New 12:     9 T2-P (SkillTrigger, SkillLoad, ContextMode, Sphincter, SkillArguments,
        //                      EnzymeType, EnzymeSubstitution, Microbiome, SkillResult)
        //             + 3 T2-C (SkillFrontmatter, SkillExecution, DigestiveHealth)
        // Total:      15 T2-P + 5 T2-C = 20 types
        assert_eq!(t2p, 15, "Expected 15 T2-P types");
        assert_eq!(t2c, 5, "Expected 5 T2-C types");
    }
}
