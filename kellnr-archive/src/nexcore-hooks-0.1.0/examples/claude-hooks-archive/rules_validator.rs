//! Rules-based validator hook.
//!
//! Uses a JSON configuration file for validation rules.

use claude_hooks::{
    exit_success, read_input, write_output,
    input::PreToolUseInput,
    output::PreToolUseOutput,
    rules::{RulesEngine, RuleAction},
    HookResult,
};
use std::env;
use std::path::PathBuf;

fn get_rules_path() -> PathBuf {
    // Check for --rules argument
    let args: Vec<String> = env::args().collect();
    for (i, arg) in args.iter().enumerate() {
        if arg == "--rules" {
            if let Some(path) = args.get(i + 1) {
                return PathBuf::from(path);
            }
        }
    }

    // Default locations
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let candidates = [
        PathBuf::from("rules.json"),
        PathBuf::from(format!("{}/.claude/rules.json", home)),
        PathBuf::from(format!("{}/.config/claude-hooks/rules.json", home)),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // Return first candidate even if it doesn't exist
    candidates[0].clone()
}

fn main() -> HookResult<()> {
    let input: PreToolUseInput = read_input()?;

    let rules_path = get_rules_path();

    // Load rules engine
    let engine = match RulesEngine::load(&rules_path) {
        Ok(e) => e,
        Err(_) => {
            // No rules file, allow everything
            exit_success();
        }
    };

    // Check based on tool type
    let result = match input.tool_name.as_str() {
        "Bash" => {
            let command = input.tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            engine.check_bash(command)
        }
        "Write" | "Edit" => {
            let path = input.tool_input
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            engine.check_file(path)
        }
        "WebFetch" | "WebSearch" => {
            let url = input.tool_input
                .get("url")
                .or_else(|| input.tool_input.get("query"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            engine.check_web(url)
        }
        _ => None,
    };

    // Handle the result
    if let Some(rule_match) = result {
        match rule_match.rule.action {
            RuleAction::Block => {
                let output = PreToolUseOutput::deny(rule_match.rule.get_message());
                write_output(&output)?;
            }
            RuleAction::Warn => {
                // Log warning but allow
                eprintln!("Warning: {}", rule_match.rule.get_message());
            }
            RuleAction::Ask => {
                let output = PreToolUseOutput::ask(rule_match.rule.get_message());
                write_output(&output)?;
            }
            RuleAction::Allow => {
                let output = PreToolUseOutput::allow(rule_match.rule.get_message());
                write_output(&output)?;
            }
        }
    }

    exit_success();
}
