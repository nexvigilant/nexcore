//! T1 primitive grounding for Borrow Miner game types.
//!
//! | Type | Primitives | Dominant | Rationale |
//! |------|-----------|----------|-----------|
//! | Score | N (quantity) | N | Accumulated point value |
//! | Combo | N (quantity) + σ (sequence) | N | Sequential multiplier counter |
//! | Depth | N (quantity) + ∂ (boundary) | N | Mining depth with floor boundary |
//! | OreType | Σ (sum) + N (quantity) | Σ | Weighted sum enum with rarity values |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::types::{Combo, Depth, OreType, Score};

impl GroundsTo for Score {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for Combo {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Quantity, 0.8)
    }
}

impl GroundsTo for Depth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for OreType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Sum, 0.75)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_grounds_to_quantity() {
        assert_eq!(Score::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert!(Score::is_pure_primitive());
    }

    #[test]
    fn combo_grounds_to_quantity() {
        assert_eq!(Combo::dominant_primitive(), Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn depth_grounds_to_quantity() {
        assert_eq!(Depth::dominant_primitive(), Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn ore_type_grounds_to_sum() {
        assert_eq!(OreType::dominant_primitive(), Some(LexPrimitiva::Sum));
    }
}
