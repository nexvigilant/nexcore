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

use crate::graph::{Graph, VertexId};
use crate::markov::{MarkovChain, StateClass};
use crate::matrix::Matrix;
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
// Graph types
// ===========================================================================

/// Graph<V, E>: T2-C (Mapping + Location + Boundary + Sequence), dominant Mapping
///
/// A directed graph with typed vertices and edges.
/// Mapping-dominant: the defining characteristic IS the vertex-to-vertex
/// relationships (edges as mappings). Location for node positions,
/// Boundary for components/communities, Sequence for paths/ordering.
impl<V, E> GroundsTo for Graph<V, E> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- vertex-to-vertex relationships
            LexPrimitiva::Location, // lambda -- nodes as positions in a network
            LexPrimitiva::Boundary, // partial -- communities, components
            LexPrimitiva::Sequence, // sigma -- paths, topological ordering
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// VertexId: T1 (Location), dominant Location
///
/// An opaque identifier for a position in the graph.
/// Pure Location primitive — it IS a position.
impl GroundsTo for VertexId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // lambda -- position in graph
        ])
        .with_dominant(LexPrimitiva::Location, 0.95)
    }
}

// ===========================================================================
// Matrix type
// ===========================================================================

/// Matrix: T2-P (Mapping + Quantity + Boundary), dominant Mapping
///
/// A dense numerical matrix with row-major storage.
/// Mapping-dominant: the defining characteristic IS the row-to-column
/// transformation. Quantity for numeric values, Boundary for dimension
/// constraints.
impl GroundsTo for Matrix {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- row-to-column transformations
            LexPrimitiva::Quantity, // N -- numeric values
            LexPrimitiva::Boundary, // partial -- dimension constraints
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ===========================================================================
// Markov chain types
// ===========================================================================

/// MarkovChain<S>: T2-C (State + Sequence + Quantity + Recursion), dominant State
///
/// A discrete-time Markov chain with typed state labels and transition matrix.
/// State-dominant: the defining characteristic IS the probabilistic state
/// transitions. Sequence for temporal evolution, Quantity for probabilities,
/// Recursion for power iteration and n-step computation.
impl<S> GroundsTo for MarkovChain<S> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- discrete states with transitions
            LexPrimitiva::Sequence,  // sigma -- state-to-state temporal evolution
            LexPrimitiva::Quantity,  // N -- transition probabilities
            LexPrimitiva::Recursion, // rho -- power iteration, n-step computation
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// StateClass: T1 (State), dominant State
///
/// Classification of a Markov chain state as Recurrent, Transient, or Absorbing.
/// Pure State primitive — it IS a classification of state behavior.
impl GroundsTo for StateClass {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State, // varsigma -- state classification
        ])
        .with_dominant(LexPrimitiva::State, 0.95)
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
    fn graph_is_t2c_mapping_dominant() {
        assert_eq!(Graph::<(), ()>::tier(), Tier::T2Composite);
        assert_eq!(
            Graph::<(), ()>::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        let comp = Graph::<(), ()>::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
    }

    #[test]
    fn vertex_id_is_t1_location_dominant() {
        assert_eq!(VertexId::tier(), Tier::T1Universal);
        assert_eq!(
            VertexId::primitive_composition().dominant,
            Some(LexPrimitiva::Location)
        );
        assert!(VertexId::is_pure_primitive());
    }

    #[test]
    fn matrix_is_t2p_mapping_dominant() {
        assert_eq!(Matrix::tier(), Tier::T2Primitive);
        assert_eq!(
            Matrix::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        let comp = Matrix::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn markov_chain_is_t2c_state_dominant() {
        assert_eq!(MarkovChain::<String>::tier(), Tier::T2Composite);
        assert_eq!(
            MarkovChain::<String>::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
        let comp = MarkovChain::<String>::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
    }

    #[test]
    fn state_class_is_t1_state_dominant() {
        assert_eq!(StateClass::tier(), Tier::T1Universal);
        assert_eq!(
            StateClass::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
        assert!(StateClass::is_pure_primitive());
    }

    #[test]
    fn tier_distribution_is_reasonable() {
        // T1: Relation, Identity, MathError, VertexId, StateClass = 5
        let t1_count = [
            Relation::tier(),
            Identity::<()>::tier(),
            MathError::tier(),
            VertexId::tier(),
            StateClass::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T1Universal)
        .count();

        // T2-P (2-3 primitives): Bounded, Proof, MeasuredBound, Matrix = 4
        let t2p_count = [
            Bounded::<()>::tier(),
            Proof::<(), ()>::tier(),
            MeasuredBound::<()>::tier(),
            Matrix::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Primitive)
        .count();

        // T2-C (4+ primitives): Graph, MarkovChain = 2
        let t2c_count = [Graph::<(), ()>::tier(), MarkovChain::<String>::tier()]
            .iter()
            .filter(|t| **t == Tier::T2Composite)
            .count();

        assert_eq!(t1_count, 5, "expected 5 T1 types");
        assert_eq!(t2p_count, 4, "expected 4 T2-P types");
        assert_eq!(t2c_count, 2, "expected 2 T2-C types");
    }
}
