//! Test classifier for the 5-category taxonomy
//!
//! Classifies tests into: POSITIVE, NEGATIVE, EDGE, STRESS, ADVERSARIAL
//! Uses pattern matching on test names and content for classification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The 5-category test taxonomy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestCategory {
    /// Happy path tests - verifies expected behavior works
    Positive,
    /// Error handling tests - verifies failures are handled correctly
    Negative,
    /// Boundary condition tests - verifies edge cases
    Edge,
    /// Performance/load tests - verifies system under stress
    Stress,
    /// Security/malicious input tests - verifies resilience
    Adversarial,
}

impl std::fmt::Display for TestCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Positive => write!(f, "POSITIVE"),
            Self::Negative => write!(f, "NEGATIVE"),
            Self::Edge => write!(f, "EDGE"),
            Self::Stress => write!(f, "STRESS"),
            Self::Adversarial => write!(f, "ADVERSARIAL"),
        }
    }
}

/// Classification result for a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedTest {
    /// Test function name
    pub name: String,
    /// Classified category
    pub category: TestCategory,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Matched patterns that led to classification
    pub matched_patterns: Vec<String>,
}

/// Classification patterns for each category
#[derive(Debug, Clone)]
pub struct ClassificationPatterns {
    /// Patterns indicating POSITIVE tests
    pub positive: Vec<&'static str>,
    /// Patterns indicating NEGATIVE tests
    pub negative: Vec<&'static str>,
    /// Patterns indicating EDGE tests
    pub edge: Vec<&'static str>,
    /// Patterns indicating STRESS tests
    pub stress: Vec<&'static str>,
    /// Patterns indicating ADVERSARIAL tests
    pub adversarial: Vec<&'static str>,
}

impl Default for ClassificationPatterns {
    fn default() -> Self {
        Self {
            positive: vec![
                "test_", "test_basic", "test_simple", "test_valid",
                "test_success", "test_works", "test_happy", "test_normal",
                "test_default", "test_standard", "test_typical", "test_expected",
                "should_", "can_", "does_", "returns_", "creates_", "gets_",
            ],
            negative: vec![
                "error", "fail", "invalid", "reject", "deny", "refuse",
                "exception", "raise", "throw", "not_found", "missing",
                "unauthorized", "forbidden", "bad_", "wrong_", "broken",
                "corrupt", "malformed", "illegal", "cannot_", "unable_",
                "should_fail", "should_raise", "should_reject", "expect_error",
            ],
            edge: vec![
                "edge", "boundary", "limit", "max", "min", "zero", "empty",
                "none", "null", "nil", "overflow", "underflow", "wrap",
                "large", "small", "huge", "tiny", "extreme", "corner",
                "special_case", "unicode", "utf8", "whitespace", "newline",
                "first", "last", "single", "multiple", "many", "few",
            ],
            stress: vec![
                "stress", "load", "perf", "performance", "benchmark", "bench",
                "concurrent", "parallel", "async", "bulk", "batch", "scale",
                "throughput", "latency", "timeout", "slow", "fast", "speed",
                "memory", "cpu", "resource", "exhaust", "flood", "spike",
                "sustained", "burst", "peak", "capacity",
            ],
            adversarial: vec![
                "adversarial", "attack", "malicious", "inject", "injection",
                "xss", "csrf", "sql", "command", "path_traversal", "fuzzy",
                "fuzz", "random", "chaos", "evil", "hostile", "untrusted",
                "sanitize", "escape", "encode", "decode", "security",
                "vulnerability", "exploit", "payload", "buffer_overflow",
            ],
        }
    }
}

/// Classify a test function by its name
///
/// Uses pattern matching against the 5-category taxonomy.
/// Falls back to POSITIVE with low confidence if no patterns match.
pub fn classify_test(name: &str, patterns: &ClassificationPatterns) -> ClassifiedTest {
    let name_lower = name.to_lowercase();

    // Score each category
    let mut scores: HashMap<TestCategory, (f64, Vec<String>)> = HashMap::new();

    // Check each category's patterns
    let categories = [
        (TestCategory::Adversarial, &patterns.adversarial), // Check first (highest priority)
        (TestCategory::Stress, &patterns.stress),
        (TestCategory::Edge, &patterns.edge),
        (TestCategory::Negative, &patterns.negative),
        (TestCategory::Positive, &patterns.positive),
    ];

    for (category, category_patterns) in categories {
        let mut matched = Vec::new();
        for pattern in category_patterns.iter() {
            if name_lower.contains(*pattern) {
                matched.push((*pattern).to_string());
            }
        }

        if !matched.is_empty() {
            // Score based on number of matches and pattern specificity
            let score = matched.len() as f64 * 0.3 + 0.5; // Base 0.5 + 0.3 per match
            scores.insert(category, (score.min(1.0), matched));
        }
    }

    // Find highest scoring category
    if let Some((&category, (confidence, patterns))) =
        scores.iter().max_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap())
    {
        return ClassifiedTest {
            name: name.to_string(),
            category,
            confidence: *confidence,
            matched_patterns: patterns.clone(),
        };
    }

    // Default to POSITIVE with low confidence
    ClassifiedTest {
        name: name.to_string(),
        category: TestCategory::Positive,
        confidence: 0.3,
        matched_patterns: vec!["default".to_string()],
    }
}

