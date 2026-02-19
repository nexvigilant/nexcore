use claude_hooks::{
    exit_success, read_input, write_output,
    input::{PostToolUseInput, ToolInput},
    output::PostToolUseOutput,
    HookResult,
};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> HookResult<()> {
    let input: PostToolUseInput = read_input()?;

    let tool_input: ToolInput = serde_json::from_value(input.tool_input.clone())
        .unwrap_or(ToolInput::Other(input.tool_input.clone()));

    let path_str = match tool_input.file_path() {
        Some(p) if p.ends_with(".rs") => p,
        _ => exit_success(),
    };

    let path = Path::new(path_str);
    let cargo_toml = match find_cargo_toml(path) {
        Some(c) => c,
        None => exit_success(),
    };

    let cargo_dir = cargo_toml.parent().unwrap();

    // Run clippy
    let (success, output) = run_clippy(cargo_dir);

    if success {
        exit_success();
    }

    // Format and report issues
    let msg = format_issues(&output);
    let output = PostToolUseOutput::block(msg);
    write_output(&output)?;

    Ok(())
}

fn find_cargo_toml(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path);
    while let Some(dir) = current {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Some(cargo_toml);
        }
        current = dir.parent();
    }
    None
}

fn run_clippy(cargo_dir: &Path) -> (bool, String) {
    let output = Command::new("cargo")
        .current_dir(cargo_dir)
        .args([
            "clippy",
            "--message-format=short",
            "--",
            "-D", "warnings",
            "-W", "clippy::pedantic",
            "-A", "clippy::module_name_repetitions",
            "-A", "clippy::must_use_candidate",
            "-A", "clippy::missing_errors_doc",
        ])
        .output();

    match output {
        Ok(o) => {
            let combined = format!(
                "{}\n{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            );
            (o.status.success(), combined)
        }
        Err(e) => (false, format!("Failed to run clippy: {e}")),
    }
}

fn format_issues(output: &str) -> String {
    let mut issues = Vec::new();
    for line in output.lines() {
        if line.contains("warning:") || line.contains("error[") {
            issues.push(line.trim().to_string());
        }
    }

    if issues.is_empty() {
        return "Clippy failed with unknown error. Check logs.".to_string();
    }

    let mut msg = String::from("CLIPPY/RUSTC WARNINGS DETECTED (MUST FIX)\n\n");
    for issue in issues.iter().take(10) {
        msg.push_str(&format!("  {}\n", issue));
    }
    if issues.len() > 10 {
        msg.push_str(&format!("\n  ... and {} more issues\n", issues.len() - 10));
    }
    msg.push_str("\nRule: Zero-warning policy. Fix all lints before proceeding.\n");
    msg
}
