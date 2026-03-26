// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Pipeline Stage Configuration
//!
//! Typed configuration for the course generation pipeline stages.
//! Migrated from `studio/backend/app/services/stage_config_service.py`.
//!
//! ## Primitive Grounding
//!
//! | Component | Primitive | Meaning |
//! |-----------|-----------|---------|
//! | Stage enum | Σ | Sum type over pipeline phases |
//! | Progress range | ∂ | Boundary of each stage's progress window |
//! | Stage order | σ | Sequence dependency between stages |
//! | Duration/tokens | N | Quantified resource limits |

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Pipeline Stages
// ---------------------------------------------------------------------------

/// Course generation pipeline stages.
///
/// Two pipeline variants exist: Legacy (prototyping) and Quality (production).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PipelineStage {
    // --- Legacy Pipeline ---
    /// Decompose topic into Knowledge, Skills, Behaviors.
    KsbDecomposition,
    /// Research each KSB component.
    Research,
    /// Generate lesson content from research.
    ContentGeneration,
    /// Validate content quality via AI agents.
    QualityValidation,
    /// Format content into Academy JSON structure.
    AcademyFormatting,
    /// Validate course against 43 checks.
    CourseValidation,
    /// Upload to Firestore.
    AcademyUpload,

    // --- Quality Pipeline ---
    /// Research-validate KSBs against library.
    KsbValidation,
    /// High-fidelity research with caching.
    HighQualityResearch,
    /// Generate individual lessons.
    LessonGeneration,
    /// Validate individual lessons.
    LessonValidation,
    /// Publish to Academy.
    AcademyPublishing,
    /// Update KSB library with new entries.
    KsbLibraryUpdate,
}

impl PipelineStage {
    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::KsbDecomposition => "KSB Decomposition",
            Self::Research => "Research",
            Self::ContentGeneration => "Content Generation",
            Self::QualityValidation => "Quality Validation",
            Self::AcademyFormatting => "Academy Formatting",
            Self::CourseValidation => "Course Validation",
            Self::AcademyUpload => "Academy Upload",
            Self::KsbValidation => "KSB Validation",
            Self::HighQualityResearch => "High-Quality Research",
            Self::LessonGeneration => "Lesson Generation",
            Self::LessonValidation => "Lesson Validation",
            Self::AcademyPublishing => "Academy Publishing",
            Self::KsbLibraryUpdate => "KSB Library Update",
        }
    }

    /// Progress percentage range [start, end] for this stage.
    pub fn progress_range(&self) -> (u8, u8) {
        match self {
            Self::KsbDecomposition => (0, 20),
            Self::Research => (20, 40),
            Self::ContentGeneration => (40, 70),
            Self::QualityValidation => (70, 85),
            Self::AcademyFormatting => (85, 90),
            Self::CourseValidation => (85, 90),
            Self::AcademyUpload => (90, 100),
            Self::KsbValidation => (0, 15),
            Self::HighQualityResearch => (15, 40),
            Self::LessonGeneration => (40, 70),
            Self::LessonValidation => (70, 85),
            Self::AcademyPublishing => (85, 100),
            Self::KsbLibraryUpdate => (85, 100),
        }
    }

    /// Whether this stage requires AI model calls.
    pub fn requires_ai(&self) -> bool {
        !matches!(
            self,
            Self::AcademyFormatting
                | Self::AcademyUpload
                | Self::AcademyPublishing
                | Self::KsbLibraryUpdate
        )
    }

    /// Legacy pipeline stages in order.
    pub const LEGACY: [PipelineStage; 7] = [
        Self::KsbDecomposition,
        Self::Research,
        Self::ContentGeneration,
        Self::QualityValidation,
        Self::AcademyFormatting,
        Self::CourseValidation,
        Self::AcademyUpload,
    ];

    /// Quality pipeline stages in order.
    pub const QUALITY: [PipelineStage; 6] = [
        Self::KsbValidation,
        Self::HighQualityResearch,
        Self::LessonGeneration,
        Self::LessonValidation,
        Self::AcademyPublishing,
        Self::KsbLibraryUpdate,
    ];
}

// ---------------------------------------------------------------------------
// AI Model Configuration
// ---------------------------------------------------------------------------

/// AI model configuration for a pipeline stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelConfig {
    /// Model identifier (e.g., "claude-sonnet-4-5-20250929").
    pub model: String,
    /// Sampling temperature (0.0–2.0).
    pub temperature: f32,
    /// Maximum output tokens.
    pub max_tokens: u32,
}

/// Pre-configured AI model settings.
pub mod models {
    use super::AiModelConfig;

    /// Claude Sonnet for general content generation.
    pub fn claude_sonnet() -> AiModelConfig {
        AiModelConfig {
            model: "claude-sonnet-4-5-20250929".into(),
            temperature: 1.0,
            max_tokens: 8000,
        }
    }

    /// Claude Sonnet with low temperature for validation.
    pub fn claude_sonnet_validation() -> AiModelConfig {
        AiModelConfig {
            model: "claude-sonnet-4-5-20250929".into(),
            temperature: 0.3,
            max_tokens: 8000,
        }
    }

    /// Perplexity Sonar Pro for research.
    pub fn perplexity_sonar() -> AiModelConfig {
        AiModelConfig {
            model: "sonar-pro".into(),
            temperature: 0.7,
            max_tokens: 4000,
        }
    }
}

