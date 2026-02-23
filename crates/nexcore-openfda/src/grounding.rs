//! T1 primitive grounding for OpenFDA types.
//!
//! | Type | Primitives | Dominant | Rationale |
//! |------|-----------|----------|-----------|
//! | OpenFdaClient | μ (mapping) + λ (location) | μ | API endpoint dispatch mapping |
//! | OpenFdaResponse | σ (sequence) + N (quantity) | σ | Ordered result set with counts |
//! | FanOutResults | σ (sequence) + μ (mapping) + N (quantity) | μ | Multi-endpoint concurrent mapping |
//! | OpenFdaError | ∂ (boundary) | ∂ | Error boundary classification |
//! | QueryParams | μ (mapping) + ∂ (boundary) | μ | Search param mapping with limit bounds |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::client::{OpenFdaClient, QueryParams};
use crate::error::OpenFdaError;
use crate::search::FanOutResults;

impl GroundsTo for OpenFdaClient {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

impl GroundsTo for QueryParams {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.8)
    }
}

impl GroundsTo for FanOutResults {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.7)
    }
}

impl GroundsTo for OpenFdaError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_grounds_to_mapping() {
        assert_eq!(
            OpenFdaClient::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn query_params_grounds_to_mapping() {
        assert_eq!(
            QueryParams::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn fan_out_results_grounds_to_mapping() {
        assert_eq!(
            FanOutResults::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn error_grounds_to_boundary() {
        assert_eq!(
            OpenFdaError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert!(OpenFdaError::is_pure_primitive());
    }
}
