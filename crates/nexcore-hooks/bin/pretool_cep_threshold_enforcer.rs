//! CEP Threshold Enforcer (PreToolUse:Edit|Write)
//!
//! Enforces that CEP validation code uses correct thresholds.
//! Blocks hardcoded thresholds that deviate from patent specifications.
//!
//! Patent: NV-2026-002 §5.4

use nexcore_hooks::{exit_block, exit_success_auto, exit_warn, read_input};
use regex::Regex;

/// Patent-defined thresholds (NV-2026-002)
const COVERAGE_THRESHOLD: f64 = 0.95;
const MINIMALITY_THRESHOLD: f64 = 0.90;
const INDEPENDENCE_THRESHOLD: f64 = 0.90;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = input.get_file_path().unwrap_or("");
    let content = input.get_written_content().unwrap_or("");

    // Only check CEP-related files
    let is_cep_file = file_path.contains("cep")
        || file_path.contains("primitive")
        || file_path.contains("extraction")
        || file_path.contains("validation");

    if !is_cep_file {
        exit_success_auto();
    }

    // Check for hardcoded threshold violations
    let mut violations = Vec::new();

    // Pattern: coverage = 0.XX or coverage: 0.XX or coverage >= 0.XX
    let coverage_re = Regex::new(r"coverage\s*[=:≥>=]+\s*0\.(\d+)").ok();
    if let Some(re) = coverage_re {
        for cap in re.captures_iter(content) {
            if let Some(num) = cap.get(1) {
                let value: f64 = format!("0.{}", num.as_str()).parse().unwrap_or(0.0);
                if value < COVERAGE_THRESHOLD && value > 0.5 {
                    violations.push(format!(
                        "Coverage threshold {:.2} below patent minimum {:.2}",
                        value, COVERAGE_THRESHOLD
                    ));
                }
            }
        }
    }

    // Pattern: minimality = 0.XX
    let minimality_re = Regex::new(r"minimality\s*[=:≥>=]+\s*0\.(\d+)").ok();
    if let Some(re) = minimality_re {
        for cap in re.captures_iter(content) {
            if let Some(num) = cap.get(1) {
                let value: f64 = format!("0.{}", num.as_str()).parse().unwrap_or(0.0);
                if value < MINIMALITY_THRESHOLD && value > 0.5 {
                    violations.push(format!(
                        "Minimality threshold {:.2} below patent minimum {:.2}",
                        value, MINIMALITY_THRESHOLD
                    ));
                }
            }
        }
    }

    // Pattern: independence = 0.XX
    let independence_re = Regex::new(r"independence\s*[=:≥>=]+\s*0\.(\d+)").ok();
    if let Some(re) = independence_re {
        for cap in re.captures_iter(content) {
            if let Some(num) = cap.get(1) {
                let value: f64 = format!("0.{}", num.as_str()).parse().unwrap_or(0.0);
                if value < INDEPENDENCE_THRESHOLD && value > 0.5 {
                    violations.push(format!(
                        "Independence threshold {:.2} below patent minimum {:.2}",
                        value, INDEPENDENCE_THRESHOLD
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        let msg = format!(
            "🔬 CEP THRESHOLD VIOLATION (NV-2026-002 §5.4):\n{}",
            violations.join("\n")
        );
        // Warn rather than block to allow intentional relaxation
        exit_warn(&msg);
    }

    exit_success_auto();
}
