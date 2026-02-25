//! # GroundsTo implementations for nexcore-primitives
//!
//! Primitive grounding for all public types across the chemistry, quantum,
//! transfer, and dynamics modules.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

// ============================================================================
// Chemistry Module
// ============================================================================

// -- threshold_gating --

impl GroundsTo for crate::chemistry::ThresholdGate {
    fn primitive_composition() -> PrimitiveComposition {
        // Arrhenius activation: threshold × quantity × frequency
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

impl GroundsTo for crate::chemistry::ThresholdError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- saturation --

impl GroundsTo for crate::chemistry::SaturationKinetics {
    fn primitive_composition() -> PrimitiveComposition {
        // Michaelis-Menten: maximum × cause × ratio
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

impl GroundsTo for crate::chemistry::SaturationError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- feasibility --

impl GroundsTo for crate::chemistry::FeasibilityAssessment {
    fn primitive_composition() -> PrimitiveComposition {
        // Gibbs: quantity × comparison × state
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

impl GroundsTo for crate::chemistry::Favorability {
    fn primitive_composition() -> PrimitiveComposition {
        // Enum: favorable/unfavorable/borderline
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for crate::chemistry::FeasibilityError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- dependency (rate law) --

impl GroundsTo for crate::chemistry::RateLaw {
    fn primitive_composition() -> PrimitiveComposition {
        // Rate law: dependency × frequency × quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

impl GroundsTo for crate::chemistry::DependencyError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- buffer_stability --

impl GroundsTo for crate::chemistry::BufferSystem {
    fn primitive_composition() -> PrimitiveComposition {
        // Henderson-Hasselbalch: state × ratio × persists
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Quantity,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for crate::chemistry::BufferError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- signal_intensity --

impl GroundsTo for crate::chemistry::SignalDetector {
    fn primitive_composition() -> PrimitiveComposition {
        // Beer-Lambert: signal × proportion × quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for crate::chemistry::SignalError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- decay_kinetics --

impl GroundsTo for crate::chemistry::DecayKinetics {
    fn primitive_composition() -> PrimitiveComposition {
        // Half-life: frequency × quantity × irreversibility
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
            LexPrimitiva::Irreversibility,
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
    }
}

impl GroundsTo for crate::chemistry::DecayError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- equilibrium --

impl GroundsTo for crate::chemistry::EquilibriumSystem {
    fn primitive_composition() -> PrimitiveComposition {
        // Equilibrium: state × persistence × quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

impl GroundsTo for crate::chemistry::EquilibriumError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- cooperativity --

impl GroundsTo for crate::chemistry::CooperativeBinding {
    fn primitive_composition() -> PrimitiveComposition {
        // Hill equation: causality × quantity × comparison
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

impl GroundsTo for crate::chemistry::CooperativityType {
    fn primitive_composition() -> PrimitiveComposition {
        // Enum: Positive/Negative/None
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for crate::chemistry::CooperativityError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- electrochemical --

impl GroundsTo for crate::chemistry::ElectrochemicalCell {
    fn primitive_composition() -> PrimitiveComposition {
        // Nernst equation: quantity × boundary × state
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

impl GroundsTo for crate::chemistry::PotentialState {
    fn primitive_composition() -> PrimitiveComposition {
        // Enum: Above/Below/At threshold
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for crate::chemistry::ElectrochemicalError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- inhibition --

impl GroundsTo for crate::chemistry::CompetitiveInhibition {
    fn primitive_composition() -> PrimitiveComposition {
        // Competitive inhibition: boundary × quantity × causality
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::chemistry::InhibitionStrength {
    fn primitive_composition() -> PrimitiveComposition {
        // Enum: Weak/Moderate/Strong/Overwhelming
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for crate::chemistry::InhibitionError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- transition_state --

impl GroundsTo for crate::chemistry::TransitionState {
    fn primitive_composition() -> PrimitiveComposition {
        // Eyring: boundary × frequency × quantity × causality
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::chemistry::ActivationParameters {
    fn primitive_composition() -> PrimitiveComposition {
        // Thermodynamic activation: quantity × product
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for crate::chemistry::TransitionStateError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- adsorption --

impl GroundsTo for crate::chemistry::LangmuirIsotherm {
    fn primitive_composition() -> PrimitiveComposition {
        // Langmuir: boundary × quantity × state
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for crate::chemistry::CompetitiveLangmuir {
    fn primitive_composition() -> PrimitiveComposition {
        // Multi-species competition: comparison × quantity × boundary
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

impl GroundsTo for crate::chemistry::CoverageState {
    fn primitive_composition() -> PrimitiveComposition {
        // Enum: Empty/Low/Medium/High/Full
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for crate::chemistry::AdsorptionError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// -- aggregation_pipeline --

impl GroundsTo for crate::chemistry::AggregationPipeline {
    fn primitive_composition() -> PrimitiveComposition {
        // Pipeline: sequence × mapping × state
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// -- PvMapping --

impl GroundsTo for crate::chemistry::PvMapping {
    fn primitive_composition() -> PrimitiveComposition {
        // Cross-domain mapping with confidence
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
            LexPrimitiva::Product,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ============================================================================
// Quantum Module (wave, operator, state, core submodules)
// ============================================================================

// -- wave --

impl GroundsTo for crate::quantum::wave::Amplitude {
    fn primitive_composition() -> PrimitiveComposition {
        // Magnitude + frequency: N (Quantity) + ν (Frequency)
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Frequency])
            .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for crate::quantum::wave::Phase {
    fn primitive_composition() -> PrimitiveComposition {
        // Angle + offset: N (Quantity) + σ (Sequence)
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for crate::quantum::wave::Interference {
    fn primitive_composition() -> PrimitiveComposition {
        // Superposition of amplitudes and phases: Σ (Sum) + N (Quantity)
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// -- operator --

impl GroundsTo for crate::quantum::operators::Eigenstate {
    fn primitive_composition() -> PrimitiveComposition {
        // Fixed point of operator: ς (State) + N (Quantity) + κ (Comparison)
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::quantum::operators::Observable {
    fn primitive_composition() -> PrimitiveComposition {
        // Measurable property with eigenstates: μ (Mapping) + N (Quantity) + ∃ (Existence)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

impl GroundsTo for crate::quantum::operators::Hermiticity {
    fn primitive_composition() -> PrimitiveComposition {
        // Self-adjoint constraint: ∂ (Boundary) + κ (Comparison)
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

impl GroundsTo for crate::quantum::operators::Unitarity {
    fn primitive_composition() -> PrimitiveComposition {
        // Norm-preserving transformation: ∂ (Boundary) + N (Quantity)
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// -- state --

impl GroundsTo for crate::quantum::state::Superposition {
    fn primitive_composition() -> PrimitiveComposition {
        // Weighted combination of states: Σ (Sum) + ς (State) + N (Quantity)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::State,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

impl GroundsTo for crate::quantum::state::Measurement {
    fn primitive_composition() -> PrimitiveComposition {
        // Collapse: ∝ (Irreversibility) + ∃ (Existence) + N (Quantity) + ς (State)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Existence,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.75)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::quantum::state::Uncertainty {
    fn primitive_composition() -> PrimitiveComposition {
        // Conjugate uncertainty: ∂ (Boundary) + N (Quantity) + κ (Comparison)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// -- core --

impl GroundsTo for crate::quantum::domain::Qubit {
    fn primitive_composition() -> PrimitiveComposition {
        // Two-state quantum: ς (State) + Σ (Sum) + N (Quantity) + ∂ (Boundary)
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Sum,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::quantum::domain::Entanglement {
    fn primitive_composition() -> PrimitiveComposition {
        // Correlated multi-party state: → (Causality) + ς (State) + N (Quantity) + × (Product)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::State,
            LexPrimitiva::Quantity,
            LexPrimitiva::Product,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.75)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::quantum::domain::Decoherence {
    fn primitive_composition() -> PrimitiveComposition {
        // Information loss to environment: ∝ (Irreversibility) + ν (Frequency) + N (Quantity)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.80)
    }
}

// ============================================================================
// Transfer Module (28 types)
// ============================================================================

impl GroundsTo for crate::transfer::ThreatSignature {
    fn primitive_composition() -> PrimitiveComposition {
        // Pattern matching against known threats: κ (Comparison) + × (Product) + ∂ (Boundary)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Product,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

impl GroundsTo for crate::transfer::ResourceRatio {
    fn primitive_composition() -> PrimitiveComposition {
        // Proportional resource comparison: N (Quantity) + κ (Comparison)
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for crate::transfer::PatternMatcher {
    fn primitive_composition() -> PrimitiveComposition {
        // Regex-like matching with thresholds: κ (Comparison) + ∂ (Boundary) + N (Quantity)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

impl GroundsTo for crate::transfer::ExploreExploit {
    fn primitive_composition() -> PrimitiveComposition {
        // Trade-off between exploration and exploitation: Σ (Sum) + N (Quantity) + → (Causality)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

impl GroundsTo for crate::transfer::EventClassifier {
    fn primitive_composition() -> PrimitiveComposition {
        // Categorization of events: κ (Comparison) + μ (Mapping) + Σ (Sum)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

impl GroundsTo for crate::transfer::FeedbackLoop {
    fn primitive_composition() -> PrimitiveComposition {
        // Recursive causal cycle: → (Causality) + ρ (Recursion) + ς (State)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Recursion,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for crate::transfer::SchemaContract {
    fn primitive_composition() -> PrimitiveComposition {
        // Structural validation: ∂ (Boundary) + × (Product) + κ (Comparison)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Product,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

impl GroundsTo for crate::transfer::MessageBus {
    fn primitive_composition() -> PrimitiveComposition {
        // Ordered message delivery: σ (Sequence) + μ (Mapping) + ς (State)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

impl GroundsTo for crate::transfer::SpecializedWorker {
    fn primitive_composition() -> PrimitiveComposition {
        // Task-specific executor: μ (Mapping) + → (Causality) + × (Product)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Causality,
            LexPrimitiva::Product,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

impl GroundsTo for crate::transfer::DecayFunction {
    fn primitive_composition() -> PrimitiveComposition {
        // Time-based decay: ν (Frequency) + N (Quantity) + ∝ (Irreversibility)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
            LexPrimitiva::Irreversibility,
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
    }
}

impl GroundsTo for crate::transfer::Homeostasis {
    fn primitive_composition() -> PrimitiveComposition {
        // Self-regulating equilibrium: ς (State) + → (Causality) + ∂ (Boundary) + ρ (Recursion)
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::Recursion,
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for crate::transfer::StagedValidation {
    fn primitive_composition() -> PrimitiveComposition {
        // Sequential gate checks: σ (Sequence) + ∂ (Boundary) + κ (Comparison)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

impl GroundsTo for crate::transfer::Atomicity {
    fn primitive_composition() -> PrimitiveComposition {
        // All-or-nothing execution: ∂ (Boundary) + ∝ (Irreversibility)
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Irreversibility])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

impl GroundsTo for crate::transfer::CompareAndSwap {
    fn primitive_composition() -> PrimitiveComposition {
        // Atomic compare-and-swap: κ (Comparison) + ς (State) + ∂ (Boundary)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::transfer::ToctouWindow {
    fn primitive_composition() -> PrimitiveComposition {
        // Time-of-check vs time-of-use: ν (Frequency) + ∂ (Boundary) + ς (State)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::transfer::SerializationGuard {
    fn primitive_composition() -> PrimitiveComposition {
        // Ordering enforcement: σ (Sequence) + ∂ (Boundary) + ς (State)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::transfer::RateLimiter {
    fn primitive_composition() -> PrimitiveComposition {
        // Token bucket: ν (Frequency) + ∂ (Boundary) + N (Quantity)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
    }
}

impl GroundsTo for crate::transfer::BreakerState {
    fn primitive_composition() -> PrimitiveComposition {
        // Enum: Closed/Open/HalfOpen
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for crate::transfer::CircuitBreaker {
    fn primitive_composition() -> PrimitiveComposition {
        // Failure isolation pattern: ∂ (Boundary) + ς (State) + ν (Frequency) + → (Causality)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Frequency,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::transfer::Idempotency {
    fn primitive_composition() -> PrimitiveComposition {
        // Repeat-safe operations: κ (Comparison) + ς (State) + π (Persistence)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for crate::transfer::NegativeEvidence {
    fn primitive_composition() -> PrimitiveComposition {
        // Absence as signal: ∅ (Void) + κ (Comparison) + N (Quantity)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Void, 0.80)
    }
}

impl GroundsTo for crate::transfer::TopologicalAddress {
    fn primitive_composition() -> PrimitiveComposition {
        // Hierarchical location: λ (Location) + ρ (Recursion) + σ (Sequence)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Recursion,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Location, 0.80)
    }
}

impl GroundsTo for crate::transfer::Accumulator {
    fn primitive_composition() -> PrimitiveComposition {
        // Running total: Σ (Sum) + N (Quantity) + ς (State)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

impl GroundsTo for crate::transfer::Checkpoint {
    fn primitive_composition() -> PrimitiveComposition {
        // State snapshot for recovery: π (Persistence) + ς (State) + σ (Sequence)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

impl GroundsTo for crate::transfer::Decomposition {
    fn primitive_composition() -> PrimitiveComposition {
        // Breaking into sub-problems: ρ (Recursion) + σ (Sequence) + μ (Mapping)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.80)
    }
}

impl GroundsTo for crate::transfer::GenerationStage {
    fn primitive_composition() -> PrimitiveComposition {
        // Enum: stages of code generation
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for crate::transfer::CodeGeneration {
    fn primitive_composition() -> PrimitiveComposition {
        // Template to code: μ (Mapping) + σ (Sequence) + ∃ (Existence) + × (Product)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Existence,
            LexPrimitiva::Product,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
    }
}

impl GroundsTo for crate::transfer::PrimitiveMining {
    fn primitive_composition() -> PrimitiveComposition {
        // Extraction of patterns: κ (Comparison) + ρ (Recursion) + μ (Mapping) + σ (Sequence)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Recursion,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.75)
    }
}

// ============================================================================
// Dynamics Module
// ============================================================================

impl GroundsTo for crate::dynamics::Phasor {
    fn primitive_composition() -> PrimitiveComposition {
        // Rotating vector: N (Quantity) + ν (Frequency) + σ (Sequence)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

impl GroundsTo for crate::dynamics::EnvironmentalCoupling {
    fn primitive_composition() -> PrimitiveComposition {
        // Decoherence driver: ∝ (Irreversibility) + → (Causality) + ν (Frequency)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Causality,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.80)
    }
}

impl GroundsTo for crate::dynamics::Interaction {
    fn primitive_composition() -> PrimitiveComposition {
        // Coupling operator: → (Causality) + × (Product)
        PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

impl GroundsTo for crate::dynamics::Observer {
    fn primitive_composition() -> PrimitiveComposition {
        // Measurement collapse: ∝ (Irreversibility) + ∃ (Existence) + κ (Comparison)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Existence,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.80)
    }
}

// ============================================================================
// Entropy Module
// ============================================================================

/// EntropyResult: T2-P (Quantity + Irreversibility + Comparison)
///
/// Shannon entropy is a numerical measure of information (Quantity-dominant).
/// Irreversibility grounds the one-way nature of entropy increase.
/// Comparison grounds relative entropy and divergence operations.
impl GroundsTo for crate::entropy::EntropyResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,        // N — numerical entropy value
            LexPrimitiva::Irreversibility, // ∝ — entropy increase is one-way
            LexPrimitiva::Comparison,      // κ — normalized entropy compares to max
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// LogBase: T1 (Comparison) — unit selection for log base
///
/// Pure comparison: selects between bits/nats/hartleys.
impl GroundsTo for crate::entropy::LogBase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// EntropyError: T1 (Boundary) — constraint violation
impl GroundsTo for crate::entropy::EntropyError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

/// InformationLoss: T2-P (Irreversibility + Quantity)
///
/// Quantifies irreversible information loss between distributions.
impl GroundsTo for crate::entropy::InformationLoss {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // ∝ — lost information cannot be recovered
            LexPrimitiva::Quantity,        // N — measured in bits
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.85)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // -- Chemistry tests --

    #[test]
    fn threshold_gate_grounding() {
        let comp = crate::chemistry::ThresholdGate::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(comp.primitives.len(), 3);
    }

    #[test]
    fn saturation_kinetics_grounding() {
        let comp = crate::chemistry::SaturationKinetics::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn feasibility_assessment_grounding() {
        let comp = crate::chemistry::FeasibilityAssessment::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn favorability_grounding() {
        let comp = crate::chemistry::Favorability::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn rate_law_grounding() {
        let comp = crate::chemistry::RateLaw::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn buffer_system_grounding() {
        let comp = crate::chemistry::BufferSystem::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn signal_detector_grounding() {
        let comp = crate::chemistry::SignalDetector::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn decay_kinetics_grounding() {
        let comp = crate::chemistry::DecayKinetics::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
    }

    #[test]
    fn equilibrium_system_grounding() {
        let comp = crate::chemistry::EquilibriumSystem::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
    }

    #[test]
    fn cooperative_binding_grounding() {
        let comp = crate::chemistry::CooperativeBinding::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn electrochemical_cell_grounding() {
        let comp = crate::chemistry::ElectrochemicalCell::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn competitive_inhibition_grounding() {
        let comp = crate::chemistry::CompetitiveInhibition::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn transition_state_grounding() {
        let comp = crate::chemistry::TransitionState::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn langmuir_isotherm_grounding() {
        let comp = crate::chemistry::LangmuirIsotherm::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn aggregation_pipeline_grounding() {
        let comp = crate::chemistry::AggregationPipeline::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn pv_mapping_grounding() {
        let comp = crate::chemistry::PvMapping::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    // -- Quantum tests --

    #[test]
    fn amplitude_grounding() {
        let comp = crate::quantum::wave::Amplitude::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
    }

    #[test]
    fn phase_grounding() {
        let comp = crate::quantum::wave::Phase::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn interference_grounding() {
        let comp = crate::quantum::wave::Interference::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn eigenstate_grounding() {
        let comp = crate::quantum::operators::Eigenstate::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn observable_grounding() {
        let comp = crate::quantum::operators::Observable::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn hermiticity_grounding() {
        let comp = crate::quantum::operators::Hermiticity::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn unitarity_grounding() {
        let comp = crate::quantum::operators::Unitarity::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn superposition_grounding() {
        let comp = crate::quantum::state::Superposition::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn measurement_grounding() {
        let comp = crate::quantum::state::Measurement::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
    }

    #[test]
    fn uncertainty_grounding() {
        let comp = crate::quantum::state::Uncertainty::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn qubit_grounding() {
        let comp = crate::quantum::domain::Qubit::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn entanglement_grounding() {
        let comp = crate::quantum::domain::Entanglement::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn decoherence_grounding() {
        let comp = crate::quantum::domain::Decoherence::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
    }

    // -- Transfer tests --

    #[test]
    fn threat_signature_grounding() {
        let comp = crate::transfer::ThreatSignature::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn resource_ratio_grounding() {
        let comp = crate::transfer::ResourceRatio::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn feedback_loop_grounding() {
        let comp = crate::transfer::FeedbackLoop::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
    }

    #[test]
    fn homeostasis_grounding() {
        let comp = crate::transfer::Homeostasis::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn circuit_breaker_grounding() {
        let comp = crate::transfer::CircuitBreaker::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn idempotency_grounding() {
        let comp = crate::transfer::Idempotency::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
    }

    #[test]
    fn negative_evidence_grounding() {
        let comp = crate::transfer::NegativeEvidence::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Void));
    }

    #[test]
    fn checkpoint_grounding() {
        let comp = crate::transfer::Checkpoint::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
    }

    #[test]
    fn decomposition_grounding() {
        let comp = crate::transfer::Decomposition::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
    }

    #[test]
    fn code_generation_grounding() {
        let comp = crate::transfer::CodeGeneration::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn primitive_mining_grounding() {
        let comp = crate::transfer::PrimitiveMining::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
    }

    // -- Dynamics tests --

    #[test]
    fn phasor_grounding() {
        let comp = crate::dynamics::Phasor::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn environmental_coupling_grounding() {
        let comp = crate::dynamics::EnvironmentalCoupling::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
    }

    #[test]
    fn interaction_grounding() {
        let comp = crate::dynamics::Interaction::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn observer_grounding() {
        let comp = crate::dynamics::Observer::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
    }

    // -- Tier classification tests --

    #[test]
    fn chemistry_error_types_are_t2_primitive() {
        assert_eq!(crate::chemistry::ThresholdError::tier(), Tier::T2Primitive);
        assert_eq!(crate::chemistry::SaturationError::tier(), Tier::T2Primitive);
        assert_eq!(
            crate::chemistry::FeasibilityError::tier(),
            Tier::T2Primitive
        );
        assert_eq!(crate::chemistry::DependencyError::tier(), Tier::T2Primitive);
        assert_eq!(crate::chemistry::BufferError::tier(), Tier::T2Primitive);
        assert_eq!(crate::chemistry::SignalError::tier(), Tier::T2Primitive);
        assert_eq!(crate::chemistry::DecayError::tier(), Tier::T2Primitive);
    }

    #[test]
    fn quantum_core_types_are_t2_composite() {
        assert_eq!(crate::quantum::domain::Qubit::tier(), Tier::T2Composite);
        assert_eq!(
            crate::quantum::domain::Entanglement::tier(),
            Tier::T2Composite
        );
    }

    #[test]
    fn quantum_wave_types_are_t2_primitive() {
        assert_eq!(crate::quantum::wave::Amplitude::tier(), Tier::T2Primitive);
        assert_eq!(crate::quantum::wave::Phase::tier(), Tier::T2Primitive);
        assert_eq!(
            crate::quantum::wave::Interference::tier(),
            Tier::T2Primitive
        );
    }

    #[test]
    fn transfer_complex_types_are_t2_composite() {
        assert_eq!(crate::transfer::Homeostasis::tier(), Tier::T2Composite);
        assert_eq!(crate::transfer::CircuitBreaker::tier(), Tier::T2Composite);
        assert_eq!(crate::transfer::CodeGeneration::tier(), Tier::T2Composite);
        assert_eq!(crate::transfer::PrimitiveMining::tier(), Tier::T2Composite);
    }

    #[test]
    fn entropy_types_grounding() {
        use crate::entropy::{EntropyResult, LogBase};

        // EntropyResult: T2-P (Quantity + Irreversibility + Comparison)
        assert_eq!(EntropyResult::tier(), Tier::T2Primitive);
        assert_eq!(
            EntropyResult::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );

        // LogBase: T1 (Comparison) — pure unit selection
        assert_eq!(LogBase::tier(), Tier::T1Universal);
        assert!(LogBase::is_pure_primitive());
    }
}
