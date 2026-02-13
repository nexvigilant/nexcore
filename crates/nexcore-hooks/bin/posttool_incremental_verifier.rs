//! Incremental Verification Enforcer (HOOK-M1-05)

use nexcore_hooks::parser::extract_constructs;
use nexcore_hooks::state::SessionState;
use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, read_input};

const EDITS_BLOCK: u32 = 15; // Count edit operations, not file line count
const FILES_BLOCK: u32 = 15;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };
    // Skip verification tracking in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }
    let fp = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };
    if !is_rust_file(fp) {
        exit_success_auto();
    }
    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };
    let mut state = SessionState::load();
    let constructs = extract_constructs(content);
    // Count edit operations (1 per Edit/Write) instead of file line count
    let total_edits = state.files_since_verification + 1;

    // Detect type-level edits (struct/trait/impl) — signature changes need test compilation
    let has_type_constructs = constructs
        .iter()
        .any(|c| c.starts_with("struct ") || c.starts_with("trait ") || c.starts_with("impl "));

    if has_type_constructs {
        state.increment_type_level_edits();
    }

    if total_edits > EDITS_BLOCK {
        let type_warning = if state.has_type_level_edits() {
            format!(
                "\n⚠ TYPE-LEVEL EDITS DETECTED ({} struct/trait/impl changes)\n\
                 ALSO RUN: `cargo test --no-run -p <crate>` to catch test compilation failures.\n\
                 Reason: `cargo check` misses test-only callsite breakage from signature changes.",
                state.type_level_edits
            )
        } else {
            String::new()
        };

        exit_block(&format!(
            "INCREMENTAL VERIFICATION REQUIRED\n\
             {total_edits} edits since last verification (threshold: {EDITS_BLOCK}).\n\n\
             REQUIRED: Run `cargo check` before adding more code.{type_warning}"
        ));
    }
    state.increment_files();
    for c in &constructs {
        state.add_unverified_construct(c);
    }
    if let Err(e) = state.save() {
        eprintln!("Failed to save hook state: {e}");
    }
    exit_success_auto();
}
