//! Coverage Tracker - PostToolUse:Edit|Write
//!
//! After skill modifications, reminds to update coverage metrics.
//! Part of the ASR (Autonomous Skill Runtime) integration acceleration.
//!
//! Coverage tracking is essential for measuring skill determinism:
//! - What percentage of inputs are handled deterministically?
//! - Which patterns still require LLM fallback?
//! - Where should we add deterministic handlers next?
//!
//! This hook warns when skill code changes but coverage metrics are not updated.

use nexcore_hooks::{exit_ok, exit_warn, get_content, get_file_path, is_rust_file, read_input};
use std::path::Path;

/// Skill-related path patterns
const SKILL_PATTERNS: &[&str] = &[
    "skills/",
    "skill-",
    "/skill",
    "skill_",
    "nexcore-skill",
    "executor",
    "dispatcher",
    "handler",
];

/// Coverage-related code indicators
const COVERAGE_INDICATORS: &[&str] = &[
    "coverage",
    "Coverage",
    "hit_count",
    "miss_count",
    "deterministic_ratio",
    "fallback_count",
    "execution_stats",
    "ExecutionStats",
    "track_execution",
    "record_hit",
    "record_miss",
];

/// Check if content has coverage tracking
fn has_coverage_tracking(content: &str) -> bool {
    COVERAGE_INDICATORS.iter().any(|i| content.contains(i))
}

/// Check if this is a skill-related file
fn is_skill_related(path: &str) -> bool {
    SKILL_PATTERNS.iter().any(|p| path.contains(p))
}

/// Check if this is a core logic file (not tests/docs)
fn is_core_logic_file(path: &str) -> bool {
    is_rust_file(path)
        && !path.contains("/tests/")
        && !path.contains("_test.rs")
        && !path.contains("/benches/")
        && !path.contains("/examples/")
}

/// Analyze what kind of changes were made
fn analyze_changes(content: &str) -> ChangeAnalysis {
    ChangeAnalysis {
        adds_handler: content.contains("fn handle")
            || content.contains("fn execute")
            || content.contains("fn dispatch")
            || content.contains("fn process")
            || content.contains("impl Handler"),

        adds_match_arm: content.contains("=> {")
            || content.contains("=> Ok(")
            || content.contains("=> Some(")
            || content.matches("=>").count() >= 3,

        adds_new_pattern: content.contains("Pattern::")
            || content.contains("Command::")
            || content.contains("Action::")
            || content.contains("Event::"),

        modifies_routing: content.contains("route")
            || content.contains("dispatch")
            || content.contains("delegate")
            || content.contains("forward"),
    }
}

struct ChangeAnalysis {
    adds_handler: bool,
    adds_match_arm: bool,
    adds_new_pattern: bool,
    modifies_routing: bool,
}

impl ChangeAnalysis {
    fn is_significant(&self) -> bool {
        self.adds_handler || self.adds_match_arm || self.adds_new_pattern || self.modifies_routing
    }

    fn description(&self) -> String {
        let mut changes = Vec::new();
        if self.adds_handler {
            changes.push("new handler");
        }
        if self.adds_match_arm {
            changes.push("match arms");
        }
        if self.adds_new_pattern {
            changes.push("new patterns");
        }
        if self.modifies_routing {
            changes.push("routing logic");
        }
        changes.join(", ")
    }
}

/// Check if coverage.json or similar exists in the skill directory
fn coverage_file_likely_exists(path: &str) -> bool {
    // Check common coverage file locations relative to the skill
    if let Some(skill_root) = find_skill_root(path) {
        let coverage_paths = [
            format!("{}/coverage.json", skill_root),
            format!("{}/metrics/coverage.json", skill_root),
            format!("{}/stats/coverage.json", skill_root),
            format!("{}/.coverage.json", skill_root),
        ];

        for cov_path in coverage_paths {
            if Path::new(&cov_path).exists() {
                return true;
            }
        }
    }
    false
}

/// Find the root directory of the skill
fn find_skill_root(path: &str) -> Option<String> {
    let mut current = Path::new(path);
    while let Some(parent) = current.parent() {
        let parent_str = parent.to_string_lossy();
        if parent_str.contains("skills/") || parent_str.ends_with("skill") {
            // Check if SKILL.md exists
            let skill_md = parent.join("SKILL.md");
            if skill_md.exists() {
                return Some(parent_str.to_string());
            }
        }
        current = parent;
    }
    None
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

    // Only check skill-related core logic files
    if !is_skill_related(&file_path) || !is_core_logic_file(&file_path) {
        exit_ok();
    }

    // Get content
    let content = match get_content(tool_input) {
        Some(c) => c,
        None => exit_ok(),
    };

    // Analyze the changes
    let changes = analyze_changes(&content);

    // If significant changes were made
    if changes.is_significant() {
        // Check if coverage tracking exists in the content
        let has_coverage = has_coverage_tracking(&content);
        let coverage_file_exists = coverage_file_likely_exists(&file_path);

        if !has_coverage && !coverage_file_exists {
            exit_warn(&format!(
                "Skill changes detected ({}). Consider adding coverage tracking: \
                 deterministic_count/total_count ratio to measure ASR effectiveness. \
                 Target: >80% deterministic execution.",
                changes.description()
            ));
        }

        // If there's a coverage file, remind to update it
        if coverage_file_exists && !has_coverage {
            exit_warn(&format!(
                "Skill logic modified ({}). Remember to update coverage.json \
                 with new deterministic handlers.",
                changes.description()
            ));
        }
    }

    exit_ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_skill_related() {
        assert!(is_skill_related("~/.claude/skills/my-skill/src/lib.rs"));
        assert!(is_skill_related("crates/nexcore-skill-verify/src/main.rs"));
        assert!(is_skill_related("src/executor/mod.rs"));
        assert!(!is_skill_related("src/api/routes.rs"));
    }

    #[test]
    fn test_has_coverage_tracking() {
        assert!(has_coverage_tracking("let coverage = Coverage::new();"));
        assert!(has_coverage_tracking("self.hit_count += 1;"));
        assert!(has_coverage_tracking("track_execution(cmd);"));
        assert!(!has_coverage_tracking("fn main() {}"));
    }

    #[test]
    fn test_analyze_changes() {
        let handler_code = "fn handle_command(cmd: &str) -> Result<()> { Ok(()) }";
        let analysis = analyze_changes(handler_code);
        assert!(analysis.adds_handler);
        assert!(analysis.is_significant());

        let match_code = r#"
            match cmd {
                "a" => Ok(1),
                "b" => Ok(2),
                _ => Err(e),
            }
        "#;
        let analysis = analyze_changes(match_code);
        assert!(analysis.adds_match_arm);
        assert!(analysis.is_significant());
    }

    #[test]
    fn test_change_description() {
        let analysis = ChangeAnalysis {
            adds_handler: true,
            adds_match_arm: true,
            adds_new_pattern: false,
            modifies_routing: false,
        };
        let desc = analysis.description();
        assert!(desc.contains("handler"));
        assert!(desc.contains("match"));
    }
}
