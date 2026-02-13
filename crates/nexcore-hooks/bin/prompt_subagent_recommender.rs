//! Proactive Subagent Recommender Hook
//!
//! Event: UserPromptSubmit
//!
//! Analyzes user prompts and proactively recommends specialized subagents
//! when the task would benefit from delegation. Unlike the agent detector
//! which triggers on specific keywords, this hook uses heuristics to identify
//! complex tasks that warrant subagent usage.
//!
//! Recommendations are based on:
//! - Task complexity indicators (multi-step, architecture, etc.)
//! - Domain specificity (Rust patterns, PV signals, strategy)
//! - Exploration depth needed (codebase understanding)
//! - Parallel work opportunities (multiple independent analyses)
//!
//! Exit codes:
//! - 0: Success with optional recommendation context

use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    let prompt = match input.get_prompt() {
        Some(p) => p,
        None => exit_skip_prompt(),
    };

    // Skip very short prompts
    if prompt.len() < 20 {
        exit_skip_prompt();
    }

    // Check for subagent opportunities
    if let Some(recommendation) = analyze_for_subagent(prompt) {
        exit_with_context(&recommendation);
    }

    exit_skip_prompt();
}

fn analyze_for_subagent(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let word_count = words.len();

    // Collect all applicable recommendations
    let mut recommendations = Vec::new();

    // 1. Exploration tasks -> Explore agent
    if matches_exploration(&lower) {
        recommendations.push(SubagentRec {
            agent: "Explore",
            reason: "Codebase exploration or understanding",
            model: "sonnet",
        });
    }

    // 2. Architecture/planning tasks -> Plan agent
    if matches_planning(&lower) {
        recommendations.push(SubagentRec {
            agent: "Plan",
            reason: "Architecture design or implementation planning",
            model: "sonnet",
        });
    }

    // 3. Rust-specific complex tasks -> rust-anatomy-expert
    if matches_rust_complex(&lower) {
        recommendations.push(SubagentRec {
            agent: "rust-anatomy-expert",
            reason: "Complex Rust patterns or architecture",
            model: "sonnet",
        });
    }

    // 4. Strategy/business tasks -> strat-dev
    if matches_strategy(&lower) {
        recommendations.push(SubagentRec {
            agent: "strat-dev",
            reason: "Strategic planning or capability analysis",
            model: "sonnet",
        });
    }

    // 5. Primitive decomposition -> primitive-extractor
    if matches_primitives(&lower) {
        recommendations.push(SubagentRec {
            agent: "primitive-extractor",
            reason: "Domain decomposition or primitive analysis",
            model: "sonnet",
        });
    }

    // 6. Skill/hook development -> specialized agents
    if matches_extensibility(&lower) {
        recommendations.push(SubagentRec {
            agent: "extensibility-mastery",
            reason: "Hook, skill, or MCP development",
            model: "sonnet",
        });
    }

    // 7. Complex multi-step tasks (heuristic: long prompt + action words)
    if word_count > 30 && has_multiple_actions(&lower) {
        recommendations.push(SubagentRec {
            agent: "general-purpose",
            reason: "Complex multi-step task requiring autonomous execution",
            model: "sonnet",
        });
    }

    // 8. Validation/testing tasks -> ctvp-validator
    if matches_validation(&lower) {
        recommendations.push(SubagentRec {
            agent: "ctvp-validator",
            reason: "Test validation or coverage analysis",
            model: "sonnet",
        });
    }

    // Format recommendations if any
    if recommendations.is_empty() {
        return None;
    }

    let mut output = String::new();
    output.push_str("\n🤖 **SUBAGENT RECOMMENDATIONS** ─────────────────────────────\n");

    for (i, rec) in recommendations.iter().enumerate() {
        if i == 0 {
            output.push_str("   **Primary recommendation:**\n");
        } else if i == 1 {
            output.push_str("\n   **Also consider:**\n");
        }

        output.push_str(&["   • ", rec.agent, " (", rec.model, ")\n"].concat());
        output.push_str(&["     Reason: ", rec.reason, "\n"].concat());
    }

    output.push_str("\n   Use: Task tool with subagent_type parameter\n");
    output.push_str("───────────────────────────────────────────────────────────\n");

    Some(output)
}

struct SubagentRec {
    agent: &'static str,
    reason: &'static str,
    model: &'static str,
}

fn matches_exploration(s: &str) -> bool {
    let patterns = [
        "where is",
        "where are",
        "find all",
        "search for",
        "look for",
        "what files",
        "which files",
        "how does",
        "understand the",
        "explore",
        "codebase",
        "structure of",
        "architecture of",
    ];
    patterns.iter().any(|p| s.contains(p))
}

fn matches_planning(s: &str) -> bool {
    let patterns = [
        "plan",
        "design",
        "architect",
        "implement a",
        "build a",
        "create a system",
        "refactor",
        "restructure",
        "migrate",
        "how should i",
        "what approach",
        "best way to",
    ];
    patterns.iter().any(|p| s.contains(p))
}

fn matches_rust_complex(s: &str) -> bool {
    if !s.contains("rust") {
        return false;
    }
    let patterns = [
        "lifetime",
        "borrow",
        "trait",
        "generic",
        "async",
        "unsafe",
        "macro",
        "workspace",
        "crate structure",
        "ownership",
        "smart pointer",
        "typestate",
    ];
    patterns.iter().any(|p| s.contains(p))
}

fn matches_strategy(s: &str) -> bool {
    let patterns = [
        "strategy",
        "strategic",
        "business",
        "compete",
        "market",
        "capability",
        "capabilities",
        "playing to win",
        "ptw",
        "where to play",
        "how to win",
    ];
    patterns.iter().any(|p| s.contains(p))
}

fn matches_primitives(s: &str) -> bool {
    let patterns = [
        "primitive",
        "decompose",
        "break down",
        "building blocks",
        "fundamental",
        "atomic",
        "t1",
        "t2",
        "extract concept",
    ];
    patterns.iter().any(|p| s.contains(p))
}

fn matches_extensibility(s: &str) -> bool {
    let patterns = [
        "create hook",
        "new hook",
        "create skill",
        "new skill",
        "mcp server",
        "mcp tool",
        "subagent",
        "extend claude",
    ];
    patterns.iter().any(|p| s.contains(p))
}

fn matches_validation(s: &str) -> bool {
    let patterns = [
        "validate",
        "test coverage",
        "testing",
        "ctvp",
        "production ready",
        "mock theater",
        "five problems",
    ];
    patterns.iter().any(|p| s.contains(p))
}

fn has_multiple_actions(s: &str) -> bool {
    let action_words = [
        "and",
        "then",
        "also",
        "next",
        "after",
        "before",
        "first",
        "second",
        "finally",
        "additionally",
    ];
    let count = action_words.iter().filter(|w| s.contains(*w)).count();
    count >= 2
}
