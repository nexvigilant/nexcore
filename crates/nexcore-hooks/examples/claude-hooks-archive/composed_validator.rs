//! Composed validator example.
//!
//! Demonstrates chaining multiple validators with metrics.

use claude_hooks::{
    exit_success, read_input, write_output,
    input::PreToolUseInput,
    output::PreToolUseOutput,
    rules::{RuleBuilder, RuleSet, RuleAction},
    compose::{HookChain, AggregationStrategy},
    metrics::{HookMetrics, LogLevel, CheckTimer},
    HookResult,
};

/// Build security rules (highest priority).
fn security_rules() -> RuleSet {
    RuleSet::default()
        .add_bash_rule(
            RuleBuilder::new()
                .id("block-rm-rf-root")
                .description("Block rm -rf on root")
                .pattern(r"rm\s+(-[rf]+\s+)*/$")
                .action(RuleAction::Block)
                .priority(100)
                .build()
                .unwrap()
        )
        .add_bash_rule(
            RuleBuilder::new()
                .id("block-curl-bash")
                .description("Block curl piped to bash")
                .pattern(r"curl.*\|.*bash")
                .action(RuleAction::Block)
                .priority(100)
                .build()
                .unwrap()
        )
}

/// Build project-specific rules.
fn project_rules() -> RuleSet {
    RuleSet::default()
        .add_file_rule(
            RuleBuilder::new()
                .id("protect-env")
                .description("Protect .env files")
                .pattern(r"\.env")
                .action(RuleAction::Ask)
                .priority(50)
                .build()
                .unwrap()
        )
}

fn main() -> HookResult<()> {
    let input: PreToolUseInput = read_input()?;

    // Initialize metrics
    let mut metrics = HookMetrics::new("composed_validator")
        .with_session(input.common.session_id.clone());

    metrics.log(LogLevel::Info, format!("Validating tool: {}", input.tool_name));

    // Build the validation chain
    let chain = HookChain::new()
        .with_strategy(AggregationStrategy::FirstBlock)
        .add_rules("security", security_rules())
        .add_rules("project", project_rules());

    // Run validation based on tool type
    let timer = CheckTimer::start();
    let result = match input.tool_name.as_str() {
        "Bash" => {
            let command = input.tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            chain.check_bash(command)
        }
        "Write" | "Edit" => {
            let path = input.tool_input
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            chain.check_file(path)
        }
        _ => chain.check_bash(""), // No rules for this tool
    };
    let duration_us = timer.elapsed_us();

    // Record check result
    metrics.record_check(&input.tool_name, result.is_blocked(), duration_us);

    // Handle result
    if result.is_blocked() {
        let message = result.message.unwrap_or_else(|| "Blocked by policy".to_string());
        metrics.record_tool_decision(&input.tool_name, "block", &message);
        metrics.log(LogLevel::Warn, format!("Blocked: {}", message));

        let output = PreToolUseOutput::deny(&message);
        write_output(&output)?;
    } else if result.should_ask() {
        let message = result.message.unwrap_or_else(|| "Requires confirmation".to_string());
        metrics.record_tool_decision(&input.tool_name, "ask", &message);

        let output = PreToolUseOutput::ask(&message);
        write_output(&output)?;
    } else {
        metrics.record_tool_decision(&input.tool_name, "allow", "No rules matched");
    }

    // Log summary
    let summary = metrics.summary();
    metrics.log(LogLevel::Info, format!("Completed: {}", summary));

    // Save metrics (optional - could be disabled in production)
    if let Ok(home) = std::env::var("HOME") {
        let metrics_path = format!("{}/.claude/hooks/metrics/composed_validator.jsonl", home);
        let _ = metrics.save_to_file(metrics_path);
    }

    exit_success();
}
