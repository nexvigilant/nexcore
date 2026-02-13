use claude_hooks::{
    exit_success, read_input, write_output,
    input::PostToolUseInput,
    output::PostToolUseOutput,
    HookResult,
};
use regex::Regex;
use std::collections::HashSet;

fn main() -> HookResult<()> {
    let input: PostToolUseInput = read_input()?;

    // Only process Bash tools running cargo
    if input.tool_name != "Bash" {
        exit_success();
    }

    let command = input.tool_input.get("command").and_then(|v| v.as_str()).unwrap_or("");
    if !command.contains("cargo") {
        exit_success();
    }

    // Extract stdout and stderr from tool_response
    let stdout = input.tool_response.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
    let stderr = input.tool_response.get("stderr").and_then(|v| v.as_str()).unwrap_or("");
    let combined = format!("{}\n{}", stdout, stderr);

    // Extract error codes
    let mut codes = HashSet::new();
    let re = Regex::new(r"error\[E(\d{4})\]").unwrap();
    for cap in re.captures_iter(&combined) {
        codes.insert(format!("E{}", &cap[1]));
    }

    if codes.is_empty() {
        exit_success();
    }

    // Categorize and generate advice
    let mut advice = Vec::new();
    advice.push("--- RUST COMPILATION ANALYSIS ---".to_string());

    let hallucination: Vec<_> = codes.iter().filter(|c| is_hallucination(c)).collect();
    if !hallucination.is_empty() {
        advice.push(format!("⚠️ Hallucination Warning: {} (Crate/Module not found)", hallucination.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
        advice.push("→ Did you reference a non-existent crate or forget to add it to Cargo.toml?".to_string());
    }

    let borrow: Vec<_> = codes.iter().filter(|c| is_borrow_error(c)).collect();
    if !borrow.is_empty() {
        advice.push(format!("🧠 Borrow Checker Issue: {}", borrow.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
        advice.push("→ Review ownership and lifetimes. Consider using .clone() or references.".to_string());
    }

    let type_err: Vec<_> = codes.iter().filter(|c| is_type_error(c)).collect();
    if !type_err.is_empty() {
        advice.push(format!("📐 Type Mismatch: {}", type_err.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
        advice.push("→ Check function signatures and expected vs actual types.".to_string());
    }

    advice.push("---------------------------------".to_string());

    // Inject advice as context for the next turn
    let output = PostToolUseOutput::with_context(advice.join("\n"));
    write_output(&output)?;

    Ok(())
}

fn is_hallucination(code: &str) -> bool {
    matches!(code, "E0432" | "E0433" | "E0463")
}

fn is_borrow_error(code: &str) -> bool {
    matches!(code, "E0382" | "E0499" | "E0502" | "E0503" | "E0505" | "E0506" | "E0507" | "E0515" | "E0597" | "E0716")
}

fn is_type_error(code: &str) -> bool {
    matches!(code, "E0277" | "E0308" | "E0412")
}
