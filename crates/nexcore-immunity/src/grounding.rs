// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # GroundsTo implementations for nexcore-immunity types
//!
//! Connects the innate immune layer to the Lex Primitiva type system.
//!
//! ## Innate vs. Adaptive
//!
//! Where `nexcore-antibodies` (adaptive immunity) focuses on learned, specific
//! recognition (κ-dominant), `nexcore-immunity` (innate immunity) focuses on
//! broad pattern-based detection -- sensing threats via compiled regex patterns
//! and responding with predetermined strategies. The dominant primitive here
//! is **Comparison (κ)** for the scanner and detection subsystem, and
//! **Causality (->)** for the response subsystem.
//!
//! ## Primitive Coverage
//!
//! | Symbol | Primitive | Role in Immunity |
//! |--------|-----------|------------------|
//! | Sigma  | Sum       | Enum classification (ThreatType, Severity, ResponseStrategy) |
//! | kappa  | Comparison | Pattern matching, severity ordering |
//! | mu     | Mapping   | Threat -> response transformation |
//! | pi     | Persistence | Registry storage, application counts |
//! | sigma  | Sequence  | Ordered scanning, pattern lists |
//! | ->     | Causality | Detection causes response |
//! | partial| Boundary  | Error boundaries, severity thresholds |
//! | exists | Existence | Threat existence validation |
//! | N      | Quantity  | Confidence scores, counters |
//! | lambda | Location  | Source file location tracking |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::error::ImmunityError;
use crate::scanner::ImmunityScanner;
use crate::types::{
    Antibody, AntibodyRegistry, CodePattern, Detection, ExampleCase, Examples, Response,
    ResponseStrategy, ScanMetrics, ScanResult, ThreatLevel, ThreatMatch, ThreatType, Validation,
};

// ---------------------------------------------------------------------------
// Classification types -- Sigma dominant
// ---------------------------------------------------------------------------

/// ThreatType: T1 (Sigma), dominant Sigma
///
/// Binary sum type: PAMP (external) | DAMP (internal).
/// Pure sum -- the type IS a two-variant alternation classifying threat origin.
impl GroundsTo for ThreatType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Sigma -- two-variant enum (Pamp | Damp)
        ])
        .with_dominant(LexPrimitiva::Sum, 0.95)
    }
}

/// ThreatLevel: T2-P (kappa . Sigma), dominant kappa
///
/// Ordinal threat classification: Low < Medium < High < Critical.
/// Comparison-dominant: the purpose is ordered threat comparison.
/// Derives PartialOrd/Ord for direct comparison.
impl GroundsTo for ThreatLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- ordinal ordering between levels
            LexPrimitiva::Sum,        // Sigma -- four-variant enum
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// ResponseStrategy: T2-P (-> . Sigma), dominant ->
///
/// Thirteen-variant enum prescribing the causal action to take.
/// Causality-dominant: each variant IS a cause -> effect prescription
/// (Block -> halt, Fix -> transform, Warn -> alert).
impl GroundsTo for ResponseStrategy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // -> -- each variant prescribes an effect
            LexPrimitiva::Sum,       // Sigma -- thirteen-variant alternation
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Detection types -- kappa dominant
// ---------------------------------------------------------------------------

