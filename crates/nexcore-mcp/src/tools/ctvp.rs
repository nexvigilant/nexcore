//! CTVP MCP tools — Clinical Trial Validation Paradigm for software.
//!
//! 5-phase validation: Preclinical → Safety → Efficacy → Confirmation → Surveillance
//! Plus Five Problems Protocol for testing gap discovery.
//!
//! ## T1 Primitive Grounding
//! - Phase 0 (Preclinical): ∃(Existence) — does it compile?
//! - Phase 1 (Safety): ∂(Boundary) — no panics, no unsafe?
//! - Phase 2 (Efficacy): κ(Comparison) — does it do what it claims?
//! - Phase 3 (Confirmation): σ(Sequence) + ν(Frequency) — under stress?
//! - Phase 4 (Surveillance): π(Persistence) + ν(Frequency) — in production?

use crate::params::ctvp::{CtvpFiveProblemsParams, CtvpPhasesListParams, CtvpScoreParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

/// Resolve the cargo binary path, avoiding the systemd-run alias trap.
/// Checks CARGO env var first (set by rustup), then absolute path, then PATH fallback.
fn cargo_bin() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("CARGO") {
        return std::path::PathBuf::from(p);
    }
    if let Ok(home) = std::env::var("HOME") {
        let abs = std::path::PathBuf::from(home).join(".cargo/bin/cargo");
        if abs.exists() {
            return abs;
        }
    }
    std::path::PathBuf::from("cargo")
}

struct Phase {
    number: u8,
    name: &'static str,
    clinical_analog: &'static str,
    question: &'static str,
    checks: &'static [&'static str],
    primitives: &'static str,
}

const PHASES: &[Phase] = &[
    Phase {
        number: 0,
        name: "Preclinical",
        clinical_analog: "Lab testing",
        question: "Does it compile and have basic structure?",
        checks: &[
            "Cargo.toml exists",
            "src/ exists",
            "cargo check succeeds",
            "No syntax errors",
        ],
        primitives: "∃",
    },
    Phase {
        number: 1,
        name: "Safety",
        clinical_analog: "Phase I (safety in humans)",
        question: "Is it safe? No panics, no undefined behavior?",
        checks: &[
            "No unsafe blocks",
            "No unwrap/expect",
            "Error types defined",
            "clippy clean",
        ],
        primitives: "∂+∝",
    },
    Phase {
        number: 2,
        name: "Efficacy",
        clinical_analog: "Phase II (does it work?)",
        question: "Does it do what it claims?",
        checks: &[
            "Unit tests exist",
            "Tests pass",
            "Public API documented",
            "Integration points tested",
        ],
        primitives: "κ+→",
    },
    Phase {
        number: 3,
        name: "Confirmation",
        clinical_analog: "Phase III (large-scale trials)",
        question: "Does it work under stress and at scale?",
        checks: &[
            "Property tests exist",
            "Fuzz tests exist",
            "Benchmark exists",
            "Edge cases covered",
        ],
        primitives: "σ+ν+N",
    },
    Phase {
        number: 4,
        name: "Surveillance",
        clinical_analog: "Phase IV (post-market)",
        question: "Is it monitored in production?",
        checks: &[
            "Logging present",
            "Metrics exported",
            "Error tracking",
            "Version in registry",
        ],
        primitives: "π+ν",
    },
];

