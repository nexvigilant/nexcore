//! Test harness module for comprehensive test coverage analysis
//!
//! This module provides high-performance test analysis capabilities:
//! - **Metrics**: Jaccard similarity, set operations for coverage analysis
//! - **Scanner**: Parallel file discovery with rayon (~30-100x faster)
//! - **Classifier**: 5-category taxonomy (POSITIVE, NEGATIVE, EDGE, STRESS, ADVERSARIAL)
//! - **AST**: Python primitive extraction (function/class names)
//!
//! # Architecture
//!
//! The test harness is designed as a hybrid Rust/Python system:
//! - Rust handles compute-intensive operations (scanning, metrics, extraction)
//! - Python wrapper handles pytest integration and report generation
//!
//! # Example
//!
//! ```ignore
//! use rust_skills::harness::{scanner, ast, metrics, classifier};
//!
//! // Scan for Python files
//! let files = scanner::scan_python_files(Path::new("./src"));
//!
//! // Extract primitives from source and test files
//! let source_files: Vec<_> = files.iter().filter(|f| !f.is_test).collect();
//! let test_files: Vec<_> = files.iter().filter(|f| f.is_test).collect();
//!
//! // Calculate coverage
//! let report = metrics::calculate_coverage(&source_primitives, &test_primitives);
//!
//! // Classify tests
//! let classified = classifier::classify_tests(&test_names);
//! ```

pub mod ast;
pub mod classifier;
pub mod metrics;
pub mod scanner;

// Re-export commonly used types
pub use ast::{extract_primitives, get_function_names, get_test_function_names, Primitive, PrimitiveKind};
pub use classifier::{classify_test, classify_tests, ClassifiedTest, TestCategory};
pub use metrics::{calculate_coverage, jaccard_similarity, CoverageReport};
pub use scanner::{scan_directory, scan_python_files, scan_test_files, ScannedFile, ScanOptions};

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Full analysis report combining all harness components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessReport {
    /// File scanning summary
    pub scan_summary: scanner::ScanSummary,
    /// Coverage analysis
    pub coverage: CoverageReport,
    /// Test classification summary
    pub classification: classifier::ClassificationSummary,
    /// AST extraction summary
    pub extraction: ast::ExtractionSummary,
    /// Recommendations for improving test coverage
    pub recommendations: Vec<String>,
}

/// Run full analysis on a directory
///
/// This is the main entry point for the test harness.
/// Scans files, extracts primitives, calculates coverage, and classifies tests.
pub fn analyze_directory(root: &Path) -> HarnessReport {
    // Step 1: Scan for files
    let files = scan_python_files(root);
    let scan_summary = scanner::summarize_scan(&files);

    // Step 2: Separate source and test files
    let source_files: Vec<_> = files.iter().filter(|f| !f.is_test).collect();
    let test_files: Vec<_> = files.iter().filter(|f| f.is_test).collect();

    // Step 3: Extract primitives from all files
    let mut all_source_primitives = Vec::new();
    let mut all_test_primitives = Vec::new();

    for file in &source_files {
        if let Ok(primitives) = ast::extract_from_file(&file.path) {
            all_source_primitives.extend(primitives);
        }
    }

    for file in &test_files {
        if let Ok(primitives) = ast::extract_from_file(&file.path) {
            all_test_primitives.extend(primitives);
        }
    }

    let extraction = ast::summarize_extraction(
        &all_source_primitives
            .iter()
            .chain(all_test_primitives.iter())
            .cloned()
            .collect::<Vec<_>>(),
    );

    // Step 4: Get function names for coverage analysis
    let source_functions = get_function_names(&all_source_primitives);
    let test_functions = get_test_function_names(&all_test_primitives);

    // Step 5: Calculate coverage
    let coverage = calculate_coverage(&source_functions, &test_functions);

    // Step 6: Classify tests
    let classified = classify_tests(&test_functions);
    let classification = classifier::summarize_classification(&classified);

    // Step 7: Generate recommendations
    let mut recommendations = Vec::new();

    // Coverage recommendations
    if coverage.coverage_ratio < 0.5 {
        recommendations.push(format!(
            "Coverage is low ({:.1}%). Consider adding tests for: {}",
            coverage.coverage_ratio * 100.0,
            coverage.uncovered_primitives.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        ));
    }

    // Distribution recommendations
    recommendations.extend(classifier::analyze_distribution(&classified));

    HarnessReport {
        scan_summary,
        coverage,
        classification,
        extraction,
        recommendations,
    }
}

/// Quick coverage check (fast mode)
///
/// Only calculates coverage ratio without full classification.
pub fn quick_coverage(root: &Path) -> f64 {
    let files = scan_python_files(root);

    let source_files: Vec<_> = files.iter().filter(|f| !f.is_test).collect();
    let test_files: Vec<_> = files.iter().filter(|f| f.is_test).collect();

    let mut source_functions = Vec::new();
    let mut test_functions = Vec::new();

    for file in &source_files {
        if let Ok(primitives) = ast::extract_from_file(&file.path) {
            source_functions.extend(get_function_names(&primitives));
        }
    }

    for file in &test_files {
        if let Ok(primitives) = ast::extract_from_file(&file.path) {
            test_functions.extend(get_test_function_names(&primitives));
        }
    }

    let report = calculate_coverage(&source_functions, &test_functions);
    report.coverage_ratio
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create source file
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/main.py"),
            r#"
def add(a, b):
    return a + b

def subtract(a, b):
    return a - b

def multiply(a, b):
    return a * b

class Calculator:
    def divide(self, a, b):
        return a / b
"#,
        )
        .unwrap();

        // Create test file
        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(
            root.join("tests/test_main.py"),
            r#"
def test_add():
    assert add(1, 2) == 3

def test_subtract():
    assert subtract(3, 1) == 2

def test_add_negative():
    assert add(-1, -2) == -3

def test_add_edge_zero():
    assert add(0, 0) == 0
"#,
        )
        .unwrap();

        dir
    }

    #[test]
    fn test_analyze_directory() {
        let dir = create_test_project();
        let report = analyze_directory(dir.path());

        // Check scan summary
        assert!(report.scan_summary.source_files > 0);
        assert!(report.scan_summary.test_files > 0);

        // Check coverage
        assert!(report.coverage.coverage_ratio > 0.0);
        assert!(!report.coverage.covered_primitives.is_empty());

        // Check classification
        assert!(report.classification.total > 0);
    }

    #[test]
    fn test_quick_coverage() {
        let dir = create_test_project();
        let coverage = quick_coverage(dir.path());

        // Should have some coverage (add and subtract are tested)
        assert!(coverage > 0.0);
        // But not 100% (multiply and divide are not tested)
        assert!(coverage < 1.0);
    }
}
