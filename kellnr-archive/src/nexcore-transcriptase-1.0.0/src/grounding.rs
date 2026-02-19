//! # GroundsTo implementations for nexcore-transcriptase types
//!
//! Connects the bidirectional data-to-schema engine to the Lex Primitiva type system.
//!
//! ## Biological Analogy
//!
//! Reverse transcriptase synthesizes DNA from RNA. This engine synthesizes
//! structural knowledge (schema) from observed data (JSON).
//!
//! ## Key Primitive Mapping
//!
//! - Schema inference: mu (Mapping) -- data -> schema
//! - Schema merging: Sigma (Sum) -- combining observations
//! - Violation synthesis: partial (Boundary) -- boundary assertions
//! - Fidelity check: kappa (Comparison) -- round-trip comparison

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    Config, DiagnosticLevel, Engine, Fidelity, Schema, SchemaKind, SchemaViolation, Stats,
    TranscriptaseError, TranscriptionOutput,
};

// ---------------------------------------------------------------------------
// Schema types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// Schema: T2-P (mu + rho + N), dominant mu
///
/// Inferred schema from observed data: name, kind, observation count.
/// Mapping-dominant: it maps data observations to structural knowledge.
/// Recursion is secondary (schemas can be recursive: Record contains Schema).
/// Quantity is tertiary (observation count).
impl GroundsTo for Schema {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- data -> structure mapping
            LexPrimitiva::Recursion, // rho -- recursive schema nesting
            LexPrimitiva::Quantity,  // N -- observation count
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// SchemaKind: T2-P (Sigma + rho), dominant Sigma
///
/// Eight-variant enum: Null, Bool, Int, Float, Str, Array, Record, Mixed.
/// Sum-dominant: the type IS a categorical alternation of data types.
/// Recursion is secondary (Array and Record contain nested schemas).
impl GroundsTo for SchemaKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Sigma -- type variant alternation
            LexPrimitiva::Recursion, // rho -- recursive nesting
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Violation types -- partial (Boundary) dominant
// ---------------------------------------------------------------------------

/// Violation: T2-C (partial + kappa + Sigma + lambda), dominant partial
///
/// A boundary violation assertion.
/// Boundary-dominant: it IS a boundary that should not be crossed.
/// Comparison is secondary (assertion evaluates a condition).
/// Sum is tertiary (severity classification).
/// Location is quaternary (field path).
impl GroundsTo for SchemaViolation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- boundary assertion
            LexPrimitiva::Comparison, // kappa -- assertion evaluation
            LexPrimitiva::Sum,        // Sigma -- severity classification
            LexPrimitiva::Location,   // lambda -- field path
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// DiagnosticLevel: T2-P (Sigma + kappa), dominant Sigma
///
/// Violation severity: Critical, Warning, Info.
impl GroundsTo for DiagnosticLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- severity variant
            LexPrimitiva::Comparison, // kappa -- ordinal comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Fidelity types -- kappa (Comparison) dominant
// ---------------------------------------------------------------------------

/// Fidelity: T2-P (kappa + Sigma), dominant kappa
///
/// Round-trip fidelity result: Exact, Approximate, Failed.
/// Comparison-dominant: it compares original vs round-tripped data.
impl GroundsTo for Fidelity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- round-trip comparison
            LexPrimitiva::Sum,        // Sigma -- result variant
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Engine types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// Config: T2-P (varsigma + partial), dominant varsigma
///
/// Engine configuration: synthesis flags, fidelity check, source name.
impl GroundsTo for Config {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- configuration state
            LexPrimitiva::Boundary, // partial -- config boundaries
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// Stats: T2-P (N + varsigma), dominant N
///
/// Engine statistics: record counts, merge counts, fidelity counts.
impl GroundsTo for Stats {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric counters
            LexPrimitiva::State,    // varsigma -- stats snapshot
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// Engine: T3 (mu + varsigma + sigma + rho + partial + kappa), dominant mu
///
/// The full transcription engine: observes, infers, merges, synthesizes.
/// Mapping-dominant: the engine maps data observations to schemas.
/// State is secondary (mutable engine state with merged schema).
/// Sequence is tertiary (observation processing pipeline).
/// Recursion is quaternary (recursive schema merging).
impl GroundsTo for Engine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- data -> schema mapping
            LexPrimitiva::State,      // varsigma -- engine state
            LexPrimitiva::Sequence,   // sigma -- processing pipeline
            LexPrimitiva::Recursion,  // rho -- recursive merging
            LexPrimitiva::Boundary,   // partial -- violation synthesis
            LexPrimitiva::Comparison, // kappa -- fidelity checking
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// TranscriptionOutput: T2-C (mu + partial + kappa + N), dominant mu
///
/// Complete output from a transcription run: schema, violations, fidelity, stats.
impl GroundsTo for TranscriptionOutput {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- transcription result mapping
            LexPrimitiva::Boundary,   // partial -- violation boundaries
            LexPrimitiva::Comparison, // kappa -- fidelity results
            LexPrimitiva::Quantity,   // N -- statistics
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// TranscriptaseError: T2-P (partial + Sigma), dominant partial
impl GroundsTo for TranscriptaseError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error boundary
            LexPrimitiva::Sum,      // Sigma -- error variant
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
    fn schema_is_t2p() {
        assert_eq!(Schema::tier(), Tier::T2Primitive);
        assert_eq!(Schema::dominant_primitive(), Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn schema_kind_is_t2p() {
        assert_eq!(SchemaKind::tier(), Tier::T2Primitive);
        assert_eq!(SchemaKind::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn schema_violation_is_t2c() {
        assert_eq!(SchemaViolation::tier(), Tier::T2Composite);
        assert_eq!(
            SchemaViolation::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn fidelity_comparison_dominant() {
        assert_eq!(
            Fidelity::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn engine_is_t3() {
        assert_eq!(Engine::tier(), Tier::T3DomainSpecific);
        assert_eq!(Engine::dominant_primitive(), Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn schema_contains_recursion() {
        let comp = Schema::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            Schema::primitive_composition(),
            SchemaKind::primitive_composition(),
            SchemaViolation::primitive_composition(),
            DiagnosticLevel::primitive_composition(),
            Fidelity::primitive_composition(),
            Config::primitive_composition(),
            Stats::primitive_composition(),
            Engine::primitive_composition(),
            TranscriptionOutput::primitive_composition(),
            TranscriptaseError::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