/// Classify multiple tests
pub fn classify_tests(names: &[String]) -> Vec<ClassifiedTest> {
    let patterns = ClassificationPatterns::default();
    names.iter().map(|n| classify_test(n, &patterns)).collect()
}

/// Summary of test classification results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationSummary {
    /// Total tests classified
    pub total: usize,
    /// Count by category
    pub by_category: HashMap<String, usize>,
    /// Average confidence
    pub avg_confidence: f64,
    /// Tests with low confidence (< 0.5)
    pub low_confidence_count: usize,
}

/// Generate summary from classification results
pub fn summarize_classification(tests: &[ClassifiedTest]) -> ClassificationSummary {
    let mut by_category: HashMap<String, usize> = HashMap::new();

    for test in tests {
        *by_category.entry(test.category.to_string()).or_insert(0) += 1;
    }

    let avg_confidence = if tests.is_empty() {
        0.0
    } else {
        tests.iter().map(|t| t.confidence).sum::<f64>() / tests.len() as f64
    };

    let low_confidence_count = tests.iter().filter(|t| t.confidence < 0.5).count();

    ClassificationSummary {
        total: tests.len(),
        by_category,
        avg_confidence,
        low_confidence_count,
    }
}

/// Check if test distribution is balanced across categories
///
/// Returns recommendations for improving test coverage
pub fn analyze_distribution(tests: &[ClassifiedTest]) -> Vec<String> {
    let summary = summarize_classification(tests);
    let mut recommendations = Vec::new();

    let total = summary.total as f64;
    if total == 0.0 {
        return vec!["No tests found to analyze".to_string()];
    }

    // Check for missing categories
    let categories = ["POSITIVE", "NEGATIVE", "EDGE", "STRESS", "ADVERSARIAL"];
    for cat in categories {
        let count = summary.by_category.get(cat).copied().unwrap_or(0);
        let ratio = count as f64 / total;

        if count == 0 {
            recommendations.push(format!("Missing {} tests - add tests for this category", cat));
        } else if ratio < 0.05 {
            recommendations.push(format!(
                "Low {} coverage ({:.1}%) - consider adding more tests",
                cat,
                ratio * 100.0
            ));
        }
    }

    // Check for over-reliance on positive tests
    let positive_count = summary.by_category.get("POSITIVE").copied().unwrap_or(0);
    let positive_ratio = positive_count as f64 / total;
    if positive_ratio > 0.8 {
        recommendations.push(format!(
            "Test suite is {:.1}% POSITIVE tests - add more negative/edge/adversarial tests",
            positive_ratio * 100.0
        ));
    }

    // Check for low confidence classifications
    if summary.low_confidence_count > tests.len() / 2 {
        recommendations.push(format!(
            "{} tests have low classification confidence - consider using clearer naming conventions",
            summary.low_confidence_count
        ));
    }

    if recommendations.is_empty() {
        recommendations.push("Test distribution looks balanced across categories".to_string());
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_positive() {
        let patterns = ClassificationPatterns::default();
        let result = classify_test("test_user_creation_works", &patterns);
        assert_eq!(result.category, TestCategory::Positive);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_classify_negative() {
        let patterns = ClassificationPatterns::default();
        let result = classify_test("test_invalid_email_raises_error", &patterns);
        assert_eq!(result.category, TestCategory::Negative);
        assert!(result.matched_patterns.iter().any(|p| p.contains("invalid") || p.contains("error")));
    }

    #[test]
    fn test_classify_edge() {
        let patterns = ClassificationPatterns::default();
        let result = classify_test("test_empty_list_boundary", &patterns);
        assert_eq!(result.category, TestCategory::Edge);
    }

    #[test]
    fn test_classify_stress() {
        let patterns = ClassificationPatterns::default();
        let result = classify_test("test_concurrent_load_performance", &patterns);
        assert_eq!(result.category, TestCategory::Stress);
    }

    #[test]
    fn test_classify_adversarial() {
        let patterns = ClassificationPatterns::default();
        let result = classify_test("test_sql_injection_attack", &patterns);
        assert_eq!(result.category, TestCategory::Adversarial);
    }

    #[test]
    fn test_default_classification() {
        let patterns = ClassificationPatterns::default();
        let result = classify_test("mysterious_function", &patterns);
        // Should default to POSITIVE with low confidence
        assert_eq!(result.category, TestCategory::Positive);
        assert!(result.confidence < 0.5);
    }

    #[test]
    fn test_summarize_classification() {
        let tests = vec![
            ClassifiedTest {
                name: "test_a".to_string(),
                category: TestCategory::Positive,
                confidence: 0.8,
                matched_patterns: vec![],
            },
            ClassifiedTest {
                name: "test_b".to_string(),
                category: TestCategory::Negative,
                confidence: 0.9,
                matched_patterns: vec![],
            },
        ];

        let summary = summarize_classification(&tests);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.by_category.get("POSITIVE"), Some(&1));
        assert_eq!(summary.by_category.get("NEGATIVE"), Some(&1));
    }

    #[test]
    fn test_analyze_distribution_missing_category() {
        let tests = vec![
            ClassifiedTest {
                name: "test_a".to_string(),
                category: TestCategory::Positive,
                confidence: 0.8,
                matched_patterns: vec![],
            },
        ];

        let recommendations = analyze_distribution(&tests);
        assert!(recommendations.iter().any(|r| r.contains("Missing")));
    }
}
