//! Structured error diagnostics for AI consumption.
//!
//! Converts raw `DnaError` values into machine-parseable `Diagnostic` structs
//! with error codes, line numbers, suggestions, and source context.
//!
//! ## ROADMAP Phase 10.3
//!
//! ```json
//! {
//!   "error": "undefined_variable",
//!   "name": "y",
//!   "line": 3,
//!   "suggestion": "Did you mean 'x'?",
//!   "context": "let x = 5\nlet z = 10\ny + 1"
//! }
//! ```
//!
//! Tier: T2-C (∂ Boundary + μ Mapping + → Causality + σ Sequence)

use crate::error::DnaError;

use core::fmt;

// ---------------------------------------------------------------------------
// ErrorCode — categorized error type
// ---------------------------------------------------------------------------

/// Machine-parseable error code for structured diagnostics.
///
/// Tier: T2-P (κ Comparison + ∂ Boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Lexer errors
    /// Invalid character in source. ∂[lexer]
    InvalidCharacter,
    /// Number literal could not be parsed. ∂[lexer]
    InvalidLiteral,

    // Parser errors
    /// Unexpected token where something else was expected. ∂[parser]
    UnexpectedToken,
    /// A specific token was expected but not found. ∂[parser]
    ExpectedToken,
    /// An identifier was expected (e.g. after `let`). ∂[parser]
    ExpectedIdentifier,
    /// Unmatched parenthesis. ∂[parser]
    UnmatchedParen,

    // Compiler/semantic errors
    /// Variable used but never defined. ∂[compiler]
    UndefinedVariable,
    /// Function called but never defined. ∂[compiler]
    UndefinedFunction,
    /// Nested function definitions not supported. ∂[compiler]
    NestedFunction,

    // General
    /// Error that does not fit another category. ∂[general]
    Unknown,
}

impl ErrorCode {
    /// Machine-readable string name for this error code.
    pub fn name(self) -> &'static str {
        match self {
            Self::InvalidCharacter => "invalid_character",
            Self::InvalidLiteral => "invalid_literal",
            Self::UnexpectedToken => "unexpected_token",
            Self::ExpectedToken => "expected_token",
            Self::ExpectedIdentifier => "expected_identifier",
            Self::UnmatchedParen => "unmatched_paren",
            Self::UndefinedVariable => "undefined_variable",
            Self::UndefinedFunction => "undefined_function",
            Self::NestedFunction => "nested_function",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ---------------------------------------------------------------------------
// Diagnostic — structured error
// ---------------------------------------------------------------------------

/// Structured error diagnostic with machine-parseable fields.
///
/// AI can parse these to understand errors and self-correct.
///
/// Tier: T2-C (∂ Boundary + μ Mapping + → Causality + σ Sequence)
pub struct Diagnostic {
    /// Categorized error code.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Source line number (1-based, 0 if unknown).
    pub line: usize,
    /// Suggested fix, if available.
    pub suggestion: Option<String>,
    /// Source context around the error (the offending line).
    pub context: Option<String>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] line {}: {}", self.code, self.line, self.message)?;
        if let Some(ref sug) = self.suggestion {
            write!(f, " ({sug})")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Diagnostic")
            .field("code", &self.code)
            .field("message", &self.message)
            .field("line", &self.line)
            .field("suggestion", &self.suggestion)
            .field("context", &self.context)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Levenshtein edit distance (zero dependencies)
// ---------------------------------------------------------------------------

/// Compute the Levenshtein edit distance between two strings.
///
/// Classic O(m*n) dynamic programming. Used for did-you-mean suggestions.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let m = a_bytes.len();
    let n = b_bytes.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    // Single-row DP
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] {
                0
            } else {
                1
            };
            let del = prev[j] + 1;
            let ins = curr[j - 1] + 1;
            let sub = prev[j - 1] + cost;
            curr[j] = del.min(ins).min(sub);
        }
        core::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Find the closest match for `name` among `candidates`.
///
/// Returns `Some("Did you mean '<best>'?")` if the best match has
/// edit distance ≤ 2. Returns `None` if no close match exists.
pub fn did_you_mean(name: &str, candidates: &[&str]) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }

    let mut best_dist = usize::MAX;
    let mut best_name = "";

    for &candidate in candidates {
        // Skip if lengths differ too much (can't be within distance 2)
        let len_diff = name.len().abs_diff(candidate.len());
        if len_diff > 2 {
            continue;
        }

        let dist = levenshtein(name, candidate);
        if dist < best_dist {
            best_dist = dist;
            best_name = candidate;
        }
    }

    if best_dist <= 2 && !best_name.is_empty() && best_name != name {
        Some(format!("Did you mean '{best_name}'?"))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Source context extraction
// ---------------------------------------------------------------------------

/// Extract the source line at the given 1-based line number.
///
/// Returns the line content, or `None` if the line number is out of range.
pub fn source_line(source: &str, line: usize) -> Option<String> {
    if line == 0 {
        return None;
    }
    source.lines().nth(line - 1).map(|s| s.to_string())
}

/// Extract context: up to 3 lines centered on the error line.
pub fn source_context(source: &str, line: usize) -> Option<String> {
    if line == 0 {
        return None;
    }
    let lines: Vec<&str> = source.lines().collect();
    if line > lines.len() {
        return None;
    }

    let start = line.saturating_sub(2);
    let end = (line + 1).min(lines.len());

    let context: Vec<String> = lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let ln = start + i + 1;
            if ln == line {
                format!("→ {ln:>3} | {l}")
            } else {
                format!("  {ln:>3} | {l}")
            }
        })
        .collect();

    Some(context.join("\n"))
}

