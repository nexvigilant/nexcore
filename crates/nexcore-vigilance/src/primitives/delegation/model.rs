//! Model definitions and capability mapping
//!
//! T1 Primitive: Mapping (Model → Capabilities)

use serde::{Deserialize, Serialize};

/// AI model variants for task routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Model {
    /// Deep reasoning, architecture decisions
    ClaudeOpus,
    /// Balanced speed/quality
    ClaudeSonnet,
    /// Fast responses, simple tasks
    ClaudeHaiku,
    /// Speed, large context, bulk generation
    GeminiFlash,
    /// Multimodal, long context
    GeminiPro,
}

impl Model {
    /// Get model's primary strength
    #[must_use]
    pub fn primary_strength(&self) -> ModelStrength {
        match self {
            Self::ClaudeOpus => ModelStrength::DeepReasoning,
            Self::ClaudeSonnet => ModelStrength::BalancedQuality,
            Self::ClaudeHaiku => ModelStrength::Speed,
            Self::GeminiFlash => ModelStrength::BulkGeneration,
            Self::GeminiPro => ModelStrength::Multimodal,
        }
    }

    /// Get all capabilities for this model
    #[must_use]
    pub fn capabilities(&self) -> Vec<ModelCapability> {
        match self {
            Self::ClaudeOpus => vec![
                ModelCapability::Architecture,
                ModelCapability::NovelProblems,
                ModelCapability::NuancedJudgment,
            ],
            Self::ClaudeSonnet => vec![
                ModelCapability::CodeGeneration,
                ModelCapability::Refactoring,
                ModelCapability::CodeReview,
            ],
            Self::ClaudeHaiku => vec![
                ModelCapability::SimpleQueries,
                ModelCapability::Classification,
            ],
            Self::GeminiFlash => vec![
                ModelCapability::BulkGeneration,
                ModelCapability::PatternMatching,
                ModelCapability::TemplateExpansion,
            ],
            Self::GeminiPro => vec![
                ModelCapability::ImageAnalysis,
                ModelCapability::DocumentProcessing,
                ModelCapability::LongContext,
            ],
        }
    }

    /// Error tolerance level (0.0 = no tolerance, 1.0 = high tolerance)
    #[must_use]
    pub fn error_tolerance(&self) -> f64 {
        match self {
            Self::ClaudeOpus => 0.1,   // Low - high stakes
            Self::ClaudeSonnet => 0.3, // Medium
            Self::ClaudeHaiku => 0.5,  // Medium-high
            Self::GeminiFlash => 0.8,  // High - volume work
            Self::GeminiPro => 0.4,    // Medium
        }
    }
}

/// Model strength categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStrength {
    DeepReasoning,
    BalancedQuality,
    Speed,
    BulkGeneration,
    Multimodal,
}

/// Specific model capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelCapability {
    // Claude Opus
    Architecture,
    NovelProblems,
    NuancedJudgment,
    // Claude Sonnet
    CodeGeneration,
    Refactoring,
    CodeReview,
    // Claude Haiku
    SimpleQueries,
    Classification,
    // Gemini Flash
    BulkGeneration,
    PatternMatching,
    TemplateExpansion,
    // Gemini Pro
    ImageAnalysis,
    DocumentProcessing,
    LongContext,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_capabilities() {
        assert!(
            Model::GeminiFlash
                .capabilities()
                .contains(&ModelCapability::BulkGeneration)
        );
        assert!(
            Model::ClaudeOpus
                .capabilities()
                .contains(&ModelCapability::Architecture)
        );
    }

    #[test]
    fn test_error_tolerance() {
        assert!(Model::GeminiFlash.error_tolerance() > Model::ClaudeOpus.error_tolerance());
    }
}
