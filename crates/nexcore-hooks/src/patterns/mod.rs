//! Pattern matching utilities for hooks.
//!
//! Provides safe regex construction and common pattern types.

pub mod rust;
pub mod secrets;

use once_cell::sync::Lazy;
use regex::Regex;

/// Create a regex with safe fallback.
///
/// If the pattern is invalid, returns a never-match regex and logs the error.
/// This prevents hooks from crashing due to bad regex patterns.
///
/// # Example
///
/// ```
/// use nexcore_hooks::patterns::safe_regex;
///
/// let re = safe_regex(r"\bfn\s+\w+");
/// assert!(re.is_match("fn main()"));
/// ```
pub fn safe_regex(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|e| {
        eprintln!("[nexcore-hooks] Invalid regex pattern '{}': {}", pattern, e);
        // Return a never-match regex as fallback
        // SAFETY: This pattern is known to be valid
        Regex::new(r"^\b$").expect("fallback regex is valid")
    })
}

/// Create a lazy-initialized regex.
///
/// Use this macro to define static regexes that are compiled once on first use.
///
/// # Example
///
/// ```ignore
/// lazy_regex!(FUNCTION_RE, r"fn\s+(\w+)");
/// ```
#[macro_export]
macro_rules! lazy_regex {
    ($name:ident, $pattern:expr) => {
        static $name: once_cell::sync::Lazy<regex::Regex> =
            once_cell::sync::Lazy::new(|| $crate::patterns::safe_regex($pattern));
    };
}

/// Common file extension patterns
pub mod extensions {
    use super::*;

    /// Matches Rust source files (.rs)
    pub static RUST: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.rs$"));

    /// Matches Python files (.py)
    pub static PYTHON: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.py$"));

    /// Matches TypeScript files (.ts, .tsx)
    pub static TYPESCRIPT: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.tsx?$"));

    /// Matches JavaScript files (.js, .jsx)
    pub static JAVASCRIPT: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.jsx?$"));

    /// Matches YAML files (.yml, .yaml)
    pub static YAML: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.ya?ml$"));

    /// Matches JSON files (.json)
    pub static JSON: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.json$"));

    /// Matches TOML files (.toml)
    pub static TOML: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.toml$"));

    /// Matches Markdown files (.md)
    pub static MARKDOWN: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.md$"));
}

/// Check if a path matches a pattern (case-insensitive)
pub fn path_matches(path: &str, pattern: &Regex) -> bool {
    pattern.is_match(path)
}

/// Check if content contains any of the given substrings (case-insensitive)
pub fn contains_any_ci(content: &str, patterns: &[&str]) -> bool {
    let lower = content.to_ascii_lowercase();
    patterns
        .iter()
        .any(|p| lower.contains(&p.to_ascii_lowercase()))
}

/// Check if content contains any of the given substrings (case-sensitive)
pub fn contains_any(content: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| content.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_regex_valid() {
        let re = safe_regex(r"\bfn\b");
        assert!(re.is_match("fn main"));
        assert!(!re.is_match("function"));
    }

    #[test]
    fn test_safe_regex_invalid_returns_fallback() {
        // Invalid regex (unclosed group)
        let re = safe_regex(r"(unclosed");
        // Should return a never-match regex
        assert!(!re.is_match("anything"));
    }

    #[test]
    fn test_extensions() {
        assert!(extensions::RUST.is_match("src/main.rs"));
        assert!(extensions::PYTHON.is_match("script.py"));
        assert!(extensions::TYPESCRIPT.is_match("app.tsx"));
        assert!(extensions::YAML.is_match("config.yml"));
        assert!(extensions::YAML.is_match("config.yaml"));
    }

    #[test]
    fn test_contains_any_ci() {
        assert!(contains_any_ci("API_KEY=secret", &["api_key", "password"]));
        assert!(contains_any_ci("password=123", &["api_key", "password"]));
        assert!(!contains_any_ci("hello world", &["api_key", "password"]));
    }

    #[test]
    fn test_contains_any() {
        assert!(contains_any("API_KEY=secret", &["API_KEY"]));
        assert!(!contains_any("api_key=secret", &["API_KEY"])); // case sensitive
    }
}
