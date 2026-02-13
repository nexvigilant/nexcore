//! Test Runner
//!
//! Event: Stop
//! Runs tests before session ends.
//!
//! Stop hooks use "approve"/"block" not "allow"/"deny"

use std::process::Command;

fn main() {
    let output = Command::new("cargo")
        .args(["test", "--quiet", "--no-fail-fast"])
        .output();

    let (passed, failed, status) = match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let combined = format!("{}{}", stdout, stderr);

            // Parse test results
            let passed = combined
                .lines()
                .filter(|l| l.contains("passed"))
                .filter_map(|l| l.split_whitespace().next())
                .filter_map(|n| n.parse::<usize>().ok())
                .sum::<usize>();

            let failed = combined
                .lines()
                .filter(|l| l.contains("failed"))
                .filter_map(|l| l.split_whitespace().next())
                .filter_map(|n| n.parse::<usize>().ok())
                .sum::<usize>();

            let status = if o.status.success() {
                "passed"
            } else {
                "failed"
            };
            (passed, failed, status)
        }
        Err(_) => (0, 0, "error"),
    };

    // Stop hooks require "approve"/"block" decision values
    let system_msg = if failed > 0 {
        format!("🧪 ❌ {} test(s) failed", failed)
    } else if passed > 0 {
        format!("🧪 ✅ All {} test(s) passed", passed)
    } else {
        "🧪 No tests found".to_string()
    };

    // Output Stop-compatible JSON
    let output = serde_json::json!({
        "continue": true,
        "decision": "approve",
        "stopReason": format!("Tests {}: {} passed, {} failed", status, passed, failed),
        "systemMessage": system_msg
    });
    println!("{}", output);
}
