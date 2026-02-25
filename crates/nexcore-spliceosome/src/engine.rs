// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Spliceosome engine — the pre-translation structural expectation generator.
//!
//! ## Primitive Grounding: sigma(Sequence) + boundary(d) + mapping(mu)
//!
//! The spliceosome analyzes task specifications (mapping mu) and generates
//! ordered structural expectations (sequence sigma) with boundary constraints
//! (boundary d). It runs BEFORE pipeline execution and never sees output.
//!
//! ## Biology Analog
//!
//! In biology, the spliceosome processes pre-mRNA in the nucleus before
//! the ribosome ever sees the transcript. It deposits EJC markers at exon-exon
//! junctions as a byproduct of intron removal. These markers are later consumed
//! by the NMD surveillance system during translation.
//!
//! ## Orthogonality Invariant
//!
//! The spliceosome has ZERO dependencies on brain, immunity, cytokine, or
//! ribosome crates. It operates in a cognitively independent domain.
//! This breaks the circularity of asking the LLM to detect its own errors.

use std::collections::BTreeMap;

use crate::classifier::TaskClassifier;
use crate::error::{Result, SpliceosomeError};
use crate::templates::default_templates;
use crate::types::{EjcMarker, TaskCategory, TranscriptExpectation};

/// Version string for provenance tracking.
const GENERATOR_VERSION: &str = "spliceosome-v1.0.0";

/// The spliceosome — pre-translation structural expectation generator.
///
/// Analyzes task specifications and produces [`TranscriptExpectation`]s
/// containing EJC markers. These markers are consumed by the UPF
/// surveillance complex during co-translational monitoring.
///
/// ## Design Principles
///
/// 1. **Rules engine, not ML model** — deterministic, fast, no model calls
/// 2. **Orthogonal to production** — never sees pipeline output
/// 3. **Pre-translation only** — runs before execution begins
/// 4. **Foundation layer** — 0-3 internal dependencies
#[derive(Debug, Clone)]
pub struct Spliceosome {
    classifier: TaskClassifier,
    templates: BTreeMap<TaskCategory, Vec<EjcMarker>>,
}

impl Default for Spliceosome {
    fn default() -> Self {
        Self::new()
    }
}

impl Spliceosome {
    /// Create a spliceosome with default classifier and templates.
    #[must_use]
    pub fn new() -> Self {
        Self {
            classifier: TaskClassifier::new(),
            templates: default_templates(),
        }
    }

    /// Create with custom templates (for testing or domain-specific configs).
    #[must_use]
    pub fn with_templates(templates: BTreeMap<TaskCategory, Vec<EjcMarker>>) -> Self {
        Self {
            classifier: TaskClassifier::new(),
            templates,
        }
    }

    /// Splice a task specification into structural expectations.
    ///
    /// This is the core operation: task spec in, EJC markers out.
    /// The spliceosome never sees the pipeline's actual output.
    ///
    /// # Errors
    ///
    /// Returns [`SpliceosomeError::EmptyTaskSpec`] if the spec is empty.
    /// Returns [`SpliceosomeError::TemplateNotFound`] if no template exists
    /// for the classified category.
    pub fn splice(&self, task_spec: &str) -> Result<TranscriptExpectation> {
        if task_spec.trim().is_empty() {
            return Err(SpliceosomeError::EmptyTaskSpec);
        }

        let category = self.classifier.classify(task_spec);

        let markers = self
            .templates
            .get(&category)
            .ok_or_else(|| SpliceosomeError::TemplateNotFound(category.label().into()))?
            .clone();

        let task_hash = simple_hash(task_spec);

        Ok(TranscriptExpectation {
            task_hash,
            task_category: category,
            markers,
            generator_version: GENERATOR_VERSION.into(),
            generated_at: nexcore_chrono::DateTime::now(),
        })
    }

    /// Get the classifier for direct access.
    #[must_use]
    pub fn classifier(&self) -> &TaskClassifier {
        &self.classifier
    }

    /// Get templates for inspection.
    #[must_use]
    pub fn templates(&self) -> &BTreeMap<TaskCategory, Vec<EjcMarker>> {
        &self.templates
    }

    /// Number of categories with templates.
    #[must_use]
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }
}

/// Simple deterministic hash for task spec deduplication.
/// Not cryptographic — just for cache key generation.
fn simple_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splice_explore_task() {
        let s = Spliceosome::new();
        let result = s
            .splice("read the codebase and understand the architecture")
            .unwrap();
        assert_eq!(result.task_category, TaskCategory::Explore);
        assert!(!result.markers.is_empty());
        assert_eq!(result.generator_version, GENERATOR_VERSION);
    }

    #[test]
    fn test_splice_mutate_task() {
        let s = Spliceosome::new();
        let result = s
            .splice("implement a new REST endpoint and add error handling")
            .unwrap();
        assert_eq!(result.task_category, TaskCategory::Mutate);
        // Mutate template has 3 phases: investigate, implement, verify
        assert_eq!(result.markers.len(), 3);
    }

    #[test]
    fn test_splice_empty_fails() {
        let s = Spliceosome::new();
        let result = s.splice("");
        assert!(result.is_err());
    }

    #[test]
    fn test_splice_whitespace_only_fails() {
        let s = Spliceosome::new();
        let result = s.splice("   \n\t  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_task_hash_deterministic() {
        let s = Spliceosome::new();
        let r1 = s.splice("test task").unwrap();
        let r2 = s.splice("test task").unwrap();
        assert_eq!(r1.task_hash, r2.task_hash);
    }

    #[test]
    fn test_task_hash_different_for_different_specs() {
        let s = Spliceosome::new();
        let r1 = s.splice("read files").unwrap();
        let r2 = s.splice("write code").unwrap();
        assert_ne!(r1.task_hash, r2.task_hash);
    }

    #[test]
    fn test_all_categories_produce_expectations() {
        let s = Spliceosome::new();
        let specs = [
            "read and understand the module",
            "implement a new feature with tests",
            "spawn a team to coordinate parallel work",
            "calculate PRR signal detection for aspirin",
            "run tests and validate the build",
            "navigate to the website and click the button",
            "read the code then write a fix and test it",
        ];
        for spec in specs {
            let result = s.splice(spec);
            assert!(result.is_ok(), "Failed for: {spec}");
        }
    }

    #[test]
    fn test_template_count() {
        let s = Spliceosome::new();
        assert_eq!(s.template_count(), 7); // All TaskCategory variants
    }

    #[test]
    fn test_grounding_thresholds_escalate_for_compute() {
        let s = Spliceosome::new();
        let result = s.splice("calculate drug safety signal statistics").unwrap();
        // Compute tasks should have higher grounding thresholds
        let max_threshold = result
            .markers
            .iter()
            .map(|m| m.grounding_confidence_threshold)
            .fold(0.0f32, f32::max);
        assert!(
            max_threshold >= 0.7,
            "Compute tasks need high grounding: got {max_threshold}"
        );
    }
}
