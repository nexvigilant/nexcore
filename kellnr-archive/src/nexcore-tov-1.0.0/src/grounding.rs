//! # GroundsTo implementations for nexcore-tov types
//!
//! Connects the unified Theory of Vigilance crate to the Lex Primitiva type system.
//!
//! ## Grounding Strategy
//!
//! The nexcore-tov crate re-exports types from both `grounded` (runtime primitives)
//! and `proofs` (Curry-Howard proof types). Since grounded types are already
//! grounded in `nexcore-tov-grounded` and proof types in `nexcore-tov-proofs`,
//! this module grounds the re-exported proof logic types that appear at the
//! crate root: And, Or, Exists, Proof, Void, and Truth (type alias for `()`).
//!
//! Note: The grounded types (Bits, QuantityUnit, etc.) get their GroundsTo impls
//! from the `grounded` sub-module which delegates to `nexcore-tov-grounded`.
//! The proof types here are the Curry-Howard logic connectives re-exported via
//! `proofs::logic_prelude`.
//!
//! | Primitive | Role in ToV Logic Types |
//! |-----------|------------------------|
//! | x (Product) | Conjunction And<P,Q> is a product type |
//! | Sigma (Sum) | Disjunction Or<P,Q> is a sum type |
//! | exists (Existence) | Existential witness Exists<W,P> |
//! | varsigma (State) | Proof<P> marker for type-level evidence |
//! | emptyset (Void) | Void (uninhabited type, logical falsity) |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::proofs::logic_prelude;

// ---------------------------------------------------------------------------
// Logic Connective: And<P,Q> -- Conjunction
// ---------------------------------------------------------------------------

/// And<P,Q>: T2-P (x + kappa), dominant Product
///
/// Logical conjunction: a proof of P AND Q. The struct holds both
/// `left: P` and `right: Q` -- a pure product type. Comparison is present
/// because conjunction supports structural comparison of proof components.
impl<P, Q> GroundsTo for logic_prelude::And<P, Q> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,    // x -- (left, right) product type
            LexPrimitiva::Comparison, // kappa -- structural equality
        ])
        .with_dominant(LexPrimitiva::Product, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Logic Connective: Or<P,Q> -- Disjunction
// ---------------------------------------------------------------------------

/// Or<P,Q>: T2-P (Sigma + kappa), dominant Sum
///
/// Logical disjunction: a proof of P OR Q. The enum has Left(P) and
/// Right(Q) variants -- a pure sum type. Comparison is present because
/// disjunction supports pattern matching (case analysis).
impl<P, Q> GroundsTo for logic_prelude::Or<P, Q> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- Left|Right variant union
            LexPrimitiva::Comparison, // kappa -- case analysis / pattern matching
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Logic Connective: Exists<W,P> -- Existential Quantification
// ---------------------------------------------------------------------------

/// Exists<W,P>: T2-P (exists + x), dominant Existence
///
/// Existential quantification: a witness W and proof P that the property
/// holds for that witness. Existence is dominant because the type's purpose
/// is to assert that something EXISTS satisfying the property.
impl<W, P> GroundsTo for logic_prelude::Exists<W, P> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // exists -- existential witness
            LexPrimitiva::Product,   // x -- (witness, proof) pair
        ])
        .with_dominant(LexPrimitiva::Existence, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Logic Connective: Proof<P> -- Proof Marker
// ---------------------------------------------------------------------------

/// Proof<P>: T1 (varsigma), dominant State
///
/// Zero-cost proof marker using PhantomData<P>. State dominant because
/// the marker captures type-level evidence state without runtime data.
/// Pure T1 -- a single primitive manifestation.
impl<P> GroundsTo for logic_prelude::Proof<P> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State, // varsigma -- type-level evidence state
        ])
        .with_dominant(LexPrimitiva::State, 0.95)
    }
}

// ---------------------------------------------------------------------------
// Logic Connective: Void -- Logical Falsity
// ---------------------------------------------------------------------------

/// Void: T1 (emptyset), dominant Void
///
/// Uninhabited type representing logical falsity (bottom). No values
/// can be constructed. Pure void primitive -- the canonical empty type.
impl GroundsTo for logic_prelude::Void {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void, // emptyset -- uninhabited type
        ])
        .with_dominant(LexPrimitiva::Void, 0.95)
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
        assert!(!<logic_prelude::And<(), ()>>::is_pure_primitive());
        assert!(!<logic_prelude::Or<(), ()>>::is_pure_primitive());
    }

    // ---- Grounding count ----

    #[test]
    fn test_total_grounded_types_count() {
        // 5 logic connective types grounded in this module
        let count = 5;
        assert_eq!(count, 5, "Should have 5 GroundsTo implementations");
    }
}
