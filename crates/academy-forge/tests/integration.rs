//! Integration tests for academy-forge — exercises the full extract→validate pipeline
//! against real nexcore crates in the workspace.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use academy_forge::{CrateAnalysis, DomainAnalysis, ValidationReport};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // tests/ lives inside crates/academy-forge/, so ../../ is workspace root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("cannot resolve workspace root")
        .to_path_buf()
}

fn crate_path(name: &str) -> PathBuf {
    workspace_root().join("crates").join(name)
}

// ═══════════════════════════════════════════════════════════════════════════
// forge_extract
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extract_nexcore_tov_returns_valid_analysis() {
    let path = crate_path("nexcore-tov");
    let analysis: CrateAnalysis =
        academy_forge::extract_crate(&path, None).expect("extract_crate failed");

    assert_eq!(analysis.name, "nexcore-tov");
    assert!(!analysis.version.is_empty());
    assert!(!analysis.modules.is_empty(), "should discover modules");
    assert!(analysis.domain.is_none(), "no domain requested");
}

#[test]
fn extract_with_vigilance_domain_includes_domain_analysis() {
    let path = crate_path("nexcore-tov");
    let analysis: CrateAnalysis =
        academy_forge::extract_crate(&path, Some("vigilance")).expect("extract_crate failed");

    let domain: &DomainAnalysis = analysis.domain.as_ref().expect("domain should be present");
    assert_eq!(domain.axioms.len(), 5, "ToV has 5 axioms");
    assert_eq!(domain.harm_types.len(), 8, "ToV has 8 harm types (A-H)");
    assert_eq!(
        domain.conservation_laws.len(),
        11,
        "ToV has 11 conservation laws"
    );
    assert_eq!(domain.theorems.len(), 3, "ToV has 3 theorems");
    assert_eq!(
        domain.dependency_dag.nodes.len(),
        5,
        "DAG has 5 axiom nodes"
    );
    assert_eq!(domain.dependency_dag.edges.len(), 5, "DAG has 5 edges");

    // Signal thresholds match canonical values
    let t = &domain.signal_thresholds;
    assert!((t.prr - 2.0).abs() < f64::EPSILON);
    assert!((t.chi_square - 3.841).abs() < f64::EPSILON);
    assert!((t.eb05 - 2.0).abs() < f64::EPSILON);
}

#[test]
fn extract_nexcore_primitives_has_types() {
    let path = crate_path("nexcore-primitives");
    let analysis = academy_forge::extract_crate(&path, None).expect("extract_crate failed");

    assert_eq!(analysis.name, "nexcore-primitives");
    // Should discover public types or enums from a real crate
    let total_items =
        analysis.public_types.len() + analysis.public_enums.len() + analysis.traits.len();
    assert!(
        total_items > 0,
        "should find public items in nexcore-primitives"
    );
}

#[test]
fn extract_nonexistent_crate_returns_error() {
    let path = crate_path("nexcore-does-not-exist-12345");
    let result = academy_forge::extract_crate(&path, None);
    assert!(result.is_err(), "should error on missing crate");
}

#[test]
fn extract_unknown_domain_returns_error() {
    let path = crate_path("nexcore-tov");
    let result = academy_forge::extract_crate(&path, Some("unknown-domain"));
    assert!(result.is_err(), "should error on unknown domain");
}

// ═══════════════════════════════════════════════════════════════════════════
// forge_validate
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn validate_minimal_valid_content_passes_schema() {
    let content = serde_json::json!({
        "id": "tov-01",
        "title": "Introduction to Theory of Vigilance",
        "description": "Learn the foundational axioms of ToV",
        "stages": []
    });

    let report: ValidationReport = academy_forge::validate(&content, None);
    // No stages means no sub-field errors; only id/title/desc are required at top level
    let schema_errors: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.rule.starts_with('R') && f.rule.len() <= 2)
        .filter(|f| matches!(f.severity, academy_forge::validate::Severity::Error))
        .collect();
    assert!(
        schema_errors.is_empty(),
        "valid content should pass schema: {schema_errors:?}"
    );
}

#[test]
fn validate_empty_object_fails_r1() {
    let content = serde_json::json!({});
    let report = academy_forge::validate(&content, None);
    assert!(!report.passed, "empty object should fail");
    assert!(
        report.error_count >= 3,
        "missing id, title, description = 3 errors"
    );
    let r1_count = report.findings.iter().filter(|f| f.rule == "R1").count();
    assert_eq!(r1_count, 3);
}

