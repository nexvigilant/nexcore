//! # CEP Validation Types
//!
//! Validation metrics for primitive extraction: coverage, minimality, independence.
//! Patent: NV-2026-002 (thresholds: ≥0.95, ≥0.90, ≥0.90)

use serde::{Deserialize, Serialize};

/// Coverage score: proportion of concepts expressible from primitives.
/// Target: ≥ 0.95
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CoverageScore(pub f64);

impl CoverageScore {
    /// Creates a new coverage score.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Checks if coverage meets threshold.
    #[must_use]
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }

    /// Checks if coverage meets default threshold (0.95).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.meets_threshold(super::DEFAULT_COVERAGE_THRESHOLD)
    }
}

/// Minimality score: absence of redundant primitives.
/// Target: ≥ 0.90
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MinimalityScore(pub f64);

impl MinimalityScore {
    /// Creates a new minimality score.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Checks if minimality meets threshold.
    #[must_use]
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }

    /// Checks if minimality meets default threshold (0.90).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.meets_threshold(super::DEFAULT_MINIMALITY_THRESHOLD)
    }

    /// Calculates minimality from redundant count.
    /// minimality = 1 - (redundant / total)
    #[must_use]
    pub fn from_redundant_count(redundant: usize, total: usize) -> Self {
        if total == 0 {
            return Self(1.0);
        }
        Self::new(1.0 - (redundant as f64 / total as f64))
    }
}

/// Independence score: absence of implied relationships between primitives.
/// Target: ≥ 0.90
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IndependenceScore(pub f64);

impl IndependenceScore {
    /// Creates a new independence score.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Checks if independence meets threshold.
    #[must_use]
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }

    /// Checks if independence meets default threshold (0.90).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.meets_threshold(super::DEFAULT_INDEPENDENCE_THRESHOLD)
    }

    /// Calculates independence from implied pair count.
    /// independence = 1 - (implied_pairs / total_pairs)
    #[must_use]
    pub fn from_implied_pairs(implied: usize, total_pairs: usize) -> Self {
        if total_pairs == 0 {
            return Self(1.0);
        }
        Self::new(1.0 - (implied as f64 / total_pairs as f64))
    }
}

/// Validation thresholds configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ValidationThresholds {
    /// Coverage threshold (default: 0.95).
    pub coverage: f64,
    /// Minimality threshold (default: 0.90).
    pub minimality: f64,
    /// Independence threshold (default: 0.90).
    pub independence: f64,
}

impl Default for ValidationThresholds {
    fn default() -> Self {
        Self {
            coverage: super::DEFAULT_COVERAGE_THRESHOLD,
            minimality: super::DEFAULT_MINIMALITY_THRESHOLD,
            independence: super::DEFAULT_INDEPENDENCE_THRESHOLD,
        }
    }
}

/// Complete extraction validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionValidation {
    /// Coverage score.
    pub coverage: CoverageScore,
    /// Minimality score.
    pub minimality: MinimalityScore,
    /// Independence score.
    pub independence: IndependenceScore,
    /// Thresholds used for validation.
    pub thresholds: ValidationThresholds,
    /// Whether all thresholds are met.
    pub is_valid: bool,
    /// Issues found during validation.
    #[serde(default)]
    pub issues: Vec<ValidationIssue>,
}

impl ExtractionValidation {
    /// Creates a new validation result.
    #[must_use]
    pub fn new(coverage: f64, minimality: f64, independence: f64) -> Self {
        Self::with_thresholds(
            coverage,
            minimality,
            independence,
            ValidationThresholds::default(),
        )
    }

    /// Creates a validation result with custom thresholds.
    #[must_use]
    pub fn with_thresholds(
        coverage: f64,
        minimality: f64,
        independence: f64,
        thresholds: ValidationThresholds,
    ) -> Self {
        let cov = CoverageScore::new(coverage);
        let min = MinimalityScore::new(minimality);
        let ind = IndependenceScore::new(independence);

        let is_valid = cov.meets_threshold(thresholds.coverage)
            && min.meets_threshold(thresholds.minimality)
            && ind.meets_threshold(thresholds.independence);

        Self {
            coverage: cov,
            minimality: min,
            independence: ind,
            thresholds,
            is_valid,
            issues: Vec::new(),
        }
    }

    /// Adds a validation issue.
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Returns the weakest metric.
    #[must_use]
    pub fn weakest_metric(&self) -> (&'static str, f64) {
        let metrics = [
            ("coverage", self.coverage.0),
            ("minimality", self.minimality.0),
            ("independence", self.independence.0),
        ];
        metrics
            .into_iter()
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or(("unknown", 0.0))
    }
}

/// A validation issue or warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue category.
    pub category: ValidationCategory,
    /// Severity (0.0-1.0).
    pub severity: f64,
    /// Description.
    pub description: String,
    /// Affected primitives.
    #[serde(default)]
    pub affected: Vec<String>,
}

/// Category of validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationCategory {
    /// Coverage gap - concepts not expressible.
    CoverageGap,
    /// Redundant primitive - expressible from others.
    Redundancy,
    /// Implied relationship - one primitive implies another.
    ImpliedRelationship,
    /// Missing dependency - primitive depends on undefined concept.
    MissingDependency,
    /// Circular dependency detected.
    CircularDependency,
}

/// Result of a full validation pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Extraction validation.
    pub extraction: ExtractionValidation,
    /// Additional checks.
    pub consistency_check: bool,
    pub completeness_check: bool,
    pub implementability_check: bool,
    /// Overall pass/fail.
    pub passed: bool,
}

impl ValidationResult {
    /// Creates a validation result from extraction validation.
    #[must_use]
    pub fn from_extraction(extraction: ExtractionValidation) -> Self {
        let passed = extraction.is_valid;
        Self {
            extraction,
            consistency_check: true,
            completeness_check: true,
            implementability_check: true,
            passed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_score() {
        let valid = CoverageScore::new(0.98);
        assert!(valid.is_valid());

        let invalid = CoverageScore::new(0.80);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_minimality_from_count() {
        let score = MinimalityScore::from_redundant_count(2, 20);
        assert!((score.0 - 0.90).abs() < 0.001);
        assert!(score.is_valid());

        let invalid = MinimalityScore::from_redundant_count(5, 20);
        assert!(!invalid.is_valid()); // 0.75
    }

    #[test]
    fn test_extraction_validation() {
        let valid = ExtractionValidation::new(0.98, 0.95, 0.92);
        assert!(valid.is_valid);

        let invalid = ExtractionValidation::new(0.80, 0.95, 0.92);
        assert!(!invalid.is_valid);
    }

    #[test]
    fn test_weakest_metric() {
        let validation = ExtractionValidation::new(0.98, 0.85, 0.92);
        let (name, _) = validation.weakest_metric();
        assert_eq!(name, "minimality");
    }
}
