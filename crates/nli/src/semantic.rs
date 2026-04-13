//! Layer 2: Intent classification, entity extraction, and domain vocabulary.

use crate::{
    error::NliError,
    types::{ClassifiedIntent, IntentKind, Slot, SlotValue, Transcript},
};
use std::collections::HashMap;

/// Domain vocabulary mapping raw terms to canonical forms.
#[derive(Debug, Default)]
pub struct DomainVocabulary {
    /// drug_name → canonical drug name
    drug_map: HashMap<String, String>,
    /// raw_term → canonical MedDRA term
    meddra_map: HashMap<String, String>,
}

impl DomainVocabulary {
    /// Create an empty vocabulary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load vocabulary from a YAML file.
    ///
    /// Expected YAML structure:
    /// ```yaml
    /// drugs:
    ///   semaglutide: Semaglutide
    ///   ozempic: Semaglutide
    /// meddra:
    ///   nausea: Nausea
    ///   sick: Nausea
    /// ```
    pub fn from_yaml(path: &str) -> Result<Self, NliError> {
        let content = std::fs::read_to_string(path)?;
        let raw: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| NliError::VocabularyLoadFailure(e.to_string()))?;

        let mut vocab = Self::new();

        if let Some(drugs) = raw.get("drugs").and_then(|v| v.as_mapping()) {
            for (k, v) in drugs {
                if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                    vocab.drug_map.insert(key.to_lowercase(), val.to_string());
                }
            }
        }

        if let Some(meddra) = raw.get("meddra").and_then(|v| v.as_mapping()) {
            for (k, v) in meddra {
                if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                    vocab.meddra_map.insert(key.to_lowercase(), val.to_string());
                }
            }
        }

        Ok(vocab)
    }

    /// Load vocabulary from a YAML string (used in tests / embedded configs).
    pub fn from_yaml_str(yaml: &str) -> Result<Self, NliError> {
        let raw: serde_yaml::Value = serde_yaml::from_str(yaml)
            .map_err(|e| NliError::VocabularyLoadFailure(e.to_string()))?;

        let mut vocab = Self::new();

        if let Some(drugs) = raw.get("drugs").and_then(|v| v.as_mapping()) {
            for (k, v) in drugs {
                if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                    vocab.drug_map.insert(key.to_lowercase(), val.to_string());
                }
            }
        }

        if let Some(meddra) = raw.get("meddra").and_then(|v| v.as_mapping()) {
            for (k, v) in meddra {
                if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                    vocab.meddra_map.insert(key.to_lowercase(), val.to_string());
                }
            }
        }

        Ok(vocab)
    }

    /// Normalize a drug name to its canonical form.
    /// Returns the canonical name if found, otherwise returns the original.
    pub fn normalize_drug(&self, name: &str) -> String {
        let lower = name.to_lowercase();
        self.drug_map
            .get(&lower)
            .cloned()
            .unwrap_or_else(|| name.to_string())
    }

    /// Normalize a MedDRA term to its canonical form.
    /// Returns the canonical term if found, otherwise returns the original.
    pub fn normalize_meddra(&self, term: &str) -> String {
        let lower = term.to_lowercase();
        self.meddra_map
            .get(&lower)
            .cloned()
            .unwrap_or_else(|| term.to_string())
    }
}

/// Intent classifier trait.
#[async_trait::async_trait]
pub trait IntentClassifier: Send + Sync {
    /// Classify the intent of a transcript.
    async fn classify(&self, transcript: &Transcript) -> Result<ClassifiedIntent, NliError>;
}

/// Entity extractor trait.
#[async_trait::async_trait]
pub trait EntityExtractor: Send + Sync {
    /// Extract slots/entities from a transcript, guided by vocabulary.
    async fn extract(
        &self,
        transcript: &Transcript,
        vocab: &DomainVocabulary,
    ) -> Result<Vec<Slot>, NliError>;
}

/// Semantic engine combining classification + entity extraction.
pub struct SemanticEngine {
    classifier: Box<dyn IntentClassifier>,
    extractor: Box<dyn EntityExtractor>,
    vocabulary: DomainVocabulary,
}

impl SemanticEngine {
    /// Create a new semantic engine.
    pub fn new(
        classifier: Box<dyn IntentClassifier>,
        extractor: Box<dyn EntityExtractor>,
        vocabulary: DomainVocabulary,
    ) -> Self {
        Self {
            classifier,
            extractor,
            vocabulary,
        }
    }

