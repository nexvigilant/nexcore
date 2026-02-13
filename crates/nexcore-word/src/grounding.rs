//! # GroundsTo Implementations
//!
//! Connects all `nexcore-word` types to the Lex Primitiva type system.
//!
//! ## Grounding Table
//!
//! | Type | Dominant | Confidence | Tier |
//! |------|----------|-----------|------|
//! | `Word<u8/u16/u32/u64>` | State (ς) | 1.0 | T1 |
//! | `BitCount` | Quantity (N) | 1.0 | T1 |
//! | `BitPosition` | Sequence (σ) | 0.9 | T2-P |
//! | `Parity` | Comparison (κ) | 1.0 | T1 |
//! | `Alignment` | Boundary (∂) | 1.0 | T1 |
//! | `WordError` | Boundary (∂) | 0.9 | T2-P |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::error::WordError;
use crate::properties::{Alignment, BitCount, BitPosition, Parity};
use crate::width::Word;
use crate::width::WordWidth;

// ============================================================================
// Word<W> — State (ς), T1
// ============================================================================

impl<W: WordWidth> GroundsTo for Word<W> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

// ============================================================================
// BitCount — Quantity (N), T1
// ============================================================================

impl GroundsTo for BitCount {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

// ============================================================================
// BitPosition — Sequence (σ) + Location (λ), T2-P
// ============================================================================

impl GroundsTo for BitPosition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Sequence, 0.9)
    }
}

// ============================================================================
// Parity — Comparison (κ), T1
// ============================================================================

impl GroundsTo for Parity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

// ============================================================================
// Alignment — Boundary (∂), T1
// ============================================================================

impl GroundsTo for Alignment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 1.0)
    }
}

// ============================================================================
// WordError — Boundary (∂) + Sum (Σ), T2-P
// ============================================================================

impl GroundsTo for WordError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width::{Word8, Word16, Word32, Word64};
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn word8_grounds_to_state() {
        assert_eq!(Word8::dominant_primitive(), Some(LexPrimitiva::State));
        assert!(Word8::is_pure_primitive());
        assert_eq!(Word8::tier(), Tier::T1Universal);
    }

    #[test]
    fn word16_grounds_to_state() {
        assert_eq!(Word16::dominant_primitive(), Some(LexPrimitiva::State));
        assert_eq!(Word16::tier(), Tier::T1Universal);
    }

    #[test]
    fn word32_grounds_to_state() {
        assert_eq!(Word32::dominant_primitive(), Some(LexPrimitiva::State));
        assert_eq!(Word32::tier(), Tier::T1Universal);
    }

    #[test]
    fn word64_grounds_to_state() {
        assert_eq!(Word64::dominant_primitive(), Some(LexPrimitiva::State));
        assert_eq!(Word64::tier(), Tier::T1Universal);
    }

    #[test]
    fn bitcount_grounds_to_quantity() {
        assert_eq!(BitCount::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert!(BitCount::is_pure_primitive());
        assert_eq!(BitCount::tier(), Tier::T1Universal);
    }

    #[test]
    fn bitposition_grounds_to_sequence() {
        assert_eq!(
            BitPosition::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert!(!BitPosition::is_pure_primitive());
        assert_eq!(BitPosition::tier(), Tier::T2Primitive);
    }

    #[test]
    fn parity_grounds_to_comparison() {
        assert_eq!(Parity::dominant_primitive(), Some(LexPrimitiva::Comparison));
        assert!(Parity::is_pure_primitive());
        assert_eq!(Parity::tier(), Tier::T1Universal);
    }

    #[test]
    fn alignment_grounds_to_boundary() {
        assert_eq!(
            Alignment::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert!(Alignment::is_pure_primitive());
        assert_eq!(Alignment::tier(), Tier::T1Universal);
    }

    #[test]
    fn word_error_grounds_to_boundary() {
        assert_eq!(
            WordError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert!(!WordError::is_pure_primitive());
        assert_eq!(WordError::tier(), Tier::T2Primitive);
    }
}
