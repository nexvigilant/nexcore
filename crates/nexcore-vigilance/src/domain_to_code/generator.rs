//! # Rust Code Generator
//!
//! Generates idiomatic Rust code from extracted patterns using
//! `From`/`Into` implementations for type transformations.
//!
//! ## T1 Primitives Used
//!
//! - **Mapping**: `From`/`Into` traits for type transformations
//! - **Sequence**: Generated code follows pipeline order

use super::extractor::ExtractedPattern;
use super::languages::{DomainLanguage, LanguageClassification};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors during code generation.
#[derive(Debug, Error)]
pub enum GeneratorError {
    /// Unsupported pattern type.
    #[error("Unsupported pattern type: {0}")]
    UnsupportedPattern(String),

    /// Invalid configuration.
    #[error("Invalid generator configuration: {0}")]
    InvalidConfig(String),

    /// Template rendering failed.
    #[error("Template error: {0}")]
    Template(String),
}

/// Configuration for code generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Generate `#[derive(...)]` attributes.
    pub generate_derives: bool,
    /// Default derives to add.
    pub default_derives: Vec<String>,
    /// Generate doc comments.
    pub generate_docs: bool,
    /// Generate `impl From<...>` blocks.
    pub generate_from_impls: bool,
    /// Use `thiserror` for error types.
    pub use_thiserror: bool,
    /// Visibility for generated types.
    pub visibility: Visibility,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            generate_derives: true,
            default_derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "Serialize".to_string(),
                "Deserialize".to_string(),
            ],
            generate_docs: true,
            generate_from_impls: true,
            use_thiserror: true,
            visibility: Visibility::Public,
        }
    }
}

/// Visibility modifier for generated code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Visibility {
    /// `pub`
    #[default]
    Public,
    /// `pub(crate)`
    PubCrate,
    /// `pub(super)`
    PubSuper,
    /// private (no modifier)
    Private,
}

impl Visibility {
    /// Returns the Rust visibility string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "pub ",
            Self::PubCrate => "pub(crate) ",
            Self::PubSuper => "pub(super) ",
            Self::Private => "",
        }
    }
}

/// Generated Rust code output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    /// Pattern ID this code was generated from.
    pub pattern_id: String,
    /// The generated Rust source code.
    pub rust_code: String,
    /// Required imports/use statements.
    pub imports: Vec<String>,
    /// Derive macros used.
    pub derives: Vec<String>,
}

impl GeneratedCode {
    /// Creates a new generated code block.
    #[must_use]
    pub fn new(pattern_id: impl Into<String>, rust_code: impl Into<String>) -> Self {
        Self {
            pattern_id: pattern_id.into(),
            rust_code: rust_code.into(),
            imports: Vec::new(),
            derives: Vec::new(),
        }
    }

    /// Adds an import.
    #[must_use]
    pub fn with_import(mut self, import: impl Into<String>) -> Self {
        self.imports.push(import.into());
        self
    }

    /// Adds derives.
    #[must_use]
    pub fn with_derives(mut self, derives: Vec<String>) -> Self {
        self.derives = derives;
        self
    }

    /// Returns the full code with imports.
    #[must_use]
    pub fn full_code(&self) -> String {
        let mut output = String::new();

        for import in &self.imports {
            output.push_str(import);
            output.push('\n');
        }

        if !self.imports.is_empty() {
            output.push('\n');
        }

        output.push_str(&self.rust_code);
        output
    }
}

/// Trait for generating Rust code from patterns.
pub trait RustCodeGenerator {
    /// Generates Rust code for a single pattern.
    ///
    /// # Errors
    ///
    /// Returns `GeneratorError` if generation fails.
    fn generate(
        &self,
        pattern: &ExtractedPattern,
        classification: &LanguageClassification,
    ) -> Result<GeneratedCode, GeneratorError>;

    /// Generates code for multiple patterns.
    fn generate_batch(
        &self,
        patterns: &[(ExtractedPattern, LanguageClassification)],
    ) -> Vec<Result<GeneratedCode, GeneratorError>> {
        patterns.iter().map(|(p, c)| self.generate(p, c)).collect()
    }
}

/// Default code generator with configurable options.
#[derive(Debug, Clone)]
pub struct DefaultGenerator {
    config: GeneratorConfig,
}

