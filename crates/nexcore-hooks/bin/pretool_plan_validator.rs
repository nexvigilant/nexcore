//! Plan Quality Validator - PreToolUse Event
//!
//! Validates plan file structure and completeness when writing to ~/.claude/plans/
//!
//! Validates:
//! - Has ## Objective section
//! - Has ## Implementation section
//! - Has ## Verification section
//! - Not empty or placeholder content
//!
//! Action: WARN if structure incomplete (does not block)

use nexcore_hooks::{exit_success_auto, exit_warn, read_input};
use std::path::Path;

/// Check if the file path is a plan file in ~/.claude/plans/
fn is_plan_file(path: &str) -> bool {
    let p = Path::new(path);

    // Check if it's in a plans directory
    let in_plans_dir = path.contains("/.claude/plans/") || path.contains("/plans/");

    // Check if it's a markdown file
    let is_markdown = p.extension().map(|e| e == "md").unwrap_or(false);

    in_plans_dir && is_markdown
}

/// Required sections for a quality plan
const REQUIRED_SECTIONS: &[(&str, &str)] = &[
    ("objective", "What are we trying to achieve?"),
    ("implementation", "How will we implement this?"),
    ("verification", "How will we verify it works?"),
];

/// Optional but recommended sections
const RECOMMENDED_SECTIONS: &[(&str, &str)] = &[
    ("scope", "What's in/out of scope?"),
    ("dependencies", "What does this depend on?"),
    ("risks", "What could go wrong?"),
];

/// Check for placeholder/empty content
fn has_substance(content: &str) -> bool {
    // Filter out markdown syntax and whitespace
    let text_chars: usize = content
        .lines()
        .filter(|l| !l.trim().starts_with('#'))
        .filter(|l| !l.trim().starts_with('-'))
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len())
        .sum();

    // Need at least 100 chars of actual content
    text_chars >= 100
}

/// Check for placeholder text
fn has_placeholder(content: &str) -> bool {
    let lower = content.to_lowercase();
    let placeholders = [
        "todo",
        "tbd",
        "fill in",
        "add details",
        "placeholder",
        "lorem ipsum",
        "xxx",
    ];
    placeholders.iter().any(|p| lower.contains(p))
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip in plan mode - we want to allow drafting plans
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // Only check plan files
    if !is_plan_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let lower_content = content.to_lowercase();

    // Check for required sections
    let mut missing_required: Vec<&str> = Vec::new();
    let mut missing_recommended: Vec<&str> = Vec::new();

    for (section, _desc) in REQUIRED_SECTIONS {
        // Look for "## Objective" or "# Objective" or "**Objective**"
        let has_section = lower_content.contains(&format!("# {}", section))
            || lower_content.contains(&format!("## {}", section))
            || lower_content.contains(&format!("**{}**", section));

        if !has_section {
            missing_required.push(section);
        }
    }

    for (section, _desc) in RECOMMENDED_SECTIONS {
        let has_section = lower_content.contains(&format!("# {}", section))
            || lower_content.contains(&format!("## {}", section))
            || lower_content.contains(&format!("**{}**", section));

        if !has_section {
            missing_recommended.push(section);
        }
    }

    // Check content quality
    let lacks_substance = !has_substance(content);
    let has_placeholders = has_placeholder(content);

    // Build warning message if issues found
    let mut issues: Vec<String> = Vec::new();

    if !missing_required.is_empty() {
        issues.push(format!(
            "Missing required sections: {}",
            missing_required.join(", ")
        ));
    }

    if lacks_substance {
        issues.push("Plan lacks substantive content (< 100 chars of text)".to_string());
    }

    if has_placeholders {
        issues.push("Plan contains placeholder text (TODO, TBD, etc.)".to_string());
    }

    if issues.is_empty() {
        // Plan looks good!
        if !missing_recommended.is_empty() {
            // Just a gentle suggestion
            exit_warn(&format!(
                "📋 Plan structure looks good!\n\
                 Consider adding: {}",
                missing_recommended.join(", ")
            ));
        }
        exit_success_auto();
    }

    // Build warning message
    let mut msg = String::from("📋 PLAN QUALITY WARNING\n\n");
    msg.push_str(&format!("File: {}\n\n", file_path));

    msg.push_str("Issues found:\n");
    for issue in &issues {
        msg.push_str(&format!("  ⚠️ {}\n", issue));
    }

    msg.push_str("\nA quality plan should have:\n");
    for (section, desc) in REQUIRED_SECTIONS {
        let status = if missing_required.contains(section) {
            "❌"
        } else {
            "✅"
        };
        msg.push_str(&format!(
            "  {} ## {} - {}\n",
            status,
            section.to_uppercase(),
            desc
        ));
    }

    // Warn but don't block - plans can be iteratively improved
    exit_warn(&msg);
}
