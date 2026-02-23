//! T1 primitive grounding for nucli codec types.
//!
//! Feature-gated behind `grounding` to preserve nucli's zero-dep design.
//!
//! | Type | Primitives | Dominant | Rationale |
//! |------|-----------|----------|-----------|
//! | NucliError | ∂ (boundary) | ∂ | Codec boundary violation |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::error::NucliError;

impl GroundsTo for NucliError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nucli_error_grounds_to_boundary() {
        assert_eq!(
            NucliError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert!(NucliError::is_pure_primitive());
    }
}
