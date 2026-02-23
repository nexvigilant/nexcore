//! # Typestate Pipeline
//!
//! Domain-to-code transformation pipeline using the typestate pattern.
//! Enforces compile-time stage ordering: Raw → Extracted → Classified → Generated → Validated.
//!
//! ## T1 Primitives Used
//!
//! - **State**: `Pipeline<S>` with `PhantomData` for compile-time stage tracking
//! - **Sequence**: Stages flow in order, each method returns next stage
//! - **Void**: `PhantomData<S>` - the stage marker has no runtime representation

use std::marker::PhantomData;

use super::extractor::{ExtractedPattern, ExtractionContext, ExtractionError, PatternExtractor};
use super::generator::{GeneratedCode, GeneratorError, RustCodeGenerator};
use super::languages::{LanguageClassification, LanguageClassifier};
use nexcore_error::Error;

/// Pipeline stage marker trait (sealed).
pub trait PipelineStage: private::Sealed {}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Raw {}
    impl Sealed for super::Extracted {}
    impl Sealed for super::Classified {}
    impl Sealed for super::Generated {}
    impl Sealed for super::Validated {}
}

/// Initial stage - raw text input.
#[derive(Debug, Clone, Copy)]
pub struct Raw;
impl PipelineStage for Raw {}

/// After pattern extraction.
#[derive(Debug, Clone, Copy)]
pub struct Extracted;
impl PipelineStage for Extracted {}

/// After language classification.
#[derive(Debug, Clone, Copy)]
pub struct Classified;
impl PipelineStage for Classified {}

/// After code generation.
#[derive(Debug, Clone, Copy)]
pub struct Generated;
impl PipelineStage for Generated {}

/// After validation.
#[derive(Debug, Clone, Copy)]
pub struct Validated;
impl PipelineStage for Validated {}

/// Errors during pipeline execution.
#[derive(Debug, Error)]
pub enum PipelineError {
    /// Extraction failed.
    #[error("Extraction failed: {0}")]
    Extraction(#[from] ExtractionError),

    /// Classification failed.
    #[error("Classification failed: {0}")]
    Classification(String),

    /// Generation failed.
    #[error("Generation failed: {0}")]
    Generation(String),

    /// Validation failed.
    #[error("Validation failed: {0}")]
    Validation(String),
}

impl From<GeneratorError> for PipelineError {
    fn from(e: GeneratorError) -> Self {
        Self::Generation(e.to_string())
    }
}

/// Pipeline data that accumulates through stages.
#[derive(Debug, Clone, Default)]
pub struct PipelineData {
    /// Original input text.
    pub input: String,
    /// Extraction context.
    pub context: ExtractionContext,
    /// Extracted patterns.
    pub patterns: Vec<ExtractedPattern>,
    /// Language classifications.
    pub classifications: Vec<LanguageClassification>,
    /// Generated code.
    pub generated: Vec<GeneratedCode>,
    /// Validation messages.
    pub validation_messages: Vec<String>,
    /// Whether validation passed.
    pub validation_passed: bool,
}

/// Typestate pipeline for domain-to-code transformation.
///
/// The stage type parameter `S` ensures compile-time enforcement of stage ordering.
/// You cannot call `.generate()` before `.extract()` - the types prevent it.
///
/// # Example
///
/// ```ignore
/// let result = Pipeline::new("signal_detection", context)
///     .extract(&extractor)?
///     .classify(&classifier)?
///     .generate(&generator)?
///     .validate()?;
/// ```
#[derive(Debug)]
pub struct Pipeline<S: PipelineStage> {
    /// Accumulated pipeline data.
    data: PipelineData,
    /// Stage marker (zero-sized, compile-time only).
    _stage: PhantomData<S>,
}

impl Pipeline<Raw> {
    /// Creates a new pipeline with raw input.
    #[must_use]
    pub fn new(input: impl Into<String>, context: ExtractionContext) -> Self {
        Self {
            data: PipelineData {
                input: input.into(),
                context,
                ..Default::default()
            },
            _stage: PhantomData,
        }
    }

    /// Extracts patterns from the input text.
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::Extraction` if extraction fails.
    pub fn extract(
        mut self,
        extractor: &impl PatternExtractor,
    ) -> Result<Pipeline<Extracted>, PipelineError> {
        self.data.patterns = extractor.extract(&self.data.input, &self.data.context)?;

        Ok(Pipeline {
            data: self.data,
            _stage: PhantomData,
        })
    }

    /// Returns the input text.
    #[must_use]
    pub fn input(&self) -> &str {
        &self.data.input
    }
}

impl Pipeline<Extracted> {
    /// Classifies extracted patterns into domain languages.
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::Classification` if classification fails.
    pub fn classify(
        mut self,
        classifier: &impl LanguageClassifier,
    ) -> Result<Pipeline<Classified>, PipelineError> {
        if self.data.patterns.is_empty() {
            return Err(PipelineError::Classification(
                "No patterns to classify".to_string(),
            ));
        }

        self.data.classifications = self
            .data
            .patterns
            .iter()
            .map(|p| {
                let name = p.pattern.name().unwrap_or("unnamed");
                classifier.classify(name)
            })
            .collect();

        Ok(Pipeline {
            data: self.data,
            _stage: PhantomData,
        })
    }

