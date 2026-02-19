//! Stop checker hook.
//!
//! Verifies task completion before allowing Claude to stop.

use claude_hooks::{
    exit_success, read_input, write_output,
    input::StopInput,
    output::StopOutput,
    transcript::Transcript,
    HookResult,
};

/// Check if there are any obvious incomplete tasks.
fn check_completion(transcript: &Transcript) -> Result<(), String> {
    let summary = transcript.summary();

    // Check for errors in the session
    if summary.errors > 0 {
        let errors = transcript.errors();
        let last_error = errors.last();
        if let Some(err) = last_error {
            if let Some(error_val) = &err.error {
                return Err(format!(
                    "Session has {} error(s). Last error: {}",
                    summary.errors,
                    error_val.to_string().chars().take(100).collect::<String>()
                ));
            }
        }
    }

    // Check if tests were run (if test-related tools were used)
    let test_uses = transcript.tool_uses_for("Bash");
    let has_test_command = test_uses.iter().any(|t| {
        t.tool_input
            .as_ref()
            .and_then(|v| v.get("command"))
            .and_then(|v| v.as_str())
            .map(|cmd| cmd.contains("test") || cmd.contains("cargo t"))
            .unwrap_or(false)
    });

    // If we used test commands, check they succeeded
    if has_test_command {
        let results = transcript.tool_results();
        let last_test_result = results.iter().rev().find(|r| {
            r.tool_use_id.as_ref().map(|id| {
                test_uses.iter().any(|t| t.tool_use_id.as_ref() == Some(id))
            }).unwrap_or(false)
        });

        if let Some(result) = last_test_result {
            if result.error.is_some() {
                return Err("Tests may have failed. Please verify tests pass before stopping.".into());
            }
        }
    }

    Ok(())
}

fn main() -> HookResult<()> {
    let input: StopInput = read_input()?;

    // Don't re-check if we're already in a stop hook
    if input.stop_hook_active {
        exit_success();
    }

    // Try to load and check transcript
    match Transcript::load(&input.common.transcript_path) {
        Ok(transcript) => {
            if let Err(reason) = check_completion(&transcript) {
                let output = StopOutput::block(reason);
                write_output(&output)?;
            }
        }
        Err(_) => {
            // Can't load transcript, allow stopping
        }
    }

    exit_success();
}
