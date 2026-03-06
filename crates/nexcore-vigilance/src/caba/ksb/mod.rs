//! Knowledge-Skills-Behaviors (KSB) Taxonomy
//!
//! The atomic unit of PV competency. Each KSB is classified by type and subtype.
//! (source: ~/Vaults/nexvigilant/400-projects/ksb-framework/)
//!
//! ## KSB Types
//!
//! - **Knowledge**: Facts, concepts, principles, mental models (K1-K3)
//! - **Skills**: Procedures, techniques, tool usage (S1-S3)
//! - **Behaviors**: Patterns, heuristics, decision approaches (B1-B3)
//! - **AI Integration**: Human-AI collaboration points (A1-A3)

use crate::caba::{Score, ScoreError};
use serde::{Deserialize, Serialize};

/// Knowledge-Skills-Behaviors classification.
///
/// # L0 Quark - Type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KsbType {
    // Knowledge Types (K1-K3)
    /// K1: Facts, concepts, principles
    #[serde(rename = "K1_Declarative")]
    KnowledgeDeclarative,
    /// K2: Mental models, frameworks
    #[serde(rename = "K2_Conceptual")]
    KnowledgeConceptual,
    /// K3: Step-by-step processes
    #[serde(rename = "K3_Procedural")]
    KnowledgeProcedural,

    // Skill Types (S1-S3)
    /// S1: Tool usage, coding
    #[serde(rename = "S1_Technical")]
    SkillTechnical,
    /// S2: Problem analysis, debugging
    #[serde(rename = "S2_Analytical")]
    SkillAnalytical,
    /// S3: System connection, orchestration
    #[serde(rename = "S3_Integration")]
    SkillIntegration,

    // Behavior Types (B1-B3)
    /// B1: Failure management approaches
    #[serde(rename = "B1_Error_Handling")]
    BehaviorErrorHandling,
    /// B2: Testing, validation
    #[serde(rename = "B2_Quality_Assurance")]
    BehaviorQualityAssurance,
    /// B3: Performance patterns
    #[serde(rename = "B3_Optimization")]
    BehaviorOptimization,

    // AI Integration Types (A1-A3)
    // (source: primitives/ksb.rs documents 233 AI Integration KSBs per domain)
    /// A1: AI tool selection and prompt engineering
    #[serde(rename = "A1_Tool_Selection")]
    AiToolSelection,
    /// A2: Human-AI workflow orchestration
    #[serde(rename = "A2_Workflow_Orchestration")]
    AiWorkflowOrchestration,
    /// A3: AI output validation and oversight
    #[serde(rename = "A3_Validation_Oversight")]
    AiValidationOversight,
}

impl KsbType {
    /// Get single-letter type prefix (K, S, B, or A).
    #[must_use]
    pub fn prefix(&self) -> char {
        match self {
            Self::KnowledgeDeclarative | Self::KnowledgeConceptual | Self::KnowledgeProcedural => {
                'K'
            }
            Self::SkillTechnical | Self::SkillAnalytical | Self::SkillIntegration => 'S',
            Self::BehaviorErrorHandling
            | Self::BehaviorQualityAssurance
            | Self::BehaviorOptimization => 'B',
            Self::AiToolSelection | Self::AiWorkflowOrchestration | Self::AiValidationOversight => {
                'A'
            }
        }
    }

    /// Check if this is a Knowledge type.
    #[must_use]
    pub fn is_knowledge(&self) -> bool {
        self.prefix() == 'K'
    }

    /// Check if this is a Skill type.
    #[must_use]
    pub fn is_skill(&self) -> bool {
        self.prefix() == 'S'
    }

    /// Check if this is a Behavior type.
    #[must_use]
    pub fn is_behavior(&self) -> bool {
        self.prefix() == 'B'
    }

    /// Check if this is an AI Integration type.
    #[must_use]
    pub fn is_ai_integration(&self) -> bool {
        self.prefix() == 'A'
    }
}

/// Specification for a required KSB.
///
/// Input to the KSB Research Engine - what knowledge, skill,
/// or behavior needs to be researched and codified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbRequirement {
    /// Requirement title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// KSB classification
    pub ksb_type: KsbType,
    /// Domain context
    pub domain: String,
    /// Usage context
    pub context: String,
    /// Target proficiency level (default: "competent")
    #[serde(default = "default_proficiency")]
    pub proficiency_target: String,
    /// Prerequisite KSB IDs
    #[serde(default)]
    pub prerequisites: Vec<String>,
}

fn default_proficiency() -> String {
    "competent".to_string()
}

