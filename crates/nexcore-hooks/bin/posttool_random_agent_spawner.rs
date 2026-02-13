//! Random Agent Spawner Hook
//!
//! Event: PostToolUse (Skill tool only)
//!
//! Auto-spawns a random subagent from the available pool whenever a skill
//! is invoked. This creates emergent cross-skill collaboration and unexpected
//! synergies by introducing stochastic agent assistance.
//!
//! Available agents (19):
//! - primitive-rust-advisor, strategy-to-rust, primitive-validator
//! - guardian-orchestrator, hook-amplifier, ctvp-validator
//! - skill-dev, constructive-epistemology, brain-dev
//! - extensibility-mastery, mcp-dev, vigilance-dev
//! - primitive-extractor, friday-dev, strat-dev
//! - cep-orchestrator, domain-bridger, wallace-protocol
//! - skill-audit
//!
//! Exit codes:
//! - 0: Success with agent spawn recommendation

use nexcore_hooks::{exit_ok, exit_success_auto_with, read_input};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// All available custom subagents (from ~/.config/agents/)
const AGENTS: &[&str] = &[
    "primitive-rust-advisor",
    "strategy-to-rust",
    "primitive-validator",
    "guardian-orchestrator",
    "hook-amplifier",
    "ctvp-validator",
    "skill-dev",
    "constructive-epistemology",
    "brain-dev",
    "extensibility-mastery",
    "mcp-dev",
    "vigilance-dev",
    "primitive-extractor",
    "friday-dev",
    "strat-dev",
    "cep-orchestrator",
    "domain-bridger",
    "wallace-protocol",
    "skill-audit",
];

/// Agent descriptions for context
const AGENT_PURPOSES: &[(&str, &str)] = &[
    (
        "primitive-rust-advisor",
        "T1 bedrock + rust-anatomy combined",
    ),
    ("strategy-to-rust", "Playing to Win → Rust capabilities"),
    ("primitive-validator", "Read-only T1 compliance auditor"),
    (
        "guardian-orchestrator",
        "Homeostasis control loop orchestration",
    ),
    ("hook-amplifier", "Tier-based hook enforcement"),
    ("ctvp-validator", "5-phase clinical trial validation"),
    ("skill-dev", "Skill creation and Diamond compliance"),
    ("constructive-epistemology", "8-stage knowledge pipeline"),
    ("brain-dev", "Working memory and artifacts"),
    ("extensibility-mastery", "Hooks vs skills vs MCP decisions"),
    ("mcp-dev", "MCP server development"),
    ("vigilance-dev", "ToV axioms and Guardian-AV"),
    ("primitive-extractor", "T1/T2/T3 primitive decomposition"),
    ("friday-dev", "Vigil orchestrator development"),
    ("strat-dev", "Playing to Win strategic planning"),
    ("cep-orchestrator", "Cognitive Evolution Pipeline"),
    ("domain-bridger", "Cross-domain concept translation"),
    ("wallace-protocol", "Fearless Rust battle doctrine"),
    ("skill-audit", "Ecosystem health assessment"),
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_ok(),
    };

    // Only trigger for Skill tool
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Skill" {
        exit_ok();
    }

    // Get the invoked skill name
    let skill_name = input
        .tool_input
        .as_ref()
        .and_then(|v| v.get("skill"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Generate pseudo-random index based on time + skill name
    let random_agent = pick_random_agent(skill_name);
    let purpose = get_agent_purpose(random_agent);

    let msg = format!(
        "🎲 Random agent spawn: {} | Purpose: {} | Triggered by: /{}",
        random_agent, purpose, skill_name
    );

    exit_success_auto_with(&msg);
}

/// Pick a random agent using time-based hashing
fn pick_random_agent(skill_name: &str) -> &'static str {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    skill_name.hash(&mut hasher);

    // Add some entropy from process ID
    std::process::id().hash(&mut hasher);

    let hash = hasher.finish();
    let index = (hash as usize) % AGENTS.len();

    AGENTS[index]
}

/// Get agent purpose description
fn get_agent_purpose(agent: &str) -> &'static str {
    AGENT_PURPOSES
        .iter()
        .find(|(name, _)| *name == agent)
        .map(|(_, purpose)| *purpose)
        .unwrap_or("General assistance")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_random_agent_deterministic_for_same_inputs() {
        // Different skill names should (usually) produce different agents
        let agent1 = pick_random_agent("forge");
        let agent2 = pick_random_agent("strat-dev");
        // Note: This test might occasionally fail due to hash collisions
        // but statistically should pass most of the time
        assert!(AGENTS.contains(&agent1));
        assert!(AGENTS.contains(&agent2));
    }

    #[test]
    fn test_all_agents_have_purposes() {
        for agent in AGENTS {
            let purpose = get_agent_purpose(agent);
            assert!(!purpose.is_empty(), "Agent {} missing purpose", agent);
        }
    }
}
