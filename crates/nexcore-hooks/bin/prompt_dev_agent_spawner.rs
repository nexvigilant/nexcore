//! UserPromptSubmit hook: Dev Agent Auto-Spawner
//!
//! Detects domain keywords in user input and recommends spawning
//! the appropriate -dev subagent for specialized handling.
//!
//! Pattern Matching:
//! - rust/cargo/crate → rust-dev
//! - hook/prehook/posthook → hook-lifecycle
//! - skill/SKILL.md → skill-dev
//! - mcp/server/tool → mcp-dev
//! - brain/session/artifact → brain-dev
//! - config/settings → config-dev
//! - strategy/strat/ptw → strat-dev
//! - vigil/friday/orchestrator → vigil-dev
//! - persona/style/output → persona-dev
//! - primitive/decompose/t1 → primitive-extractor
//!
//! Exit codes:
//! - 0: Success (recommendation in context)

use nexcore_hooks::{exit_success, exit_with_session_context, read_input};

/// Domain patterns and their corresponding dev agents
const DEV_AGENTS: &[(&[&str], &str, &str)] = &[
    // (keywords, agent_name, description)
    (
        &["rust", "cargo", "crate", "ownership", "lifetime", "trait"],
        "rust-dev",
        "Rust patterns and language structure",
    ),
    (
        &[
            "hook",
            "prehook",
            "posthook",
            "pretool",
            "posttool",
            "sessionstart",
        ],
        "hook-lifecycle",
        "Hook events, matchers, capabilities",
    ),
    (
        &["skill", "SKILL.md", "skill-dev", "frontmatter"],
        "skill-dev",
        "Skill creation and validation",
    ),
    (
        &["mcp", "server", "tool", "transport", "stdio"],
        "mcp-dev",
        "MCP server development",
    ),
    (
        &["brain", "session", "artifact", "memory", "implicit"],
        "brain-dev",
        "Working memory and artifacts",
    ),
    (
        &["config", "settings", "settings.json", "precedence"],
        "config-dev",
        "Configuration management",
    ),
    (
        &["strategy", "strat", "ptw", "playing to win", "capability"],
        "strat-dev",
        "Strategic planning framework",
    ),
    (
        &["vigil", "friday", "orchestrator", "event bus", "voice"],
        "vigil-dev",
        "AI orchestrator development",
    ),
    (
        &["persona", "style", "output", "tone", "compendious"],
        "persona-dev",
        "Output style configuration",
    ),
    (
        &["primitive", "decompose", "t1", "t2", "tier", "extract"],
        "primitive-extractor",
        "Primitive extraction and classification",
    ),
    (
        &["guardian", "homeostasis", "pamp", "damp", "sensing"],
        "guardian-orchestrator",
        "Guardian control loop",
    ),
    (
        &[
            "anatomy",
            "workspace",
            "crate organization",
            "smart pointer",
        ],
        "rust-anatomy-expert",
        "Rust architecture decisions",
    ),
    (
        &["forge", "autonomous", "primitive mining"],
        "forge",
        "Autonomous Rust development",
    ),
    (
        &["ctvp", "validation", "five problems", "reality"],
        "ctvp-validator",
        "Clinical trial validation paradigm",
    ),
    (
        &["smart", "goal", "achievable", "measurable"],
        "SMART-dev",
        "SMART goal framework",
    ),
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success(),
    };

    // Get user prompt
    let prompt = input.prompt.as_deref().unwrap_or("");
    if prompt.is_empty() {
        exit_success();
    }

    let prompt_lower = prompt.to_lowercase();

    // Find matching agents
    let mut matches: Vec<(&str, &str, usize)> = Vec::new();

    for (keywords, agent, desc) in DEV_AGENTS {
        let match_count = keywords
            .iter()
            .filter(|kw| prompt_lower.contains(&kw.to_lowercase()))
            .count();

        if match_count > 0 {
            matches.push((agent, desc, match_count));
        }
    }

    if matches.is_empty() {
        exit_success();
    }

    // Sort by match count (highest first)
    matches.sort_by(|a, b| b.2.cmp(&a.2));

    // Build recommendation context
    let mut context =
        String::from("🤖 **DEV AGENT RECOMMENDATIONS** ─────────────────────────────\n");

    let (top_agent, top_desc, _) = matches[0];
    context.push_str("   **Primary recommendation:**\n");
    context.push_str("   • ");
    context.push_str(top_agent);
    context.push_str("\n     ");
    context.push_str(top_desc);
    context.push('\n');

    if matches.len() > 1 {
        context.push_str("\n   **Also relevant:**\n");
        for (agent, desc, _) in matches.iter().skip(1).take(2) {
            context.push_str("   • ");
            context.push_str(agent);
            context.push_str(" - ");
            context.push_str(desc);
            context.push('\n');
        }
    }

    context.push_str("\n   Use: Task tool with subagent_type parameter\n");
    context.push_str("───────────────────────────────────────────────────────────\n");

    exit_with_session_context(&context);
}