/// Score a deliverable against CTVP phases.
pub fn score(params: CtvpScoreParams) -> Result<CallToolResult, McpError> {
    let target = Path::new(&params.target);
    if !target.exists() {
        return Err(McpError::invalid_params(
            format!("Target does not exist: {}", params.target),
            None,
        ));
    }

    let phase_filter: Option<u8> = match params.phase.as_deref() {
        None | Some("all") => None,
        Some(p) => match p.parse::<u8>() {
            Ok(n) if n <= 4 => Some(n),
            _ => {
                return Err(McpError::invalid_params(
                    format!("Invalid phase '{p}': must be 0-4 or 'all'"),
                    None,
                ));
            }
        },
    };

    let mut phase_results = Vec::new();
    let mut total_score = 0.0f64;
    let mut phase_count = 0u32;

    for phase in PHASES {
        if let Some(f) = phase_filter {
            if phase.number != f {
                continue;
            }
        }

        let (score, findings) = evaluate_phase(phase, &params.target);
        phase_results.push(json!({
            "phase": phase.number,
            "name": phase.name,
            "clinical_analog": phase.clinical_analog,
            "question": phase.question,
            "score": (score * 100.0).round() / 100.0,
            "max_score": 1.0,
            "findings": findings,
            "primitives": phase.primitives,
        }));
        total_score += score;
        phase_count += 1;
    }

    let overall = if phase_count > 0 {
        total_score / phase_count as f64
    } else {
        0.0
    };
    let highest_passed = phase_results
        .iter()
        .filter(|r| r["score"].as_f64().unwrap_or(0.0) >= 0.8)
        .filter_map(|r| r["phase"].as_u64())
        .max();

    let readiness = match highest_passed {
        Some(4) => "Production-ready (Phase IV)",
        Some(3) => "Confirmation-ready (Phase III passed)",
        Some(2) => "Efficacy-proven (Phase II passed)",
        Some(1) => "Safety-cleared (Phase I passed)",
        Some(0) => "Preclinical-only (Phase 0 passed)",
        _ => "Not validated",
    };

    let mut result = CallToolResult::success(vec![Content::text(
        json!({
            "target": params.target,
            "overall_score": (overall * 100.0).round() / 100.0,
            "readiness": readiness,
            "highest_phase_passed": highest_passed,
            "phases": phase_results,
        })
        .to_string(),
    )]);
    crate::tooling::attach_forensic_meta(&mut result, overall, Some(overall >= 0.8), "ctvp_score");
    Ok(crate::tooling::wrap_result(result))
}

/// Run Five Problems Protocol.
pub fn five_problems(params: CtvpFiveProblemsParams) -> Result<CallToolResult, McpError> {
    let target = Path::new(&params.target);
    if !target.exists() {
        return Err(McpError::invalid_params(
            format!("Target does not exist: {}", params.target),
            None,
        ));
    }

    let domain = params.domain.as_deref().unwrap_or("rust-crate");
    let problems = discover_five_problems(&params.target, domain);

    let high_count = problems
        .iter()
        .filter(|p| p.get("severity").and_then(|s| s.as_str()) == Some("high"))
        .count();
    let score = if high_count == 0 {
        1.0
    } else {
        (1.0 - (high_count as f64 * 0.2)).max(0.0)
    };

    let mut result = CallToolResult::success(vec![Content::text(
        json!({
            "target": params.target,
            "domain": domain,
            "problems": problems,
            "high_severity_count": high_count,
            "methodology": "Five Problems Protocol — systematic discovery of testing gaps",
        })
        .to_string(),
    )]);
    crate::tooling::attach_forensic_meta(
        &mut result,
        score,
        Some(high_count == 0),
        "ctvp_five_problems",
    );
    Ok(crate::tooling::wrap_result(result))
}

/// List CTVP phase definitions.
pub fn phases_list(_params: CtvpPhasesListParams) -> Result<CallToolResult, McpError> {
    let phases: Vec<serde_json::Value> = PHASES
        .iter()
        .map(|p| {
            json!({
                "phase": p.number,
                "name": p.name,
                "clinical_analog": p.clinical_analog,
                "question": p.question,
                "checks": p.checks,
                "primitives": p.primitives,
            })
        })
        .collect();

    Ok(crate::tooling::wrap_result(CallToolResult::success(vec![
        Content::text(json!({ "phases": phases, "count": phases.len() }).to_string()),
    ])))
}