/// CodePattern: T2-C (kappa . sigma . lambda), dominant kappa
///
/// A regex pattern applied to source code at a specific language/location.
/// Comparison-dominant: the regex IS the comparison operation.
impl GroundsTo for CodePattern {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- regex pattern matching
            LexPrimitiva::Sequence,   // sigma -- string pattern is sequential
            LexPrimitiva::Location,   // lambda -- language/file context
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// Detection: T2-C (kappa . sigma . mu . partial), dominant kappa
///
/// Configuration bundle for threat detection: code patterns, error patterns,
/// file contexts, exceptions. All fields serve the matching operation.
/// Comparison-dominant: detection IS pattern comparison.
impl GroundsTo for Detection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- all patterns are comparisons
            LexPrimitiva::Sequence,   // sigma -- ordered pattern lists
            LexPrimitiva::Mapping,    // mu -- pattern -> match function
            LexPrimitiva::Boundary,   // partial -- exceptions define exclusion boundaries
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Response types -- -> dominant
// ---------------------------------------------------------------------------

/// Response: T2-C (-> . mu . Sigma . sigma), dominant ->
///
/// Response configuration: strategy + template + alternatives.
/// Causality-dominant: the response IS the causal action taken upon detection.
impl GroundsTo for Response {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // -> -- response causes system change
            LexPrimitiva::Mapping,   // mu -- template transformation
            LexPrimitiva::Sum,       // Sigma -- strategy variant selection
            LexPrimitiva::Sequence,  // sigma -- ordered alternatives list
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Measurement types -- N dominant
// ---------------------------------------------------------------------------

/// ScanMetrics: T2-P (N . Sigma), dominant N
///
/// Counters: total_scanned, threats_detected, auto_fixed, false_positives.
/// Quantity-dominant: every field IS a numeric count.
impl GroundsTo for ScanMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- four numeric counters
            LexPrimitiva::Sum,      // Sigma -- categorized counts
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Evidence types -- exists dominant
// ---------------------------------------------------------------------------

/// ThreatMatch: T2-C (exists . kappa . lambda . -> . N), dominant exists
///
/// A confirmed threat detection with location, severity, and response.
/// Existence-dominant: the match IS proof that a threat exists at a location.
impl GroundsTo for ThreatMatch {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // exists -- confirmed threat presence
            LexPrimitiva::Comparison, // kappa -- severity comparison
            LexPrimitiva::Location,   // lambda -- file location (line number)
            LexPrimitiva::Causality,  // -> -- prescribed response action
            LexPrimitiva::Quantity,   // N -- confidence score
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Result types -- sigma dominant
// ---------------------------------------------------------------------------

/// ScanResult: T2-C (sigma . kappa . N . exists), dominant sigma
///
/// Ordered collection of threat matches with metrics.
/// Sequence-dominant: the result IS an ordered list of findings.
impl GroundsTo for ScanResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- ordered threat list
            LexPrimitiva::Comparison, // kappa -- clean/dirty determination
            LexPrimitiva::Quantity,   // N -- scan metrics
            LexPrimitiva::Existence,  // exists -- clean flag (threats exist?)
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Validation / Example types -- kappa dominant
// ---------------------------------------------------------------------------

/// Validation: T1 (kappa), dominant kappa
///
/// Test code for validating an antibody's detection accuracy.
/// Pure comparison: validation IS the act of comparing expected vs actual.
impl GroundsTo for Validation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- test validates correctness
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// Examples: T2-P (kappa . sigma), dominant kappa
///
/// Pass/fail example cases for an antibody.
/// Comparison-dominant: examples compare expected outcome (pass vs fail).
impl GroundsTo for Examples {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- pass/fail classification
            LexPrimitiva::Sequence,   // sigma -- ordered lists of cases
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// ExampleCase: T2-C (kappa . lambda . mu . ->), dominant kappa
///
/// A single pass/fail case with file, reason, and fix suggestion.
/// Comparison-dominant: the case IS a comparison against expected behavior.
impl GroundsTo for ExampleCase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- expected vs actual
            LexPrimitiva::Location,   // lambda -- file path context
            LexPrimitiva::Mapping,    // mu -- should_be transformation
            LexPrimitiva::Causality,  // -> -- reason explains cause
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ---------------------------------------------------------------------------
// T3 domain types
// ---------------------------------------------------------------------------

/// Antibody: T3 (mu . kappa . -> . pi . Sigma . partial . N), dominant mu
///
/// Complete innate immune recognition unit: maps threat pattern to response.
/// Unlike nexcore-antibodies' adaptive Antibody (kappa-dominant structural
/// matching), the innate Antibody is Mapping-dominant because it serves as
/// a predetermined pattern -> action transformer without learned specificity.
impl GroundsTo for Antibody {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,     // mu -- threat pattern -> response action
            LexPrimitiva::Comparison,  // kappa -- regex pattern matching
            LexPrimitiva::Causality,   // -> -- detection causes response
            LexPrimitiva::Persistence, // pi -- application count, learned_from
            LexPrimitiva::Sum,         // Sigma -- threat_type + severity enums
            LexPrimitiva::Boundary,    // partial -- confidence threshold
            LexPrimitiva::Quantity,    // N -- confidence score, counters
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// AntibodyRegistry: T2-C (pi . sigma . mu . kappa . exists), dominant pi
///
/// Persistent, indexed collection of all innate antibody definitions.
/// Persistence-dominant: the registry IS the stored immune memory loaded
/// from YAML on disk.
impl GroundsTo for AntibodyRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- loaded from persistent YAML
            LexPrimitiva::Sequence,    // sigma -- ordered antibody list
            LexPrimitiva::Mapping,     // mu -- id -> antibody lookup
            LexPrimitiva::Comparison,  // kappa -- by_type / by_severity filtering
            LexPrimitiva::Existence,   // exists -- get() returns Option
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

/// ImmunityScanner: T3 (kappa . sigma . mu . pi . -> . exists), dominant kappa
///
/// The compiled scanning engine: precompiled regex patterns applied
/// sequentially to content, mapping matches to threat responses.
/// Comparison-dominant: the scanner's core operation IS pattern comparison.
impl GroundsTo for ImmunityScanner {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,  // kappa -- regex pattern matching
            LexPrimitiva::Sequence,    // sigma -- line-by-line iteration
            LexPrimitiva::Mapping,     // mu -- match -> ThreatMatch
            LexPrimitiva::Persistence, // pi -- compiled patterns held in memory
            LexPrimitiva::Causality,   // -> -- detection -> response
            LexPrimitiva::Existence,   // exists -- threat existence check
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Error types -- partial dominant
// ---------------------------------------------------------------------------

/// ImmunityError: T2-C (partial . Sigma . -> . kappa), dominant partial
///
/// Error variants representing boundary violations in the innate immune system:
/// load failures, parse errors, invalid patterns, not-found, IO, response failures.
/// Boundary-dominant: errors ARE boundary conditions between valid and invalid states.
impl GroundsTo for ImmunityError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- violated constraints
            LexPrimitiva::Sum,        // Sigma -- six-variant error alternation
            LexPrimitiva::Causality,  // -> -- error causes failure propagation
            LexPrimitiva::Comparison, // kappa -- pattern validation
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ---- T1 types ----

