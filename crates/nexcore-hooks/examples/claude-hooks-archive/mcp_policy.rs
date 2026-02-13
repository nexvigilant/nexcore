//! mcp-policy hook - PreToolUse

use std::io::Read;

fn main() {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    let input: serde_json::Value = serde_json::from_str(&buffer).unwrap();

    
    // Example: Block dangerous commands
    if let Some(command) = input
        .get("tool_input")
        .and_then(|ti| ti.get("command"))
        .and_then(|c| c.as_str())
    {
        if command.contains("rm -rf /") {
            eprintln!("Dangerous command blocked");
            std::process::exit(2);
        }
    }

    // Default: allow the operation
}
