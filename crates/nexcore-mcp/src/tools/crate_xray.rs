//! Crate X-Ray — deep inspection, CTVP validation trials, development goals
//!
//! Combines structural analysis, quality metrics, grounding coverage, and
//! dependency graph into a single diagnostic. Uses CTVP clinical trial
//! phases (0-4) for progressive validation.

use crate::params::{CrateXrayGoalsParams, CrateXrayParams, CrateXrayTrialParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::{Path, PathBuf};

fn nexcore_root() -> PathBuf {
    PathBuf::from(
        std::env::var("NEXCORE_ROOT").unwrap_or_else(|_| format!("{}/nexcore", env!("HOME"))),
    )
}

fn normalize_crate_name(name: &str) -> String {
    // Check if the name as-is exists first (handles `signal`, `prima`, `antitransformer`, etc.)
    let as_is = nexcore_root().join("crates").join(name);
    if as_is.is_dir() {
        return name.to_string();
    }
    // Otherwise try with nexcore- prefix
    if name.starts_with("nexcore-") {
        name.to_string()
    } else {
        format!("nexcore-{name}")
    }
}

fn crate_dir(crate_name: &str) -> PathBuf {
    nexcore_root().join("crates").join(crate_name)
}

// ============================================================================
// Source analysis helpers
// ============================================================================

struct SourceStats {
    total_lines: usize,
    code_lines: usize,
    comment_lines: usize,
    blank_lines: usize,
    pub_fns: usize,
    pub_structs: usize,
    pub_enums: usize,
    impl_blocks: usize,
    test_fns: usize,
    modules: Vec<String>,
}

fn analyze_source(src_dir: &Path) -> SourceStats {
    let mut stats = SourceStats {
        total_lines: 0,
        code_lines: 0,
        comment_lines: 0,
        blank_lines: 0,
        pub_fns: 0,
        pub_structs: 0,
        pub_enums: 0,
        impl_blocks: 0,
        test_fns: 0,
        modules: Vec::new(),
    };

    let entries = match std::fs::read_dir(src_dir) {
        Ok(e) => e,
        Err(_) => return stats,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem != "lib" {
                    stats.modules.push(stem.to_string());
                }
            }

            let content = std::fs::read_to_string(&path).unwrap_or_default();
            for line in content.lines() {
                stats.total_lines += 1;
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    stats.blank_lines += 1;
                } else if trimmed.starts_with("//") || trimmed.starts_with("///") {
                    stats.comment_lines += 1;
                } else {
                    stats.code_lines += 1;
                }

                if trimmed.starts_with("pub fn ") || trimmed.starts_with("pub async fn ") {
                    stats.pub_fns += 1;
                }
                if trimmed.starts_with("pub struct ") {
                    stats.pub_structs += 1;
                }
                if trimmed.starts_with("pub enum ") {
                    stats.pub_enums += 1;
                }
                if trimmed.starts_with("impl ") {
                    stats.impl_blocks += 1;
                }
                if trimmed.starts_with("#[test]") || trimmed.starts_with("fn test_") {
                    stats.test_fns += 1;
                }
            }
        }
    }

    stats
}

// ============================================================================
// Grounding / Transfer / Safety analysis
// ============================================================================

struct GroundingInfo {
    grounded_count: usize,
    type_count: usize,
    state_mode_count: usize,
    dominant_primitives: Vec<String>,
}

