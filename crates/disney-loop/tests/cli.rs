//! Integration tests for the disney-loop CLI binary.

use std::process::Command;

fn disney_loop_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_disney-loop"))
}

#[test]
fn dry_run_exits_cleanly() {
    let output = disney_loop_bin()
        .arg("--dry-run")
        .output()
        .expect("failed to execute disney-loop");

    assert!(
        output.status.success(),
        "dry-run should exit 0, got: {}",
        output.status
    );

    // Tracing output goes to stderr — check both stdout and stderr for log message
    let all_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        all_output.contains("Dry run"),
        "Expected dry-run log message in output"
    );
}

#[test]
fn humanize_dry_run_exits_cleanly() {
    let output = disney_loop_bin()
        .args(["--mode", "humanize", "--dry-run"])
        .output()
        .expect("failed to execute disney-loop");

    assert!(
        output.status.success(),
        "humanize dry-run should exit 0, got: {}",
        output.status
    );
}

#[test]
fn version_flag_works() {
    let output = disney_loop_bin()
        .arg("--version")
        .output()
        .expect("failed to execute disney-loop");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("disney-loop"),
        "Expected version output containing binary name"
    );
}

#[test]
fn help_flag_works() {
    let output = disney_loop_bin()
        .arg("--help")
        .output()
        .expect("failed to execute disney-loop");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("disney-loop") || stdout.contains("Disney Loop"),
        "Expected binary name or title in help output, got: {stdout}"
    );
}
