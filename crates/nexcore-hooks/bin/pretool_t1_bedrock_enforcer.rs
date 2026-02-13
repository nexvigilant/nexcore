//! T1 Bedrock Enforcer - PreToolUse:Task
//!
//! Ensures Task tool calls (subagent spawning) are grounded in T1 primitives.
//! When spawning agents for Rust development, the prompt must reference:
//! - Primitive concepts (sequence, mapping, recursion, state)
//! - Primitive-first methodology
//! - Decomposition to foundations
//!
//! This ensures all agent work is built on primitive_primitive bedrock.

use nexcore_hooks::{exit_block, exit_ok, read_input};

/// T1 Universal Primitives and their synonyms
const T1_PRIMITIVE_TERMS: &[&str] = &[
    // Core T1 primitives
    "primitive",
    "primitives",
    "T1",
    "bedrock",
    "foundation",
    "fundamental",
    // Sequence primitive
    "sequence",
    "sequential",
    "iterator",
    "chain",
    "pipeline",
    // Mapping primitive
    "mapping",
    "transform",
    "conversion",
    "From",
    "Into",
    // Recursion primitive
    "recursion",
    "recursive",
    "self-reference",
    "tree",
    "graph",
    // State primitive
    "state",
    "stateful",
    "transition",
    "machine",
    "typestate",
];

/// Rust-related agent types that need primitive grounding
const RUST_AGENT_TYPES: &[&str] = &[
    "rust-dev",
    "rust-anatomy-expert",
    "forge",
    "primitive-extractor",
];

/// Check if the prompt references T1 primitives
fn has_primitive_grounding(prompt: &str) -> bool {
    let prompt_lower = prompt.to_lowercase();
    T1_PRIMITIVE_TERMS
        .iter()
        .any(|term| prompt_lower.contains(&term.to_lowercase()))
}

/// Check if this is a Rust-focused agent
fn is_rust_agent(agent_type: &str) -> bool {
    RUST_AGENT_TYPES.iter().any(|t| agent_type.contains(t))
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_ok(),
    };

    // Only check Task tool
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Task" {
        exit_ok();
    }

    // Get tool_input
    let tool_input = match &input.tool_input {
        Some(v) => v,
        None => exit_ok(),
    };

    // Extract subagent_type and prompt
    let subagent_type = tool_input
        .get("subagent_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let prompt = tool_input
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Only enforce on Rust-related agents
    if !is_rust_agent(subagent_type) {
        exit_ok();
    }

    // Check for primitive grounding - BLOCKS if missing
    if !has_primitive_grounding(prompt) {
        exit_block(&format!(
            "🛑 Rust agent '{}' requires T1 primitive grounding. \
             Reference: primitives, sequence, mapping, recursion, \
             or state in the prompt to ensure bedrock foundation.",
            subagent_type
        ));
    }

    exit_ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_grounding_detected() {
        assert!(has_primitive_grounding("Use primitive patterns for this"));
        assert!(has_primitive_grounding("Apply sequence operations"));
        assert!(has_primitive_grounding("Build on T1 foundations"));
        assert!(has_primitive_grounding("Use the mapping primitive"));
    }

    #[test]
    fn test_no_primitive_grounding() {
        assert!(!has_primitive_grounding("Just write some code"));
        assert!(!has_primitive_grounding("Implement the feature"));
        assert!(!has_primitive_grounding("Fix the bug"));
    }

    #[test]
    fn test_rust_agent_detection() {
        assert!(is_rust_agent("rust-dev"));
        assert!(is_rust_agent("rust-anatomy-expert"));
        assert!(is_rust_agent("forge"));
        assert!(!is_rust_agent("general-purpose"));
        assert!(!is_rust_agent("Explore"));
    }
}