#[test]
fn validate_with_domain_checks_accuracy() {
    let path = crate_path("nexcore-tov");
    let analysis =
        academy_forge::extract_crate(&path, Some("vigilance")).expect("extract_crate failed");
    let domain = analysis.domain.as_ref().unwrap();

    // Content that mentions all ToV concepts should pass accuracy
    let content = serde_json::json!({
        "id": "tov-01",
        "title": "Theory of Vigilance",
        "description": "Complete ToV coverage",
        "text": "System Decomposition, Hierarchical Organization, Conservation Constraints, Safety Manifold, Emergence, Acute, Cumulative, Off-Target, Cascade, Idiosyncratic, Saturation, Interaction, Population, Mass/Amount, Energy/Gradient, State Normalization, Flux Continuity, Catalyst Invariance, Entropy Increase, Momentum, Capacity/Saturation, Charge Conservation, Stoichiometry, Structural Invariant, Predictability Theorem, Attenuation Theorem, Intervention Theorem, 2.0, 3.841",
        "stages": []
    });

    let report = academy_forge::validate(&content, Some(domain));
    let accuracy_errors: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            let rule_num: u32 = f.rule.trim_start_matches('R').parse().unwrap_or(0);
            (9..=14).contains(&rule_num)
        })
        .filter(|f| matches!(f.severity, academy_forge::validate::Severity::Error))
        .collect();
    assert!(
        accuracy_errors.is_empty(),
        "complete ToV content should pass accuracy: {accuracy_errors:?}"
    );
}

