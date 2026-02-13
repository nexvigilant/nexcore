//! Python Creation Blocker - Atomic Hook
//!
//! PreToolUse:Write hook that blocks creation of new .py files.
//! Allows edits to existing .py files (for migration work).
//!
//! # Codex Compliance
//! - **Tier**: T3 (Policy Hook)
//! - **Commandments**: VI (Match), VII (Type)
//!
//! # Cytokine Integration
//! - **Block**: Emits TNF-alpha (terminate) via cytokine bridge — policy violation
//! - **Pass**: No emission (homeostasis maintained)

use nexcore_hook_lib::cytokine::emit_tool_blocked;
use nexcore_hook_lib::{
    block, file_path_or_pass, pass, read_input, require_edit_tool, require_python_file,
};
use std::path::Path;

const HOOK_NAME: &str = "python-creation-blocker";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    // Check tool type - Commandment VI (Match)
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);

    // Only check Python files
    require_python_file(file_path);

    // Check if file already exists (edit vs create)
    if Path::new(file_path).exists() {
        // File exists - this is an edit, allow it
        pass();
    }

    // Emit cytokine signal before blocking (TNF-alpha = terminate, policy violation)
    let tool_name = input
        .tool_name
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_default();
    emit_tool_blocked(
        &tool_name,
        HOOK_NAME,
        "Python file creation blocked by 100% Rust policy",
    );

    // New .py file creation - block it
    block(&format!(
        "BLOCKED: Cannot create new Python file '{}'.\n\n\
         100% Rust policy in effect.\n\n\
         Options:\n\
         1. Create equivalent Rust code instead\n\
         2. If migrating existing Python, edit the file rather than recreating\n",
        file_path
    ));
}
