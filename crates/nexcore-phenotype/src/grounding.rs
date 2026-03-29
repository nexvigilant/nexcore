//! # GroundsTo implementations for nexcore-phenotype types
//!
//! Connects adversarial test fixture generator types to the Lex Primitiva type system.
//!
//! ## Biological Analogy
//!
//! In biology, a phenotype is the observable expression of a genotype.
//! This crate mutates schemas (genotype) to produce observable drift (phenotype).
//!
//! ## Key Primitive Mapping
//!
//! - Mutation dispatch: partial (Boundary) -- conditional mutation selection
//! - Value generation: mu (Mapping) -- schema -> mutated value
//! - Type swap: kappa (Comparison) -- expected vs actual drift comparison

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{Mutation, Phenotype};

// ---------------------------------------------------------------------------
// Mutation types -- Sigma (Sum) dominant
// ---------------------------------------------------------------------------

/// Mutation: T2-P (Sigma + partial), dominant Sigma
///
/// Seven-variant enum classifying mutation strategies.
/// Sum-dominant: the type IS a categorical alternation of mutation types.
/// Boundary is secondary (each mutation tests a different boundary).
impl GroundsTo for Mutation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- mutation type alternation
            LexPrimitiva::Boundary, // partial -- boundary violation type
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Result types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// Phenotype: T2-C (mu + sigma + kappa + partial), dominant mu
///
/// A mutated JSON value with metadata about what was changed.
/// Mapping-dominant: it maps a schema + mutation to a mutated value.
/// Sequence is secondary (mutations_applied is ordered).
/// Comparison is tertiary (expected_drifts for verification).
/// Boundary is quaternary (each mutation violates a boundary).
impl GroundsTo for Phenotype {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- schema + mutation -> value
            LexPrimitiva::Sequence,   // sigma -- ordered mutations
            LexPrimitiva::Comparison, // kappa -- drift verification
            LexPrimitiva::Boundary,   // partial -- boundary violations
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
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

    #[test]
    fn mutation_is_t2p() {
        assert_eq!(Mutation::tier(), Tier::T2Primitive);
        assert_eq!(Mutation::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn phenotype_is_t2c() {
        assert_eq!(Phenotype::tier(), Tier::T2Composite);
        assert_eq!(Phenotype::dominant_primitive(), Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn phenotype_contains_boundary() {
        let comp = Phenotype::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            Mutation::primitive_composition(),
            Phenotype::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
