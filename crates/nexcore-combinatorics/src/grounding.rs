//! T1 primitive grounding for combinatorial algorithm types.
//!
//! All combinatorics algorithms are σ-dominant (Sequence), reflecting their
//! origin in ordering, arrangement, and permutation problems.
//!
//! | Algorithm | Primitives | Dominant | Puzzle Origin |
//! |-----------|-----------|----------|---------------|
//! | Catalan | σ+∂+κ | σ | #279 Barrels of Balsam |
//! | Derangement | σ+κ+N | σ | #267 Wrong Hats |
//! | Cycle decomp | σ+μ+N | σ | #238 Jampots |
//! | Josephus | σ+N+ρ | σ | #232 Catching Mice |
//! | Grid paths | σ+N+∂ | σ | #253 Bank Holiday |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

/// Grounding marker for Catalan number computations.
pub struct CatalanComputation;

/// Grounding marker for derangement computations.
pub struct DerangementComputation;

/// Grounding marker for cycle decomposition.
pub struct CycleDecomposition;

/// Grounding marker for Josephus problem solutions.
pub struct JosephusComputation;

/// Grounding marker for grid path counting.
pub struct GridPathComputation;

impl GroundsTo for CatalanComputation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.6)
    }
}

impl GroundsTo for DerangementComputation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.6)
    }
}

impl GroundsTo for CycleDecomposition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.5)
    }
}

impl GroundsTo for JosephusComputation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Recursion,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.5)
    }
}

impl GroundsTo for GridPathComputation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_combinatorics_are_sequence_dominant() {
        assert_eq!(
            CatalanComputation::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            DerangementComputation::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            CycleDecomposition::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            JosephusComputation::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            GridPathComputation::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn none_are_pure_primitives() {
        assert!(!CatalanComputation::is_pure_primitive());
        assert!(!DerangementComputation::is_pure_primitive());
    }
}