fn analyze_grounding(src_dir: &Path) -> GroundingInfo {
    let grounding_path = src_dir.join("grounding.rs");
    let content = std::fs::read_to_string(&grounding_path).unwrap_or_default();
    let grounded_count = content.matches("impl GroundsTo for").count();
    let state_mode_count = content.matches("StateMode::").count();

    // Extract dominant primitives
    let mut dominants = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("with_dominant(LexPrimitiva::") {
            if let Some(start) = trimmed.find("LexPrimitiva::") {
                let after = &trimmed[start + 14..];
                if let Some(end) = after.find(|c: char| c == ',' || c == ')') {
                    let prim = after[..end].to_string();
                    if !dominants.contains(&prim) {
                        dominants.push(prim);
                    }
                }
            }
        }
    }

    // Count types across source files
    let mut type_count = 0;
    for module in &["primitives.rs", "composites.rs", "service_models.rs"] {
        let path = src_dir.join(module);
        if path.exists() {
            let mc = std::fs::read_to_string(&path).unwrap_or_default();
            type_count += mc.matches("pub struct ").count();
            type_count += mc.matches("pub enum ").count();
        }
    }

    GroundingInfo {
        grounded_count,
        type_count,
        state_mode_count,
        dominant_primitives: dominants,
    }
}

struct TransferInfo {
    mapping_count: usize,
    domains_covered: Vec<String>,
    types_with_transfers: usize,
}

fn analyze_transfers(src_dir: &Path) -> TransferInfo {
    let transfer_path = src_dir.join("transfer.rs");
    let content = std::fs::read_to_string(&transfer_path).unwrap_or_default();

    let mapping_count =
        content.matches("TransferMapping {").count() + content.matches("TransferMapping{").count();

    let mut domains = Vec::new();
    let mut source_types = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("domain:") || trimmed.starts_with("domain :") {
            let val = trimmed.split('"').nth(1).unwrap_or("").to_string();
            if !val.is_empty() && !domains.contains(&val) {
                domains.push(val);
            }
        }
        if trimmed.starts_with("source_type:") || trimmed.starts_with("source_type :") {
            let val = trimmed.split('"').nth(1).unwrap_or("").to_string();
            if !val.is_empty() && !source_types.contains(&val) {
                source_types.push(val);
            }
        }
    }

    TransferInfo {
        mapping_count,
        domains_covered: domains,
        types_with_transfers: source_types.len(),
    }
}

struct SafetyInfo {
    forbid_unsafe: bool,
    deny_unwrap: bool,
    deny_expect: bool,
    deny_panic: bool,
    unwrap_calls: usize,
    expect_calls: usize,
    panic_calls: usize,
    todo_count: usize,
}

fn analyze_safety(src_dir: &Path) -> SafetyInfo {
    let lib_content = std::fs::read_to_string(src_dir.join("lib.rs")).unwrap_or_default();

    // Check deny/forbid context — handles both individual and combined attributes:
    //   #![cfg_attr(not(test), deny(clippy::unwrap_used))]  (individual)
    //   #![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]  (combined)
    let has_deny = lib_content.contains("deny(") || lib_content.contains("forbid(");
    let mut info = SafetyInfo {
        forbid_unsafe: lib_content.contains("forbid(unsafe_code)"),
        deny_unwrap: has_deny && lib_content.contains("clippy::unwrap_used"),
        deny_expect: has_deny && lib_content.contains("clippy::expect_used"),
        deny_panic: has_deny && lib_content.contains("clippy::panic"),
        unwrap_calls: 0,
        expect_calls: 0,
        panic_calls: 0,
        todo_count: 0,
    };

    // Scan all .rs files for violations
    if let Ok(entries) = std::fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                for line in content.lines() {
                    let trimmed = line.trim();
                    // Skip test code and comments
                    if trimmed.starts_with("//") || trimmed.starts_with("///") {
                        continue;
                    }
                    if trimmed.contains(".unwrap()") {
                        info.unwrap_calls += 1;
                    }
                    if trimmed.contains(".expect(") {
                        info.expect_calls += 1;
                    }
                    if trimmed.contains("panic!(") {
                        info.panic_calls += 1;
                    }
                    if trimmed.contains("TODO")
                        || trimmed.contains("todo!")
                        || trimmed.contains("FIXME")
                    {
                        info.todo_count += 1;
                    }
                }
            }
        }
    }

    info
}

// ============================================================================
// Dependency analysis
// ============================================================================

struct DependencyInfo {
    internal_deps: Vec<String>,
    external_deps: Vec<String>,
    internal_count: usize,
    external_count: usize,
    layer: String,
}