fn evaluate_phase(phase: &Phase, target: &str) -> (f64, Vec<serde_json::Value>) {
    let target_path = Path::new(target);
    let is_crate = target_path.join("Cargo.toml").exists();
    let src_dir = target_path.join("src");

    let mut findings = Vec::new();
    let mut passed = 0u32;
    let total = phase.checks.len() as u32;

    match phase.number {
        0 => {
            // Preclinical: exists and compiles
            if is_crate {
                passed += 1;
                findings.push(json!({"check": "Cargo.toml exists", "status": "pass"}));
            } else {
                findings.push(json!({"check": "Cargo.toml exists", "status": "fail"}));
            }

            if src_dir.exists() {
                passed += 1;
                findings.push(json!({"check": "src/ exists", "status": "pass"}));
            } else {
                findings.push(json!({"check": "src/ exists", "status": "fail"}));
            }

            if is_crate {
                let check_result = std::process::Command::new(cargo_bin())
                    .args(["check"])
                    .current_dir(target)
                    .output();
                match &check_result {
                    Ok(out) if out.status.success() => {
                        passed += 1;
                        findings.push(json!({"check": "cargo check", "status": "pass"}));
                        // Infer no syntax errors from successful cargo check
                        passed += 1;
                        findings.push(json!({"check": "No syntax errors", "status": "inferred"}));
                    }
                    Ok(_) => {
                        findings.push(json!({"check": "cargo check", "status": "fail"}));
                        findings.push(json!({"check": "No syntax errors", "status": "skipped", "reason": "cargo check failed"}));
                    }
                    Err(e) => {
                        findings.push(json!({"check": "cargo check", "status": "error", "detail": format!("spawn failed: {e}")}));
                        findings.push(json!({"check": "No syntax errors", "status": "skipped", "reason": "cargo not available"}));
                    }
                }
            } else {
                findings.push(
                    json!({"check": "cargo check", "status": "skipped", "reason": "not a crate"}),
                );
                findings.push(json!({"check": "No syntax errors", "status": "skipped", "reason": "not a crate"}));
            }
        }
        1 => {
            // Safety: no unsafe, no unwrap
            let (unsafe_count, unwrap_count) = count_unsafe_unwrap(&src_dir);
            if unsafe_count == 0 {
                passed += 1;
            }
            findings.push(json!({"check": "No unsafe blocks", "status": if unsafe_count == 0 { "pass" } else { "fail" }, "count": unsafe_count}));
            if unwrap_count == 0 {
                passed += 1;
            }
            findings.push(json!({"check": "No unwrap/expect", "status": if unwrap_count == 0 { "pass" } else { "fail" }, "count": unwrap_count}));

            // Error types
            let has_error_type =
                has_pattern_in_dir(&src_dir, "enum") && has_pattern_in_dir(&src_dir, "Error");
            if has_error_type {
                passed += 1;
            }
            findings.push(json!({"check": "Error types defined", "status": if has_error_type { "pass" } else { "fail" }}));

            // Clippy
            if is_crate {
                match std::process::Command::new(cargo_bin())
                    .args(["clippy", "--", "-D", "warnings"])
                    .current_dir(target)
                    .output()
                {
                    Ok(out) if out.status.success() => {
                        passed += 1;
                        findings.push(json!({"check": "clippy clean", "status": "pass"}));
                    }
                    Ok(_) => {
                        findings.push(json!({"check": "clippy clean", "status": "fail"}));
                    }
                    Err(e) => {
                        findings.push(json!({"check": "clippy clean", "status": "error", "detail": format!("spawn failed: {e}")}));
                    }
                }
            } else {
                findings.push(
                    json!({"check": "clippy clean", "status": "skipped", "reason": "not a crate"}),
                );
            }
        }
        2 => {
            // Efficacy: tests exist and pass
            let has_tests = has_pattern_in_dir(&src_dir, "#[test]")
                || has_pattern_in_dir(&src_dir, "#[cfg(test)]");
            if has_tests {
                passed += 1;
            }
            findings.push(json!({"check": "Unit tests exist", "status": if has_tests { "pass" } else { "fail" }}));

            if is_crate && has_tests {
                match std::process::Command::new(cargo_bin())
                    .args(["test", "--lib"])
                    .current_dir(target)
                    .output()
                {
                    Ok(out) if out.status.success() => {
                        passed += 1;
                        findings.push(json!({"check": "Tests pass", "status": "pass"}));
                    }
                    Ok(_) => {
                        findings.push(json!({"check": "Tests pass", "status": "fail"}));
                    }
                    Err(e) => {
                        findings.push(json!({"check": "Tests pass", "status": "error", "detail": format!("spawn failed: {e}")}));
                    }
                }
            } else if !is_crate {
                findings.push(
                    json!({"check": "Tests pass", "status": "skipped", "reason": "not a crate"}),
                );
            } else {
                findings.push(
                    json!({"check": "Tests pass", "status": "skipped", "reason": "no tests found"}),
                );
            }

            let has_docs = has_pattern_in_dir(&src_dir, "///");
            if has_docs {
                passed += 1;
            }
            findings.push(json!({"check": "Public API documented", "status": if has_docs { "pass" } else { "fail" }}));

            let has_integration = target_path.join("tests").exists();
            if has_integration {
                passed += 1;
            }
            findings.push(json!({"check": "Integration points tested", "status": if has_integration { "pass" } else { "fail" }}));
        }
        3 => {
            // Confirmation: property tests, fuzz, benchmarks
            let has_proptest = has_pattern_in_dir(&src_dir, "proptest")
                || has_pattern_in_dir(&src_dir, "quickcheck");
            if has_proptest {
                passed += 1;
            }
            findings.push(json!({"check": "Property tests exist", "status": if has_proptest { "pass" } else { "fail" }}));

            let has_fuzz = target_path.join("fuzz").exists();
            if has_fuzz {
                passed += 1;
            }
            findings.push(json!({"check": "Fuzz tests exist", "status": if has_fuzz { "pass" } else { "fail" }}));

            let has_bench =
                target_path.join("benches").exists() || has_pattern_in_dir(&src_dir, "#[bench]");
            if has_bench {
                passed += 1;
            }
            findings.push(json!({"check": "Benchmark exists", "status": if has_bench { "pass" } else { "fail" }}));

            // Edge cases heuristic: look for boundary value patterns
            let has_edge = has_pattern_in_dir(&src_dir, "MAX")
                || has_pattern_in_dir(&src_dir, "MIN")
                || has_pattern_in_dir(&src_dir, "empty");
            if has_edge {
                passed += 1;
            }
            findings.push(json!({"check": "Edge cases covered", "status": if has_edge { "pass" } else { "fail" }}));
        }
        4 => {
            // Surveillance: logging, metrics, monitoring
            let has_logging =
                has_pattern_in_dir(&src_dir, "tracing::") || has_pattern_in_dir(&src_dir, "log::");
            if has_logging {
                passed += 1;
            }
            findings.push(json!({"check": "Logging present", "status": if has_logging { "pass" } else { "fail" }}));

            let has_metrics = has_pattern_in_dir(&src_dir, "metrics::")
                || has_pattern_in_dir(&src_dir, "prometheus");
            if has_metrics {
                passed += 1;
            }
            findings.push(json!({"check": "Metrics exported", "status": if has_metrics { "pass" } else { "fail" }}));

            let has_error_tracking =
                has_pattern_in_dir(&src_dir, "anyhow") || has_pattern_in_dir(&src_dir, "thiserror");
            if has_error_tracking {
                passed += 1;
            }
            findings.push(json!({"check": "Error tracking", "status": if has_error_tracking { "pass" } else { "fail" }}));

            // Registry check — just presence of version in Cargo.toml
            if is_crate {
                passed += 1;
            }
            findings.push(json!({"check": "Version in registry", "status": if is_crate { "inferred" } else { "fail" }}));
        }
        _ => {}
    }

    let score = if total > 0 {
        passed as f64 / total as f64
    } else {
        0.0
    };
    (score, findings)
}

