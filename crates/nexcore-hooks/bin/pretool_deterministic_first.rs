//! Deterministic First - PreToolUse:Edit|Write
//!
//! Before generating code, encourages deterministic patterns over LLM-dependent ones.
//! Core ASR principle: exhaust deterministic options before falling back to LLM.
//!
//! Hierarchy of preference:
//! 1. Static dispatch (const, match, enum) - O(1), zero LLM cost
//! 2. Lookup tables (HashMap, BTreeMap) - O(1) or O(log n), zero LLM cost
//! 3. Pattern matching (regex, glob) - deterministic, zero LLM cost
//! 4. Heuristic rules - fast, predictable
//! 5. LLM fallback - only for truly ambiguous cases
//!
//! This hook warns when code introduces LLM dependencies without deterministic guards.

use nexcore_hooks::{exit_ok, exit_warn, get_content, get_file_path, is_rust_file, read_input};

/// Patterns indicating LLM dependency
const LLM_DEPENDENCY_PATTERNS: &[&str] = &[
    "llm",
    "LLM",
    "gpt",
    "GPT",
    "claude",
    "Claude",
    "anthropic",
    "Anthropic",
    "openai",
    "OpenAI",
    "generate_response",
    "chat_completion",
    "completion(",
    "prompt(",
    "ai_response",
    "model.generate",
    "model.complete",
    "inference(",
];

/// Patterns indicating deterministic guards
const DETERMINISTIC_GUARD_PATTERNS: &[&str] = &[
    "match ",
    "if let ",
    "HashMap::",
    "BTreeMap::",
    "lookup",
    "Lookup",
    "cache",
    "Cache",
    "const ",
    "static ",
    "enum ",
    "pattern",
    "Pattern",
    "regex",
    "Regex",
    "glob",
    "Glob",
    "rule",
    "Rule",
    "heuristic",
    "Heuristic",
    // Fallback indicators (means there's a deterministic path first)
    "fallback",
    "Fallback",
    "else {",
    "_ =>",
    "default",
    "Default",
];

/// Check if code has LLM dependencies
fn has_llm_dependency(content: &str) -> Vec<&'static str> {
    LLM_DEPENDENCY_PATTERNS
        .iter()
        .filter(|p| content.contains(*p))
        .copied()
        .collect()
}

/// Check if code has deterministic guards
fn has_deterministic_guards(content: &str) -> bool {
    let guard_count = DETERMINISTIC_GUARD_PATTERNS
        .iter()
        .filter(|p| content.contains(*p))
        .count();

    // Require at least 2 different deterministic patterns
    guard_count >= 2
}

/// Check if the LLM usage is properly guarded
fn is_properly_guarded(content: &str) -> bool {
    // Look for the "deterministic first, LLM fallback" pattern
    let has_match_with_fallback = content.contains("match ")
        && (content.contains("_ => ")
            || content.contains("else ")
            || content.contains("fallback")
            || content.contains("delegate"));

    let has_lookup_with_fallback = (content.contains("HashMap") || content.contains("BTreeMap"))
        && (content.contains(".get(")
            || content.contains("if let Some")
            || content.contains("match "));

    let has_cache_check = content.contains("cache")
        && (content.contains("if let Some") || content.contains("match "));

    has_match_with_fallback || has_lookup_with_fallback || has_cache_check
}

/// Analyze the structure of LLM usage
fn analyze_llm_usage(content: &str) -> LlmUsageAnalysis {
    let llm_deps = has_llm_dependency(content);
    let has_guards = has_deterministic_guards(content);
    let is_guarded = is_properly_guarded(content);

    LlmUsageAnalysis {
        has_llm_dependency: !llm_deps.is_empty(),
        llm_patterns_found: llm_deps,
        has_deterministic_guards: has_guards,
        is_properly_guarded: is_guarded,
    }
}