    /// Process a transcript: classify intent and extract entities.
    pub async fn process(&self, transcript: &Transcript) -> Result<ClassifiedIntent, NliError> {
        let mut intent = self.classifier.classify(transcript).await?;
        let slots = self.extractor.extract(transcript, &self.vocabulary).await?;
        intent.slots = slots;
        Ok(intent)
    }
}

/// Keyword-based intent classifier (used as a fallback / in tests).
pub struct KeywordIntentClassifier;

#[async_trait::async_trait]
impl IntentClassifier for KeywordIntentClassifier {
    async fn classify(&self, transcript: &Transcript) -> Result<ClassifiedIntent, NliError> {
        let text = transcript.text.to_lowercase();

        let (kind, confidence) = if text.contains("emergency")
            || text.contains("overdose")
            || text.contains("crisis")
        {
            (IntentKind::Crisis, 0.95)
        } else if text.contains("signal") || text.contains("prr") || text.contains("ror") {
            (IntentKind::SignalDetection, 0.85)
        } else if text.contains("causal") || text.contains("naranjo") || text.contains("who-umc") {
            (IntentKind::CausalityAssessment, 0.85)
        } else if text.contains("report") || text.contains("submit") || text.contains("icsr") {
            (IntentKind::ReportSubmission, 0.80)
        } else if text.contains("adverse")
            || text.contains("reaction")
            || text.contains("safety")
            || text.contains("adr")
        {
            (IntentKind::DrugSafetyQuery, 0.75)
        } else if text.contains("help") || text.contains("how") || text.contains("what") {
            (IntentKind::Navigation, 0.70)
        } else {
            (IntentKind::Conversational, 0.50)
        };

        Ok(ClassifiedIntent::new(kind, confidence))
    }
}

/// Keyword-based entity extractor.
pub struct KeywordEntityExtractor;

#[async_trait::async_trait]
impl EntityExtractor for KeywordEntityExtractor {
    async fn extract(
        &self,
        transcript: &Transcript,
        vocab: &DomainVocabulary,
    ) -> Result<Vec<Slot>, NliError> {
        let mut slots = Vec::new();
        let words: Vec<&str> = transcript.text.split_whitespace().collect();

        // Naive: check every word against the vocabulary maps.
        for word in &words {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            let normalized_drug = vocab.normalize_drug(clean);
            if normalized_drug != clean {
                slots.push(Slot {
                    name: "drug_name".to_string(),
                    value: SlotValue::Drug(normalized_drug),
                    confidence: 0.9,
                });
            }

            let normalized_meddra = vocab.normalize_meddra(clean);
            if normalized_meddra != clean {
                slots.push(Slot {
                    name: "event_term".to_string(),
                    value: SlotValue::MedDra(normalized_meddra),
                    confidence: 0.85,
                });
            }
        }

        Ok(slots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VOCAB_YAML: &str = r#"
drugs:
  ozempic: Semaglutide
  semaglutide: Semaglutide
meddra:
  nausea: Nausea
  vomiting: Vomiting
"#;

    #[test]
    fn vocabulary_from_yaml_str() {
        let vocab = DomainVocabulary::from_yaml_str(VOCAB_YAML).unwrap();
        assert_eq!(vocab.normalize_drug("ozempic"), "Semaglutide");
        assert_eq!(vocab.normalize_meddra("nausea"), "Nausea");
    }

    #[test]
    fn vocabulary_passthrough_unknown() {
        let vocab = DomainVocabulary::from_yaml_str(VOCAB_YAML).unwrap();
        assert_eq!(vocab.normalize_drug("metformin"), "metformin");
    }

    #[tokio::test]
    async fn keyword_classifier_crisis() {
        let clf = KeywordIntentClassifier;
        let t = Transcript {
            text: "emergency overdose situation".to_string(),
            confidence: 0.9,
            speech_detected: true,
        };
        let intent = clf.classify(&t).await.unwrap();
        assert_eq!(intent.kind, IntentKind::Crisis);
    }

    #[tokio::test]
    async fn keyword_classifier_signal() {
        let clf = KeywordIntentClassifier;
        let t = Transcript {
            text: "run signal detection prr".to_string(),
            confidence: 0.9,
            speech_detected: true,
        };
        let intent = clf.classify(&t).await.unwrap();
        assert_eq!(intent.kind, IntentKind::SignalDetection);
    }
}
