//! # GroundsTo implementations for nexcore-vigil types
//!
//! Connects the always-on AI orchestrator types to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! - **→ (Causality)**: event → action pipeline
//! - **ς (State)**: agent/event lifecycle
//! - **σ (Sequence)**: event stream ordering

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::errors::VigilError;
use crate::models::{DecisionAction, Event, ExecutorResult, ExecutorType, Interaction, Urgency};

// ---------------------------------------------------------------------------
// T2-P: Classification types
// ---------------------------------------------------------------------------

/// Urgency: T2-P (κ + N), dominant κ
///
/// Event processing urgency. Ascending: Low=0 < Critical=3.
/// Comparison-dominant: ordering.
impl GroundsTo for Urgency {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- urgency ordering
            LexPrimitiva::Quantity,   // N -- numeric weight
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// DecisionAction: T2-P (→ + Σ), dominant →
///
/// FRIDAY decision engine response actions: InvokeClaude, QuickResponse, SilentLog, etc.
/// Causality-dominant: each action causes an effect.
impl GroundsTo for DecisionAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → -- action causes effect
            LexPrimitiva::Sum,       // Σ -- variant enumeration
        ])
        .with_dominant(LexPrimitiva::Causality, 0.90)
    }
}

/// ExecutorType: T2-P (Σ + →), dominant Σ
///
/// Executor classification: Claude, Shell, Speech, Notify, Mcp, Http, Browser.
impl GroundsTo for ExecutorType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- variant enumeration
            LexPrimitiva::Causality, // → -- executor performs action
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T3: Domain types
// ---------------------------------------------------------------------------