fn analyze_dependencies(crate_path: &Path) -> DependencyInfo {
    let cargo_path = crate_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_path).unwrap_or_default();

    let mut internal = Vec::new();
    let mut external = Vec::new();

    let in_deps = content
        .lines()
        .skip_while(|l| !l.starts_with("[dependencies]"))
        .skip(1)
        .take_while(|l| !l.starts_with('['));

    for line in in_deps {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(name) = trimmed
            .split(|c: char| c == '=' || c.is_whitespace())
            .next()
        {
            let name = name.trim().to_string();
            if name.is_empty() {
                continue;
            }
            if trimmed.contains("path =")
                || name.starts_with("nexcore-")
                || name.starts_with("stem-")
                || name.starts_with("prima-")
            {
                internal.push(name);
            } else {
                external.push(name);
            }
        }
    }

    let ic = internal.len();
    let ec = external.len();

    let layer = match ic {
        0..=3 => "Foundation",
        4..=10 => "Domain",
        11..=25 => "Orchestration",
        _ => "Service",
    };

    DependencyInfo {
        internal_deps: internal,
        external_deps: external,
        internal_count: ic,
        external_count: ec,
        layer: layer.to_string(),
    }
}

// ============================================================================
// Reverse dependency analysis
// ============================================================================

fn count_reverse_deps(crate_name: &str) -> usize {
    let root = nexcore_root();
    let crates_dir = root.join("crates");
    let mut count = 0;

    if let Ok(entries) = std::fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            let cargo = entry.path().join("Cargo.toml");
            if cargo.exists() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    if dir_name == crate_name {
                        continue;
                    }
                }
                let content = std::fs::read_to_string(&cargo).unwrap_or_default();
                if content.contains(crate_name) {
                    count += 1;
                }
            }
        }
    }

    count
}

// ============================================================================
// Tool: crate_xray
// ============================================================================

pub fn xray(params: CrateXrayParams) -> Result<CallToolResult, McpError> {
    let crate_name = normalize_crate_name(&params.crate_name);
    let dir = crate_dir(&crate_name);

    if !dir.exists() {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": format!("Crate not found: {}", dir.display())}).to_string(),
        )]));
    }

    let src_dir = dir.join("src");
    let include_stats = params.include_stats.unwrap_or(true);

    // Core analyses
    let grounding = analyze_grounding(&src_dir);
    let transfers = analyze_transfers(&src_dir);
    let safety = analyze_safety(&src_dir);
    let deps = analyze_dependencies(&dir);
    let rev_deps = count_reverse_deps(&crate_name);

    let grounding_pct = if grounding.type_count > 0 {
        (grounding.grounded_count as f64 / grounding.type_count as f64 * 100.0).round()
    } else {
        0.0
    };

    let transfer_pct = if grounding.type_count > 0 {
        let expected = grounding.type_count * 3;
        (transfers.mapping_count as f64 / expected as f64 * 100.0)
            .min(100.0)
            .round()
    } else {
        0.0
    };

    let safety_score = [
        safety.forbid_unsafe,
        safety.deny_unwrap,
        safety.deny_expect,
        safety.deny_panic,
    ]
    .iter()
    .filter(|&&b| b)
    .count();

    // Overall health grade
    let health_score = (safety_score as f64 / 4.0) * 25.0
        + grounding_pct * 0.25
        + transfer_pct * 0.25
        + if rev_deps > 0 { 10.0 } else { 0.0 }
        + if deps.internal_count <= 5 { 5.0 } else { 0.0 };

    let grade = match health_score as u32 {
        90..=100 => "GOLD",
        70..=89 => "SILVER",
        50..=69 => "BRONZE",
        _ => "UNRATED",
    };

    let mut result = json!({
        "crate": crate_name,
        "path": dir.display().to_string(),
        "grade": grade,
        "health_score": format!("{:.0}/100", health_score),
        "layer": deps.layer,
        "structure": {
            "types": grounding.type_count,
            "modules": [], // filled below
        },
        "grounding": {
            "grounded": grounding.grounded_count,
            "total_types": grounding.type_count,
            "coverage": format!("{grounding_pct}%"),
            "state_modes": grounding.state_mode_count,
            "dominant_primitives": grounding.dominant_primitives,
        },
        "transfers": {
            "mappings": transfers.mapping_count,
            "domains": transfers.domains_covered,
            "types_covered": transfers.types_with_transfers,
            "coverage": format!("{transfer_pct}%"),
        },
        "safety": {
            "forbid_unsafe": safety.forbid_unsafe,
            "deny_unwrap": safety.deny_unwrap,
            "deny_expect": safety.deny_expect,
            "deny_panic": safety.deny_panic,
            "score": format!("{safety_score}/4"),
            "violations": {
                "unwrap_calls": safety.unwrap_calls,
                "expect_calls": safety.expect_calls,
                "panic_calls": safety.panic_calls,
            },
            "todos": safety.todo_count,
        },
        "dependencies": {
            "internal": deps.internal_deps,
            "external": deps.external_deps,
            "internal_count": deps.internal_count,
            "external_count": deps.external_count,
        },
        "adoption": {
            "reverse_deps": rev_deps,
            "is_orphaned": rev_deps == 0,
        },
    });

    if include_stats {
        let stats = analyze_source(&src_dir);
        result["structure"]["modules"] = json!(stats.modules);
        result["stats"] = json!({
            "total_lines": stats.total_lines,
            "code_lines": stats.code_lines,
            "comment_lines": stats.comment_lines,
            "blank_lines": stats.blank_lines,
            "doc_ratio": if stats.total_lines > 0 {
                format!("{:.0}%", stats.comment_lines as f64 / stats.total_lines as f64 * 100.0)
            } else {
                "0%".to_string()
            },
            "pub_fns": stats.pub_fns,
            "pub_structs": stats.pub_structs,
            "pub_enums": stats.pub_enums,
            "impl_blocks": stats.impl_blocks,
            "test_fns": stats.test_fns,
        });
    }

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Tool: crate_xray_trial (CTVP Phases)
// ============================================================================

