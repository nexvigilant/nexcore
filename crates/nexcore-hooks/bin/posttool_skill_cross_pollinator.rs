//! PostToolUse hook: Skill Cross-Pollinator
//!
//! Detects when a skill is modified and suggests related skills
//! that may also need updates for consistency.
//!
//! Pattern Detection:
//! - Shared vocabulary references
//! - Common primitive dependencies (T1/T2)
//! - Skill chains (A invokes B)
//! - Taxonomy siblings
//!
//! ToV Alignment:
//! - Feedback Loop (ℱ): Updates propagate across related skills
//! - Coherence: Maintains cross-skill consistency
//!
//! Exit codes:
//! - 0: Success (suggestions provided in context)

use nexcore_hooks::{exit_success, exit_with_session_context, read_input};
use std::collections::HashMap;
use std::path::PathBuf;

/// Known skill relationships (simplified graph)
/// Format: skill_name -> [related_skills]
fn build_skill_graph() -> HashMap<&'static str, Vec<&'static str>> {
    let mut graph = HashMap::new();

    // Primitive-based relationships
    graph.insert(
        "primitive-extractor",
        vec![
            "primitive-rust-foundation",
            "transfer-confidence",
            "domain-translator",
        ],
    );
    graph.insert(
        "primitive-rust-foundation",
        vec!["primitive-extractor", "rust-dev", "rust-anatomy-expert"],
    );

    // Skill development relationships
    graph.insert(
        "skill-dev",
        vec!["skill-audit", "skill-advisor", "extensibility-mastery"],
    );
    graph.insert(
        "skill-audit",
        vec!["skill-dev", "skill-advisor", "ctvp-validator"],
    );

    // Hook relationships
    graph.insert(
        "hook-lifecycle",
        vec!["hook-amplifier", "extensibility-mastery", "bond-architect"],
    );
    graph.insert(
        "hook-amplifier",
        vec!["hook-lifecycle", "polymer-composer", "bond-architect"],
    );

    // Strategy relationships
    graph.insert("strat-dev", vec!["SMART-dev", "vdag-orchestrator"]);
    graph.insert(
        "SMART-dev",
        vec!["strat-dev", "vdag-orchestrator", "ctvp-validator"],
    );

    // Validation relationships
    graph.insert(
        "ctvp-validator",
        vec!["SMART-dev", "skill-audit", "vdag-orchestrator"],
    );

    // Rust development relationships
    graph.insert(
        "rust-dev",
        vec!["rust-anatomy-expert", "primitive-rust-foundation", "forge"],
    );
    graph.insert(
        "rust-anatomy-expert",
        vec!["rust-dev", "primitive-rust-foundation", "forge"],
    );
    graph.insert(
        "forge",
        vec!["rust-dev", "rust-anatomy-expert", "primitive-extractor"],
    );

    // Persona/output relationships
    graph.insert("persona-dev", vec!["compendious-mach", "socratic-learner"]);
    graph.insert("compendious-mach", vec!["persona-dev"]);

    // Brain/memory relationships
    graph.insert("brain-dev", vec!["config-dev", "vigil-dev"]);

    // MCP relationships
    graph.insert("mcp-dev", vec!["nexcore-api-dev", "config-dev"]);

    // Epistemology relationships
    graph.insert(
        "constructive-epistemology",
        vec!["cep-pipeline", "primitive-extractor"],
    );
    graph.insert(
        "cep-pipeline",
        vec!["constructive-epistemology", "domain-translator"],
    );

    graph
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success(),
    };

    // Only trigger on Write/Edit to skill files
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if !matches!(tool_name, "Write" | "Edit") {
        exit_success();
    }

    // Check if this is a skill file
    let file_path = input
        .tool_input
        .as_ref()
        .and_then(|ti| ti.get("file_path").or_else(|| ti.get("path")))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if !file_path.contains("/.claude/skills/") {
        exit_success();
    }

    // Extract skill name from path
    let skill_name = extract_skill_name(file_path);
    if skill_name.is_empty() {
        exit_success();
    }

    // Find related skills
    let graph = build_skill_graph();
    let related = graph.get(skill_name.as_str()).cloned().unwrap_or_default();

    if related.is_empty() {
        exit_success();
    }

    // Check which related skills exist
    let skills_dir = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
        .join(".claude/skills");

    let existing_related: Vec<&str> = related
        .iter()
        .filter(|&s| skills_dir.join(s).exists())
        .copied()
        .collect();

    if existing_related.is_empty() {
        exit_success();
    }

    // Build context message
    let mut context = String::from("🔗 **CROSS-POLLINATION ALERT** ──────────────────────────\n");
    context.push_str(&format!("   Modified skill: {}\n", skill_name));
    context.push_str("   Consider reviewing related skills for consistency:\n\n");

    for related_skill in &existing_related {
        context.push_str(&format!(
            "   • {} ({})\n",
            related_skill,
            get_relationship_type(&skill_name, related_skill)
        ));
    }

    context.push_str("\n   Run `/skill-audit` to check cross-skill consistency.\n");
    context.push_str("─────────────────────────────────────────────────────────\n");

    exit_with_session_context(&context);
}

fn extract_skill_name(path: &str) -> String {
    // Extract skill name from path like ~/.claude/skills/skill-name/SKILL.md
    let path = PathBuf::from(path);

    for ancestor in path.ancestors() {
        if let Some(parent) = ancestor.parent() {
            if parent.ends_with("skills") || parent.to_string_lossy().contains("/.claude/skills") {
                if let Some(name) = ancestor.file_name() {
                    return name.to_string_lossy().to_string();
                }
            }
        }
    }

    String::new()
}

fn get_relationship_type(source: &str, target: &str) -> &'static str {
    // Determine relationship type for context
    match (source, target) {
        // Primitive relationships
        (s, t) if s.contains("primitive") || t.contains("primitive") => "primitive dependency",

        // Skill ecosystem
        (s, t) if s.contains("skill") && t.contains("skill") => "skill ecosystem",

        // Hook relationships
        (s, t) if s.contains("hook") || t.contains("hook") => "hook integration",

        // Rust development
        (s, t) if s.contains("rust") || t.contains("rust") => "Rust patterns",

        // Validation
        (s, t) if s.contains("valid") || t.contains("valid") => "validation chain",

        // Strategy
        (s, t) if s.contains("strat") || t.contains("SMART") => "strategic alignment",

        // Default
        _ => "related domain",
    }
}
