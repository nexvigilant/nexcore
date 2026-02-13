//! UACA Purity Enforcer (HOOK-M1-02)
//!
//! Event: PreToolUse (Edit/Write)
//! Enforces the "Master Standard" for code structure:
//! - L1 Atoms: < 20 logical lines
//! - L2 Molecules: < 50 logical lines

use nexcore_hooks::parser::purity::analyze_purity;
use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, is_test_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    if input.is_plan_mode() {
        exit_success_auto();
    }

    let fp = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(fp) || is_test_file(fp) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let metrics = analyze_purity(content, fp);

    match metrics.level {
        1 if metrics.logical_loc > 20 => {
            exit_block(&format!(
                "UACA PURITY VIOLATION: L1 Atom '{}' has {} logical lines (Limit: 20).\nRefactor into smaller atoms to maintain build-doctrine rigor.",
                fp, metrics.logical_loc
            ));
        }
        2 if metrics.logical_loc > 50 => {
            exit_block(&format!(
                "UACA PURITY VIOLATION: L2 Molecule '{}' has {} logical lines (Limit: 50).\nRefactor logic or delegate to L1 atoms.",
                fp, metrics.logical_loc
            ));
        }
        _ => exit_success_auto(),
    }
}
