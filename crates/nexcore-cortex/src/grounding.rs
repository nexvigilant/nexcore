//! # Lex Primitiva Grounding
//!
//! GroundsTo implementations for all nexcore-cortex public types.
//!
//! | Type | Primitives | Dominant | Tier |
//! |------|-----------|----------|------|
//! | InferenceEngine | σ μ ς N → ∂ | μ | T3 |
//! | ModelConfig | ς π ∃ | ς | T2-P |
//! | ModelFormat | ς ∂ | ς | T2-P |
//! | GenerateParams | N ∂ ν | N | T2-P |
//! | LoraConfig | N μ ∂ | N | T2-P |
//! | FineTuneJob | σ ς π N | σ | T2-C |
//! | CortexTokenizer | μ σ | μ | T2-P |
//! | LoraAdapter | N μ ∂ ς π | N | T2-C |
//! | DatasetConfig | σ N | σ | T2-P |
//! | TrainingParams | N ∂ | N | T2-P |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::cloud::{DatasetConfig, FineTuneJob, TrainingParams};
use crate::generate::GenerateParams;
use crate::lora::{LoraAdapter, LoraConfig};
use crate::model::{ModelConfig, ModelFormat};

// ============================================================================
// T2-P (2-3 unique primitives)
// ============================================================================

impl GroundsTo for ModelConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::State, 0.90)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for ModelFormat {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::State, 0.92)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for GenerateParams {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.88)
    }
}

impl GroundsTo for LoraConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

impl GroundsTo for DatasetConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

impl GroundsTo for TrainingParams {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Quantity, 0.92)
    }
}

// ============================================================================
// T2-C (4-5 unique primitives)
// ============================================================================

impl GroundsTo for LoraAdapter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for FineTuneJob {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.88)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn test_model_config_grounding() {
        let comp = ModelConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.primitives.len(), 3);
        assert!(matches!(ModelConfig::tier(), Tier::T2Primitive));
    }

    #[test]
    fn test_model_format_grounding() {
        let comp = ModelFormat::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.primitives.len(), 2);
    }

    #[test]
    fn test_generate_params_grounding() {
        let comp = GenerateParams::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.primitives.len(), 3);
        assert!(matches!(GenerateParams::tier(), Tier::T2Primitive));
    }

    #[test]
    fn test_lora_config_grounding() {
        let comp = LoraConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.primitives.len(), 3);
    }

    #[test]
    fn test_lora_adapter_grounding() {
        let comp = LoraAdapter::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.primitives.len(), 5);
        assert!(matches!(LoraAdapter::tier(), Tier::T2Composite));
    }

    #[test]
    fn test_fine_tune_job_grounding() {
        let comp = FineTuneJob::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(comp.primitives.len(), 4);
        assert!(matches!(FineTuneJob::tier(), Tier::T2Composite));
    }

    #[test]
    fn test_dataset_config_grounding() {
        let comp = DatasetConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(comp.primitives.len(), 2);
    }

    #[test]
    fn test_training_params_grounding() {
        let comp = TrainingParams::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.primitives.len(), 2);
    }
}
