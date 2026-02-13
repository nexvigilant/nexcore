//! Observability Verification (HOOK-M5-53)
//!
//! Ensures code has proper instrumentation for logging, metrics, and tracing.
//! Checks for structured logging, metrics recording, and distributed tracing.

use regex::Regex;
use std::sync::LazyLock;

/// Kind of observability issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservabilityIssue {
    /// No structured logging found
    MissingLogging,
    /// No metrics instrumentation
    MissingMetrics,
    /// No tracing/spans
    MissingTracing,
    /// Error returned without logging
    UnloggedError,
}

impl ObservabilityIssue {
    /// Returns the human-readable name of this issue.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingLogging => "Missing Logging",
            Self::MissingMetrics => "Missing Metrics",
            Self::MissingTracing => "Missing Tracing",
            Self::UnloggedError => "Unlogged Error",
        }
    }

    /// Returns the suggested fix for this issue.
    pub fn fix_suggestion(&self) -> &'static str {
        match self {
            Self::MissingLogging => "Add tracing crate for structured logging",
            Self::MissingMetrics => "Add metrics crate for instrumentation",
            Self::MissingTracing => "Add #[instrument] to key functions",
            Self::UnloggedError => "Log errors before returning: tracing::error!(...)",
        }
    }
}

/// Logging coverage analysis
#[derive(Debug, Clone, Default)]
pub struct LoggingCoverage {
    /// Has tracing/log crate import
    pub has_logging_import: bool,
    /// Has tracing instrumentation (#[instrument])
    pub has_tracing_instrument: bool,
    /// Has metrics recording
    pub has_metrics: bool,
    /// Has span creation
    pub has_spans: bool,
    /// Count of functions
    pub function_count: usize,
    /// Count of instrumented functions
    pub instrumented_count: usize,
    /// Count of error returns
    pub error_return_count: usize,
    /// Count of logged errors
    pub logged_error_count: usize,
}

impl LoggingCoverage {
    /// Calculate instrumentation percentage
    pub fn instrumentation_percentage(&self) -> f32 {
        if self.function_count == 0 {
            return 100.0;
        }
        (self.instrumented_count as f32 / self.function_count as f32) * 100.0
    }

    /// Check if coverage is adequate
    pub fn is_adequate(&self) -> bool {
        self.has_logging_import && self.instrumentation_percentage() >= 50.0
    }
}

/// A detected observability finding
#[derive(Debug, Clone)]
pub struct ObservabilityFinding {
    /// Kind of issue
    pub kind: ObservabilityIssue,
    /// Line number (1-indexed), 0 for file-level
    pub line: usize,
    /// Code preview if available
    pub code_preview: Option<String>,
    /// Suggested fix
    pub suggestion: &'static str,
}

/// Compile a regex pattern, exiting on failure (programming bug).
fn compile_regex(pattern: &str) -> Regex {
    match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FATAL: Invalid regex pattern: {e}");
            std::process::exit(1);
        }
    }
}

// Structured logging crates
static LOGGING_IMPORT: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"use\s+(tracing|log|slog|env_logger)"));

// Tracing instrumentation
static TRACING_INSTRUMENT: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"#\[(?:tracing::)?instrument"));

// Metrics recording
static METRICS_RECORD: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(counter!|gauge!|histogram!|metrics::)"));

// Span creation
static SPAN_CREATE: LazyLock<Regex> = LazyLock::new(|| {
    compile_regex(r"(span!|info_span!|debug_span!|warn_span!|error_span!|Span::)")
});

// Function definition
static FUNCTION_DEF: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(pub\s+)?(?:async\s+)?fn\s+\w+"));

// Error return patterns
static ERROR_RETURN: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(Err\([^)]+\)|return\s+Err|\?)"));

// Error logging before return
static ERROR_LOG: LazyLock<Regex> =
    LazyLock::new(|| compile_regex(r"(error!|warn!|tracing::error|tracing::warn)"));

