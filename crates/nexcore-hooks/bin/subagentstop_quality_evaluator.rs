//! SubagentStop hook: Evaluates subagent output quality
//!
//! Blocks completion if:
//! - Output appears truncated
//! - Expected deliverables are missing
//! - Quality metrics fail
//!
//! Exit codes:
//! - 0 with {"decision":"approve"}: Allow agent to finish
//! - 0 with {"decision":"block"}: Force agent to continue (mandatory completion)

use nexcore_hooks::{HookInput, read_input};
use serde_json::json;

fn main() {
    let input: HookInput = match read_input() {
        Some(i) => i,
        None => {
            output_decision("approve", None);
            std::process::exit(0);
        }
    };

    // Get agent type and check quality
    let agent_type = input.agent_type.as_deref().unwrap_or("unknown");

    // Read the agent's transcript to evaluate quality
    let quality_issues = evaluate_agent_quality(&input);

    if quality_issues.is_empty() {
        // Quality is acceptable
        output_decision("approve", None);
    } else {
        // Quality issues detected - block until fixed (mandatory completion)
        let reason = format!(
            "Agent ({}) output needs improvement:\n{}",
            agent_type,
            quality_issues.join("\n")
        );
        eprintln!("⚠️ {}", reason);

        // Block and force completion
        output_decision("block", Some(&reason));
    }

    std::process::exit(0);
}

fn evaluate_agent_quality(input: &HookInput) -> Vec<String> {
    let mut issues = Vec::new();

    // Check if transcript exists and can be read
    if let Some(transcript_path) = &input.agent_transcript_path {
        match std::fs::read_to_string(transcript_path) {
            Ok(content) => {
                // Check for truncation indicators
                if content.ends_with("...") || content.contains("[truncated]") {
                    issues.push("• Output appears truncated".to_string());
                }

                // Check for incomplete code blocks
                let code_block_opens = content.matches("```").count();
                if code_block_opens % 2 != 0 {
                    issues.push("• Unclosed code block detected".to_string());
                }

                // Check for error indicators without resolution
                if content.contains("error[E")
                    && !content.contains("Fixed")
                    && !content.contains("fixed")
                {
                    issues.push("• Compilation errors may be unresolved".to_string());
                }

                // Check for TODO/FIXME left unaddressed
                let todo_count = content.matches("TODO").count() + content.matches("FIXME").count();
                if todo_count > 3 {
                    issues.push(format!("• {} TODO/FIXME markers in output", todo_count));
                }

                // Check minimum content length for non-trivial agents
                let agent_type = input.agent_type.as_deref().unwrap_or("");
                if (agent_type == "Explore" || agent_type == "Plan") && content.len() < 500 {
                    issues
                        .push("• Output seems too brief for exploration/planning task".to_string());
                }
            }
            Err(_) => {
                // Can't read transcript, skip quality check
            }
        }
    }

    issues
}

fn output_decision(decision: &str, reason: Option<&str>) {
    let output = if let Some(r) = reason {
        json!({
            "decision": decision,
            "reason": r
        })
    } else {
        json!({
            "decision": decision
        })
    };
    println!("{}", output);
}
