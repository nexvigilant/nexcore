use claude_hooks::{
    read_input, write_output,
    input::HookInput,
    output::{SetupOutput, SessionStartOutput},
    HookResult,
};
use std::process::Command;

fn main() -> HookResult<()> {
    let input: HookInput = read_input()?;

    let mut issues = Vec::new();
    let mut stats = Vec::new();

    // Check rustc
    match get_version("rustc") {
        Some(v) => stats.push(format!("rustc: {}", v)),
        None => issues.push("rustc not found"),
    }

    // Check cargo
    match get_version("cargo") {
        Some(v) => stats.push(format!("cargo: {}", v)),
        None => issues.push("cargo not found"),
    }

    // Check clippy
    if check_tool("cargo", &["clippy", "--version"]) {
        stats.push("clippy: installed".to_string());
    } else {
        issues.push("clippy not found (run: rustup component add clippy)");
    }

    let mut msg = String::from("🦀 RUST TOOLCHAIN STATUS\n\n");
    for s in &stats {
        msg.push_str(&format!("  ✓ {}\n", s));
    }

    if !issues.is_empty() {
        msg.push_str("\n⚠️ ISSUES DETECTED:\n");
        for i in &issues {
            msg.push_str(&format!("  - {}\n", i));
        }
    }

    match input {
        HookInput::Setup(_) => {
            let output = SetupOutput::with_context(msg);
            write_output(&output)?;
        }
        HookInput::SessionStart(_) => {
            let output = SessionStartOutput::with_context(msg);
            write_output(&output)?;
        }
        _ => {
            // Ignore other events
        }
    }

    Ok(())
}

fn get_version(tool: &str) -> Option<String> {
    Command::new(tool)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn check_tool(tool: &str, args: &[&str]) -> bool {
    Command::new(tool)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