/// Check if the file should be skipped entirely
fn should_skip_file(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();

    // Skip test files
    if lower.contains("/test") || lower.contains("_test") || lower.contains("tests/") {
        return true;
    }

    // Skip example files
    if lower.contains("/example") || lower.contains("examples/") {
        return true;
    }

    // Only check Rust files
    !lower.ends_with(".rs")
}

/// Check if a line should be skipped
fn should_skip_line(line: &str) -> bool {
    let trimmed = line.trim();
    // Skip comments, but NOT Rust attributes (#[...])
    trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || (trimmed.starts_with('#') && !trimmed.starts_with("#["))
}

/// Calculate logging coverage for file content.
///
/// # Arguments
/// * `content` - File content to analyze
///
/// # Returns
/// Logging coverage statistics
pub fn calculate_logging_coverage(content: &str) -> LoggingCoverage {
    let mut coverage = LoggingCoverage::default();

    // Check imports
    coverage.has_logging_import = LOGGING_IMPORT.is_match(content);
    coverage.has_metrics = METRICS_RECORD.is_match(content);
    coverage.has_spans = SPAN_CREATE.is_match(content);
    coverage.has_tracing_instrument = TRACING_INSTRUMENT.is_match(content);

    // Count functions and instrumentation
    let lines: Vec<&str> = content.lines().collect();
    let mut in_function = false;
    let mut has_instrument = false;

    for (i, line) in lines.iter().enumerate() {
        if should_skip_line(line) {
            continue;
        }

        // Check for #[instrument] before function
        if TRACING_INSTRUMENT.is_match(line) {
            has_instrument = true;
        }

        // Check for function definition
        if FUNCTION_DEF.is_match(line) {
            coverage.function_count += 1;
            if has_instrument {
                coverage.instrumented_count += 1;
            }
            has_instrument = false;
            in_function = true;
        }

        // Track error returns (simplified - just count occurrences)
        if in_function && ERROR_RETURN.is_match(line) {
            coverage.error_return_count += 1;

            // Check if there's error logging in the surrounding context
            let start = i.saturating_sub(3);
            let end = (i + 1).min(lines.len());
            let context = lines[start..end].join("\n");
            if ERROR_LOG.is_match(&context) {
                coverage.logged_error_count += 1;
            }
        }

        // Simple function end detection (closing brace at start of line)
        if line.trim() == "}" {
            in_function = false;
        }
    }

    coverage
}

/// Analyze observability in file content.
///
/// # Arguments
/// * `file_path` - Path to the file
/// * `content` - File content to scan
///
/// # Returns
/// Vector of detected observability findings
pub fn analyze_observability(file_path: &str, content: &str) -> Vec<ObservabilityFinding> {
    let mut findings = Vec::new();

    if should_skip_file(file_path) {
        return findings;
    }

    let coverage = calculate_logging_coverage(content);

    // Only report issues if this looks like production code
    let is_main_or_lib = file_path.ends_with("main.rs")
        || file_path.ends_with("lib.rs")
        || file_path.contains("/src/");

    if !is_main_or_lib {
        return findings;
    }

    // Check for logging import
    if !coverage.has_logging_import && coverage.function_count > 0 {
        findings.push(ObservabilityFinding {
            kind: ObservabilityIssue::MissingLogging,
            line: 0,
            code_preview: None,
            suggestion: ObservabilityIssue::MissingLogging.fix_suggestion(),
        });
    }

    // Check instrumentation coverage
    if coverage.function_count >= 3 && coverage.instrumentation_percentage() < 30.0 {
        findings.push(ObservabilityFinding {
            kind: ObservabilityIssue::MissingTracing,
            line: 0,
            code_preview: Some(format!(
                "{}/{} functions instrumented ({:.0}%)",
                coverage.instrumented_count,
                coverage.function_count,
                coverage.instrumentation_percentage()
            )),
            suggestion: ObservabilityIssue::MissingTracing.fix_suggestion(),
        });
    }

    // Check for unlogged errors (only if there are many)
    if coverage.error_return_count > 3 && coverage.logged_error_count == 0 {
        findings.push(ObservabilityFinding {
            kind: ObservabilityIssue::UnloggedError,
            line: 0,
            code_preview: Some(format!(
                "{} error returns without logging",
                coverage.error_return_count
            )),
            suggestion: ObservabilityIssue::UnloggedError.fix_suggestion(),
        });
    }

    findings
}

