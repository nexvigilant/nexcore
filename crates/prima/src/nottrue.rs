// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Not-True (¬.true) — Antipattern Assertion Engine
//!
//! The inverse of Prima's `.true` philosophy.
//!
//! In Prima, code that compiles is true (grounds to 1).
//! In `.not.true`, every expression MUST be false, void, or error.
//! If any expression evaluates to a truthy value, a violation is raised.
//!
//! ## Use Cases
//!
//! 1. **Negative assertions**: Define invariants that must never hold
//! 2. **Auto-generated test suite**: Each expression becomes a test case
//! 3. **Antipattern monitoring**: Detect when "impossible" states become possible
//!
//! ## File Format
//!
//! ```text
//! // safety_invariants.not.true
//! // Every expression here MUST fail or be falsy.
//!
//! 1 / 0                    // Division by zero — must error
//! undefined_variable       // Undefined reference — must error
//! 0 > 1                    // False comparison — must be false
//! false                    // Literal false — trivially not-true
//! ```
//!
//! ## Mathematical Foundation
//!
//! `.true` grounds to 1 (existence). `.not.true` grounds to 0 (absence).
//! The antipattern engine verifies: ∀ expr ∈ file: eval(expr) ∉ Truthy
//!
//! ## Tier: T2-C (∂ + κ + σ)

use crate::error::{PrimaError, PrimaResult};
use crate::value::Value;
use crate::{eval, tokenize};

/// File extension for antipattern files.
pub const FILE_EXTENSION_NOT: &str = "not.true";

/// Result of checking a single antipattern assertion.
#[derive(Debug, Clone)]
pub struct AssertionResult {
    /// The source expression that was checked.
    pub expression: String,
    /// Line number in the `.not.true` file (1-indexed).
    pub line: usize,
    /// Whether this assertion passed (expression was NOT true).
    pub passed: bool,
    /// What happened when the expression was evaluated.
    pub outcome: AssertionOutcome,
}

/// What happened when an antipattern expression was evaluated.
#[derive(Debug, Clone)]
pub enum AssertionOutcome {
    /// Expression produced an error (PASS — antipattern correctly caught).
    Error(String),
    /// Expression evaluated to a falsy value (PASS — not true).
    Falsy(String),
    /// Expression evaluated to void/∅ (PASS — absence).
    Void,
    /// Expression evaluated to a truthy value (FAIL — violation!).
    Truthy(String),
}

/// Summary of a `.not.true` file check.
#[derive(Debug, Clone)]
pub struct CheckReport {
    /// Path or name of the checked file.
    pub source_name: String,
    /// Individual assertion results.
    pub assertions: Vec<AssertionResult>,
    /// Number of assertions that passed.
    pub passed: usize,
    /// Number of assertions that failed (violations).
    pub failed: usize,
    /// Total assertions checked.
    pub total: usize,
}

impl CheckReport {
    /// Returns true if all assertions passed (no violations).
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Get only the violations (failed assertions).
    #[must_use]
    pub fn violations(&self) -> Vec<&AssertionResult> {
        self.assertions.iter().filter(|a| !a.passed).collect()
    }
}

impl std::fmt::Display for CheckReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "¬.true check: {}", self.source_name)?;
        writeln!(f, "─────────────────────────────────────")?;

        for result in &self.assertions {
            let status = if result.passed {
                "✓ PASS"
            } else {
                "✗ FAIL"
            };
            let detail = match &result.outcome {
                AssertionOutcome::Error(e) => format!("error: {e}"),
                AssertionOutcome::Falsy(v) => format!("falsy: {v}"),
                AssertionOutcome::Void => "void: ∅".to_string(),
                AssertionOutcome::Truthy(v) => format!("TRUTHY: {v} ← VIOLATION"),
            };
            writeln!(
                f,
                "  [{status}] L{line}: {expr}  →  {detail}",
                line = result.line,
                expr = result.expression.trim(),
            )?;
        }

        writeln!(f, "─────────────────────────────────────")?;
        writeln!(
            f,
            "Result: {passed}/{total} passed, {failed} violation(s)",
            passed = self.passed,
            total = self.total,
            failed = self.failed,
        )?;

        if self.all_passed() {
            writeln!(f, "∎ All antipatterns correctly caught.")?;
        } else {
            writeln!(f, "⚠ {} antipattern(s) were NOT caught!", self.failed)?;
        }

        Ok(())
    }
}

