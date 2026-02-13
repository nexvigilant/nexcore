//! Test Specification Hook - PostToolUse (Bash - cargo test)
use nexcore_hooks::state::SessionState;
use nexcore_hooks::{exit_success_auto, exit_warn, read_input};
use regex::Regex;
use std::collections::HashSet;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };
    let ti = match &input.tool_input {
        Some(v) => v,
        None => exit_success_auto(),
    };
    let cmd = ti.get("command").and_then(|v| v.as_str()).unwrap_or("");
    if !cmd.contains("cargo test") {
        exit_success_auto();
    }

    let resp = match &input.tool_response {
        Some(v) => v,
        None => exit_success_auto(),
    };
    let out = resp.get("output").and_then(|v| v.as_str()).unwrap_or("");
    let err = resp.get("stderr").and_then(|v| v.as_str()).unwrap_or("");
    let combined = format!("{}\n{}", out, err);

    let Ok(re) = Regex::new(r"test\s+([\w:]+)\s+\.\.\.\s+(ok|FAILED|ignored)") else {
        exit_success_auto()
    };
    let tests: Vec<_> = re
        .captures_iter(&combined)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect();
    if tests.is_empty() {
        exit_success_auto();
    }

    let passed = tests.iter().filter(|(_, s)| s == "ok").count();
    let failed = tests.iter().filter(|(_, s)| s == "FAILED").count();

    let state = SessionState::load();
    let reqs: Vec<_> = state
        .explicit_requirements
        .iter()
        .chain(state.implicit_requirements.iter())
        .cloned()
        .collect();
    let cov = if reqs.is_empty() {
        100
    } else {
        let cov_set: HashSet<_> = tests
            .iter()
            .filter_map(|(n, _)| {
                reqs.iter().position(|r| {
                    r.to_lowercase()
                        .split_whitespace()
                        .any(|w| w.len() > 3 && n.to_lowercase().contains(w))
                })
            })
            .collect();
        (cov_set.len() * 100) / reqs.len().max(1)
    };

    let msg = format!(
        "TESTS: {} passed, {} failed | COVERAGE: {}%",
        passed, failed, cov
    );
    if failed > 0 {
        exit_warn(&msg);
    }
    if cov < 80 && !reqs.is_empty() {
        exit_warn(&format!("{} (below 80%)", msg));
    }
    eprintln!("{}", msg);
    exit_success_auto();
}