fn count_unsafe_unwrap(dir: &Path) -> (u32, u32) {
    let mut unsafe_count = 0u32;
    let mut unwrap_count = 0u32;

    if let Ok(entries) = collect_rs(dir) {
        for path in entries {
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    let t = line.trim();
                    if t.starts_with("unsafe ") || t.contains("unsafe {") {
                        unsafe_count += 1;
                    }
                    if t.contains(".unwrap()") || t.contains(".expect(") {
                        unwrap_count += 1;
                    }
                }
            }
        }
    }
    (unsafe_count, unwrap_count)
}

fn has_pattern_in_dir(dir: &Path, pattern: &str) -> bool {
    if let Ok(entries) = collect_rs(dir) {
        for path in entries {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if content.contains(pattern) {
                    return true;
                }
            }
        }
    }
    false
}

/// Max recursion depth for directory walking (prevents symlink cycles and runaway scans).
const MAX_WALK_DEPTH: u32 = 20;
/// Max number of .rs files to collect (prevents OOM on workspace-level scans).
const MAX_RS_FILES: usize = 500;

fn collect_rs(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    collect_rs_rec(dir, &mut files, 0)?;
    Ok(files)
}

fn collect_rs_rec(
    dir: &Path,
    files: &mut Vec<std::path::PathBuf>,
    depth: u32,
) -> Result<(), std::io::Error> {
    if depth > MAX_WALK_DEPTH || files.len() >= MAX_RS_FILES {
        return Ok(());
    }
    if dir.is_dir() && !dir.is_symlink() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && !path.is_symlink() {
                collect_rs_rec(&path, files, depth.saturating_add(1))?;
            } else if path.extension().is_some_and(|e| e == "rs") {
                files.push(path);
                if files.len() >= MAX_RS_FILES {
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

fn discover_five_problems(target: &str, domain: &str) -> Vec<serde_json::Value> {
    let target_path = Path::new(target);
    let src_dir = target_path.join("src");
    let mut problems = Vec::new();

    // Problem 1: Testing Theater (tests that don't test anything meaningful)
    let mock_heavy = has_pattern_in_dir(&src_dir, "mock") || has_pattern_in_dir(&src_dir, "Mock");
    let assert_only_eq =
        !has_pattern_in_dir(&src_dir, "assert_ne") && !has_pattern_in_dir(&src_dir, "assert!(");
    problems.push(json!({
        "number": 1,
        "name": "Testing Theater",
        "description": "Tests that exist but don't catch real bugs",
        "indicators": {
            "mock_heavy": mock_heavy,
            "only_assert_eq": assert_only_eq,
        },
        "severity": if mock_heavy && assert_only_eq { "high" } else { "low" },
        "domain": domain,
    }));

    // Problem 2: Happy Path Bias
    let has_err_tests =
        has_pattern_in_dir(&src_dir, "Err(") && has_pattern_in_dir(&src_dir, "#[test]");
    problems.push(json!({
        "number": 2,
        "name": "Happy Path Bias",
        "description": "Only testing success cases, not failure modes",
        "indicators": {
            "error_path_tests": has_err_tests,
        },
        "severity": if !has_err_tests { "high" } else { "low" },
    }));

    // Problem 3: Integration Gaps
    let has_integration_tests = target_path.join("tests").exists();
    problems.push(json!({
        "number": 3,
        "name": "Integration Gaps",
        "description": "Unit tests pass but components fail when composed",
        "indicators": {
            "integration_test_dir": has_integration_tests,
        },
        "severity": if !has_integration_tests { "high" } else { "low" },
    }));

    // Problem 4: Boundary Blindness
    let has_boundary = has_pattern_in_dir(&src_dir, "MAX")
        || has_pattern_in_dir(&src_dir, "overflow")
        || has_pattern_in_dir(&src_dir, "saturating");
    problems.push(json!({
        "number": 4,
        "name": "Boundary Blindness",
        "description": "Missing edge case and boundary value tests",
        "indicators": {
            "boundary_awareness": has_boundary,
        },
        "severity": if !has_boundary { "medium" } else { "low" },
    }));

    // Problem 5: Regression Amnesia
    let has_regression = has_pattern_in_dir(&src_dir, "regression")
        || has_pattern_in_dir(&src_dir, "bug")
        || has_pattern_in_dir(&src_dir, "fix");
    problems.push(json!({
        "number": 5,
        "name": "Regression Amnesia",
        "description": "Fixed bugs that don't get pinning tests",
        "indicators": {
            "regression_markers": has_regression,
        },
        "severity": if !has_regression { "medium" } else { "low" },
    }));

    problems
}
