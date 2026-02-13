//! Integration tests for skill execution.

use nexcore_skill_exec::{CompositeExecutor, ExecutionRequest};
use serde_json::json;
use std::path::PathBuf;

/// Test discovering skills from the standard directory.
#[test]
fn test_discover_skill_dev() {
    let executor = CompositeExecutor::new();

    // skill-dev should exist and have scripts
    let skill = executor.discover_skill("skill-dev").unwrap();

    assert_eq!(skill.name, "skill-dev");
    assert!(skill.path.exists());
    assert!(
        skill.has_shell_scripts(),
        "skill-dev should have shell scripts in scripts/"
    );
}

/// Test discovering a skill that doesn't exist.
#[test]
fn test_discover_nonexistent_skill() {
    let executor = CompositeExecutor::new();

    let result = executor.discover_skill("this-skill-does-not-exist-12345");
    assert!(result.is_err());
}

/// Test discovering ctvp-validator (has Rust library).
#[test]
fn test_discover_ctvp_validator() {
    let executor = CompositeExecutor::new();

    let skill = executor.discover_skill("ctvp-validator").unwrap();
    assert_eq!(skill.name, "ctvp-validator");
    assert!(skill.path.exists());

    // It has a ctvp_lib Rust crate in scripts/
    let ctvp_lib = skill.path.join("scripts").join("ctvp_lib");
    assert!(ctvp_lib.exists(), "ctvp-validator should have ctvp_lib");
}

/// Test custom skills directory.
#[test]
fn test_custom_skills_dir() {
    let custom_dir = PathBuf::from("/tmp/nonexistent-skills-dir");
    let executor = CompositeExecutor::with_skills_dir(&custom_dir);

    let result = executor.discover_skill("any-skill");
    assert!(result.is_err());
}

/// Test execution request builder.
#[test]
fn test_execution_request_builder() {
    let request = ExecutionRequest::new("skill-dev", json!({"path": "/some/path"}))
        .with_timeout(std::time::Duration::from_secs(30))
        .with_env("DEBUG", "1")
        .with_working_dir("/tmp");

    assert_eq!(request.skill_name, "skill-dev");
    assert_eq!(request.timeout.as_secs(), 30);
    assert_eq!(request.env.get("DEBUG"), Some(&"1".to_string()));
    assert_eq!(request.working_dir, Some(PathBuf::from("/tmp")));
}

/// Integration test: execute skill-dev verify.sh
///
/// This test actually runs the verify.sh script.
#[tokio::test]
async fn test_execute_skill_dev_verify() {
    let executor = CompositeExecutor::new();

    // Execute verify.sh on skill-dev itself (self-verification)
    let request = ExecutionRequest::new("skill-dev", json!({}));

    let result = executor.execute(&request).await.unwrap();

    // The script should run (may pass or fail based on skill state)
    assert_eq!(result.skill_name, "skill-dev");

    // Should have some output
    assert!(
        !result.stdout.is_empty() || !result.stderr.is_empty(),
        "verify.sh should produce output"
    );

    println!("Exit code: {:?}", result.exit_code);
    println!("Status: {:?}", result.status);
    println!("Stdout:\n{}", result.stdout);
    if !result.stderr.is_empty() {
        println!("Stderr:\n{}", result.stderr);
    }
}