// ---------------------------------------------------------------------------
// Name extraction from source (for did-you-mean candidates)
// ---------------------------------------------------------------------------

/// Scan source for `let <name>` and `fn <name>` to build a candidate list.
///
/// Uses a lightweight scan — no full parse needed. This works even when
/// the source has errors (which is exactly when we need suggestions).
fn extract_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let words: Vec<&str> = source.split_whitespace().collect();

    for window in words.windows(2) {
        if window[0] == "let" || window[0] == "fn" {
            // Next word is a name (strip trailing parens for fn)
            let raw = window[1];
            let name = raw.split('(').next().unwrap_or(raw);
            let name = name.split('=').next().unwrap_or(name).trim();
            if !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                names.push(name.to_string());
            }
        }
    }

    names
}

// ---------------------------------------------------------------------------
// Error classification
// ---------------------------------------------------------------------------

/// Classify a `DnaError` into an `ErrorCode` by inspecting its message.
fn classify_error(err: &DnaError) -> ErrorCode {
    let msg = format!("{err}");

    if msg.contains("unexpected character") {
        return ErrorCode::InvalidCharacter;
    }
    if msg.contains("invalid literal") {
        return ErrorCode::InvalidLiteral;
    }
    if msg.contains("expected identifier")
        || msg.contains("expected function name")
        || msg.contains("expected parameter name")
    {
        return ErrorCode::ExpectedIdentifier;
    }
    if msg.contains("expected ')'") || msg.contains("expected ')' but") {
        return ErrorCode::UnmatchedParen;
    }
    if msg.contains("expected ") {
        return ErrorCode::ExpectedToken;
    }
    if msg.contains("unexpected token") {
        return ErrorCode::UnexpectedToken;
    }
    if msg.contains("undefined variable") {
        return ErrorCode::UndefinedVariable;
    }
    if msg.contains("undefined function") {
        return ErrorCode::UndefinedFunction;
    }
    if msg.contains("nested function") {
        return ErrorCode::NestedFunction;
    }

    ErrorCode::Unknown
}

