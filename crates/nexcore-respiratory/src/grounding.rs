//! # GroundsTo implementations for nexcore-respiratory types

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    BreathingRate, ExchangeResult, Exhaled, Extracted, Inhaled, InputSource, RespiratoryError,
};

// Claude Code infrastructure types
use crate::claude_code::{
    BreathCycle, ContextFork, ContextSource, DeadSpace, Exhalation, GasExchange, Inhalation,
    RespiratoryHealth, TidalVolume, VitalCapacity,
};

impl GroundsTo for InputSource {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- 4-variant source classification
            LexPrimitiva::Location, // lambda -- where input comes from
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

impl GroundsTo for Inhaled {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- filtered at intake
            LexPrimitiva::Sequence, // sigma -- ordered intake items
            LexPrimitiva::Mapping,  // mu -- source -> intake items
            LexPrimitiva::Quantity, // N -- count of items per source
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.60)
    }
}

impl GroundsTo for Extracted {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- raw intake -> useful data
            LexPrimitiva::Mapping,   // mu -- extraction transformation
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

impl GroundsTo for Exhaled {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,     // empty -- waste expulsion
            LexPrimitiva::Sequence, // sigma -- ordered waste items
        ])
        .with_dominant(LexPrimitiva::Void, 0.85)
    }
}

impl GroundsTo for ExchangeResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,   // times -- composite of extracted + waste
            LexPrimitiva::Boundary,  // partial -- useful/waste boundary
            LexPrimitiva::Causality, // arrow -- intake -> exchange result
        ])
        .with_dominant(LexPrimitiva::Product, 0.65)
    }
}

impl GroundsTo for BreathingRate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,  // nu -- rate of breathing
            LexPrimitiva::Quantity,   // N -- numeric rate value
            LexPrimitiva::Comparison, // kappa -- rate vs demand
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.75)
    }
}

impl GroundsTo for RespiratoryError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error = boundary violation
            LexPrimitiva::Sum,      // Sigma -- error variants
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ============================================================================
// Claude Code Infrastructure — Context Window Respiratory Analogy
// ============================================================================

impl GroundsTo for ContextSource {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Sigma -- 5-variant enum (Tool, McpCall, SystemPrompt, UserMessage, SkillOutput)
            LexPrimitiva::Location, // lambda -- specifies where data enters context from
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for Inhalation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered intake of context tokens
            LexPrimitiva::Quantity, // N -- token count consumed
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

impl GroundsTo for Exhalation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered expulsion of stale context
            LexPrimitiva::Quantity, // N -- tokens freed
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

impl GroundsTo for GasExchange {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- input tokens map to output tokens
            LexPrimitiva::Quantity,   // N -- numeric token counts
            LexPrimitiva::Comparison, // kappa -- exchange ratio (efficiency)
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.70)
    }
}

impl GroundsTo for DeadSpace {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,     // empty -- wasted context overhead
            LexPrimitiva::Quantity, // N -- token counts
            LexPrimitiva::Boundary, // partial -- boundary between useful/waste
        ])
        .with_dominant(LexPrimitiva::Void, 0.70)
    }
}

impl GroundsTo for ContextFork {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // rho -- parent session spawns child (recursive structure)
            LexPrimitiva::Quantity,  // N -- shared fraction measured
            LexPrimitiva::Boundary,  // partial -- isolation boundary
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.75)
    }
}

impl GroundsTo for TidalVolume {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- working context token count
            LexPrimitiva::Boundary, // partial -- max tokens boundary
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for VitalCapacity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- max/residual/tidal token counts
            LexPrimitiva::Boundary,   // partial -- residual volume boundary
            LexPrimitiva::Comparison, // kappa -- utilization ratio
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
    }
}

impl GroundsTo for BreathCycle {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered phases: inhale -> exchange -> exhale
            LexPrimitiva::Frequency, // nu -- cycle rate
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

impl GroundsTo for RespiratoryHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- health state (healthy/unhealthy)
            LexPrimitiva::Comparison, // kappa -- ratio comparisons (dead space, utilization)
            LexPrimitiva::Boundary,   // partial -- threshold violations
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn input_source_is_sum() {
        assert_eq!(InputSource::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn inhaled_is_boundary() {
        assert_eq!(Inhaled::dominant_primitive(), Some(LexPrimitiva::Boundary));
        assert_eq!(Inhaled::tier(), Tier::T2Composite);
    }

    #[test]
    fn extracted_is_causality() {
        assert_eq!(
            Extracted::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn exhaled_is_void() {
        assert_eq!(Exhaled::dominant_primitive(), Some(LexPrimitiva::Void));
    }

    #[test]
    fn breathing_rate_has_frequency() {
        let comp = BreathingRate::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Frequency));
    }

    #[test]
    fn all_types_have_dominant() {
        assert!(InputSource::dominant_primitive().is_some());
        assert!(Inhaled::dominant_primitive().is_some());
        assert!(Extracted::dominant_primitive().is_some());
        assert!(Exhaled::dominant_primitive().is_some());
        assert!(ExchangeResult::dominant_primitive().is_some());
        assert!(BreathingRate::dominant_primitive().is_some());
        assert!(RespiratoryError::dominant_primitive().is_some());
    }

    // ========================================================================
    // Claude Code Infrastructure Tests
    // ========================================================================

    #[test]
    fn context_source_is_sum() {
        assert_eq!(ContextSource::dominant_primitive(), Some(LexPrimitiva::Sum));
        let comp = ContextSource::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Location));
        assert_eq!(ContextSource::tier(), Tier::T2Primitive);
    }

