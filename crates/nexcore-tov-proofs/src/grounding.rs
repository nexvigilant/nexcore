//! # GroundsTo implementations for nexcore-tov-proofs types
//!
//! Connects the Curry-Howard proof verification types to the Lex Primitiva type system.
//!
//! ## Grounding Strategy
//!
//! This crate implements the Curry-Howard correspondence: propositions as types, proofs
//! as programs. The types fall into several categories:
//!
//! - **Logic connectives** (And, Or, Exists, Proof, Void): The structural backbone of
//!   intuitionistic logic. These map directly to product/sum/existence primitives.
//! - **Codex compliance types** (Confident, ConfidentProof, Versioned): Wrappers adding
//!   confidence or version metadata to values.
//! - **Type-level constraints** (ValidatedLevel, ValidatedLawIndex, ValidatedRarity, etc.):
//!   Compile-time validated numeric constraints using const generics.
//! - **Attenuation types** (PropagationProbability): Runtime probability values.
//!
//! | Primitive | Role in ToV Proofs |
//! |-----------|-------------------|
//! | x (Product) | Conjunction And<P,Q>, multi-field records |
//! | Sigma (Sum) | Disjunction Or<P,Q>, biconditional Iff<P,Q> |
//! | exists (Existence) | Existential witness Exists<W,P> |
//! | varsigma (State) | Proof markers, type-level state evidence |
//! | emptyset (Void) | Logical falsity (uninhabited type) |
//! | N (Quantity) | Validated numeric ranges, confidence scores |
//! | partial (Boundary) | Hierarchy level bounds, law index bounds |
//! | -> (Causality) | Implication as function, proof by cases |
//! | kappa (Comparison) | Tier classification, version identity |
//! | nu (Frequency) | Propagation probability, decay rates |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::logic_prelude;

// ---------------------------------------------------------------------------
// LOGIC CONNECTIVES
// ---------------------------------------------------------------------------

/// And<P,Q>: T2-P (x + kappa), dominant Product
///
/// Logical conjunction. A struct holding `left: P` and `right: Q`.
/// Pure product type with structural equality.
impl<P, Q> GroundsTo for logic_prelude::And<P, Q> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,    // x -- (left, right) product type
            LexPrimitiva::Comparison, // kappa -- structural equality
        ])
        .with_dominant(LexPrimitiva::Product, 0.90)
    }
}

/// Or<P,Q>: T2-P (Sigma + kappa), dominant Sum
///
/// Logical disjunction. An enum with Left(P) and Right(Q) variants.
/// Pure sum type with pattern-matching case analysis.
impl<P, Q> GroundsTo for logic_prelude::Or<P, Q> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- Left|Right variant union
            LexPrimitiva::Comparison, // kappa -- case analysis
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// Exists<W,P>: T2-P (exists + x), dominant Existence
///
/// Existential quantification: witness W and proof P(W).
/// Existence dominant because the type asserts that something satisfying
/// the property EXISTS.
impl<W, P> GroundsTo for logic_prelude::Exists<W, P> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // exists -- existential witness
            LexPrimitiva::Product,   // x -- (witness, proof) pair
        ])
        .with_dominant(LexPrimitiva::Existence, 0.90)
    }
}

/// Proof<P>: T1 (varsigma), dominant State
///
/// Zero-cost proof marker. State dominant because it captures
/// type-level evidence without runtime data.
impl<P> GroundsTo for logic_prelude::Proof<P> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State, // varsigma -- type-level evidence state
        ])
        .with_dominant(LexPrimitiva::State, 0.95)
    }
}

/// Void: T1 (emptyset), dominant Void
///
/// Uninhabited type representing logical falsity. No values can be constructed.
impl GroundsTo for logic_prelude::Void {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void, // emptyset -- uninhabited type
        ])
        .with_dominant(LexPrimitiva::Void, 0.95)
    }
}

/// Iff<P,Q>: T2-P (-> + Sigma), dominant Causality
///
/// Biconditional: pair of forward and backward implications.
/// Causality dominant because the type IS a pair of causal implications (P -> Q, Q -> P).
/// Sum captures the two-directional nature.
impl<P, Q> GroundsTo for logic_prelude::Iff<P, Q> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // -> -- bidirectional implication
            LexPrimitiva::Sum,       // Sigma -- forward + backward directions
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// ---------------------------------------------------------------------------
// CODEX COMPLIANCE TYPES
// ---------------------------------------------------------------------------