/// Check whether a value is "truthy" in Prima.
///
/// Truthy: non-zero int, non-zero float, true, non-empty string,
/// non-empty sequence, non-empty mapping, functions, symbols.
/// Falsy: 0, 0.0, false, empty string, empty sequence, empty mapping, void.
fn is_truthy(value: &Value) -> bool {
    use crate::value::ValueData;
    match &value.data {
        ValueData::Void => false,
        ValueData::Int(n) => *n != 0,
        ValueData::Float(f) => *f != 0.0,
        ValueData::Bool(b) => *b,
        ValueData::String(s) => !s.is_empty(),
        ValueData::Sequence(seq) => !seq.is_empty(),
        ValueData::Mapping(map) => !map.is_empty(),
        ValueData::Function(_) | ValueData::Builtin(_) => true,
        ValueData::Symbol(_) => true,
        ValueData::Quoted(_) => true,
    }
}

/// Parse a `.not.true` source into individual expressions.
///
/// Rules:
/// - Lines starting with `//` are comments (skipped)
/// - Empty lines are skipped
/// - Each non-comment, non-empty line is one expression
/// - Multi-line expressions use `\` continuation (future)
fn parse_assertions(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                None
            } else {
                Some((i + 1, trimmed.to_string()))
            }
        })
        .collect()
}

/// Check a single expression against the not-true invariant.
fn check_one(expr: &str) -> AssertionResult {
    // First verify it's valid syntax by tokenizing
    match tokenize(expr) {
        Err(e) => {
            // Tokenization error — expression is definitely not-true
            return AssertionResult {
                expression: expr.to_string(),
                line: 0, // caller sets this
                passed: true,
                outcome: AssertionOutcome::Error(format!("{e}")),
            };
        }
        Ok(_tokens) => {} // Continue to evaluation
    }

    // Evaluate the expression
    match eval(expr) {
        Err(e) => {
            // Runtime error — expression is not-true (PASS)
            AssertionResult {
                expression: expr.to_string(),
                line: 0,
                passed: true,
                outcome: AssertionOutcome::Error(format!("{e}")),
            }
        }
        Ok(value) => {
            if matches!(value.data, crate::value::ValueData::Void) {
                // Void — absence, not-true (PASS)
                AssertionResult {
                    expression: expr.to_string(),
                    line: 0,
                    passed: true,
                    outcome: AssertionOutcome::Void,
                }
            } else if !is_truthy(&value) {
                // Falsy value — not-true (PASS)
                AssertionResult {
                    expression: expr.to_string(),
                    line: 0,
                    passed: true,
                    outcome: AssertionOutcome::Falsy(format!("{value}")),
                }
            } else {
                // TRUTHY — this should NOT be true! VIOLATION
                AssertionResult {
                    expression: expr.to_string(),
                    line: 0,
                    passed: false,
                    outcome: AssertionOutcome::Truthy(format!("{value}")),
                }
            }
        }
    }
}

/// Check an entire `.not.true` source file.
///
/// Every expression in the file must evaluate to false, void, or error.
/// Returns a report with pass/fail for each assertion.
pub fn check(source: &str, source_name: &str) -> CheckReport {
    let expressions = parse_assertions(source);
    let mut assertions = Vec::with_capacity(expressions.len());

    for (line, expr) in &expressions {
        let mut result = check_one(expr);
        result.line = *line;
        assertions.push(result);
    }

    let passed = assertions.iter().filter(|a| a.passed).count();
    let failed = assertions.iter().filter(|a| !a.passed).count();
    let total = assertions.len();

    CheckReport {
        source_name: source_name.to_string(),
        assertions,
        passed,
        failed,
        total,
    }
}