/// Format observability findings for terminal output.
///
/// # Arguments
/// * `file_path` - Path to the scanned file
/// * `findings` - Vector of detected findings
///
/// # Returns
/// Formatted string with a visual report
pub fn format_observability_findings(file_path: &str, findings: &[ObservabilityFinding]) -> String {
    if findings.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    output.push_str(
        "┌─────────────────────────────────────────────────────────────────────────────┐\n",
    );
    output.push_str(
        "│ ⚠️  OBSERVABILITY ADVISORY                                                   │\n",
    );
    output.push_str(
        "├─────────────────────────────────────────────────────────────────────────────┤\n",
    );

    let display_path = if file_path.len() > 70 {
        &file_path[..70]
    } else {
        file_path
    };
    output.push_str(&format!("│ File: {:<70} │\n", display_path));

    for finding in findings {
        output.push_str(&format!("│   ⚠️  {:<70} │\n", finding.kind.as_str()));

        if let Some(preview) = &finding.code_preview {
            let preview_display = if preview.len() > 65 {
                &preview[..65]
            } else {
                preview.as_str()
            };
            output.push_str(&format!("│      {:<71} │\n", preview_display));
        }

        let suggestion = if finding.suggestion.len() > 65 {
            &finding.suggestion[..65]
        } else {
            finding.suggestion
        };
        output.push_str(&format!("│      Fix: {:<66} │\n", suggestion));
    }

    output.push_str(
        "├─────────────────────────────────────────────────────────────────────────────┤\n",
    );
    output.push_str(&format!(
        "│ Total: {} finding(s) - advisory only, not blocking                          │\n",
        findings.len()
    ));
    output.push_str(
        "└─────────────────────────────────────────────────────────────────────────────┘\n",
    );

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_missing_logging() {
        let content = r#"
            fn foo() -> i32 { 42 }
            fn bar() -> String { "hello".into() }
            fn baz() -> bool { true }
        "#;

        let findings = analyze_observability("src/main.rs", content);
        assert!(!findings.is_empty());
        assert!(
            findings
                .iter()
                .any(|f| f.kind == ObservabilityIssue::MissingLogging)
        );
    }

    #[test]
    fn test_logging_present() {
        let content = r#"
            use tracing::{info, error};

            #[tracing::instrument]
            fn foo() -> i32 {
                info!("doing something");
                42
            }
        "#;

        let coverage = calculate_logging_coverage(content);
        assert!(coverage.has_logging_import);
        assert!(coverage.has_tracing_instrument);
    }

    #[test]
    fn test_instrumentation_coverage() {
        let content = r#"
            use tracing::instrument;

            #[instrument]
            fn instrumented() {}

            fn not_instrumented() {}

            fn also_not_instrumented() {}
        "#;

        let coverage = calculate_logging_coverage(content);
        assert_eq!(coverage.function_count, 3);
        assert_eq!(coverage.instrumented_count, 1);
        assert!(coverage.instrumentation_percentage() < 50.0);
    }

    #[test]
    fn test_skip_test_files() {
        let content = r#"
            fn test_something() {
                // no logging needed in tests
            }
        "#;

        let findings = analyze_observability("tests/unit_test.rs", content);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_format_findings() {
        let findings = vec![ObservabilityFinding {
            kind: ObservabilityIssue::MissingLogging,
            line: 0,
            code_preview: None,
            suggestion: "Add tracing crate",
        }];

        let formatted = format_observability_findings("src/main.rs", &findings);
        assert!(formatted.contains("OBSERVABILITY"));
        assert!(formatted.contains("Missing Logging"));
    }
}
