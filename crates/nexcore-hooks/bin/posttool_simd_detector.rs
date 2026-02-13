//! SIMD Opportunity Detector (Hook 48)
//!
//! Detects loops and operations that could benefit from SIMD vectorization.
//!
//! Patterns detected:
//! - Element-wise array operations in loops
//! - Reduction operations (sum, fold, min, max)
//! - Numeric loops with predictable array access
//!
//! This hook provides informational suggestions (warnings) rather than blocks,
//! since SIMD optimization is often a trade-off decision.

use nexcore_hooks::parser::performance::detect_simd_opportunities;
use nexcore_hooks::{HookOutput, exit_success_auto, is_rust_file, is_test_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) || is_test_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let opportunities = detect_simd_opportunities(content);
    if opportunities.is_empty() {
        exit_success_auto();
    }

    // SIMD opportunities are informational - warn but don't block
    let mut msg = format!(
        "SIMD OPPORTUNITY DETECTOR - {} potential optimization(s)\n\n",
        opportunities.len()
    );

    for opp in &opportunities {
        msg.push_str(&format!(
            "Line {}: {}\n  Pattern: {}\n  Opportunity: {}\n  Suggestion: {}\n  Potential: {}\n\n",
            opp.line, opp.code, opp.pattern, opp.opportunity, opp.suggestion, opp.speedup
        ));
    }

    msg.push_str("These are optimization suggestions, not errors.\n");
    msg.push_str("Justify with // SIMD: or // VECTORIZED: if intentionally scalar.");

    // Advisory output
    HookOutput::warn(&msg).emit();
    std::process::exit(0);
}