/// Event: T3 (→ + σ + ς + κ + λ + N), dominant →
///
/// A discrete event flowing through the system.
/// Causality-dominant: events cause reactions.
impl GroundsTo for Event {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // → -- event triggers action
            LexPrimitiva::Sequence,   // σ -- temporal ordering
            LexPrimitiva::State,      // ς -- payload data
            LexPrimitiva::Comparison, // κ -- priority ordering
            LexPrimitiva::Location,   // λ -- source identity
            LexPrimitiva::Quantity,   // N -- timestamp
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// Interaction: T3 (→ + μ + ς + σ + N + π), dominant →
///
/// Record of an LLM interaction. Causality-dominant: prompt → response.
impl GroundsTo for Interaction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,   // → -- prompt → response
            LexPrimitiva::Mapping,     // μ -- event → output
            LexPrimitiva::State,       // ς -- interaction snapshot
            LexPrimitiva::Sequence,    // σ -- temporal ordering
            LexPrimitiva::Quantity,    // N -- tokens_used
            LexPrimitiva::Persistence, // π -- recorded for later recall
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// ExecutorResult: T2-C (→ + κ + ∅ + μ), dominant →
///
/// Result from an executor processing an action.
/// Causality-dominant: execution produces result.
impl GroundsTo for ExecutorResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // → -- execution → result
            LexPrimitiva::Comparison, // κ -- success/failure
            LexPrimitiva::Void,       // ∅ -- optional output/error
            LexPrimitiva::Mapping,    // μ -- metadata mapping
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Nervous System bridge types
// ---------------------------------------------------------------------------

use crate::bridge::nervous_system::{
    CytokineBridge, EnergyGovernor, GuardianBridge, HormonalModulator, ImmunitySensor,
    NervousSystem, SynapticLearner,
};
use crate::runtime::VigilRuntime;

/// CytokineBridge: T2-P (→ + μ + ν), dominant →
///
/// Translates cytokine signals to Vigil events.
/// Causality-dominant: cytokine emission causes event propagation.
impl GroundsTo for CytokineBridge {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → -- cytokine causes event
            LexPrimitiva::Mapping,   // μ -- family→priority mapping
            LexPrimitiva::Frequency, // ν -- signal frequency
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// HormonalModulator: T2-P (ς + ∂ + μ), dominant ς
///
/// Maintains behavioral state and modulates decision thresholds.
/// State-dominant: endocrine state drives behavioral changes.
impl GroundsTo for HormonalModulator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- behavioral state
            LexPrimitiva::Boundary, // ∂ -- threshold bounds
            LexPrimitiva::Mapping,  // μ -- stimulus→hormone mapping
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// SynapticLearner: T2-P (ν + ρ + π), dominant ν
///
/// Learns from repeated observations of decision outcomes.
/// Frequency-dominant: observation count drives consolidation.
impl GroundsTo for SynapticLearner {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,   // ν -- repeated observations
            LexPrimitiva::Recursion,   // ρ -- self-reinforcing learning
            LexPrimitiva::Persistence, // π -- persistent synapses
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
    }
}

/// ImmunitySensor: T2-P (∃ + κ + ∂), dominant ∃
///
/// Detects existence of code threats at boundaries.
/// Existence-dominant: primary function is threat detection.
impl GroundsTo for ImmunitySensor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // ∃ -- threat detection
            LexPrimitiva::Comparison, // κ -- pattern matching
            LexPrimitiva::Boundary,   // ∂ -- code boundary scanning
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

/// EnergyGovernor: T2-P (N + ∂ + ς), dominant N
///
/// Quantitative threshold gating for LLM invocations.
/// Quantity-dominant: energy charge is a numeric threshold.
impl GroundsTo for EnergyGovernor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- energy charge threshold
            LexPrimitiva::Boundary, // ∂ -- regime boundaries
            LexPrimitiva::State,    // ς -- pool state
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// GuardianBridge: T2-P (→ + ρ + ς), dominant →
///
/// Guardian homeostasis feedback into EventBus.
/// Causality-dominant: Guardian signals cause Vigil reactions.
impl GroundsTo for GuardianBridge {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → -- Guardian signal causes event
            LexPrimitiva::Recursion, // ρ -- feedback loop
            LexPrimitiva::State,     // ς -- iteration state
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// VigilRuntime: T2-P (σ + ς + ∂), dominant σ
///
/// Ordered lifecycle management for daemon/embedded modes.
/// Sequence-dominant: start → run → stop lifecycle.
impl GroundsTo for VigilRuntime {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- lifecycle ordering
            LexPrimitiva::State,    // ς -- running/stopped
            LexPrimitiva::Boundary, // ∂ -- mode boundaries
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// NervousSystem: T3 (→ + ς + ν + N + ∃ + μ + ρ + π + ∂ + σ), dominant →
///
/// The complete Central Nervous System aggregate.
/// Causality-dominant: signal propagation across all subsystems.
impl GroundsTo for NervousSystem {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,   // → -- signal propagation
            LexPrimitiva::State,       // ς -- hormonal/synaptic state
            LexPrimitiva::Frequency,   // ν -- learning frequency
            LexPrimitiva::Quantity,    // N -- energy budget
            LexPrimitiva::Existence,   // ∃ -- threat detection
            LexPrimitiva::Mapping,     // μ -- bridge mappings
            LexPrimitiva::Recursion,   // ρ -- feedback loops
            LexPrimitiva::Persistence, // π -- persistent state
            LexPrimitiva::Boundary,    // ∂ -- threshold bounds
            LexPrimitiva::Sequence,    // σ -- lifecycle ordering
        ])
        .with_dominant(LexPrimitiva::Causality, 0.75)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// VigilError: T2-C (∂ + → + ∅ + Σ), dominant ∂
///
/// Vigil errors: I/O, API, executor, context, LLM failures.
impl GroundsTo for VigilError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ -- constraint violations
            LexPrimitiva::Causality, // → -- operation failures
            LexPrimitiva::Void,      // ∅ -- missing/unknown
            LexPrimitiva::Sum,       // Σ -- error variant
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn urgency_is_comparison_dominant() {
        assert_eq!(
            Urgency::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn decision_action_is_causality_dominant() {
        assert_eq!(
            DecisionAction::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn event_is_t3() {
        assert_eq!(Event::tier(), Tier::T3DomainSpecific);
        assert_eq!(Event::dominant_primitive(), Some(LexPrimitiva::Causality));
    }

    #[test]
    fn interaction_is_t3() {
        assert_eq!(Interaction::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            Interaction::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn executor_result_is_causality_dominant() {
        assert_eq!(
            ExecutorResult::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn vigil_error_is_boundary_dominant() {
        assert_eq!(
            VigilError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn executor_type_is_sum_dominant() {
        assert_eq!(ExecutorType::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    // --- Nervous System bridge grounding tests ---

    #[test]
    fn cytokine_bridge_is_causality_dominant() {
        assert_eq!(
            CytokineBridge::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(CytokineBridge::tier(), Tier::T2Primitive);
    }

    #[test]
    fn hormonal_modulator_is_state_dominant() {
        assert_eq!(
            HormonalModulator::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
        assert_eq!(HormonalModulator::tier(), Tier::T2Primitive);
    }

    #[test]
    fn synaptic_learner_is_frequency_dominant() {
        assert_eq!(
            SynapticLearner::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
        assert_eq!(SynapticLearner::tier(), Tier::T2Primitive);
    }

    #[test]
    fn immunity_sensor_is_existence_dominant() {
        assert_eq!(
            ImmunitySensor::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(ImmunitySensor::tier(), Tier::T2Primitive);
    }

    #[test]
    fn energy_governor_is_quantity_dominant() {
        assert_eq!(
            EnergyGovernor::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(EnergyGovernor::tier(), Tier::T2Primitive);
    }

    #[test]
    fn guardian_bridge_is_causality_dominant() {
        assert_eq!(
            GuardianBridge::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(GuardianBridge::tier(), Tier::T2Primitive);
    }

    #[test]
    fn vigil_runtime_is_sequence_dominant() {
        assert_eq!(
            VigilRuntime::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(VigilRuntime::tier(), Tier::T2Primitive);
    }

    #[test]
    fn nervous_system_is_t3() {
        assert_eq!(NervousSystem::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            NervousSystem::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }
}
