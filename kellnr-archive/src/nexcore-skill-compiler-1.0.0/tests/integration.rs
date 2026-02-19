//! Integration tests for the skill compiler pipeline.

use nexcore_skill_compiler::spec::CompoundSpec;

#[test]
fn parse_and_validate_minimal_spec() {
    let toml = r#"
[compound]
name = "test-pipeline"
description = "Integration test"
strategy = "sequential"

[[skills]]
name = "skill-a"

[[skills]]
name = "skill-b"
"#;
    let spec = CompoundSpec::parse(toml).expect("should parse");
    assert_eq!(spec.compound.name, "test-pipeline");
    assert_eq!(spec.skills.len(), 2);
}

#[test]
fn codegen_produces_files() {
    let toml = r#"
[compound]
name = "codegen-test"
description = "Codegen integration test"
strategy = "sequential"
tags = ["test"]

[[skills]]
name = "alpha"
required = true
timeout_seconds = 30

[[skills]]
name = "beta"
required = false
timeout_seconds = 15
"#;
    let spec = CompoundSpec::parse(toml).expect("should parse");
    let dir = tempfile::tempdir().expect("tempdir");
    let generated = nexcore_skill_compiler::codegen::generate(&spec, dir.path()).expect("codegen");

    assert!(generated.main_rs.exists());
    assert!(generated.cargo_toml.exists());
    assert!(generated.skill_md.exists());

    let main_src = std::fs::read_to_string(&generated.main_rs).expect("read");
    assert!(main_src.contains("alpha"));
    assert!(main_src.contains("beta"));

    let skill_md = std::fs::read_to_string(&generated.skill_md).expect("read");
    assert!(skill_md.contains("nested-skills"));
}

#[test]
fn codegen_parallel_strategy() {
    let toml = r#"
[compound]
name = "parallel-test"
strategy = "parallel"

[[skills]]
name = "fast"

[[skills]]
name = "slow"
"#;
    let spec = CompoundSpec::parse(toml).expect("parse");
    let dir = tempfile::tempdir().expect("tempdir");
    let generated = nexcore_skill_compiler::codegen::generate(&spec, dir.path()).expect("codegen");
    let src = std::fs::read_to_string(&generated.main_rs).expect("read");
    assert!(src.contains("thread::spawn"));
}

#[test]
fn codegen_feedback_loop_strategy() {
    let toml = r#"
[compound]
name = "feedback-test"
strategy = "feedback_loop"

[[skills]]
name = "refiner"

[[skills]]
name = "evaluator"

[feedback]
max_iterations = 3
convergence_field = "quality"
convergence_threshold = 0.9
"#;
    let spec = CompoundSpec::parse(toml).expect("parse");
    let dir = tempfile::tempdir().expect("tempdir");
    let generated = nexcore_skill_compiler::codegen::generate(&spec, dir.path()).expect("codegen");
    let src = std::fs::read_to_string(&generated.main_rs).expect("read");
    assert!(src.contains("0..3u32"));
    assert!(src.contains("quality"));
}

#[test]
fn spec_from_params_roundtrips() {
    let skills = vec!["a".into(), "b".into(), "c".into()];
    let toml_text = nexcore_skill_compiler::spec_from_params(&skills, "parallel", "my-compound")
        .expect("generate");
    let spec = CompoundSpec::parse(&toml_text).expect("roundtrip");
    assert_eq!(spec.compound.name, "my-compound");
    assert_eq!(spec.skills.len(), 3);
}

#[test]
fn check_reports_missing_skills() {
    let skills = vec!["nonexistent-xyz".into(), "also-missing".into()];
    let report = nexcore_skill_compiler::check(&skills, "sequential").expect("check");
    assert!(!report.can_compile);
    assert!(!report.blockers.is_empty());
}

#[test]
fn check_rejects_single_skill() {
    let result = nexcore_skill_compiler::check(&["only-one".into()], "sequential");
    assert!(result.is_err());
}