    #[test]
    fn inhalation_is_sequence() {
        assert_eq!(
            Inhalation::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        let comp = Inhalation::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Quantity));
        assert_eq!(Inhalation::tier(), Tier::T2Primitive);
    }

    #[test]
    fn exhalation_is_sequence() {
        assert_eq!(
            Exhalation::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(Exhalation::tier(), Tier::T2Primitive);
    }

    #[test]
    fn gas_exchange_is_mapping() {
        assert_eq!(
            GasExchange::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        let comp = GasExchange::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Quantity));
        assert!(comp.unique().contains(&LexPrimitiva::Comparison));
        // 3 primitives = T2Primitive (tier system: 2-3 = T2-P, 4-5 = T2-C)
        assert_eq!(GasExchange::tier(), Tier::T2Primitive);
    }

    #[test]
    fn dead_space_is_void() {
        assert_eq!(DeadSpace::dominant_primitive(), Some(LexPrimitiva::Void));
        let comp = DeadSpace::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
        // 3 primitives = T2Primitive
        assert_eq!(DeadSpace::tier(), Tier::T2Primitive);
    }

    #[test]
    fn context_fork_is_recursion() {
        assert_eq!(
            ContextFork::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
        let comp = ContextFork::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
        // 3 primitives = T2Primitive
        assert_eq!(ContextFork::tier(), Tier::T2Primitive);
    }

    #[test]
    fn tidal_volume_is_quantity() {
        assert_eq!(
            TidalVolume::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(TidalVolume::tier(), Tier::T2Primitive);
    }

    #[test]
    fn vital_capacity_is_quantity() {
        assert_eq!(
            VitalCapacity::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        let comp = VitalCapacity::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
        assert!(comp.unique().contains(&LexPrimitiva::Comparison));
        // 3 primitives = T2Primitive
        assert_eq!(VitalCapacity::tier(), Tier::T2Primitive);
    }

    #[test]
    fn breath_cycle_is_sequence() {
        assert_eq!(
            BreathCycle::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        let comp = BreathCycle::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Frequency));
        assert_eq!(BreathCycle::tier(), Tier::T2Primitive);
    }

    #[test]
    fn respiratory_health_is_state() {
        assert_eq!(
            RespiratoryHealth::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
        let comp = RespiratoryHealth::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Comparison));
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
        // 3 primitives = T2Primitive
        assert_eq!(RespiratoryHealth::tier(), Tier::T2Primitive);
    }

    #[test]
    fn all_claude_code_types_have_dominant() {
        assert!(ContextSource::dominant_primitive().is_some());
        assert!(Inhalation::dominant_primitive().is_some());
        assert!(Exhalation::dominant_primitive().is_some());
        assert!(GasExchange::dominant_primitive().is_some());
        assert!(DeadSpace::dominant_primitive().is_some());
        assert!(ContextFork::dominant_primitive().is_some());
        assert!(TidalVolume::dominant_primitive().is_some());
        assert!(VitalCapacity::dominant_primitive().is_some());
        assert!(BreathCycle::dominant_primitive().is_some());
        assert!(RespiratoryHealth::dominant_primitive().is_some());
    }

    #[test]
    fn claude_code_primitive_counts() {
        // 2 primitives
        assert_eq!(ContextSource::primitive_composition().unique().len(), 2);
        assert_eq!(Inhalation::primitive_composition().unique().len(), 2);
        assert_eq!(Exhalation::primitive_composition().unique().len(), 2);
        assert_eq!(TidalVolume::primitive_composition().unique().len(), 2);
        assert_eq!(BreathCycle::primitive_composition().unique().len(), 2);

        // 3 primitives (all still T2Primitive per tier system: 2-3 = T2-P)
        assert_eq!(GasExchange::primitive_composition().unique().len(), 3);
        assert_eq!(DeadSpace::primitive_composition().unique().len(), 3);
        assert_eq!(ContextFork::primitive_composition().unique().len(), 3);
        assert_eq!(VitalCapacity::primitive_composition().unique().len(), 3);
        assert_eq!(RespiratoryHealth::primitive_composition().unique().len(), 3);
    }
}
