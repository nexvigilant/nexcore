//! Crate Validator - PreToolUse (Edit|Write)
//! Validates crates exist, prevents hallucinations, caches verifications.

use nexcore_hooks::state::SessionState;
use nexcore_hooks::{exit_block, exit_success_auto, exit_warn, read_input};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

const WORKSPACE_PREFIXES: &[&str] = &["nexcore", "guardian"];
const HALLUCINATION_PATTERNS: &[&str] = &["_utils$", "_helpers$", "^easy_", "^simple_"];
const BUILTIN: &[&str] = &["crate", "self", "super", "std", "core", "alloc"];

fn get_workspace_members(cwd: &str) -> HashSet<String> {
    let mut members = HashSet::new();
    let cargo_path = Path::new(cwd).join("Cargo.toml");
    if let Ok(content) = fs::read_to_string(&cargo_path) {
        if let Ok(re) = Regex::new(r#"members\s*=\s*\[([\s\S]*?)\]"#) {
            if let Some(cap) = re.captures(&content) {
                for m in cap[1].split(',') {
                    let name = m.trim().trim_matches('"').trim_matches('\'');
                    if !name.is_empty() {
                        members.insert(name.replace('-', "_"));
                    }
                }
            }
        }
    }
    members
}

fn get_cargo_dependencies(cwd: &str) -> HashSet<String> {
    let mut deps = HashSet::new();
    let cargo_path = Path::new(cwd).join("Cargo.toml");
    if let Ok(content) = fs::read_to_string(&cargo_path) {
        // Use (?m) for multiline mode
        if let Ok(re) = Regex::new(r"(?m)^\s*(\w[\w-]*)\s*=") {
            for cap in re.captures_iter(&content) {
                deps.insert(cap[1].replace('-', "_"));
            }
        }
    }
    // Add common dependencies
    for dep in [
        "serde",
        "serde_json",
        "tokio",
        "anyhow",
        "thiserror",
        "regex",
        "chrono",
    ] {
        deps.insert(dep.to_string());
    }
    deps
}

fn is_workspace_crate(name: &str) -> bool {
    WORKSPACE_PREFIXES.iter().any(|p| name.starts_with(p))
}

fn matches_hallucination_pattern(name: &str) -> bool {
    for pat in HALLUCINATION_PATTERNS {
        if let Ok(re) = Regex::new(pat) {
            if re.is_match(name) {
                return true;
            }
        }
    }
    false
}

fn verify_crate_exists(name: &str) -> bool {
    Command::new("cargo")
        .args(["search", "--limit", "1", name])
        .output()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains(name))
        .unwrap_or(false)
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip validation in plan mode - avoid API calls during planning
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !file_path.ends_with(".rs") && !file_path.ends_with("Cargo.toml") {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let cargo_deps = get_cargo_dependencies(&input.cwd);
    let workspace = get_workspace_members(&input.cwd);
    let mut state = SessionState::load();

    let Ok(re) = Regex::new(r"(?:extern\s+crate|use)\s+(\w+)") else {
        exit_success_auto()
    };

    let mut unverified = Vec::new();
    let mut hallucinations = Vec::new();

    for cap in re.captures_iter(content) {
        let name = cap[1].to_string();
        if BUILTIN.contains(&name.as_str()) {
            continue;
        }
        if cargo_deps.contains(&name) || workspace.contains(&name) {
            continue;
        }
        if is_workspace_crate(&name) {
            continue;
        }
        if state.is_crate_verified(&name) {
            continue;
        }

        if matches_hallucination_pattern(&name) {
            hallucinations.push(name.clone());
        } else if verify_crate_exists(&name) {
            state.add_verified_crate(&name);
        } else {
            unverified.push(name);
        }
    }

    let _ = state.save();

    if !hallucinations.is_empty() {
        exit_block(&format!(
            "HALLUCINATION DETECTED: {} - suspicious crate name pattern",
            hallucinations.join(", ")
        ));
    }

    if !unverified.is_empty() {
        exit_warn(&format!(
            "Unverified crates (not on crates.io): {}",
            unverified.join(", ")
        ));
    }

    exit_success_auto();
}