/// Documentation of a research source used to codify a KSB.
///
/// Tracks where information came from and its quality/relevance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSource {
    /// Source URL
    pub url: String,
    /// Source title
    pub title: String,
    /// Relevance score [0.0, 1.0]
    pub relevance_score: Score,
    /// Key extracts from the source
    pub key_extracts: Vec<String>,
    /// Source type: "documentation", "tutorial", "best_practice", "academic"
    pub source_type: String,
    /// Authority score [0.0, 1.0]
    pub authority_score: Score,
}

impl ResearchSource {
    /// Create a new research source with score validation.
    ///
    /// # Errors
    /// Returns error if scores are not in [0.0, 1.0] range.
    pub fn new(
        url: String,
        title: String,
        relevance: f64,
        authority: f64,
        source_type: String,
    ) -> Result<Self, ScoreError> {
        Ok(Self {
            url,
            title,
            relevance_score: Score::new(relevance)?,
            key_extracts: Vec::new(),
            source_type,
            authority_score: Score::new(authority)?,
        })
    }
}

/// Fully codified KSB ready for competency integration.
///
/// Output of the KSB Research Engine - a machine-executable
/// representation of knowledge, skill, or behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodedKsb {
    /// Unique identifier (format: K-XXX-001, S-XXX-001, B-XXX-001)
    pub id: String,
    /// KSB classification
    pub ksb_type: KsbType,
    /// Category within the KSB type
    pub category: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Structured content (domain-specific)
    pub structured_content: serde_json::Value,
    /// Code representation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_representation: Option<String>,
    /// Decision logic (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_logic: Option<serde_json::Value>,
    /// Templates
    #[serde(default)]
    pub templates: Vec<serde_json::Value>,
    /// Validation tests
    #[serde(default)]
    pub validation_tests: Vec<serde_json::Value>,
    /// Quality criteria
    #[serde(default)]
    pub quality_criteria: serde_json::Value,
    /// Domain context
    #[serde(default)]
    pub domain: String,
    /// Prerequisite KSB IDs
    #[serde(default)]
    pub prerequisites: Vec<String>,
    /// Automation potential [0.0, 1.0]
    #[serde(default)]
    pub automation_potential: Score,
    /// Research sources
    #[serde(default)]
    pub research_sources: Vec<ResearchSource>,
    /// Confidence score [0.0, 1.0]
    #[serde(default)]
    pub confidence_score: Score,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last validation timestamp (ISO 8601)
    pub last_validated: String,
    /// Version string
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

impl CodedKsb {
    /// Check if this is a Knowledge KSB.
    #[must_use]
    pub fn is_knowledge(&self) -> bool {
        self.ksb_type.is_knowledge()
    }

    /// Check if this is a Skill KSB.
    #[must_use]
    pub fn is_skill(&self) -> bool {
        self.ksb_type.is_skill()
    }

    /// Check if this is a Behavior KSB.
    #[must_use]
    pub fn is_behavior(&self) -> bool {
        self.ksb_type.is_behavior()
    }

    /// Check if this is an AI Integration KSB.
    #[must_use]
    pub fn is_ai_integration(&self) -> bool {
        self.ksb_type.is_ai_integration()
    }

    /// Get the single-letter type prefix (K, S, B, or A).
    #[must_use]
    pub fn type_prefix(&self) -> char {
        self.ksb_type.prefix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ksb_type_prefix() {
        assert_eq!(KsbType::KnowledgeDeclarative.prefix(), 'K');
        assert_eq!(KsbType::SkillTechnical.prefix(), 'S');
        assert_eq!(KsbType::BehaviorErrorHandling.prefix(), 'B');
        assert_eq!(KsbType::AiToolSelection.prefix(), 'A');
    }

    #[test]
    fn test_ksb_type_classification() {
        assert!(KsbType::KnowledgeConceptual.is_knowledge());
        assert!(!KsbType::KnowledgeConceptual.is_skill());
        assert!(KsbType::SkillAnalytical.is_skill());
        assert!(KsbType::BehaviorOptimization.is_behavior());
        assert!(KsbType::AiWorkflowOrchestration.is_ai_integration());
        assert!(!KsbType::AiValidationOversight.is_knowledge());
    }

    #[test]
    fn test_research_source_validation() {
        let valid = ResearchSource::new(
            "https://example.com".to_string(),
            "Test Source".to_string(),
            0.8,
            0.9,
            "documentation".to_string(),
        );
        assert!(valid.is_ok());

        let invalid = ResearchSource::new(
            "https://example.com".to_string(),
            "Test Source".to_string(),
            1.5, // Out of bounds
            0.9,
            "documentation".to_string(),
        );
        assert!(invalid.is_err());
    }
}
