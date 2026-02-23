//! T1 primitive grounding for topological data analysis types.
//!
//! | Type | Primitives | Dominant | Rationale |
//! |------|-----------|----------|-----------|
//! | Simplex | N (quantity) | N | Vertex count defines dimension |
//! | SimplicialComplex | Σ (sum) + N | Σ | Collection of simplices |
//! | PersistenceDiagram | π (persistence) + σ (sequence) | π | Lifetime of topological features |
//! | BettiNumbers | Σ (sum) + N | Σ | Counting connected components/holes |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::betti::BettiNumbers;
use crate::diagram::{PersistenceDiagram, PersistencePoint};
use crate::simplex::{Simplex, SimplicialComplex};

impl GroundsTo for Simplex {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for SimplicialComplex {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Sum, 0.7)
    }
}

impl GroundsTo for PersistencePoint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Persistence, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Persistence, 0.8)
    }
}

impl GroundsTo for PersistenceDiagram {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Sequence,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.6)
    }
}

impl GroundsTo for BettiNumbers {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Sum, 0.8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplex_grounds_to_quantity() {
        assert_eq!(Simplex::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert!(Simplex::is_pure_primitive());
    }

    #[test]
    fn complex_grounds_to_sum() {
        assert_eq!(
            SimplicialComplex::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert!(!SimplicialComplex::is_pure_primitive());
    }

    #[test]
    fn persistence_diagram_grounds_to_persistence() {
        assert_eq!(
            PersistenceDiagram::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn betti_grounds_to_sum() {
        assert_eq!(BettiNumbers::dominant_primitive(), Some(LexPrimitiva::Sum));
    }
}
