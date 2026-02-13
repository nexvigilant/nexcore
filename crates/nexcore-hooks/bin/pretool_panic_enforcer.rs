//! Panic-Free Guarantee Enforcer (HOOK-M2-22)

use nexcore_hooks::parser::check_panic_patterns;
use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, is_test_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };
    // Skip enforcement in plan mode
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
    let violations = check_panic_patterns(content);
    if violations.is_empty() {
        exit_success_auto();
    }
    let mut msg = String::from("PANIC PATH DETECTED\n\n");
    for v in &violations {
        msg.push_str(&format!(
            "Line {}: {}\n  Fix: {}\n\n",
            v.line, v.pattern, v.fix
        ));
    }
    msg.push_str("If truly impossible to fail, use:\n  // INVARIANT: reason\n");
    exit_block(&msg);
}
