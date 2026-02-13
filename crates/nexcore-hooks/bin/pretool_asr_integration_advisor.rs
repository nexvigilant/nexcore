//! ASR Integration Advisor - PreToolUse:Edit|Write
//!
//! When working on nexcore skill-related crates, suggests ASR (Autonomous Skill Runtime)
//! patterns to accelerate integration.
//!
//! ASR T2 Composites to apply:
//! - decision_tree: recursive condition nodes for deterministic branching
//! - coverage_metric: track deterministic/total execution ratio
//! - fallback_routing: LLM delegation only for ambiguous cases
//! - flywheel_loop: log -> analyze -> improve cycle
//!
//! Architecture patterns encouraged:
//! 1. Hybrid Execution: deterministic path + LLM fallback for holes
//! 2. Coverage Tracking: measure deterministic execution percentage
//! 3. Flywheel: log fallbacks -> pattern analysis -> corpus improvement
//! 4. Hook Extraction: @hook: annotations for shell commands

use nexcore_hooks::{exit_ok, exit_warn, get_content, get_file_path, is_rust_file, read_input};

/// ASR-related crate patterns
const ASR_CRATE_PATTERNS: &[&str] = &[
    "nexcore-skill",
    "nexcore-mcp",
    "skills/",
    "skill-",
    "/skill",
    "skill_",
    "runtime",
    "executor",
    "dispatcher",
];

/// T2 composite patterns to check for
struct AsrPatternAnalysis {
    has_decision_tree: bool,
    has_coverage_metric: bool,
    has_fallback_routing: bool,
    has_flywheel_loop: bool,
    has_deterministic_first: bool,
}

impl AsrPatternAnalysis {
    fn score(&self) -> usize {
        let mut score = 0;
        if self.has_decision_tree {
            score += 1;
        }
        if self.has_coverage_metric {
            score += 1;
        }
        if self.has_fallback_routing {
            score += 1;
        }
        if self.has_flywheel_loop {
            score += 1;
        }
        if self.has_deterministic_first {
            score += 1;
        }
        score
    }

    fn missing_patterns(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !self.has_decision_tree {
            missing.push("decision_tree (recursive condition nodes)");
        }
        if !self.has_coverage_metric {
            missing.push("coverage_metric (deterministic/total ratio)");
        }
        if !self.has_fallback_routing {
            missing.push("fallback_routing (LLM delegation for ambiguity)");
        }
        if !self.has_flywheel_loop {
            missing.push("flywheel_loop (log->analyze->improve)");
        }
        if !self.has_deterministic_first {
            missing.push("deterministic_first (prefer match/enum over LLM)");
        }
        missing
    }
}

/// Analyze content for ASR T2 composite patterns
fn analyze_asr_patterns(content: &str) -> AsrPatternAnalysis {
    AsrPatternAnalysis {
        // Decision tree: recursive enums with match arms
        has_decision_tree: content.contains("enum Decision")
            || content.contains("enum Node")
            || content.contains("enum Action")
            || (content.contains("match ") && content.matches("match ").count() >= 3)
            || content.contains("DecisionTree")
            || content.contains("ConditionNode"),

        // Coverage metric: tracking deterministic vs total
        has_coverage_metric: content.contains("coverage")
            || content.contains("Coverage")
            || content.contains("deterministic_count")
            || content.contains("total_count")
            || content.contains("execution_ratio")
            || content.contains("hit_rate"),

        // Fallback routing: LLM delegation when deterministic fails
        has_fallback_routing: content.contains("fallback")
            || content.contains("Fallback")
            || content.contains("delegate_to_llm")
            || content.contains("llm_fallback")
            || content.contains("ambiguous_case")
            || (content.contains("Err(") && content.contains("llm")),

        // Flywheel loop: log -> analyze -> improve
        has_flywheel_loop: content.contains("flywheel")
            || content.contains("Flywheel")
            || content.contains("log_for_improvement")
            || content.contains("analyze_patterns")
            || content.contains("corpus_improvement")
            || (content.contains("log") && content.contains("analyze")),

        // Deterministic first: prefer static dispatch
        has_deterministic_first: content.contains("match ")
            || content.contains("if let ")
            || content.contains("enum ")
            || content.contains("const ")
            || content.contains("static ")
            || content.contains("lookup_table")
            || content.contains("HashMap::"),
    }
}

/// Check if the file path is related to skills/ASR
fn is_asr_related(path: &str) -> bool {
    ASR_CRATE_PATTERNS.iter().any(|p| path.contains(p))
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

    // Only check Rust files in ASR-related paths
    if !is_rust_file(&file_path) || !is_asr_related(&file_path) {
        exit_ok();
    }

    // Get content
    let content = match get_content(tool_input) {
        Some(c) => c,
        None => exit_ok(),
    };

    // Skip small changes
    let line_count = content.lines().count();
    if line_count < 15 {
        exit_ok();
    }

    // Analyze ASR patterns
    let analysis = analyze_asr_patterns(&content);

    // If score is low, suggest missing patterns
    if analysis.score() < 2 {
        let missing = analysis.missing_patterns();
        if !missing.is_empty() {
            let suggestions = missing.join(", ");
            exit_warn(&format!(
                "ASR patterns missing. Consider adding: {}. \
                 ASR architecture: deterministic path + LLM fallback for holes.",
                suggestions
            ));
        }
    }

    exit_ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_asr_related() {
        assert!(is_asr_related("crates/nexcore-skill-verify/src/lib.rs"));
        assert!(is_asr_related("~/.claude/skills/my-skill/src/lib.rs"));
        assert!(is_asr_related("crates/nexcore-mcp/src/tools.rs"));
        assert!(!is_asr_related("crates/nexcore-api/src/routes.rs"));
    }

    #[test]
    fn test_analyze_decision_tree() {
        let code = r#"
            enum Decision {
                Accept,
                Reject,
                Delegate,
            }
            match input {
                Pattern::A => Decision::Accept,
                Pattern::B => Decision::Reject,
                _ => Decision::Delegate,
            }
        "#;
        let analysis = analyze_asr_patterns(code);
        assert!(analysis.has_decision_tree);
        assert!(analysis.has_deterministic_first);
    }

    #[test]
    fn test_analyze_coverage_metric() {
        let code = r#"
            struct CoverageMetric {
                deterministic_count: usize,
                total_count: usize,
            }
            impl CoverageMetric {
                fn coverage_ratio(&self) -> f64 {
                    self.deterministic_count as f64 / self.total_count as f64
                }
            }
        "#;
        let analysis = analyze_asr_patterns(code);
        assert!(analysis.has_coverage_metric);
    }

    #[test]
    fn test_analyze_fallback_routing() {
        let code = r#"
            fn execute(cmd: &str) -> Result<Output, Error> {
                match lookup_deterministic(cmd) {
                    Some(result) => Ok(result),
                    None => delegate_to_llm(cmd),
                }
            }
        "#;
        let analysis = analyze_asr_patterns(code);
        assert!(analysis.has_fallback_routing);
        assert!(analysis.has_deterministic_first);
    }

    #[test]
    fn test_analyze_flywheel() {
        let code = r#"
            fn flywheel_iteration(logs: &[FallbackLog]) {
                let patterns = analyze_patterns(logs);
                for pattern in patterns {
                    corpus_improvement(pattern);
                }
            }
        "#;
        let analysis = analyze_asr_patterns(code);
        assert!(analysis.has_flywheel_loop);
    }
}
