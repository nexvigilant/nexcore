//! API-First Design Enforcer - Ensures public APIs have documentation.

use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip enforcement in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let violations = check_undocumented_pub(content);
    if violations.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("API-FIRST CHECKPOINT\n\n");
    msg.push_str("Public items without documentation:\n");
    for (line, item) in &violations {
        msg.push_str(&format!("  Line {line}: {item}\n"));
    }
    msg.push_str("\nREQUIRED for public APIs:\n");
    msg.push_str("  /// Brief description\n");
    msg.push_str("  /// # Arguments / # Returns / # Errors / # Example\n");
    exit_block(&msg);
}

fn check_undocumented_pub(content: &str) -> Vec<(usize, &'static str)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check for pub items (but not pub(crate) etc.)
        let is_pub = (trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub async fn ")
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("pub trait ")
            || trimmed.starts_with("pub type ")
            || trimmed.starts_with("pub const "))
            && !trimmed.starts_with("pub(");

        if is_pub {
            // Check if preceded by doc comment or attribute
            let has_doc = i > 0
                && lines
                    .get(i.saturating_sub(1))
                    .is_some_and(|l| l.trim().starts_with("///") || l.trim().starts_with("#["));

            if !has_doc {
                let item = if trimmed.contains("fn ") {
                    "pub fn"
                } else if trimmed.contains("struct ") {
                    "pub struct"
                } else if trimmed.contains("enum ") {
                    "pub enum"
                } else if trimmed.contains("trait ") {
                    "pub trait"
                } else {
                    "pub item"
                };
                violations.push((i + 1, item));
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_undocumented_pub_fn() {
        let code = "pub fn foo() {}";
        let violations = check_undocumented_pub(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_allows_documented_pub_fn() {
        let code = "/// Does something\npub fn foo() {}";
        let violations = check_undocumented_pub(code);
        assert!(violations.is_empty());
    }
}