impl DefaultGenerator {
    /// Creates a new generator with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GeneratorConfig::default(),
        }
    }

    /// Creates a generator with custom config.
    #[must_use]
    pub fn with_config(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Generates a struct based on language classification.
    fn generate_struct(&self, name: &str, language: DomainLanguage) -> String {
        let vis = self.config.visibility.as_str();
        let derives = if self.config.generate_derives {
            format!("#[derive({})]\n", self.config.default_derives.join(", "))
        } else {
            String::new()
        };

        let doc = if self.config.generate_docs {
            format!(
                "/// {} type generated from {} language pattern.\n",
                name, language
            )
        } else {
            String::new()
        };

        let fields = self.fields_for_language(language);

        format!("{doc}{derives}{vis}struct {name} {{\n{fields}}}\n")
    }

    /// Returns field definitions based on language.
    fn fields_for_language(&self, language: DomainLanguage) -> String {
        let vis = self.config.visibility.as_str();

        match language {
            DomainLanguage::Risk => format!(
                "    /// Probability value [0.0, 1.0].\n    {vis}probability: f64,\n    /// Confidence interval.\n    {vis}confidence: (f64, f64),\n"
            ),
            DomainLanguage::Optimization => format!(
                "    /// Objective function value.\n    {vis}objective: f64,\n    /// Active constraints.\n    {vis}constraints: Vec<String>,\n"
            ),
            DomainLanguage::Network => format!(
                "    /// Node identifier.\n    {vis}node_id: String,\n    /// Connected edges.\n    {vis}edges: Vec<String>,\n"
            ),
            DomainLanguage::Information => format!(
                "    /// Signal value.\n    {vis}signal: Vec<u8>,\n    /// Entropy estimate.\n    {vis}entropy: f64,\n"
            ),
            DomainLanguage::Resource => format!(
                "    /// Current capacity.\n    {vis}capacity: usize,\n    /// Current usage.\n    {vis}usage: usize,\n"
            ),
            DomainLanguage::Emergence => format!(
                "    /// Hierarchy level.\n    {vis}level: usize,\n    /// Emergent properties.\n    {vis}properties: Vec<String>,\n"
            ),
            DomainLanguage::Adaptation => format!(
                "    /// Current state.\n    {vis}state: String,\n    /// Learning rate.\n    {vis}learning_rate: f64,\n"
            ),
        }
    }

    /// Sanitizes a name for use as a Rust identifier.
    fn sanitize_name(name: &str) -> String {
        name.replace('-', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
    }
}

impl Default for DefaultGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RustCodeGenerator for DefaultGenerator {
    fn generate(
        &self,
        pattern: &ExtractedPattern,
        classification: &LanguageClassification,
    ) -> Result<GeneratedCode, GeneratorError> {
        let name = Self::sanitize_name(&pattern.id);

        let struct_code = self.generate_struct(&name, classification.primary);

        let mut code = GeneratedCode::new(&pattern.id, struct_code)
            .with_derives(self.config.default_derives.clone());

        // Add imports based on derives
        if self
            .config
            .default_derives
            .contains(&"Serialize".to_string())
        {
            code = code.with_import("use serde::{Serialize, Deserialize};");
        }

        Ok(code)
    }
}

/// Code emitter that writes generated code to a target.
pub trait CodeEmitter {
    /// Emits generated code to a target (file, buffer, etc.).
    ///
    /// # Errors
    ///
    /// Returns error if emission fails.
    fn emit(&self, code: &GeneratedCode) -> Result<(), std::io::Error>;
}

/// Emitter that writes to a string buffer.
#[derive(Debug, Default)]
pub struct StringEmitter {
    buffer: std::cell::RefCell<String>,
}

impl StringEmitter {
    /// Creates a new string emitter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the accumulated code.
    #[must_use]
    pub fn into_string(self) -> String {
        self.buffer.into_inner()
    }
}

impl CodeEmitter for StringEmitter {
    fn emit(&self, code: &GeneratedCode) -> Result<(), std::io::Error> {
        let mut buffer = self.buffer.borrow_mut();

        // Add imports
        for import in &code.imports {
            buffer.push_str(import);
            buffer.push('\n');
        }

        if !code.imports.is_empty() {
            buffer.push('\n');
        }

        // Add code
        buffer.push_str(&code.rust_code);
        buffer.push_str("\n\n");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_to_code::patterns::DomainPattern;

    #[test]
    fn test_default_generator() {
        let generator = DefaultGenerator::new();
        let pattern = ExtractedPattern::new(
            "TEST-001",
            DomainPattern::concept("test", DomainLanguage::Risk),
        );
        let classification = LanguageClassification::single(DomainLanguage::Risk, 0.9, "Test");

        let result = generator.generate(&pattern, &classification);
        assert!(result.is_ok());

        let code = result.expect("test: generation should succeed");
        // Sanitizer replaces - with _, so TEST-001 becomes TEST_001
        assert!(code.rust_code.contains("struct TEST_001"));
        assert!(code.rust_code.contains("probability"));
    }

    #[test]
    fn test_visibility_as_str() {
        assert_eq!(Visibility::Public.as_str(), "pub ");
        assert_eq!(Visibility::PubCrate.as_str(), "pub(crate) ");
        assert_eq!(Visibility::Private.as_str(), "");
    }

    #[test]
    fn test_string_emitter() {
        let emitter = StringEmitter::new();
        let code = GeneratedCode::new("TEST", "struct Test;").with_import("use std::io;");

        emitter.emit(&code).unwrap();

        let output = emitter.into_string();
        assert!(output.contains("use std::io;"));
        assert!(output.contains("struct Test;"));
    }

    #[test]
    fn test_generated_code_full() {
        let code = GeneratedCode::new("TEST", "struct Test;")
            .with_import("use std::io;")
            .with_import("use std::fs;");

        let full = code.full_code();
        assert!(full.starts_with("use std::io;"));
        assert!(full.contains("use std::fs;"));
        assert!(full.contains("struct Test;"));
    }
}
