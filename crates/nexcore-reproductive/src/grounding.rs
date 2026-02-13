//! # GroundsTo implementations for nexcore-reproductive types
//!
//! Primitive decomposition for all reproductive system types.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::claude_code::{
    BranchMutation, CiPipeline, CiStage, DeployTarget, DeploymentBirth, KnowledgeTransfer,
    MergeEvent, ReproductiveSystemHealth, ScalingEvent, TrimesterGate,
};
use crate::{
    Differentiation, Embryo, Fertilization, Gamete, GameteFitness, Mutation, ReproductiveHealth,
    Trimester,
};

// ══════════════════════════════════════════════════════════════════════════════
// Core Type Groundings
// ══════════════════════════════════════════════════════════════════════════════

impl GroundsTo for GameteFitness {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for Gamete {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Existence, 0.70)
    }
}

impl GroundsTo for Fertilization {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

impl GroundsTo for Trimester {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for Embryo {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Existence, 0.65)
    }
}

impl GroundsTo for Differentiation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.70)
    }
}

impl GroundsTo for Mutation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.70)
    }
}

impl GroundsTo for ReproductiveHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Claude Code Type Groundings
// ══════════════════════════════════════════════════════════════════════════════

impl GroundsTo for CiStage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for CiPipeline {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.70)
    }
}

impl GroundsTo for DeployTarget {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for DeploymentBirth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Causality,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Existence, 0.65)
    }
}

impl GroundsTo for BranchMutation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.70)
    }
}

impl GroundsTo for MergeEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

impl GroundsTo for TrimesterGate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

impl GroundsTo for ScalingEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Product,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
    }
}

impl GroundsTo for KnowledgeTransfer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.70)
    }
}

impl GroundsTo for ReproductiveSystemHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // Core type grounding tests

    #[test]
    fn test_gamete_fitness_grounds_to() {
        let comp = GameteFitness::primitive_composition();
        assert_eq!(comp.primitives.len(), 2);
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(comp.confidence, 0.85);
    }

    #[test]
    fn test_gamete_grounds_to() {
        let comp = Gamete::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
        assert_eq!(comp.confidence, 0.70);
    }

    #[test]
    fn test_fertilization_grounds_to() {
        let comp = Fertilization::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert_eq!(comp.confidence, 0.70);
    }

    #[test]
    fn test_trimester_grounds_to() {
        let comp = Trimester::primitive_composition();
        assert_eq!(comp.primitives.len(), 2);
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(comp.confidence, 0.85);
    }

    #[test]
    fn test_embryo_grounds_to() {
        let comp = Embryo::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
        assert_eq!(comp.confidence, 0.65);
    }

    #[test]
    fn test_differentiation_grounds_to() {
        let comp = Differentiation::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(comp.confidence, 0.70);
    }

    #[test]
    fn test_mutation_grounds_to() {
        let comp = Mutation::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert_eq!(comp.confidence, 0.70);
    }

    #[test]
    fn test_reproductive_health_grounds_to() {
        let comp = ReproductiveHealth::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.confidence, 0.65);
    }

    // Claude Code type grounding tests

    #[test]
    fn test_ci_stage_grounds_to() {
        let comp = CiStage::primitive_composition();
        assert_eq!(comp.primitives.len(), 2);
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(comp.confidence, 0.85);
    }

    #[test]
    fn test_ci_pipeline_grounds_to() {
        let comp = CiPipeline::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(comp.confidence, 0.70);
    }

    #[test]
    fn test_deploy_target_grounds_to() {
        let comp = DeployTarget::primitive_composition();
        assert_eq!(comp.primitives.len(), 2);
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(comp.confidence, 0.85);
    }

    #[test]
    fn test_deployment_birth_grounds_to() {
        let comp = DeploymentBirth::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
        assert_eq!(comp.confidence, 0.65);
    }

    #[test]
    fn test_branch_mutation_grounds_to() {
        let comp = BranchMutation::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert_eq!(comp.confidence, 0.70);
    }

    #[test]
    fn test_merge_event_grounds_to() {
        let comp = MergeEvent::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert_eq!(comp.confidence, 0.70);
    }

    #[test]
    fn test_trimester_gate_grounds_to() {
        let comp = TrimesterGate::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(comp.confidence, 0.70);
    }

    #[test]
    fn test_scaling_event_grounds_to() {
        let comp = ScalingEvent::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Product));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.confidence, 0.75);
    }

    #[test]
    fn test_knowledge_transfer_grounds_to() {
        let comp = KnowledgeTransfer::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert_eq!(comp.confidence, 0.70);
    }

    #[test]
    fn test_reproductive_system_health_grounds_to() {
        let comp = ReproductiveSystemHealth::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.confidence, 0.65);
    }
}