pub fn trial(params: CrateXrayTrialParams) -> Result<CallToolResult, McpError> {
    let crate_name = normalize_crate_name(&params.crate_name);
    let dir = crate_dir(&crate_name);

    if !dir.exists() {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": format!("Crate not found: {}", dir.display())}).to_string(),
        )]));
    }

    let src_dir = dir.join("src");
    let run_all = params.phase.is_none();
    let phase = params.phase.unwrap_or(255); // 255 = run all

    let mut phases = Vec::new();

    // Phase 0: Preclinical — does it have the right structure?
    if run_all || phase == 0 {
        let lib_exists = src_dir.join("lib.rs").exists();
        let cargo_exists = dir.join("Cargo.toml").exists();
        let has_src = src_dir.exists();

        let required = [
            "primitives.rs",
            "composites.rs",
            "grounding.rs",
            "transfer.rs",
            "prelude.rs",
        ];
        let present: Vec<&str> = required
            .iter()
            .filter(|m| src_dir.join(m).exists())
            .copied()
            .collect();
        let missing: Vec<&str> = required
            .iter()
            .filter(|m| !src_dir.join(m).exists())
            .copied()
            .collect();

        let pass = lib_exists && cargo_exists && has_src && missing.is_empty();
        phases.push(json!({
            "phase": 0,
            "name": "Preclinical",
            "description": "Structural integrity — files, modules, Cargo.toml",
            "status": if pass { "PASS" } else { "FAIL" },
            "checks": {
                "lib_rs": lib_exists,
                "cargo_toml": cargo_exists,
                "src_dir": has_src,
                "gold_modules_present": present,
                "gold_modules_missing": missing,
            }
        }));
    }

    // Phase 1: Safety — denials, no violations
    if run_all || phase == 1 {
        let safety = analyze_safety(&src_dir);
        let all_denials =
            safety.forbid_unsafe && safety.deny_unwrap && safety.deny_expect && safety.deny_panic;
        let no_violations =
            safety.unwrap_calls == 0 && safety.expect_calls == 0 && safety.panic_calls == 0;
        let pass = all_denials && no_violations;

        phases.push(json!({
            "phase": 1,
            "name": "Safety",
            "description": "Safety denials enforced, no panic paths in non-test code",
            "status": if pass { "PASS" } else if all_denials { "WARN" } else { "FAIL" },
            "checks": {
                "forbid_unsafe": safety.forbid_unsafe,
                "deny_unwrap": safety.deny_unwrap,
                "deny_expect": safety.deny_expect,
                "deny_panic": safety.deny_panic,
                "unwrap_violations": safety.unwrap_calls,
                "expect_violations": safety.expect_calls,
                "panic_violations": safety.panic_calls,
                "todo_markers": safety.todo_count,
            },
            "remediation": if !all_denials {
                "Add #![forbid(unsafe_code)] #![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)] to lib.rs"
            } else if !no_violations {
                "Replace .unwrap()/.expect()/panic!() with Result returns or .unwrap_or_default()"
            } else {
                "None needed"
            }
        }));
    }

    // Phase 2: Efficacy — grounding, transfers, serde
    if run_all || phase == 2 {
        let grounding = analyze_grounding(&src_dir);
        let transfers = analyze_transfers(&src_dir);

        let grounding_complete =
            grounding.grounded_count >= grounding.type_count && grounding.type_count > 0;
        let expected_transfers = grounding.type_count * 3;
        let transfers_complete =
            transfers.mapping_count >= expected_transfers && expected_transfers > 0;

        // Check serde derives
        let mut serde_count = 0;
        for module in &["primitives.rs", "composites.rs", "service_models.rs"] {
            let path = src_dir.join(module);
            if path.exists() {
                let c = std::fs::read_to_string(&path).unwrap_or_default();
                serde_count += c.matches("Serialize, Deserialize").count();
            }
        }
        let serde_complete = serde_count >= grounding.type_count;

        let pass = grounding_complete && transfers_complete && serde_complete;

        phases.push(json!({
            "phase": 2,
            "name": "Efficacy",
            "description": "All types grounded, transfers mapped, serde derives complete",
            "status": if pass { "PASS" } else { "FAIL" },
            "checks": {
                "grounding": {
                    "complete": grounding_complete,
                    "grounded": grounding.grounded_count,
                    "total": grounding.type_count,
                    "gap": grounding.type_count.saturating_sub(grounding.grounded_count),
                },
                "transfers": {
                    "complete": transfers_complete,
                    "mappings": transfers.mapping_count,
                    "expected": expected_transfers,
                    "gap": expected_transfers.saturating_sub(transfers.mapping_count),
                    "domains": transfers.domains_covered,
                },
                "serde": {
                    "complete": serde_complete,
                    "with_serde": serde_count,
                    "total": grounding.type_count,
                },
                "dominant_primitives": grounding.dominant_primitives,
                "state_modes_declared": grounding.state_mode_count,
            }
        }));
    }

    // Phase 3: Confirmation — tests exist, code coverage indicators
    if run_all || phase == 3 {
        let stats = analyze_source(&src_dir);
        let has_tests = stats.test_fns > 0;
        let tc = count_types(&src_dir);
        let test_ratio = if tc > 0 {
            stats.test_fns as f64 / tc as f64
        } else {
            0.0
        };

        // Check for test categories
        let all_src = glob_rs_content(&src_dir);
        let has_serde_tests =
            all_src.contains("serde_json::to_string") && all_src.contains("serde_json::from_str");
        let has_grounding_tests =
            all_src.contains("primitive_composition()") || all_src.contains("GroundsTo");
        let has_transfer_tests =
            all_src.contains("transfer_confidence") || all_src.contains("transfers_for_type");
        let has_boundary_tests = all_src.contains("test")
            && (all_src.contains("negative") || all_src.contains("-1") || all_src.contains("0.0"));

        let pass = has_tests && has_serde_tests && has_grounding_tests;

        phases.push(json!({
            "phase": 3,
            "name": "Confirmation",
            "description": "Tests exist for behavior, serde, grounding, transfers, boundaries",
            "status": if pass { "PASS" } else if has_tests { "PARTIAL" } else { "FAIL" },
            "checks": {
                "test_count": stats.test_fns,
                "tests_per_type": format!("{:.1}", test_ratio),
                "categories": {
                    "behavioral": has_tests,
                    "serde_roundtrip": has_serde_tests,
                    "grounding_verification": has_grounding_tests,
                    "transfer_mapping": has_transfer_tests,
                    "boundary_testing": has_boundary_tests,
                },
                "missing_categories": {
                    "serde": !has_serde_tests,
                    "grounding": !has_grounding_tests,
                    "transfer": !has_transfer_tests,
                    "boundary": !has_boundary_tests,
                }
            }
        }));
    }

    // Phase 4: Surveillance — adoption, blast radius, ecosystem integration
    if run_all || phase == 4 {
        let rev_deps = count_reverse_deps(&crate_name);
        let deps = analyze_dependencies(&dir);
        let stats = analyze_source(&src_dir);

        let is_adopted = rev_deps > 0;
        let is_prelude_exported = {
            let lib = std::fs::read_to_string(src_dir.join("lib.rs")).unwrap_or_default();
            lib.contains("pub mod prelude")
        };

        phases.push(json!({
            "phase": 4,
            "name": "Surveillance",
            "description": "Ecosystem integration — reverse deps, blast radius, adoption",
            "status": if is_adopted { "ACTIVE" } else { "ORPHANED" },
            "checks": {
                "reverse_dependencies": rev_deps,
                "layer": deps.layer,
                "internal_deps": deps.internal_count,
                "external_deps": deps.external_count,
                "has_prelude": is_prelude_exported,
                "blast_radius": if rev_deps > 10 { "HIGH" } else if rev_deps > 3 { "MEDIUM" } else { "LOW" },
                "doc_lines": stats.comment_lines,
                "doc_ratio": if stats.total_lines > 0 {
                    format!("{:.0}%", stats.comment_lines as f64 / stats.total_lines as f64 * 100.0)
                } else {
                    "0%".to_string()
                },
            }
        }));
    }

    // Summary
    let pass_count = phases
        .iter()
        .filter(|p| {
            let s = p["status"].as_str().unwrap_or("");
            s == "PASS" || s == "ACTIVE"
        })
        .count();

    let overall = if pass_count == phases.len() {
        "ALL TRIALS PASSED"
    } else if pass_count > phases.len() / 2 {
        "PARTIAL — some trials need attention"
    } else {
        "NEEDS WORK — multiple trial failures"
    };

    let result = json!({
        "crate": crate_name,
        "trial_summary": overall,
        "passed": pass_count,
        "total": phases.len(),
        "phases": phases,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// Count types across standard module files
fn count_types(src_dir: &Path) -> usize {
    let mut count = 0;
    for module in &["primitives.rs", "composites.rs", "service_models.rs"] {
        let path = src_dir.join(module);
        if path.exists() {
            let c = std::fs::read_to_string(&path).unwrap_or_default();
            count += c.matches("pub struct ").count();
            count += c.matches("pub enum ").count();
        }
    }
    count
}

fn glob_rs_content(src_dir: &Path) -> String {
    let mut all = String::new();
    if let Ok(entries) = std::fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    all.push_str(&content);
                }
            }
        }
    }
    all
}

