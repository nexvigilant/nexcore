//! Skill-Subagent Sync Enforcer - PostToolUse Hook
//!
//! PostToolUse:Write hook that enforces bidirectional skill-subagent creation.
//! When a skill is created, warns if no corresponding agent exists.
//! When an agent is created, warns if no corresponding skill exists.
//!
//! Locations checked:
//! - Skills: ~/.claude/skills/{name}/SKILL.md
//! - Agents: ~/.claude/agents/{name}.md
//!
//! Action: Warn when skill exists without agent or vice versa
//! Exit: 0 = pass (both exist or neither), 1 = warn (mismatch detected)
//!
//! # Codex Compliance
//! - **Tier**: T3 (Policy Hook)
//! - **Commandments**: VI (Match), XII (Enforce)
//! - **Grounding**: Path strings → T1(String), existence check → T1(bool)

use nexcore_hook_lib::cytokine::emit_check_failed;
use nexcore_hook_lib::{file_path_or_pass, pass, read_input, require_edit_tool, warn};
use std::path::Path;

const HOOK_NAME: &str = "skill-subagent-sync";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
    // Check tool type - Commandment VI (Match)
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);

    // Check if this is a skill creation
    if is_skill_file(file_path)
        && let Some(name) = extract_skill_name(file_path)
    {
        check_agent_exists(&name, file_path);
    }

    // Check if this is an agent creation
    if is_agent_file(file_path)
        && let Some(name) = extract_agent_name(file_path)
    {
        check_skill_exists(&name, file_path);
    }

    pass();
}

/// Check if path is a skill file (SKILL.md in ~/.claude/skills/)
fn is_skill_file(path: &str) -> bool {
    path.contains("/.claude/skills/") && path.ends_with("/SKILL.md")
}

/// Check if path is an agent file (.md in ~/.claude/agents/)
fn is_agent_file(path: &str) -> bool {
    path.contains("/.claude/agents/") && path.ends_with(".md")
}

/// Extract skill name from path like ~/.claude/skills/my-skill/SKILL.md
fn extract_skill_name(path: &str) -> Option<String> {
    let path = Path::new(path);
    // Get parent directory (the skill name)
    path.parent()?.file_name()?.to_str().map(String::from)
}

/// Extract agent name from path like ~/.claude/agents/my-agent.md
fn extract_agent_name(path: &str) -> Option<String> {
    let path = Path::new(path);
    let file_name = path.file_name()?.to_str()?;
    // Remove .md extension
    Some(file_name.strip_suffix(".md")?.to_string())
}

/// Check if corresponding agent exists for a skill
fn check_agent_exists(skill_name: &str, skill_path: &str) {
    let home = std::env::var("HOME").unwrap_or_default();
    let agent_path = format!("{home}/.claude/agents/{skill_name}.md");

    if !Path::new(&agent_path).exists() {
        emit_check_failed(HOOK_NAME, &format!("missing agent for skill: {skill_name}"));
        warn(&format!(
            "⚠️  SKILL-SUBAGENT SYNC WARNING\n\n\
             Skill created: {skill_path}\n\
             Missing agent: {agent_path}\n\n\
             Per policy, skills and subagents must be created together.\n\
             Create the corresponding agent file at:\n  {agent_path}\n\n\
             Agent template:\n\
             ---\n\
             name: {skill_name}\n\
             description: \"Use this agent when...\"\n\
             model: inherit\n\
             ---\n\n\
             [Agent system prompt here]",
        ));
    }
}

/// Check if corresponding skill exists for an agent
fn check_skill_exists(agent_name: &str, agent_path: &str) {
    let home = std::env::var("HOME").unwrap_or_default();
    let skill_path = format!("{home}/.claude/skills/{agent_name}/SKILL.md");

    if !Path::new(&skill_path).exists() {
        emit_check_failed(HOOK_NAME, &format!("missing skill for agent: {agent_name}"));
        warn(&format!(
            "⚠️  SKILL-SUBAGENT SYNC WARNING\n\n\
             Agent created: {agent_path}\n\
             Missing skill: {skill_path}\n\n\
             Per policy, skills and subagents must be created together.\n\
             Create the corresponding skill at:\n  {skill_path}\n\n\
             Skill template:\n\
             ---\n\
             name: {agent_name}\n\
             description: \"...\"\n\
             user-invocable: true\n\
             ---\n\n\
             # {agent_name}\n\n\
             [Skill content here]",
        ));
    }
}
