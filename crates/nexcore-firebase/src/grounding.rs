//! T1 primitive grounding for Firebase types.
//!
//! | Type | Primitives | Dominant | Rationale |
//! |------|-----------|----------|-----------|
//! | AuthResponse | ∃ (existence) + ∂ (boundary) | ∃ | Identity token proving user exists within auth boundary |
//! | SignInRequest | μ (mapping) + ∂ (boundary) | μ | Credential mapping at auth boundary |
//! | AuthErrorResponse | ∂ (boundary) | ∂ | Authentication boundary violation |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::auth::{AuthErrorResponse, AuthResponse, SignInRequest};

impl GroundsTo for AuthResponse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

impl GroundsTo for SignInRequest {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.8)
    }
}

impl GroundsTo for AuthErrorResponse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_response_grounds_to_existence() {
        assert_eq!(
            AuthResponse::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn sign_in_request_grounds_to_mapping() {
        assert_eq!(
            SignInRequest::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn auth_error_grounds_to_boundary() {
        assert_eq!(
            AuthErrorResponse::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert!(AuthErrorResponse::is_pure_primitive());
    }
}
