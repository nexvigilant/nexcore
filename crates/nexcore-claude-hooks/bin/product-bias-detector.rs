//! Product Bias Detector
//!
//! PreToolUse:Write hook that detects categorical dual omission.
//! Warns when a Rust module has ≥5 structs and 0 enums.
//!
//! # T1 Primitive Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|--------------|--------|
//! | Struct count | Quantity | N |
//! | Enum count | Quantity | N |
//! | Threshold | Boundary | ∂ |
//! | Comparison | Comparison | κ |
//! | Dual detection | Sum | Σ |

use nexcore_hook_lib::cytokine::emit_check_failed;
use nexcore_hook_lib::{
    content_or_pass, file_path_or_pass, is_rust_file, pass, read_input, require_edit_tool, warn,
};
use regex::Regex;

const HOOK_NAME: &str = "product-bias-detector";

/// Minimum structs to trigger check
const STRUCT_THRESHOLD: usize = 5;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    require_edit_tool(input.tool_name.clone());

    let path = file_path_or_pass(&input);
    if !is_rust_file(path) {
        pass();
    }

    let content = content_or_pass(&input);

    // Skip test modules
    if is_test_module(content) {
        pass();
    }

    let struct_count = count_structs(content);
    let enum_count = count_enums(content);

    // Check for product bias
    if struct_count >= STRUCT_THRESHOLD && enum_count == 0 {
        let message = format_warning(path, struct_count);
        // Emit cytokine signal before warning (IL-6 = acute response)
        emit_check_failed(
            HOOK_NAME,
            &format!("product bias: {} structs, 0 enums", struct_count),
        );
        warn(&message);
    }

    pass();
}

fn is_test_module(content: &str) -> bool {
    content.contains("#[cfg(test)]") && !content.contains("mod tests")
}

fn count_structs(content: &str) -> usize {
    let re = Regex::new(r"(?m)^(?:pub\s+)?struct\s+\w+").unwrap_or_else(|_| pass());
    re.find_iter(content).count()
}

fn count_enums(content: &str) -> usize {
    let re = Regex::new(r"(?m)^(?:pub\s+)?enum\s+\w+").unwrap_or_else(|_| pass());
    // Exclude derive-only enums (they're usually from macros)
    re.find_iter(content)
        .filter(|m| !is_derive_only_enum(content, m.start()))
        .count()
}

fn is_derive_only_enum(content: &str, enum_pos: usize) -> bool {
    // Check if enum is preceded only by derive macros
    let before = &content[..enum_pos];
    let lines: Vec<&str> = before.lines().rev().take(3).collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("#[derive") {
            continue;
        }
        // If we hit non-derive, non-empty line, it's a real enum
        return false;
    }
    false
}

fn format_warning(path: &str, struct_count: usize) -> String {
    format!(
        r#"⚠️ PRODUCT BIAS DETECTED (DUAL-001)

File: {path}
Structs: {struct_count}
Enums: 0

State (ς) requires its categorical dual Sum (Σ).

Consider: Are any of these structs actually variants?

Common conversions:
  • Status structs → enum State {{ ... }}
  • Error types → enum Error {{ ... }}
  • Handler structs → enum Handler {{ ... }}

T-Set Axiom: Product without Sum = incomplete type system

See: ~/.claude/immunity/antibodies.yaml (DUAL-001)"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_structs() {
        let content = r#"
pub struct Foo {}
struct Bar {}
pub struct Baz { x: i32 }
"#;
        assert_eq!(count_structs(content), 3);
    }

    #[test]
    fn test_count_enums() {
        let content = r#"
pub enum State { A, B }
enum Error { X, Y }
"#;
        assert_eq!(count_enums(content), 2);
    }

    #[test]
    fn test_no_bias_with_enums() {
        let content = r#"
struct A {}
struct B {}
struct C {}
struct D {}
struct E {}
enum State { X, Y }
"#;
        let structs = count_structs(content);
        let enums = count_enums(content);
        assert_eq!(structs, 5);
        assert_eq!(enums, 1);
        // No warning should fire
    }

    #[test]
    fn test_bias_detected() {
        let content = r#"
struct A {}
struct B {}
struct C {}
struct D {}
struct E {}
"#;
        let structs = count_structs(content);
        let enums = count_enums(content);
        assert_eq!(structs, 5);
        assert_eq!(enums, 0);
        // Warning should fire
    }

    #[test]
    fn test_below_threshold() {
        let content = r#"
struct A {}
struct B {}
"#;
        let structs = count_structs(content);
        assert!(structs < STRUCT_THRESHOLD);
    }
}
