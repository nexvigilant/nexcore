//! PV Safety Validator (HOOK-PV-01)
//!
//! Enforces pharmacovigilance safety standards:
//! - Prevents hardcoded thresholds (e.g., prr > 2.0)
//! - Ensures use of SignalCriteria for threshold comparisons
//! - Flags missing international causality (WHO-UMC) when Naranjo is used

use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, is_test_file, read_input};
use regex::Regex;

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

    let mut violations = Vec::new();

    // 1. Detect hardcoded PRR/ROR thresholds
    let prr_re = Regex::new(r"prr\s*[>=]+\s*[0-9.]+").unwrap();
    let ror_re = Regex::new(r"ror\s*[>=]+\s*[0-9.]+").unwrap();

    for (i, line) in content.lines().enumerate() {
        if line.contains("// SAFETY:") || line.contains("// INVARIANT:") {
            continue;
        }

        if (prr_re.is_match(line) || ror_re.is_match(line))
            && !line.contains("SignalCriteria")
            && !line.contains("threshold")
        {
            violations.push(format!("Line {}: Hardcoded threshold detected. Use SignalCriteria::evans() or ema_gvp_ix() instead.", i + 1));
        }

        // 2. Detect Naranjo without WHO-UMC mention (encouraging dual assessment)
        if line.to_lowercase().contains("naranjo")
            && !content.to_lowercase().contains("who_umc")
            && !content.to_lowercase().contains("who-umc")
        {
            violations.push(format!("Line {}: Naranjo used without WHO-UMC. International standards recommend dual assessment where feasible.", i + 1));
        }
    }

    if violations.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("PV SAFETY VIOLATION\n\n");
    for v in violations {
        msg.push_str(&format!("- {}\n", v));
    }

    exit_block(&msg);
}