struct LlmUsageAnalysis {
    has_llm_dependency: bool,
    llm_patterns_found: Vec<&'static str>,
    has_deterministic_guards: bool,
    is_properly_guarded: bool,
}

impl LlmUsageAnalysis {
    fn needs_warning(&self) -> bool {
        self.has_llm_dependency && !self.is_properly_guarded
    }

    fn severity(&self) -> &'static str {
        if self.has_llm_dependency && !self.has_deterministic_guards {
            "high" // No guards at all
        } else if self.has_llm_dependency && !self.is_properly_guarded {
            "medium" // Has some guards but not proper fallback pattern
        } else {
            "low"
        }
    }
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_ok(),
    };

    // Only check Write and Edit tools
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Write" && tool_name != "Edit" {
        exit_ok();
    }

    // Get tool_input
    let tool_input = match &input.tool_input {
        Some(v) => v,
        None => exit_ok(),
    };

    // Get file path
    let file_path = match get_file_path(tool_input) {
        Some(p) => p,
        None => exit_ok(),
    };

    // Only check Rust files
    if !is_rust_file(&file_path) {
        exit_ok();
    }

    // Skip test files (more lenient)
    if file_path.contains("/tests/") || file_path.contains("_test.rs") {
        exit_ok();
    }

    // Get content
    let content = match get_content(tool_input) {
        Some(c) => c,
        None => exit_ok(),
    };

    // Skip small changes
    if content.lines().count() < 10 {
        exit_ok();
    }

    // Analyze LLM usage
    let analysis = analyze_llm_usage(&content);

    if analysis.needs_warning() {
        let patterns = analysis.llm_patterns_found.join(", ");
        let severity = analysis.severity();

        let suggestion = if !analysis.has_deterministic_guards {
            "Add deterministic guards: match/enum for known cases, \
             HashMap for lookup, then LLM only for unknown cases."
        } else {
            "Ensure LLM is only called in fallback path: \
             match known_patterns {{ ... _ => llm_fallback() }}"
        };

        exit_warn(&format!(
            "LLM dependency detected ({}) without proper deterministic guards. \
             Severity: {}. {}",
            patterns, severity, suggestion
        ));
    }

    exit_ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_llm_dependency() {
        assert!(!has_llm_dependency("fn main() {}").is_empty() == false);
        assert!(!has_llm_dependency("let response = llm.generate()").is_empty());
        assert!(!has_llm_dependency("use anthropic::Client").is_empty());
    }

    #[test]
    fn test_has_deterministic_guards() {
        let guarded = r#"
            match command {
                "help" => show_help(),
                "version" => show_version(),
                _ => fallback_to_llm(),
            }
        "#;
        assert!(has_deterministic_guards(guarded));

        let unguarded = "let response = llm.generate(prompt);";
        assert!(!has_deterministic_guards(unguarded));
    }

    #[test]
    fn test_is_properly_guarded() {
        let good_pattern = r#"
            match lookup_cache.get(&key) {
                Some(cached) => cached,
                None => {
                    let result = llm_generate(key);
                    lookup_cache.insert(key, result);
                    result
                }
            }
        "#;
        assert!(is_properly_guarded(good_pattern));

        let bad_pattern = "let result = llm_generate(input);";
        assert!(!is_properly_guarded(bad_pattern));
    }

    #[test]
    fn test_analyze_llm_usage() {
        let unguarded_llm = "let response = claude.generate(prompt);";
        let analysis = analyze_llm_usage(unguarded_llm);
        assert!(analysis.has_llm_dependency);
        assert!(!analysis.is_properly_guarded);
        assert!(analysis.needs_warning());

        let guarded_llm = r#"
            match cache.get(&key) {
                Some(v) => v,
                None => claude.generate(key),
            }
        "#;
        let analysis = analyze_llm_usage(guarded_llm);
        assert!(analysis.has_llm_dependency);
        assert!(analysis.is_properly_guarded);
        assert!(!analysis.needs_warning());
    }
}
