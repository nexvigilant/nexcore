//! # Rust Test Extractor
//!
//! Extract `#[test]` functions from Rust source files using `syn` AST parsing.

use std::fs;
use std::path::Path;

use nexcore_error::Error;
use syn::{Attribute, File, Item, ItemFn, ItemMod};

use super::test_taxonomy::{
    ClassificationPatterns, ClassifiedTest, TestCategory, TestClassification, build_classification,
};

/// Extraction errors.
#[derive(Debug, Error)]
pub enum ExtractorError {
    /// Failed to read file.
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),
    /// Failed to parse Rust source.
    #[error("Failed to parse Rust source: {0}")]
    ParseError(String),
    /// Path is not a file or directory.
    #[error("Path not found: {0}")]
    PathNotFound(String),
}

/// Check if a function has the `#[test]` attribute.
fn is_test_fn(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().get_ident().map_or(false, |id| id == "test"))
}

/// Extract test function info from an `ItemFn`.
fn extract_test_info(
    func: &ItemFn,
    file_path: &str,
    patterns: &ClassificationPatterns,
) -> Option<ClassifiedTest> {
    if !is_test_fn(&func.attrs) {
        return None;
    }

    let name = func.sig.ident.to_string();
    let (category, matched_patterns, confidence) = patterns.classify(&name);

    Some(ClassifiedTest {
        name,
        file: file_path.to_string(),
        line: 0, // syn doesn't provide line numbers without span feature
        category,
        confidence,
        patterns: matched_patterns,
    })
}

/// Recursively extract tests from items (handles nested modules).
fn extract_from_items(
    items: &[Item],
    file_path: &str,
    patterns: &ClassificationPatterns,
    tests: &mut Vec<ClassifiedTest>,
) {
    for item in items {
        match item {
            Item::Fn(func) => {
                if let Some(test) = extract_test_info(func, file_path, patterns) {
                    tests.push(test);
                }
            }
            Item::Mod(module) => {
                extract_from_module(module, file_path, patterns, tests);
            }
            _ => {}
        }
    }
}

/// Extract tests from a module (inline or external).
fn extract_from_module(
    module: &ItemMod,
    file_path: &str,
    patterns: &ClassificationPatterns,
    tests: &mut Vec<ClassifiedTest>,
) {
    // Only process inline modules (with content)
    if let Some((_, items)) = &module.content {
        extract_from_items(items, file_path, patterns, tests);
    }
}

/// Extract tests from a single Rust file.
pub fn extract_from_file(path: &Path) -> Result<Vec<ClassifiedTest>, ExtractorError> {
    let content = fs::read_to_string(path)?;
    let file_path = path.display().to_string();
    let patterns = ClassificationPatterns::default();

    let syntax: File =
        syn::parse_file(&content).map_err(|e| ExtractorError::ParseError(e.to_string()))?;

    let mut tests = Vec::new();
    extract_from_items(&syntax.items, &file_path, &patterns, &mut tests);

    Ok(tests)
}

/// Extract tests from a directory (recursively scans .rs files).
pub fn extract_from_directory(path: &Path) -> Result<Vec<ClassifiedTest>, ExtractorError> {
    let mut all_tests = Vec::new();

    if !path.exists() {
        return Err(ExtractorError::PathNotFound(path.display().to_string()));
    }

    fn walk_dir(dir: &Path, tests: &mut Vec<ClassifiedTest>) -> Result<(), ExtractorError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                walk_dir(&path, tests)?;
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                match extract_from_file(&path) {
                    Ok(file_tests) => tests.extend(file_tests),
                    Err(ExtractorError::ParseError(_)) => {
                        // Skip files that fail to parse (e.g., proc macro crates)
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    walk_dir(path, &mut all_tests)?;
    Ok(all_tests)
}

/// Classify tests in a path (file or directory).
pub fn classify_tests(path: &Path) -> Result<TestClassification, ExtractorError> {
    let tests = if path.is_file() {
        extract_from_file(path)?
    } else {
        extract_from_directory(path)?
    };

    Ok(build_classification(path, tests))
}

/// Classify tests and return category distribution.
#[must_use]
pub fn get_category_distribution(
    classification: &TestClassification,
) -> Vec<(TestCategory, usize, f64)> {
    let total = classification.total_tests.max(1) as f64;
    TestCategory::all()
        .iter()
        .map(|cat| {
            let count = classification.counts.get(*cat);
            let percentage = (count as f64 / total) * 100.0;
            (*cat, count, percentage)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_test_fn() {
        let code = r#"
            #[test]
            fn test_something() {}
        "#;
        let file: File = syn::parse_str(code).expect("parse");
        if let Item::Fn(func) = &file.items[0] {
            assert!(is_test_fn(&func.attrs));
        }
    }

    #[test]
    fn test_classification_from_code() {
        let code = r#"
            #[test]
            fn test_login_success() {}

            #[test]
            fn test_invalid_password_error() {}

            #[test]
            fn test_max_boundary() {}
        "#;
        let file: File = syn::parse_str(code).expect("parse");
        let patterns = ClassificationPatterns::default();
        let mut tests = Vec::new();
        extract_from_items(&file.items, "test.rs", &patterns, &mut tests);

        assert_eq!(tests.len(), 3);
        assert_eq!(tests[0].category, TestCategory::Positive);
        assert_eq!(tests[1].category, TestCategory::Negative);
        assert_eq!(tests[2].category, TestCategory::Edge);
    }
}
