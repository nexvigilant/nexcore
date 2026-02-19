//! Raw knowledge ingestion — converts diverse sources into fragments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extraction::{ConceptExtractor, ExtractedConcept, ExtractedPrimitive};
use crate::scoring::{CompendiousScorer, ScoreResult};

/// Source type for ingested knowledge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSource {
    BrainDistillation,
    BrainArtifact,
    ImplicitKnowledge,
    FreeText,
    Lesson,
    SessionReflection,
}

impl std::fmt::Display for KnowledgeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BrainDistillation => write!(f, "brain_distillation"),
            Self::BrainArtifact => write!(f, "brain_artifact"),
            Self::ImplicitKnowledge => write!(f, "implicit_knowledge"),
            Self::FreeText => write!(f, "free_text"),
            Self::Lesson => write!(f, "lesson"),
            Self::SessionReflection => write!(f, "session_reflection"),
        }
    }
}

/// Raw knowledge before processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawKnowledge {
    pub text: String,
    pub source: KnowledgeSource,
    pub domain: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// A processed knowledge fragment — the unit of knowledge in a pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFragment {
    pub id: String,
    pub text: String,
    pub source: KnowledgeSource,
    pub domain: String,
    pub concepts: Vec<ExtractedConcept>,
    pub primitives: Vec<ExtractedPrimitive>,
    pub score: ScoreResult,
    pub created_at: DateTime<Utc>,
}

/// Ingest raw text into a knowledge fragment.
pub fn ingest(raw: RawKnowledge) -> crate::error::Result<KnowledgeFragment> {
    if raw.text.trim().is_empty() {
        return Err(crate::error::KnowledgeEngineError::EmptyInput);
    }

    let concepts = ConceptExtractor::extract_concepts(&raw.text);
    let primitives = ConceptExtractor::extract_primitives(&raw.text);
    let score = CompendiousScorer::score(&raw.text, &[]);

    // Auto-classify domain from concepts if not provided
    let domain = raw.domain.unwrap_or_else(|| {
        concepts
            .iter()
            .find_map(|c| c.domain.clone())
            .unwrap_or_else(|| "general".to_string())
    });

    Ok(KnowledgeFragment {
        id: Uuid::new_v4().to_string(),
        text: raw.text,
        source: raw.source,
        domain,
        concepts,
        primitives,
        score,
        created_at: raw.timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_free_text() {
        let raw = RawKnowledge {
            text: "Signal detection uses PRR for disproportionality analysis.".to_string(),
            source: KnowledgeSource::FreeText,
            domain: None,
            timestamp: Utc::now(),
        };
        let frag = ingest(raw).unwrap();
        assert_eq!(frag.domain, "pv");
        assert!(!frag.concepts.is_empty());
        assert!(frag.score.compendious_score > 0.0);
    }

    #[test]
    fn ingest_empty_fails() {
        let raw = RawKnowledge {
            text: "  ".to_string(),
            source: KnowledgeSource::FreeText,
            domain: None,
            timestamp: Utc::now(),
        };
        assert!(ingest(raw).is_err());
    }

    #[test]
    fn ingest_with_explicit_domain() {
        let raw = RawKnowledge {
            text: "Homeostasis control loop monitors system health.".to_string(),
            source: KnowledgeSource::BrainArtifact,
            domain: Some("guardian".to_string()),
            timestamp: Utc::now(),
        };
        let frag = ingest(raw).unwrap();
        assert_eq!(frag.domain, "guardian");
    }
}
