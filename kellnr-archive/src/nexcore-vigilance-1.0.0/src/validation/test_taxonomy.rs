//! # Test Taxonomy
//!
//! Classification of test cases into five categories based on name patterns and attributes.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Test category classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestCategory {
    /// Happy path, expected behavior.
    Positive,
    /// Error handling, invalid inputs.
    Negative,
    /// Boundary conditions, limits.
    Edge,
    /// Performance, load, concurrency.
    Stress,
    /// Security, malicious inputs.
    Adversarial,
}

impl TestCategory {
    /// All categories as a slice.
    #[must_use]
    pub const fn all() -> &'static [TestCategory] {
        &[
            TestCategory::Positive,
            TestCategory::Negative,
            TestCategory::Edge,
            TestCategory::Stress,
            TestCategory::Adversarial,
        ]
    }

    /// Category display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            TestCategory::Positive => "Positive",
            TestCategory::Negative => "Negative",
            TestCategory::Edge => "Edge",
            TestCategory::Stress => "Stress",
            TestCategory::Adversarial => "Adversarial",
        }
    }

    /// Category description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            TestCategory::Positive => "Happy path, expected behavior",
            TestCategory::Negative => "Error handling, invalid inputs",
            TestCategory::Edge => "Boundary conditions, limits",
            TestCategory::Stress => "Performance, load, concurrency",
            TestCategory::Adversarial => "Security, malicious inputs",
        }
    }
}

/// A classified test function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedTest {
    /// Test function name.
    pub name: String,
    /// File path where the test is defined.
    pub file: String,
    /// Line number in the file.
    pub line: usize,
    /// Classified category.
    pub category: TestCategory,
    /// Confidence score (0.0-1.0).
    pub confidence: f64,
    /// Matching patterns that led to classification.
    pub patterns: Vec<String>,
}

/// Test classification result for a path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestClassification {
    /// Path that was analyzed.
    pub path: String,
    /// Total tests found.
    pub total_tests: usize,
    /// Classified tests by category.
    pub tests: Vec<ClassifiedTest>,
    /// Category counts.
    pub counts: CategoryCounts,
    /// Coverage metrics.
    pub coverage: CoverageMetrics,
}

/// Counts by category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryCounts {
    /// Positive (happy path) tests.
    pub positive: usize,
    /// Negative (error handling) tests.
    pub negative: usize,
    /// Edge (boundary) tests.
    pub edge: usize,
    /// Stress (performance) tests.
    pub stress: usize,
    /// Adversarial (security) tests.
    pub adversarial: usize,
}

impl CategoryCounts {
    /// Get count for a specific category.
    #[must_use]
    pub const fn get(&self, category: TestCategory) -> usize {
        match category {
            TestCategory::Positive => self.positive,
            TestCategory::Negative => self.negative,
            TestCategory::Edge => self.edge,
            TestCategory::Stress => self.stress,
            TestCategory::Adversarial => self.adversarial,
        }
    }

    /// Increment count for a category.
    pub fn increment(&mut self, category: TestCategory) {
        match category {
            TestCategory::Positive => self.positive += 1,
            TestCategory::Negative => self.negative += 1,
            TestCategory::Edge => self.edge += 1,
            TestCategory::Stress => self.stress += 1,
            TestCategory::Adversarial => self.adversarial += 1,
        }
    }

    /// Total count across all categories.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.positive + self.negative + self.edge + self.stress + self.adversarial
    }
}

/// Coverage metrics for test categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMetrics {
    /// Categories with at least one test.
    pub covered_categories: usize,
    /// Total possible categories (5).
    pub total_categories: usize,
    /// Category coverage percentage.
    pub category_coverage: f64,
    /// Missing categories.
    pub missing_categories: Vec<TestCategory>,
    /// Recommendations for improving coverage.
    pub recommendations: Vec<String>,
}

impl Default for CoverageMetrics {
    fn default() -> Self {
        Self {
            covered_categories: 0,
            total_categories: 5,
            category_coverage: 0.0,
            missing_categories: TestCategory::all().to_vec(),
            recommendations: Vec::new(),
        }
    }
}

/// Classification patterns for test names.
#[derive(Debug)]
pub struct ClassificationPatterns {
    /// Patterns indicating positive tests.
    pub positive: Vec<&'static str>,
    /// Patterns indicating negative tests.
    pub negative: Vec<&'static str>,
    /// Patterns indicating edge tests.
    pub edge: Vec<&'static str>,
    /// Patterns indicating stress tests.
    pub stress: Vec<&'static str>,
    /// Patterns indicating adversarial tests.
    pub adversarial: Vec<&'static str>,
}

