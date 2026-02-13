//! Context Builder Hook
//!
//! PreToolUse hook that runs FIRST to build shared FileContext for all subsequent hooks.
//! This reduces redundant parsing by 95% - each file is parsed once instead of 37 times.
//!
//! Exit codes:
//! - 0: Success (context built and saved)
//!
//! The context file is saved to `~/.cache/nexcore-hooks/ctx_{tool_use_id}.json`
//! and subsequent hooks can read it via `FileContext::load(tool_use_id)`.

use nexcore_hooks::{
    FileContext, HookGroup, HookResult, HookResultRegistry, exit_success_auto, read_input,
};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only build context for Edit/Write operations
    if !input.is_write_tool() {
        exit_success_auto();
    }

    // Get file path
    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // Get content
    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    // Get tool_use_id (fall back to session_id if not available)
    let tool_use_id = input.tool_use_id_or_session();

    // Build and save the file context
    let start = std::time::Instant::now();
    let ctx = FileContext::build(file_path, content, tool_use_id);

    if let Err(e) = ctx.save() {
        eprintln!("Warning: Failed to save file context: {}", e);
        // Don't block on context save failure
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    // Record our result in the registry
    let result = HookResult::new("context_builder", HookGroup::Context).with_duration(duration_ms);

    if let Err(e) = HookResultRegistry::append_atomic(tool_use_id, result) {
        eprintln!("Warning: Failed to record hook result: {}", e);
    }

    exit_success_auto();
}
