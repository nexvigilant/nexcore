//! pretool_task_turn_limiter - Enforces max_turns ≤ 3 on all Task tool calls
//!
//! Fires on PreToolUse:Task. Warns if max_turns is missing or exceeds 3.
//! Protocol: Exit 0 (pass), Exit 1 (warn with guidance)

use serde::Deserialize;
use serde_json::Value;
use std::io::{self, Write};

#[derive(Deserialize)]
struct HookInput {
    tool_name: String,
    tool_input: Value,
}

fn main() {
    let stdin = io::stdin();
    let input: HookInput = match serde_json::from_reader(stdin.lock()) {
        Ok(i) => i,
        Err(_) => std::process::exit(0),
    };

    if input.tool_name != "Task" {
        std::process::exit(0);
    }

    let max_turns = input.tool_input.get("max_turns").and_then(|v| v.as_u64());

    match max_turns {
        Some(n) if n <= 3 => {
            std::process::exit(0);
        }
        Some(n) => {
            let stderr = io::stderr();
            let mut handle = stderr.lock();
            if writeln!(
                handle,
                "⚠️  TURN LIMIT EXCEEDED: max_turns={n} is above hard limit of 3. Set max_turns: 3."
            )
            .is_err()
            {
                eprintln!("Turn limit exceeded: {n}");
            }
            std::process::exit(1);
        }
        None => {
            let stderr = io::stderr();
            let mut handle = stderr.lock();
            if writeln!(
                handle,
                "⚠️  MISSING max_turns on Task call. CLAUDE.md requires max_turns: 3 on all agents."
            )
            .is_err()
            {
                eprintln!("Missing max_turns parameter");
            }
            std::process::exit(1);
        }
    }
}