    #[test]
    fn threat_type_is_sum_dominant_t1() {
        let comp = ThreatType::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(ThreatType::tier(), Tier::T1Universal);
    }

    #[test]
    fn validation_is_comparison_dominant_t1() {
        let comp = Validation::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert_eq!(Validation::tier(), Tier::T1Universal);
    }

    // ---- T2-P types (2-3 unique primitives) ----

    #[test]
    fn threat_level_is_comparison_dominant_t2p() {
        let comp = ThreatLevel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert_eq!(ThreatLevel::tier(), Tier::T2Primitive);
    }

    #[test]
    fn response_strategy_is_causality_dominant_t2p() {
        let comp = ResponseStrategy::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert_eq!(ResponseStrategy::tier(), Tier::T2Primitive);
    }

    #[test]
    fn scan_metrics_is_quantity_dominant_t2p() {
        let comp = ScanMetrics::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(ScanMetrics::tier(), Tier::T2Primitive);
    }

    #[test]
    fn examples_is_comparison_dominant_t2p() {
        let comp = Examples::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(Examples::tier(), Tier::T2Primitive);
    }

    // ---- T2-C types (4-5 unique primitives) ----

    #[test]
    fn code_pattern_is_comparison_dominant_t2p() {
        let comp = CodePattern::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
    }

    #[test]
    fn detection_is_comparison_dominant_t2c() {
        let comp = Detection::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(Detection::tier(), Tier::T2Composite);
    }

    #[test]
    fn response_is_causality_dominant_t2c() {
        let comp = Response::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert_eq!(Response::tier(), Tier::T2Composite);
    }

    #[test]
    fn threat_match_is_existence_dominant_t2c() {
        let comp = ThreatMatch::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(ThreatMatch::tier(), Tier::T2Composite);
    }

