//! # GroundsTo implementations for nexcore-muscular types

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    AntagonisticPair, Fatigue, ModelEscalation, MotorActivation, MuscleType, MuscularHealth,
    SizePrinciple, ToolClassification,
};

// ============================================================================
// MuscleType -> T2-P (Sum + Sequence) dominant Sum [0.85]
// ============================================================================

impl GroundsTo for MuscleType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- 3-variant enum classification
            LexPrimitiva::Sequence, // sigma -- recruitment order matters
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ============================================================================
// ToolClassification -> T2-P (Mapping + Comparison) dominant Mapping [0.80]
// ============================================================================

impl GroundsTo for ToolClassification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- tool name -> muscle type
            LexPrimitiva::Comparison, // kappa -- voluntary vs involuntary
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ============================================================================
// AntagonisticPair -> T2-P (Causality + Product) dominant Causality [0.85]
// ============================================================================

impl GroundsTo for AntagonisticPair {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- agonist causes opposite of antagonist
            LexPrimitiva::Product,   // times -- paired conjunction
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// ============================================================================
// SizePrinciple -> T2-P (Sequence + Comparison) dominant Sequence [0.80]
// ============================================================================

impl GroundsTo for SizePrinciple {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- ordered escalation
            LexPrimitiva::Comparison, // kappa -- complexity threshold matching
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ============================================================================
// Fatigue -> T2-P (Quantity + Boundary) dominant Quantity [0.80]
// ============================================================================

impl GroundsTo for Fatigue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- token counts, ratios
            LexPrimitiva::Boundary, // partial -- exhaustion threshold
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ============================================================================
// ModelEscalation -> T2-P (Sequence + Quantity) dominant Sequence [0.75]
// ============================================================================

impl GroundsTo for ModelEscalation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- escalation ladder
            LexPrimitiva::Quantity, // N -- compute cost, complexity threshold
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

// ============================================================================
// MotorActivation -> T2-C (Quantity + Comparison + Sequence) dominant Quantity [0.70]
// ============================================================================

impl GroundsTo for MotorActivation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- counts and ratios
            LexPrimitiva::Comparison, // kappa -- balance checking
            LexPrimitiva::Sequence,   // sigma -- recruitment order
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.70)
    }
}

// ============================================================================
// MuscularHealth -> T2-C (State + Comparison + Boundary) dominant State [0.65]
// ============================================================================

impl GroundsTo for MuscularHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- overall health state
            LexPrimitiva::Comparison, // kappa -- checking compliance
            LexPrimitiva::Boundary,   // partial -- fatigue thresholds
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn muscle_type_is_t2p_sum() {
        assert_eq!(MuscleType::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(MuscleType::tier(), Tier::T2Primitive);
    }

    #[test]
    fn tool_classification_is_t2p_mapping() {
        assert_eq!(
            ToolClassification::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(ToolClassification::tier(), Tier::T2Primitive);
    }

    #[test]
    fn antagonistic_pair_is_t2p_causality() {
        assert_eq!(
            AntagonisticPair::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(AntagonisticPair::tier(), Tier::T2Primitive);
    }

    #[test]
    fn size_principle_is_t2p_sequence() {
        assert_eq!(
            SizePrinciple::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(SizePrinciple::tier(), Tier::T2Primitive);
    }

    #[test]
    fn fatigue_is_t2p_quantity() {
        assert_eq!(Fatigue::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(Fatigue::tier(), Tier::T2Primitive);
    }

    #[test]
    fn motor_activation_is_t2c_quantity() {
        assert_eq!(
            MotorActivation::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(MotorActivation::tier(), Tier::T2Primitive);
    }

    #[test]
    fn muscular_health_is_t2c_state() {
        assert_eq!(
            MuscularHealth::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
        assert_eq!(MuscularHealth::tier(), Tier::T2Primitive);
    }

    #[test]
    fn all_types_have_dominant() {
        assert!(MuscleType::dominant_primitive().is_some());
        assert!(ToolClassification::dominant_primitive().is_some());
        assert!(AntagonisticPair::dominant_primitive().is_some());
        assert!(SizePrinciple::dominant_primitive().is_some());
        assert!(Fatigue::dominant_primitive().is_some());
        assert!(MotorActivation::dominant_primitive().is_some());
        assert!(MuscularHealth::dominant_primitive().is_some());
        assert!(ModelEscalation::dominant_primitive().is_some());
    }

    #[test]
    fn fatigue_composition_has_boundary() {
        let comp = Fatigue::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn muscular_health_composition_has_state() {
        let comp = MuscularHealth::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::State));
    }
}