/// Check a `.not.true` source and return an error if any violations found.
///
/// This is the strict version — any truthy expression is a hard error.
pub fn check_strict(source: &str, source_name: &str) -> PrimaResult<CheckReport> {
    let report = check(source, source_name);
    if report.all_passed() {
        Ok(report)
    } else {
        let violations: Vec<String> = report
            .violations()
            .iter()
            .map(|v| format!("L{}: {} was truthy", v.line, v.expression))
            .collect();
        Err(PrimaError::grounding(format!(
            "¬.true violations in {}: {}",
            source_name,
            violations.join("; ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_false_literal_passes() {
        let report = check("false", "test");
        assert!(report.all_passed());
        assert_eq!(report.total, 1);
        assert_eq!(report.passed, 1);
    }

    #[test]
    fn test_zero_passes() {
        let report = check("0", "test");
        assert!(report.all_passed());
    }

    #[test]
    fn test_true_literal_fails() {
        let report = check("true", "test");
        assert!(!report.all_passed());
        assert_eq!(report.failed, 1);
    }

    #[test]
    fn test_truthy_int_fails() {
        let report = check("42", "test");
        assert!(!report.all_passed());
    }

    #[test]
    fn test_division_by_zero_passes() {
        let report = check("1 / 0", "test");
        assert!(report.all_passed());
        if let Some(first) = report.assertions.first() {
            assert!(matches!(first.outcome, AssertionOutcome::Error(_)));
        }
    }

    #[test]
    fn test_undefined_variable_passes() {
        let report = check("nonexistent_variable", "test");
        assert!(report.all_passed());
    }

    #[test]
    fn test_false_comparison_passes() {
        let report = check("0 > 1", "test");
        assert!(report.all_passed());
    }

    #[test]
    fn test_comments_skipped() {
        let source = "// this is a comment\n// another comment\n0";
        let report = check(source, "test");
        assert_eq!(report.total, 1);
        assert!(report.all_passed());
    }

    #[test]
    fn test_empty_lines_skipped() {
        let source = "\n\n0\n\n";
        let report = check(source, "test");
        assert_eq!(report.total, 1);
    }

    #[test]
    fn test_mixed_pass_fail() {
        let source = "false\ntrue\n0\n42";
        let report = check(source, "test");
        assert_eq!(report.total, 4);
        assert_eq!(report.passed, 2); // false, 0
        assert_eq!(report.failed, 2); // true, 42
    }

    #[test]
    fn test_strict_mode_returns_error_on_violation() {
        let result = check_strict("true", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_strict_mode_returns_ok_on_all_pass() {
        let result = check_strict("false\n0", "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_report_display() {
        let report = check("false\ntrue", "test.not.true");
        let display = format!("{report}");
        assert!(display.contains("PASS"));
        assert!(display.contains("FAIL"));
        assert!(display.contains("VIOLATION"));
    }

    #[test]
    fn test_violations_method() {
        let report = check("false\ntrue\n0\n1", "test");
        let violations = report.violations();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_line_numbers_correct() {
        let source = "// comment\nfalse\n\ntrue";
        let report = check(source, "test");
        assert_eq!(report.assertions.len(), 2);
        assert_eq!(report.assertions[0].line, 2); // "false" is on line 2
        assert_eq!(report.assertions[1].line, 4); // "true" is on line 4
    }

    #[test]
    fn test_empty_string_passes() {
        // Empty string is falsy
        let report = check("\"\"", "test");
        assert!(report.all_passed());
    }

    #[test]
    fn test_nonempty_string_fails() {
        let report = check("\"hello\"", "test");
        assert!(!report.all_passed());
    }

    #[test]
    fn test_empty_sequence_passes() {
        let report = check("σ[]", "test");
        assert!(report.all_passed());
    }

    #[test]
    fn test_file_extension_constant() {
        assert_eq!(FILE_EXTENSION_NOT, "not.true");
    }
}
