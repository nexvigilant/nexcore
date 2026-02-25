//! GroundsTo implementations for QBR types.
//!
//! Every type declares its T1 primitive composition, enabling
//! cross-domain transfer analysis and structural complexity metrics.
//!
//! Tier: T3-D (Domain Composite)

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::types::{BenefitRiskInput, HillCurveParams, IntegrationBounds, QBR, QbrMethodDetails};

impl GroundsTo for QBR {
    fn primitive_composition() -> PrimitiveComposition {
        // QBR = Comparison (benefit/risk ratio) + Quantity (signal strengths)
        //     + Causality (exposure→outcome) + Boundary (therapeutic window)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

impl GroundsTo for BenefitRiskInput {
    fn primitive_composition() -> PrimitiveComposition {
        // Input = Quantity (contingency counts) + Causality (exposure→outcome)
        //       + Comparison (benefit vs risk framing)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.9)
    }
}

impl GroundsTo for HillCurveParams {
    fn primitive_composition() -> PrimitiveComposition {
        // Hill curve = Quantity (dose, response) + Boundary (saturation)
        //            + Causality (dose→response)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.9)
    }
}

impl GroundsTo for IntegrationBounds {
    fn primitive_composition() -> PrimitiveComposition {
        // Bounds = Boundary (min/max limits) + Quantity (interval count)
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

impl GroundsTo for QbrMethodDetails {
    fn primitive_composition() -> PrimitiveComposition {
        // Audit trail = Quantity (signal values) + Comparison (method choice)
        //             + Persistence (record-keeping)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qbr_grounding() {
        assert_eq!(QBR::dominant_primitive(), Some(LexPrimitiva::Comparison));
        assert!(!QBR::is_pure_primitive());
    }

    #[test]
    fn test_input_grounding() {
        assert_eq!(
            BenefitRiskInput::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn test_hill_grounding() {
        assert_eq!(
            HillCurveParams::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn test_bounds_grounding() {
        assert_eq!(
            IntegrationBounds::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn test_details_grounding() {
        assert_eq!(
            QbrMethodDetails::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }
}
