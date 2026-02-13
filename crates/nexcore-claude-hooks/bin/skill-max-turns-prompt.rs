//! Skill Max Turns Prompt Hook (PreToolUse:Skill)
//!
//! Prompts Matthew to consider max_turns when invoking skills.
//! Exit 1 (warn) reminds without blocking execution.

use nexcore_hook_lib::cytokine::emit_check_failed;
use nexcore_hook_lib::{ToolName, pass, read_input, warn};

const HOOK_NAME: &str = "skill-max-turns-prompt";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    // Check if this is a Skill tool invocation
    let is_skill = matches!(
        &input.tool_name,
        Some(ToolName::Unknown(name)) if name == "Skill"
    );

    if !is_skill {
        pass();
    }

    // Extract skill name from prompt if available
    let skill_name = input
        .tool_input
        .as_ref()
        .and_then(|t| t.prompt.as_deref())
        .unwrap_or("unknown");

    // Emit cytokine signal (IL-6 = acute warning, check failed)
    emit_check_failed(
        HOOK_NAME,
        &format!("skill invocation without max_turns: {skill_name}"),
    );

    // Warn with max_turns reminder (exit 1 = allow with message)
    warn(&format!(
        "⚡ SKILL INVOCATION: {skill_name}\n\
         Consider setting max_turns for token budget control.\n\
         Recommended: Explore=10, Plan=15, forge=30, general=20"
    ));
}