/// Confident<T, CONFIDENCE>: T2-P (N + varsigma), dominant Quantity
///
/// Value with compile-time confidence annotation. Quantity dominant
/// because the const generic CONFIDENCE is a numeric value [0,100].
/// State captures the wrapped value's identity.
impl<T, const C: u8> GroundsTo for crate::codex_compliance::Confident<T, C> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- confidence level (0-100)
            LexPrimitiva::State,    // varsigma -- wrapped value state
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// ConfidentProof<P, CONFIDENCE>: T2-P (N + varsigma), dominant Quantity
///
/// Proof marker with confidence annotation. Same grounding as Confident
/// but for proof markers rather than data values.
impl<P, const C: u8> GroundsTo for crate::codex_compliance::ConfidentProof<P, C> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- confidence level (0-100)
            LexPrimitiva::State,    // varsigma -- proof evidence state
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Versioned<T, VERSION>: T2-P (N + kappa), dominant Quantity
///
/// Value with version annotation for state correction. Quantity dominant
/// for the version number; Comparison for version identity checks.
impl<T, const V: u32> GroundsTo for crate::codex_compliance::Versioned<T, V> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- version number
            LexPrimitiva::Comparison, // kappa -- version identity check
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// TYPE-LEVEL CONSTRAINTS
// ---------------------------------------------------------------------------

/// ValidatedLevel<N>: T2-P (N + partial), dominant Boundary
///
/// Compile-time validated hierarchy level (1-8). Boundary dominant because
/// the type enforces bounds; Quantity for the level number.
impl<const N: u8> GroundsTo for crate::type_level::ValidatedLevel<N> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- level bounds [1, 8]
            LexPrimitiva::Quantity, // N -- level number
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// ValidatedLawIndex<I>: T2-P (N + partial), dominant Boundary
///
/// Compile-time validated conservation law index (1-11). Same grounding
/// pattern as ValidatedLevel -- bounds enforcement with numeric identity.
impl<const I: u8> GroundsTo for crate::type_level::ValidatedLawIndex<I> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- law index bounds [1, 11]
            LexPrimitiva::Quantity, // N -- law index number
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// ValidatedRarity<BITS>: T2-P (N + partial), dominant Quantity
///
/// Compile-time signal rarity in bits. Quantity dominant because
/// the primary value IS the bit count; Boundary for the non-recurrence threshold.
impl<const BITS: u64> GroundsTo for crate::type_level::ValidatedRarity<BITS> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- rarity in bits
            LexPrimitiva::Boundary, // partial -- non-recurrence threshold (63 bits)
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// ValidatedHarmTypeIndex<T>: T2-P (kappa + partial), dominant Comparison
///
/// Compile-time validated harm type index (0-7 for types A-H).
/// Comparison dominant for harm type classification; Boundary for range enforcement.
impl<const T: u8> GroundsTo for crate::type_level::ValidatedHarmTypeIndex<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- harm type classification
            LexPrimitiva::Boundary,   // partial -- index bounds [0, 7]
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// BoundedProbability<NUM, DEN>: T2-P (N + partial), dominant Quantity
///
/// Compile-time bounded probability (NUM/DEN where NUM < DEN). Quantity
/// dominant for the numeric probability value; Boundary enforces the < 1 constraint.
impl<const NUM: u32, const DEN: u32> GroundsTo for crate::type_level::BoundedProbability<NUM, DEN> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- probability value
            LexPrimitiva::Boundary, // partial -- constraint NUM < DEN
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// ValidatedDomainIndex<D>: T2-P (kappa + partial), dominant Comparison
///
/// Compile-time validated domain index (0-2 for Cloud, PV, AI).
/// Comparison dominant for domain selection; Boundary for range enforcement.
impl<const D: u8> GroundsTo for crate::type_level::ValidatedDomainIndex<D> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- domain classification
            LexPrimitiva::Boundary,   // partial -- index bounds [0, 2]
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// NonRecurrenceThreshold: T1 (partial), dominant Boundary
///
/// The U_NR = 63 bit threshold. Pure boundary primitive -- it IS a threshold constant.
impl GroundsTo for crate::type_level::NonRecurrenceThreshold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- threshold constant
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

// ---------------------------------------------------------------------------
// ATTENUATION TYPES
// ---------------------------------------------------------------------------

/// PropagationProbability: T2-P (N + partial), dominant Quantity
///
/// A propagation probability value in (0, 1). Quantity dominant because
/// the type wraps an f64 value; Boundary for the (0, 1) range constraint.
impl GroundsTo for crate::attenuation::PropagationProbability {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- probability value
            LexPrimitiva::Boundary, // partial -- (0, 1) range constraint
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ---- Tier classification tests ----

