//! Error Path Test Verifier
//!
//! Checks that error variants have corresponding tests.
//! This is advisory only (exit 0).

use nexcore_hooks::{HookOutput, exit_success_auto, is_rust_file, read_input};
use std::fs;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => exit_success_auto(),
    };

    // Find error enum variants
    let error_variants = find_error_variants(&content);
    if error_variants.is_empty() {
        exit_success_auto();
    }

    // Check if tests exist for each variant
    let untested: Vec<_> = error_variants
        .iter()
        .filter(|v| !content.contains(&format!("Err({v})")))
        .collect();

    if untested.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("ERROR PATHS UNTESTED\n");
    for variant in &untested {
        msg.push_str(&format!("  {variant} - add test\n"));
    }
    // Advisory output
    HookOutput::warn(&msg).emit();
    std::process::exit(0);
}

fn find_error_variants(content: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let mut in_error_enum = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("enum") && trimmed.contains("Error") {
            in_error_enum = true;
            continue;
        }
        if in_error_enum {
            if trimmed.starts_with('}') {
                in_error_enum = false;
            } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
                let variant = trimmed.split(['(', '{', ',']).next().unwrap_or("");
                if !variant.is_empty() && variant.chars().next().is_some_and(|c| c.is_uppercase()) {
                    variants.push(variant.to_string());
                }
            }
        }
    }
    variants
}