impl Default for ClassificationPatterns {
    fn default() -> Self {
        Self {
            positive: vec![
                "_success",
                "_works",
                "_valid",
                "_ok",
                "_pass",
                "_creates",
                "_returns",
                "_computes",
                "_parses",
                "_serializes",
                "_happy",
                "_basic",
                "_simple",
            ],
            negative: vec![
                "_error",
                "_fails",
                "_invalid",
                "_reject",
                "_deny",
                "_missing",
                "_malformed",
                "_empty",
                "_null",
                "_none",
                "_bad",
                "_wrong",
            ],
            edge: vec![
                "_boundary",
                "_max",
                "_min",
                "_zero",
                "_limit",
                "_overflow",
                "_underflow",
                "_empty_",
                "_single",
                "_large",
                "_small",
            ],
            stress: vec![
                "_concurrent",
                "_load",
                "_stress",
                "_perf",
                "_performance",
                "_parallel",
                "_async",
                "_bulk",
                "_batch",
                "_heavy",
            ],
            adversarial: vec![
                "_inject",
                "_overflow",
                "_malicious",
                "_attack",
                "_security",
                "_xss",
                "_sql",
                "_escape",
                "_fuzz",
                "_adversarial",
                "_untrusted",
            ],
        }
    }
}

impl ClassificationPatterns {
    /// Classify a test name, returning the category and matching patterns.
    #[must_use]
    pub fn classify(&self, name: &str) -> (TestCategory, Vec<String>, f64) {
        let lower = name.to_lowercase();
        let mut matches: Vec<(TestCategory, Vec<String>)> = Vec::new();

        // Check each category's patterns
        let categories = [
            (TestCategory::Adversarial, &self.adversarial),
            (TestCategory::Stress, &self.stress),
            (TestCategory::Edge, &self.edge),
            (TestCategory::Negative, &self.negative),
            (TestCategory::Positive, &self.positive),
        ];

        for (category, patterns) in &categories {
            let matched: Vec<String> = patterns
                .iter()
                .filter(|p| lower.contains(*p))
                .map(|s| (*s).to_string())
                .collect();
            if !matched.is_empty() {
                matches.push((*category, matched));
            }
        }

        // Return first match (priority order: adversarial > stress > edge > negative > positive)
        if let Some((category, patterns)) = matches.first() {
            let confidence = (patterns.len() as f64 * 0.3).min(1.0);
            return (*category, patterns.clone(), confidence + 0.5);
        }

        // Default to Positive with low confidence if no patterns match
        (TestCategory::Positive, Vec::new(), 0.3)
    }
}

/// Build a test classification from a list of test functions.
#[must_use]
pub fn build_classification(path: &Path, tests: Vec<ClassifiedTest>) -> TestClassification {
    let total_tests = tests.len();
    let mut counts = CategoryCounts::default();

    for test in &tests {
        counts.increment(test.category);
    }

    // Calculate coverage
    let mut covered = 0;
    let mut missing = Vec::new();
    for category in TestCategory::all() {
        if counts.get(*category) > 0 {
            covered += 1;
        } else {
            missing.push(*category);
        }
    }

    let category_coverage = if total_tests > 0 {
        (covered as f64 / 5.0) * 100.0
    } else {
        0.0
    };

    // Generate recommendations
    let mut recommendations = Vec::new();
    for category in &missing {
        recommendations.push(format!(
            "Add {} tests ({})",
            category.name(),
            category.description()
        ));
    }

    if counts.positive > 0 && counts.negative == 0 {
        recommendations.push("Consider adding error handling tests".into());
    }
    if counts.positive > 0 && counts.edge == 0 {
        recommendations.push("Consider adding boundary condition tests".into());
    }

    let coverage = CoverageMetrics {
        covered_categories: covered,
        total_categories: 5,
        category_coverage,
        missing_categories: missing,
        recommendations,
    };

    TestClassification {
        path: path.display().to_string(),
        total_tests,
        tests,
        counts,
        coverage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_classification() {
        let patterns = ClassificationPatterns::default();

        let (cat, _, _) = patterns.classify("test_login_success");
        assert_eq!(cat, TestCategory::Positive);

        let (cat, _, _) = patterns.classify("test_invalid_input_error");
        assert_eq!(cat, TestCategory::Negative);

        let (cat, _, _) = patterns.classify("test_max_boundary");
        assert_eq!(cat, TestCategory::Edge);

        let (cat, _, _) = patterns.classify("test_concurrent_load");
        assert_eq!(cat, TestCategory::Stress);

        let (cat, _, _) = patterns.classify("test_sql_inject");
        assert_eq!(cat, TestCategory::Adversarial);
    }

    #[test]
    fn test_category_counts() {
        let mut counts = CategoryCounts::default();
        counts.increment(TestCategory::Positive);
        counts.increment(TestCategory::Positive);
        counts.increment(TestCategory::Negative);

        assert_eq!(counts.positive, 2);
        assert_eq!(counts.negative, 1);
        assert_eq!(counts.total(), 3);
    }
}