// ============================================================================
// Tool: crate_xray_goals
// ============================================================================

pub fn goals(params: CrateXrayGoalsParams) -> Result<CallToolResult, McpError> {
    let crate_name = normalize_crate_name(&params.crate_name);
    let dir = crate_dir(&crate_name);

    if !dir.exists() {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": format!("Crate not found: {}", dir.display())}).to_string(),
        )]));
    }

    let src_dir = dir.join("src");
    let max_goals = params.max_goals.unwrap_or(10);

    let safety = analyze_safety(&src_dir);
    let grounding = analyze_grounding(&src_dir);
    let transfers = analyze_transfers(&src_dir);
    let deps = analyze_dependencies(&dir);
    let stats = analyze_source(&src_dir);
    let rev_deps = count_reverse_deps(&crate_name);
    let type_count = grounding.type_count;

    let mut dev_goals = Vec::new();

    // Priority 1: Safety gaps (blocking)
    if !safety.forbid_unsafe {
        dev_goals.push(json!({
            "priority": "P0-BLOCKING",
            "goal": "Add #![forbid(unsafe_code)] to lib.rs",
            "impact": "Prevents all unsafe code paths",
            "effort": "trivial",
            "ctvp_phase": 1,
        }));
    }
    if !safety.deny_unwrap || !safety.deny_expect || !safety.deny_panic {
        dev_goals.push(json!({
            "priority": "P0-BLOCKING",
            "goal": "Add #![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)] to lib.rs",
            "impact": "Catches panic paths at compile time",
            "effort": "trivial",
            "ctvp_phase": 1,
        }));
    }
    if safety.unwrap_calls > 0 || safety.expect_calls > 0 || safety.panic_calls > 0 {
        dev_goals.push(json!({
            "priority": "P1-HIGH",
            "goal": format!("Replace {} unwrap + {} expect + {} panic calls with Result returns",
                safety.unwrap_calls, safety.expect_calls, safety.panic_calls),
            "impact": "Eliminates runtime panics",
            "effort": if safety.unwrap_calls + safety.expect_calls + safety.panic_calls > 10 { "medium" } else { "small" },
            "ctvp_phase": 1,
        }));
    }

    // Priority 2: Structure gaps
    let required_modules = [
        "primitives.rs",
        "composites.rs",
        "grounding.rs",
        "transfer.rs",
        "prelude.rs",
    ];
    let missing_modules: Vec<&&str> = required_modules
        .iter()
        .filter(|m| !src_dir.join(m).exists())
        .collect();
    if !missing_modules.is_empty() {
        dev_goals.push(json!({
            "priority": "P1-HIGH",
            "goal": format!("Create missing modules: {:?}", missing_modules),
            "impact": "Completes gold standard structure",
            "effort": "medium",
            "ctvp_phase": 0,
        }));
    }

    // Priority 3: Grounding gaps
    if grounding.grounded_count < type_count {
        let gap = type_count - grounding.grounded_count;
        dev_goals.push(json!({
            "priority": "P1-HIGH",
            "goal": format!("Add GroundsTo implementations for {} ungrounded types", gap),
            "impact": format!("Grounding coverage: {}% → 100%",
                if type_count > 0 { grounding.grounded_count * 100 / type_count } else { 0 }),
            "effort": if gap > 10 { "large" } else { "medium" },
            "ctvp_phase": 2,
        }));
    }

    // Priority 4: Transfer gaps
    let expected_transfers = type_count * 3;
    if transfers.mapping_count < expected_transfers {
        let gap = expected_transfers - transfers.mapping_count;
        dev_goals.push(json!({
            "priority": "P2-MEDIUM",
            "goal": format!("Add {} transfer mappings (3 per type: PV, Biology, Economics)", gap),
            "impact": "Enables cross-domain reasoning",
            "effort": if gap > 15 { "large" } else { "medium" },
            "ctvp_phase": 2,
        }));
    }

    // Priority 5: Test gaps
    if stats.test_fns == 0 {
        dev_goals.push(json!({
            "priority": "P1-HIGH",
            "goal": format!("Add tests: behavioral + serde roundtrip + grounding verification for {} types", type_count),
            "impact": "No tests = no confidence in correctness",
            "effort": "medium",
            "ctvp_phase": 3,
        }));
    } else if stats.test_fns < type_count {
        dev_goals.push(json!({
            "priority": "P2-MEDIUM",
            "goal": format!("Add {} more tests ({} types, {} tests — target 2x)",
                type_count * 2 - stats.test_fns, type_count, stats.test_fns),
            "impact": "Better test coverage per type",
            "effort": "medium",
            "ctvp_phase": 3,
        }));
    }

    let all_src = glob_rs_content(&src_dir);
    if !all_src.contains("serde_json::from_str") && type_count > 0 {
        dev_goals.push(json!({
            "priority": "P2-MEDIUM",
            "goal": "Add serde round-trip tests for all types",
            "impact": "Validates serialization correctness",
            "effort": "small",
            "ctvp_phase": 3,
        }));
    }

    // Priority 6: Adoption
    if rev_deps == 0 && type_count > 0 {
        dev_goals.push(json!({
            "priority": "P3-LOW",
            "goal": "Wire crate into at least one consumer (nexcore-mcp or domain crate)",
            "impact": "Orphaned crates provide no value",
            "effort": "small",
            "ctvp_phase": 4,
        }));
    }

    // Priority 7: Documentation
    let doc_ratio = if stats.total_lines > 0 {
        stats.comment_lines as f64 / stats.total_lines as f64
    } else {
        0.0
    };
    if doc_ratio < 0.15 && type_count > 0 {
        dev_goals.push(json!({
            "priority": "P3-LOW",
            "goal": format!("Improve doc coverage ({:.0}% → target 20%+)", doc_ratio * 100.0),
            "impact": "Better discoverability and cross-domain transfer docs",
            "effort": "medium",
            "ctvp_phase": 4,
        }));
    }

    // Priority 8: TODOs
    if safety.todo_count > 0 {
        dev_goals.push(json!({
            "priority": "P2-MEDIUM",
            "goal": format!("Resolve {} TODO/FIXME markers", safety.todo_count),
            "impact": "Incomplete implementations",
            "effort": if safety.todo_count > 5 { "medium" } else { "small" },
            "ctvp_phase": 2,
        }));
    }

    // Priority 9: Dependency minimality
    if deps.external_count > 5 {
        dev_goals.push(json!({
            "priority": "P3-LOW",
            "goal": format!("Reduce external dependencies ({} → target ≤5)", deps.external_count),
            "impact": "Faster compilation, smaller blast radius",
            "effort": "medium",
            "ctvp_phase": 4,
        }));
    }

    // Truncate to max
    dev_goals.truncate(max_goals);

    // Compute progress summary
    let total_checks = 9; // all categories above
    let passing_checks = [
        safety.forbid_unsafe && safety.deny_unwrap && safety.deny_expect && safety.deny_panic,
        safety.unwrap_calls == 0 && safety.expect_calls == 0 && safety.panic_calls == 0,
        missing_modules.is_empty(),
        grounding.grounded_count >= type_count && type_count > 0,
        transfers.mapping_count >= expected_transfers && expected_transfers > 0,
        stats.test_fns >= type_count,
        rev_deps > 0,
        doc_ratio >= 0.15,
        safety.todo_count == 0,
    ]
    .iter()
    .filter(|&&b| b)
    .count();

    let progress_pct = if total_checks > 0 {
        (passing_checks as f64 / total_checks as f64 * 100.0).round()
    } else {
        0.0
    };

    let result = json!({
        "crate": crate_name,
        "progress": format!("{progress_pct}%"),
        "passing": format!("{passing_checks}/{total_checks}"),
        "goal_count": dev_goals.len(),
        "goals": dev_goals,
        "summary": {
            "types": type_count,
            "grounding_coverage": format!("{}%", if type_count > 0 { grounding.grounded_count * 100 / type_count } else { 0 }),
            "transfer_coverage": format!("{}%", if expected_transfers > 0 { (transfers.mapping_count * 100).min(expected_transfers * 100) / expected_transfers } else { 0 }),
            "safety_score": format!("{}/4", [safety.forbid_unsafe, safety.deny_unwrap, safety.deny_expect, safety.deny_panic].iter().filter(|&&b| b).count()),
            "test_count": stats.test_fns,
            "reverse_deps": rev_deps,
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
