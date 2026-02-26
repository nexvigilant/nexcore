//! Raw knowledge ingestion — converts diverse sources into fragments.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

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
    pub timestamp: DateTime,
}

/// A processed knowledge fragment — the unit of knowledge in a pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFragment {
    pub id: crate::KnowledgeId,
    pub text: String,
    pub source: KnowledgeSource,
    pub domain: String,
    pub concepts: Vec<ExtractedConcept>,
    pub primitives: Vec<ExtractedPrimitive>,
    pub score: ScoreResult,
    pub created_at: DateTime,
}

/// Ingest raw text into a knowledge fragment.
///
/// Extracts concepts, primitives, and computes the Compendious Score.
/// Domain is auto-classified from concepts if not provided.
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use nexcore_knowledge_engine::ingest::{ingest, RawKnowledge, KnowledgeSource};
/// use nexcore_chrono::DateTime;
///
/// let raw = RawKnowledge {
///     text: "Signal detection uses PRR for safety analysis.".to_string(),
///     source: KnowledgeSource::FreeText,
///     domain: None,
///     timestamp: DateTime::now(),
/// };
/// let frag = ingest(raw)?;
/// assert_eq!(frag.domain, "pv");
/// assert!(frag.score.compendious_score > 0.0);
/// # Ok(())
/// # }
/// ```
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
        id: NexId::v4().to_string(),
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
            timestamp: DateTime::now(),
        };
        let frag = ingest(raw).unwrap();
        assert_eq!(frag.domain, "pv");
        // The text has 7 unique non-stopword tokens in 8 words: I = 7*4 = 28, E = 8
        // density = 3.5, C = 1.0, R = 1.0 → Cs ≥ 2.0 (Efficient or better)
        assert!(
            frag.score.compendious_score >= 2.0,
            "dense PV text should score >= 2.0, got {:.3}",
            frag.score.compendious_score
        );
        // "signal" and "detection" must be extracted as concepts
        let terms: Vec<&str> = frag.concepts.iter().map(|c| c.term.as_str()).collect();
        assert!(
            terms.contains(&"signal") || terms.contains(&"detection"),
            "expected pv-domain concepts in: {:?}",
            terms
        );
    }

    #[test]
    fn ingest_empty_fails() {
        let raw = RawKnowledge {
            text: "  ".to_string(),
            source: KnowledgeSource::FreeText,
            domain: None,
            timestamp: DateTime::now(),
        };
        assert!(ingest(raw).is_err());
    }

    #[test]
    fn ingest_with_explicit_domain() {
        let raw = RawKnowledge {
            text: "Homeostasis control loop monitors system health.".to_string(),
            source: KnowledgeSource::BrainArtifact,
            domain: Some("guardian".to_string()),
            timestamp: DateTime::now(),
        };
        let frag = ingest(raw).unwrap();
        assert_eq!(frag.domain, "guardian");
    }
}
