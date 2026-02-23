//! [`GroundsTo`] implementations for `stem-complex` types.
//!
//! ## Primitive Profile
//!
//! - **`Complex`**: T2-C (Quantity + Location + Boundary)
//!   A complex number IS a quantity (magnitude) located in the 2D plane with
//!   convergence boundary semantics (e.g., the unit circle, poles).
//!
//! - **`ComplexError`**: T1 (Boundary)
//!   Each error variant represents a failed mathematical boundary constraint.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::complex::Complex;
use crate::error::ComplexError;

/// `Complex`: T2-P (Quantity + Location + Boundary), dominant Quantity.
///
/// - `Quantity` (N) — magnitude/modulus, the primary measurement
/// - `Location` (λ) — position in the complex plane (Re, Im axes)
/// - `Boundary` (∂) — convergence radius, analytic boundary, pole structure
impl GroundsTo for Complex {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N   — magnitude
            LexPrimitiva::Location, // λ   — position in complex plane
            LexPrimitiva::Boundary, // ∂   — convergence boundary
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// `ComplexError`: T1 (Boundary), dominant Boundary.
///
/// Each variant is a failed constraint — division by zero, log singularity,
/// or an undefined operation at a mathematical boundary.
impl GroundsTo for ComplexError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — operation failure at a boundary
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn complex_is_t2p_quantity_dominant() {
        assert_eq!(Complex::tier(), Tier::T2Primitive);
        assert_eq!(
            Complex::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
        let comp = Complex::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn complex_error_is_t1_boundary() {
        assert_eq!(ComplexError::tier(), Tier::T1Universal);
        assert_eq!(
            ComplexError::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        assert!(ComplexError::is_pure_primitive());
    }
}