    #[test]
    fn scan_result_is_sequence_dominant_t2c() {
        let comp = ScanResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert_eq!(ScanResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn example_case_is_comparison_dominant_t2c() {
        let comp = ExampleCase::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(ExampleCase::tier(), Tier::T2Composite);
    }

    #[test]
    fn antibody_registry_is_persistence_dominant_t2c() {
        let comp = AntibodyRegistry::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert_eq!(AntibodyRegistry::tier(), Tier::T2Composite);
    }

    #[test]
    fn immunity_error_is_boundary_dominant_t2c() {
        let comp = ImmunityError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert_eq!(ImmunityError::tier(), Tier::T2Composite);
    }

    // ---- T3 domain types (6+ unique primitives) ----

    #[test]
    fn antibody_is_mapping_dominant_t3() {
        let comp = Antibody::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(Antibody::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn immunity_scanner_is_comparison_dominant_t3() {
        let comp = ImmunityScanner::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert_eq!(ImmunityScanner::tier(), Tier::T3DomainSpecific);
    }

    // ---- Cross-cutting invariant checks ----

    #[test]
    fn innate_antibody_differs_from_adaptive() {
        // Innate immunity Antibody is mu-dominant (mapping: pattern -> action)
        // Adaptive immunity Antibody is kappa-dominant (structural matching)
        let comp = Antibody::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        // This intentionally differs from nexcore-antibodies::Antibody (kappa-dominant)
    }

    #[test]
    fn all_types_have_dominants() {
        // Every grounded type must declare a dominant primitive
        assert!(ThreatType::dominant_primitive().is_some());
        assert!(ThreatLevel::dominant_primitive().is_some());
        assert!(ResponseStrategy::dominant_primitive().is_some());
        assert!(CodePattern::dominant_primitive().is_some());
        assert!(Detection::dominant_primitive().is_some());
        assert!(Response::dominant_primitive().is_some());
        assert!(ScanMetrics::dominant_primitive().is_some());
        assert!(ThreatMatch::dominant_primitive().is_some());
        assert!(ScanResult::dominant_primitive().is_some());
        assert!(Validation::dominant_primitive().is_some());
        assert!(Examples::dominant_primitive().is_some());
        assert!(ExampleCase::dominant_primitive().is_some());
        assert!(Antibody::dominant_primitive().is_some());
        assert!(AntibodyRegistry::dominant_primitive().is_some());
        assert!(ImmunityScanner::dominant_primitive().is_some());
        assert!(ImmunityError::dominant_primitive().is_some());
    }

    #[test]
    fn tier_distribution_covers_all_levels() {
        // T1: ThreatType, Validation
        assert_eq!(ThreatType::tier(), Tier::T1Universal);
        assert_eq!(Validation::tier(), Tier::T1Universal);

        // T2-P: ThreatLevel, ResponseStrategy, ScanMetrics, Examples
        assert_eq!(ThreatLevel::tier(), Tier::T2Primitive);
        assert_eq!(ResponseStrategy::tier(), Tier::T2Primitive);
        assert_eq!(ScanMetrics::tier(), Tier::T2Primitive);
        assert_eq!(Examples::tier(), Tier::T2Primitive);

        // T2-C: CodePattern, Detection, Response, ThreatMatch, ScanResult,
        //        ExampleCase, AntibodyRegistry, ImmunityError
        assert_eq!(Detection::tier(), Tier::T2Composite);
        assert_eq!(Response::tier(), Tier::T2Composite);
        assert_eq!(ThreatMatch::tier(), Tier::T2Composite);
        assert_eq!(ScanResult::tier(), Tier::T2Composite);
        assert_eq!(ExampleCase::tier(), Tier::T2Composite);
        assert_eq!(AntibodyRegistry::tier(), Tier::T2Composite);
        assert_eq!(ImmunityError::tier(), Tier::T2Composite);

        // T3: Antibody, ImmunityScanner
        assert_eq!(Antibody::tier(), Tier::T3DomainSpecific);
        assert_eq!(ImmunityScanner::tier(), Tier::T3DomainSpecific);
    }
}
