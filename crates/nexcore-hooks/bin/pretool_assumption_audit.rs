//! # Assumption Audit Trail (PreToolUse Hook)
//!
//! Tracks accumulated assumptions during a session and enforces verification
//! before allowing further tool use when uncertainty exceeds thresholds.
//!
//! ## Purpose
//!
//! Implements Codex Commandment IX (MEASURE): all values carry confidence.
//! This hook monitors the session's epistemic state and blocks execution
//! when too many unverified assumptions accumulate.
//!
//! ## Thresholds
//!
//! | Level | Count | Action |
//! |-------|-------|--------|
//! | Normal | < 3 | Continue (exit 0) |
//! | Warning | 3-4 | Warn, suggest `cargo check` (exit 1) |
//! | Block | ≥ 5 | Block until verification (exit 2) |
//!
//! ## Integration
//!
//! - Reads from `SessionState::get_unverified_assumptions()`
//! - Reads from `SessionState::get_uncertain_count()`
//! - Pairs with `posttool_assumption_tracker` for write side
//!
//! ## Exit Codes
//!
//! - 0: Pass (auto-approve)
//! - 1: Warn (continue with message)
//! - 2: Block (requires verification)

use nexcore_hooks::state::SessionState;
use nexcore_hooks::{exit_block, exit_success_auto, exit_warn, read_input};

const WARN_THRESHOLD: usize = 3;
const BLOCK_THRESHOLD: usize = 5;

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let state = SessionState::load();
    let unverified = state.get_unverified_assumptions();
    let uncertain_count = state.get_uncertain_count();

    if uncertain_count >= BLOCK_THRESHOLD {
        exit_block(&format!(
            "Too many low-confidence assumptions ({} >= {}). Run verification before continuing.",
            uncertain_count, BLOCK_THRESHOLD
        ));
    }

    if unverified.len() >= WARN_THRESHOLD {
        exit_warn(&format!(
            "Accumulating assumptions: {} unverified, {} low-confidence. Consider running cargo check.",
            unverified.len(),
            uncertain_count
        ));
    }

    exit_success_auto();
}