#[test]
fn validate_missing_axiom_flags_r9() {
    let path = crate_path("nexcore-tov");
    let analysis =
        academy_forge::extract_crate(&path, Some("vigilance")).expect("extract_crate failed");
    let domain = analysis.domain.as_ref().unwrap();

    // Mention 4 of 5 axioms — missing "Safety Manifold"
    let content = serde_json::json!({
        "id": "tov-01",
        "title": "Incomplete ToV",
        "description": "Missing an axiom",
        "text": "System Decomposition, Hierarchical Organization, Conservation Constraints, Emergence",
        "stages": []
    });

    let report = academy_forge::validate(&content, Some(domain));
    let r9s: Vec<_> = report.findings.iter().filter(|f| f.rule == "R9").collect();
    assert_eq!(r9s.len(), 1, "should flag 1 missing axiom");
    assert!(
        r9s[0].message.contains("Safety Manifold"),
        "should identify Safety Manifold as missing"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Serialization round-trip (ensures IR is JSON-serializable for MCP)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn crate_analysis_serializes_to_json() {
    let path = crate_path("nexcore-tov");
    let analysis =
        academy_forge::extract_crate(&path, Some("vigilance")).expect("extract_crate failed");

    let json = serde_json::to_value(&analysis).expect("CrateAnalysis should serialize");
    assert!(json.get("name").is_some());
    assert!(json.get("modules").is_some());
    assert!(json.get("domain").is_some());

    let domain = json.get("domain").unwrap();
    assert!(domain.get("axioms").is_some());
    assert!(domain.get("harm_types").is_some());
    assert!(domain.get("signal_thresholds").is_some());
}

#[test]
fn validate_tov_01_content_file_passes() {
    let content_path = workspace_root().join("content/pathways/tov-01.json");
    let raw = std::fs::read_to_string(&content_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", content_path.display()));
    let content: serde_json::Value =
        serde_json::from_str(&raw).expect("tov-01.json is not valid JSON");

    // Extract domain IR for accuracy checking (R9-R14)
    let crate_path = crate_path("nexcore-tov");
    let analysis =
        academy_forge::extract_crate(&crate_path, Some("vigilance")).expect("extract_crate failed");
    let domain = analysis.domain.as_ref().unwrap();

    let report: ValidationReport = academy_forge::validate(&content, Some(domain));

    // Print all findings for diagnostics
    for f in &report.findings {
        eprintln!(
            "[{:?}] {} — {} ({})",
            f.severity,
            f.rule,
            f.message,
            f.field_path.as_deref().unwrap_or("-")
        );
    }

    let errors: Vec<_> = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, academy_forge::validate::Severity::Error))
        .collect();
    assert!(
        errors.is_empty(),
        "tov-01.json should have zero errors, got {}: {:?}",
        errors.len(),
        errors
    );
    assert!(
        report.passed,
        "tov-01.json validation should pass (got {} findings: {} errors, {} warnings, {} advisories)",
        report.total_findings, report.error_count, report.warning_count, report.advisory_count,
    );
}

#[test]
fn validation_report_serializes_to_json() {
    let content = serde_json::json!({
        "id": "tov-01",
        "title": "Test",
        "description": "Test",
        "stages": []
    });

    let report = academy_forge::validate(&content, None);
    let json = serde_json::to_value(&report).expect("ValidationReport should serialize");
    assert!(json.get("passed").is_some());
    assert!(json.get("total_findings").is_some());
    assert!(json.get("findings").is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// Experiential pipeline: extract → scaffold → validate
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn scaffold_pipeline_extract_scaffold_validate() {
    // Step 1: Extract domain IR from nexcore-tov
    let path = crate_path("nexcore-tov");
    let analysis =
        academy_forge::extract_crate(&path, Some("vigilance")).expect("extract_crate failed");
    let domain = analysis.domain.as_ref().expect("domain should be present");

    // Step 2: Generate scaffold
    let params = academy_forge::ScaffoldParams {
        pathway_id: "tov-99".to_string(),
        title: "Pipeline Test Pathway".to_string(),
        domain: "vigilance".to_string(),
    };
    let scaffold = academy_forge::scaffold(domain, &params);

    // Step 3: Verify scaffold is valid JSON with expected structure
    assert_eq!(scaffold["id"], "tov-99");
    assert_eq!(scaffold["domain"], "vigilance");
    let stages = scaffold["stages"]
        .as_array()
        .expect("stages should be array");
    assert!(
        stages.len() >= 4,
        "should have at least 4 stages (axioms + harm + conservation + theorems)"
    );

    // Step 4: Validate scaffold output through the 27-rule engine
    let report: academy_forge::ValidationReport = academy_forge::validate(&scaffold, Some(domain));

    // Print all findings for diagnostics
    for f in &report.findings {
        eprintln!(
            "[{:?}] {} — {} ({})",
            f.severity,
            f.rule,
            f.message,
            f.field_path.as_deref().unwrap_or("-")
        );
    }

    let errors: Vec<_> = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, academy_forge::validate::Severity::Error))
        .collect();
    assert!(
        errors.is_empty(),
        "scaffold output should pass validation with zero errors, got {}: {:?}",
        errors.len(),
        errors
    );

    // Step 5: Verify Bloom progression is monotonic non-decreasing
    let bloom_order = [
        "Remember",
        "Understand",
        "Apply",
        "Analyze",
        "Evaluate",
        "Create",
    ];
    let mut last_bloom_idx = 0;
    for stage in stages {
        if let Some(bloom) = stage.get("bloomLevel").and_then(|b| b.as_str()) {
            let idx = bloom_order.iter().position(|&b| b == bloom).unwrap_or(0);
            assert!(
                idx >= last_bloom_idx,
                "Bloom regression: {} after {}",
                bloom,
                bloom_order[last_bloom_idx]
            );
            last_bloom_idx = idx;
        }
    }

    // Step 6: Verify passing scores are non-decreasing
    let mut last_score = 0;
    for stage in stages {
        if let Some(score) = stage.get("passingScore").and_then(|s| s.as_u64()) {
            assert!(
                score >= last_score,
                "Score regression: {} after {}",
                score,
                last_score
            );
            last_score = score;
        }
    }

    eprintln!(
        "Pipeline complete: {} stages, {} findings ({} errors, {} warnings, {} advisories)",
        stages.len(),
        report.total_findings,
        report.error_count,
        report.warning_count,
        report.advisory_count,
    );

    // Verify scaffold serializes cleanly (MCP tools return JSON string)
    let _pretty = serde_json::to_string_pretty(&scaffold).expect("serialize scaffold");
}

// ═══════════════════════════════════════════════════════════════════════════
// forge_compile — JSON→TypeScript pipeline
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compile_tov_01_generates_typescript_files() {
    let input_path = workspace_root().join("content/pathways/tov-01.json");
    if !input_path.exists() {
        // Skip gracefully outside full workspace.
        return;
    }

    let tmp = tempfile::TempDir::new().unwrap();
    let output_dir = tmp.path().to_path_buf();

    let params = academy_forge::CompileParams {
        input_path,
        output_dir: output_dir.clone(),
        overwrite: true,
    };

    let result = academy_forge::compile_pathway(&params).expect("compile should succeed");

    // tov-01.json has 8 stages → 8 stage files + config.ts + index.ts = 10
    assert!(
        result.stages_compiled >= 7,
        "should compile at least 7 stages, got {}",
        result.stages_compiled
    );
    assert!(
        result.files_written.len() >= 9,
        "should write at least 9 files (stages + config + index), got {}",
        result.files_written.len()
    );
    assert!(
        result.warnings.is_empty(),
        "no warnings expected: {:?}",
        result.warnings
    );

    // Verify stage files exist under stages/
    let stages_dir = output_dir.join("stages");
    assert!(stages_dir.exists(), "stages/ directory must exist");
    assert!(
        stages_dir.join("01-system-decomposition.ts").exists(),
        "first stage file must exist"
    );

    // Verify config.ts has pathway metadata
    let config = std::fs::read_to_string(output_dir.join("config.ts")).unwrap();
    assert!(
        config.contains("id: 'tov-01'"),
        "config must have pathway id"
    );
    assert!(
        config.contains("domain: 'vigilance'"),
        "config must have domain"
    );

    // Verify index.ts assembles stages
    let index = std::fs::read_to_string(output_dir.join("index.ts")).unwrap();
    assert!(
        index.contains("import { stage01 }"),
        "index must import stage01"
    );
    assert!(
        index.contains("export const pathway"),
        "index must export pathway"
    );

    // Verify stage file has correct Studio-compatible structure
    let stage01 = std::fs::read_to_string(stages_dir.join("01-system-decomposition.ts")).unwrap();
    assert!(
        stage01.contains("import type { CapabilityStage }"),
        "stage must import CapabilityStage type"
    );
    assert!(
        stage01.contains("export const stage01: CapabilityStage"),
        "stage must export typed const"
    );
    assert!(
        stage01.contains("lessons:"),
        "stage must use 'lessons' (not 'activities')"
    );
    assert!(
        !stage01.contains("activities:"),
        "stage must NOT use 'activities' key"
    );

    eprintln!(
        "Compile integration: {} stages, {} files written",
        result.stages_compiled,
        result.files_written.len()
    );
}
