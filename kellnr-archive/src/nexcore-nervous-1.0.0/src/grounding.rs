//! # GroundsTo implementations for nexcore-nervous types
//!
//! Maps all nervous system types to their Lex Primitiva compositions,
//! grounding biological nervous system concepts in the 16 primitive symbols.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::claude_code::{
    ContextAssembly, EventBusRoute, HookChain, HookDispatch, HookEvent, MyelinationCache,
    NervousSystemHealth, ReflexResponse, SignalLatency, ToolChain,
};
use crate::{
    Impulse, MotorCommand, Myelination, NervousHealth, NeuronType, ReflexArc, SensoryInput,
    SignalSpeed,
};

// Core nervous system types

impl GroundsTo for NeuronType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for Impulse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

impl GroundsTo for ReflexArc {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.75)
    }
}

impl GroundsTo for Myelination {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.70)
    }
}

impl GroundsTo for SensoryInput {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

impl GroundsTo for MotorCommand {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

impl GroundsTo for SignalSpeed {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Frequency])
            .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

impl GroundsTo for NervousHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// Claude Code types

impl GroundsTo for HookEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Frequency])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for HookDispatch {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

impl GroundsTo for HookChain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Causality,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.70)
    }
}

impl GroundsTo for EventBusRoute {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Location,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

impl GroundsTo for ContextAssembly {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.65)
    }
}

impl GroundsTo for ToolChain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Causality,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.70)
    }
}

impl GroundsTo for SignalLatency {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
    }
}

impl GroundsTo for MyelinationCache {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.70)
    }
}

impl GroundsTo for ReflexResponse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Frequency,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

impl GroundsTo for NervousSystemHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_type_grounding() {
        let comp = NeuronType::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(comp.primitives.len(), 2);
    }

    #[test]
    fn test_impulse_grounding() {
        let comp = Impulse::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.confidence >= 0.70);
    }

    #[test]
    fn test_reflex_arc_grounding() {
        let comp = ReflexArc::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.confidence >= 0.75);
    }

    #[test]
    fn test_myelination_grounding() {
        let comp = Myelination::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert!(comp.confidence >= 0.70);
    }

    #[test]
    fn test_sensory_input_grounding() {
        let comp = SensoryInput::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!(comp.confidence >= 0.80);
    }

    #[test]
    fn test_motor_command_grounding() {
        let comp = MotorCommand::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.confidence >= 0.80);
    }

    #[test]
    fn test_signal_speed_grounding() {
        let comp = SignalSpeed::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Frequency));
        assert!(comp.confidence >= 0.85);
    }

    #[test]
    fn test_nervous_health_grounding() {
        let comp = NervousHealth::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(comp.confidence >= 0.65);
    }

    #[test]
    fn test_hook_event_grounding() {
        let comp = HookEvent::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!(comp.confidence >= 0.85);
    }

    #[test]
    fn test_hook_dispatch_grounding() {
        let comp = HookDispatch::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.confidence >= 0.70);
    }

    #[test]
    fn test_hook_chain_grounding() {
        let comp = HookChain::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!(comp.confidence >= 0.70);
    }

    #[test]
    fn test_event_bus_route_grounding() {
        let comp = EventBusRoute::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.confidence >= 0.70);
    }

    #[test]
    fn test_context_assembly_grounding() {
        let comp = ContextAssembly::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!(comp.confidence >= 0.65);
    }

    #[test]
    fn test_tool_chain_grounding() {
        let comp = ToolChain::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!(comp.confidence >= 0.70);
    }

    #[test]
    fn test_signal_latency_grounding() {
        let comp = SignalLatency::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!(comp.confidence >= 0.75);
    }

    #[test]
    fn test_myelination_cache_grounding() {
        let comp = MyelinationCache::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert!(comp.confidence >= 0.70);
    }

    #[test]
    fn test_reflex_response_grounding() {
        let comp = ReflexResponse::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.confidence >= 0.80);
    }

    #[test]
    fn test_nervous_system_health_grounding() {
        let comp = NervousSystemHealth::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(comp.confidence >= 0.65);
    }
}
