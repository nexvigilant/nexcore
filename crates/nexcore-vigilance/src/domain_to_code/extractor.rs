//! # Pattern Extractor
//!
//! Extracts domain patterns from text using T1 primitive analysis.

use super::languages::DomainLanguage;
use super::patterns::DomainPattern;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors during pattern extraction.
#[derive(Debug, Error)]
pub enum ExtractionError {
    /// No patterns found in input.
    #[error("No patterns found in input")]
    NoPatterns,

    /// Input too short for meaningful extraction.
    #[error("Input too short (minimum {min} chars, got {actual})")]
    InputTooShort {
        /// Minimum required length.
        min: usize,
        /// Actual length.
        actual: usize,
    },

    /// Invalid domain context.
    #[error("Invalid domain context: {0}")]
    InvalidContext(String),
}

/// Context for pattern extraction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractionContext {
    /// Source domain name.
    pub domain: String,

    /// Additional hints for extraction.
    #[serde(default)]
    pub hints: Vec<String>,

    /// Minimum confidence threshold.
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,

    /// Maximum patterns to extract.
    #[serde(default = "default_max_patterns")]
    pub max_patterns: usize,
}

fn default_min_confidence() -> f64 {
    0.5
}

fn default_max_patterns() -> usize {
    20
}

impl ExtractionContext {
    /// Creates a new context for a domain.
    #[must_use]
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            hints: Vec::new(),
            min_confidence: default_min_confidence(),
            max_patterns: default_max_patterns(),
        }
    }

    /// Adds extraction hints.
    #[must_use]
    pub fn with_hints(mut self, hints: Vec<String>) -> Self {
        self.hints = hints;
        self
    }

    /// Sets minimum confidence threshold.
    #[must_use]
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// An extracted pattern with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPattern {
    /// Unique identifier for this extraction.
    pub id: String,

    /// The extracted pattern.
    pub pattern: DomainPattern,

    /// Extraction confidence [0.0, 1.0].
    pub confidence: f64,

    /// Source text span (start, end).
    #[serde(default)]
    pub source_span: Option<(usize, usize)>,

    /// Extraction method used.
    #[serde(default)]
    pub method: Option<String>,
}

impl ExtractedPattern {
    /// Creates a new extracted pattern.
    #[must_use]
    pub fn new(id: impl Into<String>, pattern: DomainPattern) -> Self {
        Self {
            id: id.into(),
            pattern,
            confidence: 1.0,
            source_span: None,
            method: None,
        }
    }

    /// Sets the confidence score.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Sets the source span.
    #[must_use]
    pub fn with_span(mut self, start: usize, end: usize) -> Self {
        self.source_span = Some((start, end));
        self
    }

    /// Sets the extraction method.
    #[must_use]
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }
}

/// Trait for extracting patterns from domain text.
pub trait PatternExtractor {
    /// Extracts patterns from text.
    ///
    /// # Errors
    ///
    /// Returns `ExtractionError` if extraction fails.
    fn extract(
        &self,
        text: &str,
        context: &ExtractionContext,
    ) -> Result<Vec<ExtractedPattern>, ExtractionError>;

    /// Extracts a single dominant pattern.
    fn extract_dominant(
        &self,
        text: &str,
        context: &ExtractionContext,
    ) -> Result<ExtractedPattern, ExtractionError> {
        let patterns = self.extract(text, context)?;

        // Find pattern with highest confidence using fold instead of unwrap
        patterns
            .into_iter()
            .fold(None, |acc, p| match acc {
                None => Some(p),
                Some(best) if p.confidence > best.confidence => Some(p),
                Some(best) => Some(best),
            })
            .ok_or(ExtractionError::NoPatterns)
    }
}

/// Simple keyword-based pattern extractor.
#[derive(Debug, Default)]
pub struct PrimitiveExtractor {
    /// Minimum text length for extraction.
    pub min_length: usize,
}

impl PrimitiveExtractor {
    /// Creates a new primitive extractor.
    #[must_use]
    pub fn new() -> Self {
        Self { min_length: 10 }
    }

    /// Identifies the dominant language from text.
    fn identify_language(&self, text: &str) -> DomainLanguage {
        let text_lower = text.to_lowercase();

        // Simple keyword matching - find language with most keyword matches
        DomainLanguage::ALL
            .iter()
            .map(|&lang| {
                let count = lang
                    .keywords()
                    .iter()
                    .filter(|kw| text_lower.contains(*kw))
                    .count();
                (lang, count)
            })
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang)
            .unwrap_or(DomainLanguage::Information)
    }
}

impl PatternExtractor for PrimitiveExtractor {
    fn extract(
        &self,
        text: &str,
        context: &ExtractionContext,
    ) -> Result<Vec<ExtractedPattern>, ExtractionError> {
        if text.len() < self.min_length {
            return Err(ExtractionError::InputTooShort {
                min: self.min_length,
                actual: text.len(),
            });
        }

        let language = self.identify_language(text);

        // Create a concept pattern from the domain
        let pattern_id = format!("{}-PATTERN-001", context.domain.to_uppercase());
        let pattern = DomainPattern::concept(&context.domain, language);

        let extracted = ExtractedPattern::new(pattern_id, pattern)
            .with_confidence(0.7)
            .with_span(0, text.len())
            .with_method("primitive_keyword_extraction");

        Ok(vec![extracted])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_context() {
        let ctx = ExtractionContext::new("pharmacovigilance")
            .with_hints(vec!["safety".to_string()])
            .with_min_confidence(0.7);

        assert_eq!(ctx.domain, "pharmacovigilance");
        assert_eq!(ctx.hints.len(), 1);
        assert!((ctx.min_confidence - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_extracted_pattern() {
        let pattern = DomainPattern::concept("risk_score", DomainLanguage::Risk);
        let extracted = ExtractedPattern::new("TEST-001", pattern)
            .with_confidence(0.85)
            .with_span(10, 50);

        assert_eq!(extracted.id, "TEST-001");
        assert!((extracted.confidence - 0.85).abs() < 0.001);
        assert_eq!(extracted.source_span, Some((10, 50)));
    }

    #[test]
    fn test_primitive_extractor() {
        let extractor = PrimitiveExtractor::new();
        let context = ExtractionContext::new("test_domain");

        let result = extractor.extract(
            "Calculate the probability of an adverse event occurring",
            &context,
        );

        assert!(result.is_ok());
        let patterns = result.unwrap();
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn test_primitive_extractor_too_short() {
        let extractor = PrimitiveExtractor::new();
        let context = ExtractionContext::new("test");

        let result = extractor.extract("short", &context);

        assert!(matches!(result, Err(ExtractionError::InputTooShort { .. })));
    }
}
