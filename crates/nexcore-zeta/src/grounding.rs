//! [`GroundsTo`] implementations for `nexcore-zeta` types.
//!
//! ## Primitive Profiles
//!
//! - **`ZetaZero`**: T2-C (Location + Quantity + Boundary)
//!   Position on the critical line (λ), ordinal count (N), convergence residual (∂).
//!
//! - **`RhVerification`**: T3 (Boundary + Quantity + Sequence + Existence)
//!   Verification boundary height (∂), zero count (N), ordered zeros (σ), existence witness (∃).
//!
//! - **`DirichletCharacter`**: T2-C (Sequence + Mapping + Boundary)
//!   Periodic values (σ), n → χ(n) mapping (μ), modular arithmetic boundary (∂).
//!
//! - **`ZetaError`**: T1 (Boundary)
//!   Each error variant represents a failed mathematical boundary constraint.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::error::ZetaError;
use crate::l_functions::DirichletCharacter;
use crate::zeros::{RhVerification, ZetaZero};

/// `ZetaZero`: T2-C (Location + Quantity + Boundary), dominant Location.
impl GroundsTo for ZetaZero {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ — position on critical line
            LexPrimitiva::Quantity, // N — ordinal, z_value
            LexPrimitiva::Boundary, // ∂ — convergence residual
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

/// `RhVerification`: T3 (Boundary + Quantity + Sequence + Existence), dominant Boundary.
impl GroundsTo for RhVerification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ — verification height limit
            LexPrimitiva::Quantity,  // N — zero count
            LexPrimitiva::Sequence,  // σ — ordered zeros
            LexPrimitiva::Existence, // ∃ — existence of verification
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

/// `DirichletCharacter`: T2-C (Sequence + Mapping + Boundary), dominant Mapping.
impl GroundsTo for DirichletCharacter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ — periodic values
            LexPrimitiva::Mapping,  // μ — n → χ(n)
            LexPrimitiva::Boundary, // ∂ — modular boundary
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// `ZetaError`: T1 (Boundary), dominant Boundary.
impl GroundsTo for ZetaError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — operation failure at a boundary
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn zeta_zero_grounding() {
        let comp = ZetaZero::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(ZetaZero::tier(), Tier::T2Primitive);
    }

    #[test]
    fn rh_verification_grounding() {
        let comp = RhVerification::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
    }

    #[test]
    fn dirichlet_character_grounding() {
        let comp = DirichletCharacter::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
    }

    #[test]
    fn zeta_error_is_t1() {
        assert_eq!(ZetaError::tier(), Tier::T1Universal);
        assert_eq!(
            ZetaError::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
    }
}