/// Extract a name from an error message like "undefined variable: 'foo'".
fn extract_name_from_error(err: &DnaError) -> Option<String> {
    let msg = format!("{err}");
    // Look for pattern: 'name'
    if let Some(start) = msg.find('\'') {
        let rest = &msg[start + 1..];
        if let Some(end) = rest.find('\'') {
            let name = &rest[..end];
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Public API: diagnose
// ---------------------------------------------------------------------------

/// Try to compile source and return structured diagnostics for any errors.
///
/// Returns an empty vec if compilation succeeds. Otherwise returns one
/// or more `Diagnostic` values with error codes, line info, suggestions,
/// and source context.
///
/// This is the primary API for ROADMAP Phase 10.3.
pub fn diagnose(source: &str) -> Vec<Diagnostic> {
    use crate::lang::compiler;

    match compiler::compile(source) {
        Ok(_) => Vec::new(),
        Err(err) => {
            let code = classify_error(&err);
            let line = extract_line(&err);
            let message = format!("{err}");

            // Build suggestion
            let suggestion = match code {
                ErrorCode::UndefinedVariable | ErrorCode::UndefinedFunction => {
                    let error_name = extract_name_from_error(&err);
                    let candidates = extract_names(source);
                    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();

                    // Add keywords for identifier errors
                    let all_keywords = [
                        "let", "if", "else", "elif", "while", "fn", "end", "do", "and", "or",
                        "not", "return", "for", "to", "in", "true", "false", "print", "abs", "min",
                        "max", "pow", "sqrt", "sign", "clamp", "log2", "assert",
                    ];

                    let mut all_candidates = candidate_refs;
                    all_candidates.extend_from_slice(&all_keywords);

                    if let Some(name) = &error_name {
                        did_you_mean(name, &all_candidates)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let context = source_context(source, line);

            vec![Diagnostic {
                code,
                message,
                line,
                suggestion,
                context,
            }]
        }
    }
}

/// Extract the line number from a DnaError.
fn extract_line(err: &DnaError) -> usize {
    match err {
        DnaError::SyntaxError(line, _) => *line,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// JSON serialization (zero dependencies)
// ---------------------------------------------------------------------------

/// Serialize a `Diagnostic` to a JSON string.
pub fn diagnostic_to_json(diag: &Diagnostic) -> String {
    let mut out = String::from("{\n");

    out.push_str(&format!("  \"error\": \"{}\",\n", diag.code));
    out.push_str(&format!(
        "  \"message\": \"{}\",\n",
        json_escape(&diag.message)
    ));
    out.push_str(&format!("  \"line\": {}", diag.line));

    if let Some(ref sug) = diag.suggestion {
        out.push_str(&format!(",\n  \"suggestion\": \"{}\"", json_escape(sug)));
    }

    if let Some(ref ctx) = diag.context {
        out.push_str(&format!(",\n  \"context\": \"{}\"", json_escape(ctx)));
    }

    out.push_str("\n}");
    out
}

/// Serialize a slice of diagnostics to a JSON array string.
pub fn diagnostics_to_json(diags: &[Diagnostic]) -> String {
    if diags.is_empty() {
        return "[]".to_string();
    }

    let parts: Vec<String> = diags.iter().map(diagnostic_to_json).collect();
    format!("[\n{}\n]", parts.join(",\n"))
}

/// Escape a string for JSON output.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Levenshtein tests ---

    #[test]
    fn levenshtein_identical() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn levenshtein_empty() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn levenshtein_one_edit() {
        assert_eq!(levenshtein("cat", "bat"), 1); // substitution
        assert_eq!(levenshtein("cat", "cats"), 1); // insertion
        assert_eq!(levenshtein("cats", "cat"), 1); // deletion
    }

    #[test]
    fn levenshtein_two_edits() {
        assert_eq!(levenshtein("kitten", "sitten"), 1);
        assert_eq!(levenshtein("abc", "axc"), 1);
        assert_eq!(levenshtein("abc", "aec"), 1);
    }

    #[test]
    fn levenshtein_completely_different() {
        assert_eq!(levenshtein("abc", "xyz"), 3);
    }

    // --- did_you_mean tests ---

    #[test]
    fn did_you_mean_close_match() {
        let candidates = &["count", "total", "sum"];
        let result = did_you_mean("ccount", candidates);
        assert!(result.is_some());
        let sug = result.unwrap_or_default();
        assert!(sug.contains("count"));
    }

    #[test]
    fn did_you_mean_no_match() {
        let candidates = &["count", "total", "sum"];
        let result = did_you_mean("xyzzy", candidates);
        assert!(result.is_none());
    }

    #[test]
    fn did_you_mean_exact_match_excluded() {
        // Exact match should not suggest itself
        let candidates = &["count"];
        let result = did_you_mean("count", candidates);
        assert!(result.is_none());
    }

    #[test]
    fn did_you_mean_empty_candidates() {
        let result = did_you_mean("foo", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn did_you_mean_single_char_off() {
        let candidates = &["let", "fn", "if", "while"];
        let result = did_you_mean("lt", candidates);
        assert!(result.is_some());
        let sug = result.unwrap_or_default();
        assert!(sug.contains("let"));
    }

    // --- ErrorCode tests ---

    #[test]
    fn error_code_name() {
        assert_eq!(ErrorCode::UndefinedVariable.name(), "undefined_variable");
        assert_eq!(ErrorCode::UnmatchedParen.name(), "unmatched_paren");
        assert_eq!(ErrorCode::Unknown.name(), "unknown");
    }

    #[test]
    fn error_code_display() {
        assert_eq!(
            format!("{}", ErrorCode::InvalidCharacter),
            "invalid_character"
        );
    }

    // --- Classification tests ---

    #[test]
    fn classify_syntax_unexpected() {
        let err = DnaError::SyntaxError(1, "unexpected character: '@'".into());
        assert_eq!(classify_error(&err), ErrorCode::InvalidCharacter);
    }

    #[test]
    fn classify_expected_identifier() {
        let err = DnaError::SyntaxError(2, "expected identifier after 'let', got 42".into());
        assert_eq!(classify_error(&err), ErrorCode::ExpectedIdentifier);
    }

    #[test]
    fn classify_unmatched_paren() {
        let err = DnaError::SyntaxError(1, "expected ')' but got \\n".into());
        assert_eq!(classify_error(&err), ErrorCode::UnmatchedParen);
    }

    #[test]
    fn classify_undefined_variable() {
        let err = DnaError::SyntaxError(0, "undefined variable: 'y'".into());
        assert_eq!(classify_error(&err), ErrorCode::UndefinedVariable);
    }

    #[test]
    fn classify_undefined_function() {
        let err = DnaError::SyntaxError(0, "undefined function: 'foo'".into());
        assert_eq!(classify_error(&err), ErrorCode::UndefinedFunction);
    }

    // --- Source context tests ---

    #[test]
    fn source_line_basic() {
        let src = "let x = 5\nlet y = 10\ny + 1";
        assert_eq!(source_line(src, 1).unwrap_or_default(), "let x = 5");
        assert_eq!(source_line(src, 2).unwrap_or_default(), "let y = 10");
        assert_eq!(source_line(src, 3).unwrap_or_default(), "y + 1");
    }

    #[test]
    fn source_line_zero() {
        assert!(source_line("hello", 0).is_none());
    }

    #[test]
    fn source_line_out_of_range() {
        assert!(source_line("hello", 99).is_none());
    }

    #[test]
    fn source_context_middle() {
        let src = "line1\nline2\nline3\nline4\nline5";
        let ctx = source_context(src, 3).unwrap_or_default();
        assert!(ctx.contains("line2"));
        assert!(ctx.contains("line3"));
        assert!(ctx.contains("line4"));
        assert!(ctx.contains('→')); // error indicator
    }

    #[test]
    fn source_context_first_line() {
        let src = "first\nsecond\nthird";
        let ctx = source_context(src, 1).unwrap_or_default();
        assert!(ctx.contains("first"));
        assert!(ctx.contains("second"));
    }

    // --- Extract names tests ---

    #[test]
    fn extract_names_let() {
        let names = extract_names("let x = 5\nlet count = 10");
        assert!(names.contains(&"x".to_string()));
        assert!(names.contains(&"count".to_string()));
    }

    #[test]
    fn extract_names_fn() {
        let names = extract_names("fn add(a, b) do\n  return a + b\nend");
        assert!(names.contains(&"add".to_string()));
    }

    #[test]
    fn extract_names_mixed() {
        let names = extract_names("let total = 0\nfn sum(n) do\n  return n\nend");
        assert!(names.contains(&"total".to_string()));
        assert!(names.contains(&"sum".to_string()));
    }

    #[test]
    fn extract_names_empty() {
        let names = extract_names("42 + 1");
        assert!(names.is_empty());
    }

    // --- Integration: diagnose tests ---

    #[test]
    fn diagnose_valid_source() {
        let diags = diagnose("let x = 42\nx + 1");
        assert!(diags.is_empty());
    }

    #[test]
    fn diagnose_syntax_error() {
        let diags = diagnose("let = 42");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, ErrorCode::ExpectedIdentifier);
        assert!(diags[0].line > 0);
    }

    #[test]
    fn diagnose_unmatched_paren() {
        let diags = diagnose("(2 + 3");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, ErrorCode::UnmatchedParen);
    }

    #[test]
    fn diagnose_undefined_variable_with_suggestion() {
        let diags = diagnose("let count = 5\ncont + 1");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, ErrorCode::UndefinedVariable);
        // Should suggest "count" for "cont"
        assert!(diags[0].suggestion.is_some());
        let sug = diags[0].suggestion.as_deref().unwrap_or("");
        assert!(sug.contains("count"));
    }

    #[test]
    fn diagnose_undefined_function_with_suggestion() {
        let diags = diagnose("fn add(a, b) do\n  return a + b\nend\nadd2(1, 2)");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, ErrorCode::UndefinedFunction);
        // Should suggest "add" for "add2" (distance 1)
        assert!(diags[0].suggestion.is_some());
        let sug = diags[0].suggestion.as_deref().unwrap_or("");
        assert!(sug.contains("add"));
    }

    #[test]
    fn diagnose_has_context() {
        let diags = diagnose("let x = 5\nlet = 10\nx + 1");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].context.is_some());
    }

    #[test]
    fn diagnose_invalid_character() {
        let diags = diagnose("let x = 5 @ 2");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, ErrorCode::InvalidCharacter);
    }

    #[test]
    fn diagnose_line_number_multiline() {
        // Error on line 3
        let diags = diagnose("let x = 5\nlet y = 10\nlet = 3");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].line, 3);
    }

    // --- JSON serialization tests ---

    #[test]
    fn json_empty_diagnostics() {
        assert_eq!(diagnostics_to_json(&[]), "[]");
    }

    #[test]
    fn json_single_diagnostic() {
        let diag = Diagnostic {
            code: ErrorCode::UndefinedVariable,
            message: "undefined variable: 'y'".into(),
            line: 3,
            suggestion: Some("Did you mean 'x'?".into()),
            context: Some("let x = 5\ny + 1".into()),
        };
        let json = diagnostic_to_json(&diag);
        assert!(json.contains("\"error\": \"undefined_variable\""));
        assert!(json.contains("\"line\": 3"));
        assert!(json.contains("\"suggestion\""));
        assert!(json.contains("Did you mean"));
    }

    #[test]
    fn json_no_suggestion() {
        let diag = Diagnostic {
            code: ErrorCode::UnexpectedToken,
            message: "unexpected token: @".into(),
            line: 1,
            suggestion: None,
            context: None,
        };
        let json = diagnostic_to_json(&diag);
        assert!(!json.contains("suggestion"));
        assert!(!json.contains("context"));
    }

    #[test]
    fn json_escapes_special_chars() {
        let diag = Diagnostic {
            code: ErrorCode::Unknown,
            message: "line has \"quotes\" and\nnewlines".into(),
            line: 1,
            suggestion: None,
            context: None,
        };
        let json = diagnostic_to_json(&diag);
        assert!(json.contains("\\\"quotes\\\""));
        assert!(json.contains("\\n"));
    }

    #[test]
    fn json_array_multiple() {
        let diags = vec![
            Diagnostic {
                code: ErrorCode::UnexpectedToken,
                message: "error 1".into(),
                line: 1,
                suggestion: None,
                context: None,
            },
            Diagnostic {
                code: ErrorCode::Unknown,
                message: "error 2".into(),
                line: 2,
                suggestion: None,
                context: None,
            },
        ];
        let json = diagnostics_to_json(&diags);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains("error 1"));
        assert!(json.contains("error 2"));
    }

    // --- Display tests ---

    #[test]
    fn diagnostic_display() {
        let diag = Diagnostic {
            code: ErrorCode::UndefinedVariable,
            message: "undefined variable: 'y'".into(),
            line: 3,
            suggestion: Some("Did you mean 'x'?".into()),
            context: None,
        };
        let s = format!("{diag}");
        assert!(s.contains("undefined_variable"));
        assert!(s.contains("line 3"));
        assert!(s.contains("Did you mean"));
    }

    #[test]
    fn diagnostic_display_no_suggestion() {
        let diag = Diagnostic {
            code: ErrorCode::UnexpectedToken,
            message: "unexpected".into(),
            line: 1,
            suggestion: None,
            context: None,
        };
        let s = format!("{diag}");
        assert!(!s.contains("Did you mean"));
    }

    // --- Extract name from error tests ---

    #[test]
    fn extract_name_from_error_var() {
        let err = DnaError::SyntaxError(0, "undefined variable: 'myvar'".into());
        let name = extract_name_from_error(&err);
        assert_eq!(name.unwrap_or_default(), "myvar");
    }

    #[test]
    fn extract_name_from_error_fn() {
        let err = DnaError::SyntaxError(0, "undefined function: 'foo'".into());
        let name = extract_name_from_error(&err);
        assert_eq!(name.unwrap_or_default(), "foo");
    }

    #[test]
    fn extract_name_from_error_none() {
        let err = DnaError::SyntaxError(1, "unexpected token: 42".into());
        let name = extract_name_from_error(&err);
        assert!(name.is_none());
    }
}
