//! # GroundsTo implementations for stem-bio types
//!
//! Connects biological system primitives to the Lex Primitiva type system.
//!
//! ## Crate Primitive Profile
//!
//! stem-bio implements SCIENCE traits for biological systems. The endocrine
//! module is the primary content, modeling hormone-based behavioral modulation.
//!
//! - **HormoneSignal**: T2-P (Mapping + Quantity) -- stimulus -> signal
//! - **StimulusCategory**: T1 (Sum) -- 5-variant classification enum
//! - **BehaviorModulation**: T2-C (Mapping + State + Quantity) -- output params
//! - **EndocrineSystem**: T3 (full SCIENCE loop) -- domain implementation
//! - **ToneProfile**: T2-C (Mapping + Quantity) -- output style parameters

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::endocrine::{
    BehaviorModulation, EndocrineSystem, HormoneSignal, StimulusCategory, ToneProfile,
};

// ===========================================================================
// Endocrine types
// ===========================================================================

/// HormoneSignal: T2-P (Mapping + Quantity), dominant Mapping
///
/// A signal detected from the environment, carrying stimulus category
/// and intensity. Mapping-dominant: it IS a transformation from
/// environmental stimulus to typed signal representation.
impl GroundsTo for HormoneSignal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- stimulus -> signal transformation
            LexPrimitiva::Quantity, // N -- intensity value [0,1]
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// StimulusCategory: T1 (Sum), dominant Sum
///
/// Five-variant enum: Stress, Reward, Social, Temporal, Urgency.
/// Pure sum type: it IS a discriminated union of stimulus categories.
impl GroundsTo for StimulusCategory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Sigma -- 5-variant discriminated union
        ])
        .with_dominant(LexPrimitiva::Sum, 0.95)
    }
}

/// BehaviorModulation: T2-C (Mapping + State + Quantity + Boundary),
/// dominant Mapping
///
/// Output parameters that modulate AI behavior. Mapping-dominant:
/// the struct IS a mapping from endocrine state to behavioral parameters.
/// Boundary appears because each parameter is clamped to [0,1].
impl GroundsTo for BehaviorModulation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- state -> behavioral parameters
            LexPrimitiva::State,    // varsigma -- parameter snapshot
            LexPrimitiva::Quantity, // N -- five f64 parameter values
            LexPrimitiva::Boundary, // partial -- [0,1] clamping
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// EndocrineSystem: T3 (Mapping + Sequence + State + Recursion + Causality + Quantity),
/// dominant State
///
/// Full SCIENCE loop implementation for biological hormone signaling.
/// State-dominant: the endocrine system IS stateful -- it holds and
/// mutates EndocrineState through stimuli.
impl GroundsTo for EndocrineSystem {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- EndocrineState holder
            LexPrimitiva::Mapping,   // mu -- Sense, Classify, Codify, Extend
            LexPrimitiva::Sequence,  // sigma -- Experiment (action -> outcome)
            LexPrimitiva::Recursion, // rho -- Infer (pattern -> prediction)
            LexPrimitiva::Causality, // arrow -- stimulus -> state change
            LexPrimitiva::Quantity,  // N -- hormone levels, intensity values
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// ToneProfile: T2-C (Mapping + Quantity + Boundary), dominant Mapping
///
/// Output style parameters for AI tone. Mapping-dominant: the profile
/// IS a mapping from behavioral modulation to tone dimensions.
impl GroundsTo for ToneProfile {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- modulation -> tone parameters
            LexPrimitiva::Quantity, // N -- five f64 tone dimensions
            LexPrimitiva::Boundary, // partial -- implied [0,1] range
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn hormone_signal_is_t2p_mapping_dominant() {
        assert_eq!(HormoneSignal::tier(), Tier::T2Primitive);
        assert_eq!(
            HormoneSignal::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn stimulus_category_is_t1_sum_dominant() {
        assert_eq!(StimulusCategory::tier(), Tier::T1Universal);
        assert_eq!(
            StimulusCategory::primitive_composition().dominant,
            Some(LexPrimitiva::Sum)
        );
        assert!(StimulusCategory::is_pure_primitive());
    }

    #[test]
    fn behavior_modulation_is_t2c_mapping_dominant() {
        // 4 primitives = T2-C
        assert_eq!(BehaviorModulation::tier(), Tier::T2Composite);
        assert_eq!(
            BehaviorModulation::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        let comp = BehaviorModulation::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn endocrine_system_is_t3_state_dominant() {
        assert_eq!(EndocrineSystem::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            EndocrineSystem::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
        let comp = EndocrineSystem::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
    }

    #[test]
    fn tone_profile_is_t2p_mapping_dominant() {
        // 3 primitives = T2-P
        assert_eq!(ToneProfile::tier(), Tier::T2Primitive);
        assert_eq!(
            ToneProfile::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn tier_distribution_is_reasonable() {
        let t1_count = [StimulusCategory::tier()]
            .iter()
            .filter(|t| **t == Tier::T1Universal)
            .count();

        // T2-P (2-3 primitives): HormoneSignal, ToneProfile = 2
        let t2p_count = [HormoneSignal::tier(), ToneProfile::tier()]
            .iter()
            .filter(|t| **t == Tier::T2Primitive)
            .count();

        // T2-C (4 primitives): BehaviorModulation = 1
        let t2c_count = [BehaviorModulation::tier()]
            .iter()
            .filter(|t| **t == Tier::T2Composite)
            .count();

        let t3_count = [EndocrineSystem::tier()]
            .iter()
            .filter(|t| **t == Tier::T3DomainSpecific)
            .count();

        assert_eq!(t1_count, 1, "expected 1 T1 type");
        assert_eq!(t2p_count, 2, "expected 2 T2-P types");
        assert_eq!(t2c_count, 1, "expected 1 T2-C type");
        assert_eq!(t3_count, 1, "expected 1 T3 type");
    }
}