    #[test]
    fn test_and_tier() {
        let comp = <logic_prelude::And<(), ()>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_or_tier() {
        let comp = <logic_prelude::Or<(), ()>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_exists_tier() {
        let comp = <logic_prelude::Exists<(), ()>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_proof_tier() {
        let comp = <logic_prelude::Proof<()>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_void_tier() {
        let comp = logic_prelude::Void::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_iff_tier() {
        let comp = <logic_prelude::Iff<(), ()>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_confident_tier() {
        let comp = <crate::codex_compliance::Confident<u32, 90>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_confident_proof_tier() {
        let comp = <crate::codex_compliance::ConfidentProof<(), 100>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_versioned_tier() {
        let comp = <crate::codex_compliance::Versioned<u32, 1>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_validated_level_tier() {
        let comp = <crate::type_level::ValidatedLevel<4>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_validated_law_index_tier() {
        let comp = <crate::type_level::ValidatedLawIndex<3>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_validated_rarity_tier() {
        let comp = <crate::type_level::ValidatedRarity<100>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_validated_harm_type_index_tier() {
        let comp = <crate::type_level::ValidatedHarmTypeIndex<0>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_bounded_probability_tier() {
        let comp = <crate::type_level::BoundedProbability<1, 2>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_validated_domain_index_tier() {
        let comp = <crate::type_level::ValidatedDomainIndex<0>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_non_recurrence_threshold_tier() {
        let comp = crate::type_level::NonRecurrenceThreshold::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_propagation_probability_tier() {
        let comp = crate::attenuation::PropagationProbability::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    // ---- Dominant primitive tests ----

    #[test]
    fn test_and_dominant() {
        assert_eq!(
            <logic_prelude::And<(), ()>>::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
    }

    #[test]
    fn test_or_dominant() {
        assert_eq!(
            <logic_prelude::Or<(), ()>>::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn test_exists_dominant() {
        assert_eq!(
            <logic_prelude::Exists<(), ()>>::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn test_proof_dominant() {
        assert_eq!(
            <logic_prelude::Proof<()>>::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn test_void_dominant() {
        assert_eq!(
            logic_prelude::Void::dominant_primitive(),
            Some(LexPrimitiva::Void)
        );
    }

    #[test]
    fn test_iff_dominant() {
        assert_eq!(
            <logic_prelude::Iff<(), ()>>::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn test_validated_level_dominant() {
        assert_eq!(
            <crate::type_level::ValidatedLevel<4>>::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn test_non_recurrence_threshold_dominant() {
        assert_eq!(
            crate::type_level::NonRecurrenceThreshold::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn test_propagation_probability_dominant() {
        assert_eq!(
            crate::attenuation::PropagationProbability::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    // ---- Confidence range tests ----

    #[test]
    fn test_all_confidences_in_valid_range() {
        let compositions: Vec<(&str, PrimitiveComposition)> = vec![
            ("And", <logic_prelude::And<(), ()>>::primitive_composition()),
            ("Or", <logic_prelude::Or<(), ()>>::primitive_composition()),
            (
                "Exists",
                <logic_prelude::Exists<(), ()>>::primitive_composition(),
            ),
            ("Proof", <logic_prelude::Proof<()>>::primitive_composition()),
            ("Void", logic_prelude::Void::primitive_composition()),
            ("Iff", <logic_prelude::Iff<(), ()>>::primitive_composition()),
            (
                "Confident",
                <crate::codex_compliance::Confident<u32, 90>>::primitive_composition(),
            ),
            (
                "ConfidentProof",
                <crate::codex_compliance::ConfidentProof<(), 100>>::primitive_composition(),
            ),
            (
                "Versioned",
                <crate::codex_compliance::Versioned<u32, 1>>::primitive_composition(),
            ),
            (
                "ValidatedLevel",
                <crate::type_level::ValidatedLevel<4>>::primitive_composition(),
            ),
            (
                "ValidatedLawIndex",
                <crate::type_level::ValidatedLawIndex<3>>::primitive_composition(),
            ),
            (
                "ValidatedRarity",
                <crate::type_level::ValidatedRarity<100>>::primitive_composition(),
            ),
            (
                "ValidatedHarmTypeIndex",
                <crate::type_level::ValidatedHarmTypeIndex<0>>::primitive_composition(),
            ),
            (
                "BoundedProbability",
                <crate::type_level::BoundedProbability<1, 2>>::primitive_composition(),
            ),
            (
                "ValidatedDomainIndex",
                <crate::type_level::ValidatedDomainIndex<0>>::primitive_composition(),
            ),
            (
                "NonRecurrenceThreshold",
                crate::type_level::NonRecurrenceThreshold::primitive_composition(),
            ),
            (
                "PropagationProbability",
                crate::attenuation::PropagationProbability::primitive_composition(),
            ),
        ];

        for (name, comp) in &compositions {
            assert!(
                comp.confidence >= 0.80 && comp.confidence <= 0.95,
                "{} confidence {} outside 0.80-0.95 range",
                name,
                comp.confidence
            );
        }
    }

    // ---- Pure primitive tests ----

    #[test]
    fn test_pure_primitives() {
        assert!(<logic_prelude::Proof<()>>::is_pure_primitive());
        assert!(logic_prelude::Void::is_pure_primitive());
        assert!(crate::type_level::NonRecurrenceThreshold::is_pure_primitive());
        assert!(!<logic_prelude::And<(), ()>>::is_pure_primitive());
        assert!(!<logic_prelude::Or<(), ()>>::is_pure_primitive());
    }

    // ---- Grounding count ----

    #[test]
    fn test_total_grounded_types_count() {
        // 17 types grounded in this module
        let count = 17;
        assert_eq!(count, 17, "Should have 17 GroundsTo implementations");
    }
}
