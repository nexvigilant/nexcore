//! Deliverable Audit - Stop
//! Generates session audit report on exit.
//!
//! Stop hooks use "approve"/"block" not "allow"/"deny"

use nexcore_hooks::state::{SessionState, verified_dir};
use std::fs;
use std::process::Command;

fn main() {
    let state = SessionState::load();

    let uncommitted = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);

    let tests_pass = Command::new("cargo")
        .args(["test", "--quiet"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let compiles = Command::new("cargo")
        .args(["check", "--quiet"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let verified = state
        .assumptions
        .iter()
        .filter(|a| a.status == "verified")
        .count();
    let disproven = state
        .assumptions
        .iter()
        .filter(|a| a.status == "disproven")
        .count();
    let unverified = state
        .assumptions
        .iter()
        .filter(|a| a.status == "assumed")
        .count();

    let mut issues: Vec<String> = Vec::new();
    if !compiles {
        issues.push("Code does not compile".to_string());
    }
    if !tests_pass {
        issues.push("Tests failing".to_string());
    }
    if uncommitted > 0 {
        issues.push(format!("{} uncommitted files", uncommitted));
    }
    if unverified > 0 {
        issues.push(format!("{} unverified assumptions", unverified));
    }
    if disproven > 0 {
        issues.push(format!("{} disproven assumptions", disproven));
    }

    let issues_str = if issues.is_empty() {
        "- None".to_string()
    } else {
        issues
            .iter()
            .map(|i| format!("- {}", i))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let report = format!(
        "# Session Audit\n\n## Summary\n- Compile: {}\n- Tests: {}\n- Uncommitted: {}\n\n\
         ## Assumptions\n- Total: {}\n- Verified: {}\n- Disproven: {}\n- Unverified: {}\n\n\
         ## Requirements\n- Verified: {}\n\n## Issues\n{}\n",
        if compiles { "✓" } else { "✗" },
        if tests_pass { "✓" } else { "✗" },
        uncommitted,
        state.assumptions.len(),
        verified,
        disproven,
        unverified,
        state.requirements_verified,
        issues_str,
    );

    let dir = verified_dir();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(
        dir.join(format!("audit_{}.md", state.session_id())),
        &report,
    );

    // Output Stop-compatible JSON with "approve" decision
    let output = serde_json::json!({
        "continue": true,
        "decision": "approve",
        "stopReason": format!("Audit: {} assumptions, {} issues", state.assumptions.len(), issues.len()),
        "systemMessage": format!("📋 Session audit saved ({} issues)", issues.len())
    });
    println!("{}", output);
}
