//! GroundsTo implementations for all public types.
//!
//! Every type in the build orchestrator is grounded to T1 primitives
//! via the Lex Primitiva type system.

use nexcore_lex_primitiva::prelude::*;

// ============================================================================
// RunStatus — T2-P: ς + Σ, dominant ς, StateMode::Modal
// ============================================================================

impl GroundsTo for crate::pipeline::state::RunStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::State, 0.95)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ============================================================================
// PipelineStage — T2-P: Σ + σ, dominant Σ
// ============================================================================

impl GroundsTo for crate::pipeline::stage::PipelineStage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

// ============================================================================
// StageConfig — T2-C: μ + ∂ + → + λ, dominant μ
// ============================================================================

impl GroundsTo for crate::pipeline::stage::StageConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ============================================================================
// PipelineDefinition — T3: σ + μ + ∂ + → + ρ + ς, dominant σ
// ============================================================================

impl GroundsTo for crate::pipeline::definition::PipelineDefinition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::Recursion,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ============================================================================
// StageRunState — T2-C: ς + N + σ + Σ, dominant ς
// ============================================================================

impl GroundsTo for crate::pipeline::state::StageRunState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Quantity,
            LexPrimitiva::Sequence,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ============================================================================
// PipelineRunState — T3: ς + σ + N + → + π + ∂ + μ, dominant ς
// ============================================================================

impl GroundsTo for crate::pipeline::state::PipelineRunState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
            LexPrimitiva::Persistence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ============================================================================
// BuildTarget — T2-C: λ + ∃ + μ + κ, dominant λ
// ============================================================================

impl GroundsTo for crate::workspace::target::BuildTarget {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Existence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

// ============================================================================
// WorkspaceScan — T3: λ + ∃ + μ + σ + N + π, dominant λ
// ============================================================================

impl GroundsTo for crate::workspace::target::WorkspaceScan {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Existence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Location, 0.80)
    }
}

// ============================================================================
// PipelineId — T2-P: π + ∃, dominant π
// ============================================================================

impl GroundsTo for crate::types::PipelineId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Persistence, LexPrimitiva::Existence])
            .with_dominant(LexPrimitiva::Persistence, 0.90)
    }
}

// ============================================================================
// StageId — T1: σ, dominant σ
// ============================================================================

impl GroundsTo for crate::types::StageId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sequence, 1.0)
    }
}

// ============================================================================
// BuildDuration — T1: N, dominant N
// ============================================================================

impl GroundsTo for crate::types::BuildDuration {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

// ============================================================================
// LogChunk — T2-P: σ + λ, dominant σ
// ============================================================================

impl GroundsTo for crate::types::LogChunk {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

// ============================================================================
// BuildSummary — T2-C: N + Σ + κ + σ, dominant N
// ============================================================================

impl GroundsTo for crate::metrics::summary::BuildSummary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Sum,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ============================================================================
// BuildOrcError — T2-C: Σ + ∂ + σ + λ, dominant Σ
// ============================================================================

impl GroundsTo for crate::error::BuildOrcError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::Boundary,
            LexPrimitiva::Sequence,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::state::RunStatus;

    #[test]
    fn run_status_grounds_to_state() {
        assert_eq!(RunStatus::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn run_status_is_modal() {
        assert_eq!(RunStatus::state_mode(), Some(StateMode::Modal));
    }

    #[test]
    fn pipeline_id_grounds_to_persistence() {
        assert_eq!(
            crate::types::PipelineId::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn build_duration_is_pure_quantity() {
        assert!(crate::types::BuildDuration::is_pure_primitive());
    }

    #[test]
    fn pipeline_definition_is_t3() {
        assert_eq!(
            crate::pipeline::definition::PipelineDefinition::tier(),
            Tier::T3DomainSpecific
        );
    }

    #[test]
    fn stage_config_is_t2c() {
        assert_eq!(
            crate::pipeline::stage::StageConfig::tier(),
            Tier::T2Composite
        );
    }

    #[test]
    fn build_target_is_t2c() {
        assert_eq!(
            crate::workspace::target::BuildTarget::tier(),
            Tier::T2Composite
        );
    }

    #[test]
    fn all_types_have_dominant() {
        // Verify no None dominants
        assert!(RunStatus::dominant_primitive().is_some());
        assert!(crate::pipeline::stage::PipelineStage::dominant_primitive().is_some());
        assert!(crate::pipeline::stage::StageConfig::dominant_primitive().is_some());
        assert!(crate::pipeline::definition::PipelineDefinition::dominant_primitive().is_some());
        assert!(crate::pipeline::state::StageRunState::dominant_primitive().is_some());
        assert!(crate::pipeline::state::PipelineRunState::dominant_primitive().is_some());
        assert!(crate::workspace::target::BuildTarget::dominant_primitive().is_some());
        assert!(crate::workspace::target::WorkspaceScan::dominant_primitive().is_some());
        assert!(crate::types::PipelineId::dominant_primitive().is_some());
        assert!(crate::types::StageId::dominant_primitive().is_some());
        assert!(crate::types::BuildDuration::dominant_primitive().is_some());
        assert!(crate::types::LogChunk::dominant_primitive().is_some());
        assert!(crate::metrics::summary::BuildSummary::dominant_primitive().is_some());
        assert!(crate::error::BuildOrcError::dominant_primitive().is_some());
    }
}