    /// Returns the extracted patterns.
    #[must_use]
    pub fn patterns(&self) -> &[ExtractedPattern] {
        &self.data.patterns
    }
}

impl Pipeline<Classified> {
    /// Generates Rust code from classified patterns.
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::Generation` if generation fails.
    pub fn generate(
        mut self,
        generator: &impl RustCodeGenerator,
    ) -> Result<Pipeline<Generated>, PipelineError> {
        if self.data.patterns.len() != self.data.classifications.len() {
            return Err(PipelineError::Generation(
                "Pattern/classification count mismatch".to_string(),
            ));
        }

        self.data.generated = self
            .data
            .patterns
            .iter()
            .zip(self.data.classifications.iter())
            .map(|(pattern, classification)| generator.generate(pattern, classification))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PipelineError::Generation(e.to_string()))?;

        Ok(Pipeline {
            data: self.data,
            _stage: PhantomData,
        })
    }

    /// Returns the classifications.
    #[must_use]
    pub fn classifications(&self) -> &[LanguageClassification] {
        &self.data.classifications
    }
}

impl Pipeline<Generated> {
    /// Validates the generated code.
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::Validation` if validation fails.
    pub fn validate(mut self) -> Result<Pipeline<Validated>, PipelineError> {
        // Basic validation: ensure we have generated code
        if self.data.generated.is_empty() {
            return Err(PipelineError::Validation("No code generated".to_string()));
        }

        // Validate each piece of generated code
        for code in &self.data.generated {
            if code.rust_code.is_empty() {
                self.data
                    .validation_messages
                    .push(format!("Empty code for pattern: {}", code.pattern_id));
            }
        }

        self.data.validation_passed = self.data.validation_messages.is_empty();

        Ok(Pipeline {
            data: self.data,
            _stage: PhantomData,
        })
    }

    /// Returns the generated code.
    #[must_use]
    pub fn generated_code(&self) -> &[GeneratedCode] {
        &self.data.generated
    }
}

impl Pipeline<Validated> {
    /// Extracts the final generated code.
    #[must_use]
    pub fn into_code(self) -> Vec<GeneratedCode> {
        self.data.generated
    }

    /// Returns whether validation passed.
    #[must_use]
    pub fn validation_passed(&self) -> bool {
        self.data.validation_passed
    }

    /// Returns validation messages.
    #[must_use]
    pub fn validation_messages(&self) -> &[String] {
        &self.data.validation_messages
    }

    /// Combines all generated code into a single module.
    #[must_use]
    pub fn into_module(self, module_name: &str) -> String {
        let mut output = format!("//! Generated module: {module_name}\n\n");

        for code in self.data.generated {
            output.push_str(&code.rust_code);
            output.push_str("\n\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_to_code::extractor::PrimitiveExtractor;
    use crate::domain_to_code::languages::KeywordClassifier;

    // Mock generator for testing
    struct MockGenerator;

    impl RustCodeGenerator for MockGenerator {
        fn generate(
            &self,
            pattern: &ExtractedPattern,
            _classification: &LanguageClassification,
        ) -> Result<GeneratedCode, GeneratorError> {
            Ok(GeneratedCode {
                pattern_id: pattern.id.clone(),
                rust_code: format!("// Generated from: {}\nstruct {};", pattern.id, pattern.id),
                imports: vec![],
                derives: vec!["Debug".to_string()],
            })
        }
    }

    #[test]
    fn test_pipeline_happy_path() {
        let context = ExtractionContext {
            domain: "test".to_string(),
            ..Default::default()
        };

        let extractor = PrimitiveExtractor::default();
        let classifier = KeywordClassifier::default();
        let generator = MockGenerator;

        let result = Pipeline::new("signal_detection with risk probability", context)
            .extract(&extractor)
            .and_then(|p| p.classify(&classifier))
            .and_then(|p| p.generate(&generator))
            .and_then(|p| p.validate());

        assert!(result.is_ok());
        let pipeline = result.unwrap();
        assert!(pipeline.validation_passed());
    }

    #[test]
    fn test_pipeline_extraction_error() {
        let context = ExtractionContext::new("test");
        let extractor = PrimitiveExtractor::new();

        let result = Pipeline::new("short", context).extract(&extractor);

        assert!(matches!(result, Err(PipelineError::Extraction(_))));
    }
}