// ---------------------------------------------------------------------------
// Quality Validation Agent Configuration
// ---------------------------------------------------------------------------

/// Configuration for a quality validation agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationAgentConfig {
    /// Agent name (identifier).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Weight in overall score calculation (0.0–1.0, all must sum to 1.0).
    pub weight: f32,
    /// Minimum passing score for this agent (0–100).
    pub pass_threshold: u32,
}

/// Default quality validation agent configurations.
///
/// 6 agents with weights summing to 1.0.
pub fn default_validation_agents() -> Vec<ValidationAgentConfig> {
    vec![
        ValidationAgentConfig {
            name: "content_quality".into(),
            description: "Content Quality & Accuracy".into(),
            weight: 0.25,
            pass_threshold: 70,
        },
        ValidationAgentConfig {
            name: "pedagogical_soundness".into(),
            description: "Pedagogical Design & Learning Effectiveness".into(),
            weight: 0.20,
            pass_threshold: 70,
        },
        ValidationAgentConfig {
            name: "accessibility".into(),
            description: "Accessibility & Inclusive Design".into(),
            weight: 0.15,
            pass_threshold: 70,
        },
        ValidationAgentConfig {
            name: "technical_accuracy".into(),
            description: "Technical Accuracy & Currency".into(),
            weight: 0.15,
            pass_threshold: 70,
        },
        ValidationAgentConfig {
            name: "engagement".into(),
            description: "Learner Engagement & Motivation".into(),
            weight: 0.15,
            pass_threshold: 70,
        },
        ValidationAgentConfig {
            name: "assessment_quality".into(),
            description: "Assessment Design & Alignment".into(),
            weight: 0.10,
            pass_threshold: 70,
        },
    ]
}

/// Compute weighted overall score from individual agent scores.
///
/// Returns None if agents list is empty or weights don't match.
pub fn compute_weighted_score(agents: &[ValidationAgentConfig], scores: &[f64]) -> Option<f64> {
    if agents.len() != scores.len() || agents.is_empty() {
        return None;
    }

    let weighted_sum: f64 = agents
        .iter()
        .zip(scores.iter())
        .map(|(agent, &score)| f64::from(agent.weight) * score)
        .sum();

    Some(weighted_sum)
}

// ---------------------------------------------------------------------------
// Research Cache Configuration
// ---------------------------------------------------------------------------

/// Research cache configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether caching is enabled.
    pub enabled: bool,
    /// Default TTL in days.
    pub ttl_days: u32,
    /// Extended TTL for popular topics.
    pub popular_ttl_days: u32,
    /// Request timeout in seconds.
    pub timeout_seconds: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_days: 30,
            popular_ttl_days: 90,
            timeout_seconds: 300,
        }
    }
}

// ---------------------------------------------------------------------------
// Overall Quality Thresholds
// ---------------------------------------------------------------------------

/// Quality thresholds for the validation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum score to deploy (0–100).
    pub deploy_threshold: f64,
    /// Minimum score for auto-fix attempt (0–100).
    pub autofix_threshold: f64,
    /// Minimum per-agent passing score.
    pub agent_pass_threshold: f64,
    /// Maximum retry attempts.
    pub max_retries: u32,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            deploy_threshold: 85.0,
            autofix_threshold: 70.0,
            agent_pass_threshold: 70.0,
            max_retries: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_display_names() {
        assert_eq!(
            PipelineStage::KsbDecomposition.display_name(),
            "KSB Decomposition"
        );
        assert_eq!(
            PipelineStage::AcademyUpload.display_name(),
            "Academy Upload"
        );
    }

    #[test]
    fn stage_progress_ranges() {
        let (start, end) = PipelineStage::Research.progress_range();
        assert_eq!(start, 20);
        assert_eq!(end, 40);
    }

    #[test]
    fn formatting_does_not_require_ai() {
        assert!(!PipelineStage::AcademyFormatting.requires_ai());
        assert!(PipelineStage::ContentGeneration.requires_ai());
    }

    #[test]
    fn validation_agent_weights_sum_to_one() {
        let agents = default_validation_agents();
        let total: f32 = agents.iter().map(|a| a.weight).sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn weighted_score_computation() {
        let agents = default_validation_agents();
        let scores = vec![90.0, 85.0, 80.0, 88.0, 75.0, 92.0];
        let result = compute_weighted_score(&agents, &scores);
        assert!(result.is_some());
        let score = result.expect("score should be Some");
        // Expected: 90*0.25 + 85*0.20 + 80*0.15 + 88*0.15 + 75*0.15 + 92*0.10
        //         = 22.5 + 17.0 + 12.0 + 13.2 + 11.25 + 9.2 = 85.15
        assert!((score - 85.15).abs() < 0.01);
    }

    #[test]
    fn weighted_score_mismatched_lengths() {
        let agents = default_validation_agents();
        let scores = vec![90.0]; // Wrong length
        assert!(compute_weighted_score(&agents, &scores).is_none());
    }

    #[test]
    fn default_thresholds() {
        let t = QualityThresholds::default();
        assert_eq!(t.deploy_threshold, 85.0);
        assert_eq!(t.max_retries, 3);
    }
}
