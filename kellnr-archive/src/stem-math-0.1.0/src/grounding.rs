//! # GroundsTo implementations for stem-math types
//!
//! Connects mathematics primitive types and the Mathematics composite to the
//! Lex Primitiva type system.
//!
//! ## Crate Primitive Profile
//!
//! stem-math defines the MATHS composite with 9 traits and 4 core value types.
//! The crate spans multiple T1 primitives: Boundary (Bounded, Bound trait),
//! Sequence (Proof, Prove, Transit), Recursion (Associate), Comparison
//! (Membership, Symmetric, Commute, Relation), State (Identity, Identify),
//! and Mapping (Homeomorph).
//!
//! - **Bounded<T>**: T2-P (Boundary + State) -- constrained range
//! - **Proof<P, C>**: T2-P (Sequence + Existence) -- premises -> conclusion
//! - **Relation**: T1 (Comparison) -- ordering enum
//! - **Identity<T>**: T1 (State) -- neutral element marker
//! - **MeasuredBound<T>**: T2-C (Boundary + State + Quantity) -- with confidence
//! - **MathError**: T1 (Boundary) -- operation failure

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{Bounded, Identity, MathError, MeasuredBound, Proof, Relation};

// ===========================================================================
// Core value types
// ===========================================================================

/// Bounded<T>: T2-P (Boundary + State), dominant Boundary
///
/// A value with optional upper and lower limits.
/// Boundary-dominant: the defining characteristic IS the constraint
/// on the value's range.
impl<T> GroundsTo for Bounded<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- upper/lower limit constraints
            LexPrimitiva::State,    // varsigma -- the value within bounds
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// Proof<P, C>: T2-P (Sequence + Existence), dominant Sequence
///
/// An ordered list of premises leading to a conclusion with validity.
/// Sequence-dominant: the proof IS an ordered chain from premises
/// to conclusion. Existence appears because the validity flag
/// determines whether the proof exists.
impl<P, C> GroundsTo for Proof<P, C> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // sigma -- premises -> conclusion ordering
            LexPrimitiva::Existence, // exists -- valid/invalid existence check
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// Relation: T1 (Comparison), dominant Comparison
///
/// Four-variant enum: LessThan, Equal, GreaterThan, Incomparable.
/// Pure comparison: it IS the result of comparing two elements.
impl GroundsTo for Relation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- ordering comparison result
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// Identity<T>: T1 (State), dominant State
///
/// A marker wrapping the neutral element of an operation.
/// Pure state: it IS a fixed element that produces no change.
impl<T> GroundsTo for Identity<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State, // varsigma -- neutral element (no-op state)
        ])
        .with_dominant(LexPrimitiva::State, 0.95)
    }
}

// ===========================================================================
// Measured types
// ===========================================================================

/// MeasuredBound<T>: T2-C (Boundary + State + Quantity), dominant Boundary
///
/// A bounded value paired with confidence. Boundary-dominant: the
/// primary purpose is the constraint, augmented with a confidence
/// score.
impl<T> GroundsTo for MeasuredBound<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- upper/lower limits
            LexPrimitiva::State,    // varsigma -- the bounded value
            LexPrimitiva::Quantity, // N -- confidence value
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ===========================================================================
// Error type
// ===========================================================================

/// MathError: T1 (Boundary), dominant Boundary
///
/// Error enum for mathematical operations. Pure boundary: each variant
/// represents a failed constraint or undefined operation.
impl GroundsTo for MathError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- operation failure boundary
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn bounded_is_t2p_boundary_dominant() {
        assert_eq!(Bounded::<i32>::tier(), Tier::T2Primitive);
        assert_eq!(
            Bounded::<i32>::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        let comp = Bounded::<i32>::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
    }

    #[test]
    fn proof_is_t2p_sequence_dominant() {
        assert_eq!(Proof::<String, String>::tier(), Tier::T2Primitive);
        assert_eq!(
            Proof::<String, String>::primitive_composition().dominant,
            Some(LexPrimitiva::Sequence)
        );
        let comp = Proof::<String, String>::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
    }

    #[test]
    fn relation_is_t1_comparison_dominant() {
        assert_eq!(Relation::tier(), Tier::T1Universal);
        assert_eq!(
            Relation::primitive_composition().dominant,
            Some(LexPrimitiva::Comparison)
        );
        assert!(Relation::is_pure_primitive());
    }

    #[test]
    fn identity_is_t1_state_dominant() {
        assert_eq!(Identity::<i32>::tier(), Tier::T1Universal);
        assert_eq!(
            Identity::<i32>::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
        assert!(Identity::<i32>::is_pure_primitive());
    }

    #[test]
    fn measured_bound_is_t2p_boundary_dominant() {
        // 3 primitives = T2-P
        assert_eq!(MeasuredBound::<f64>::tier(), Tier::T2Primitive);
        assert_eq!(
            MeasuredBound::<f64>::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        let comp = MeasuredBound::<f64>::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn math_error_is_t1_boundary_dominant() {
        assert_eq!(MathError::tier(), Tier::T1Universal);
        assert_eq!(
            MathError::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        assert!(MathError::is_pure_primitive());
    }

    #[test]
    fn tier_distribution_is_reasonable() {
        // T1: Relation, Identity, MathError = 3
        let t1_count = [Relation::tier(), Identity::<()>::tier(), MathError::tier()]
            .iter()
            .filter(|t| **t == Tier::T1Universal)
            .count();

        // T2-P (2-3 primitives): Bounded, Proof, MeasuredBound = 3
        let t2p_count = [
            Bounded::<()>::tier(),
            Proof::<(), ()>::tier(),
            MeasuredBound::<()>::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Primitive)
        .count();

        assert_eq!(t1_count, 3, "expected 3 T1 types");
        assert_eq!(t2p_count, 3, "expected 3 T2-P types");
    }
}
