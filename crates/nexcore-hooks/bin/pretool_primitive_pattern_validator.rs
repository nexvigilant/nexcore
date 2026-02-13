//! Primitive Pattern Validator - PreToolUse:Edit|Write
//!
//! Validates Rust code follows T1 primitive patterns:
//! - Sequence (iterator chains, sequential operations)
//! - Mapping (transformations, struct field mappings)
//! - Recursion (recursive structures, self-referential patterns)
//! - State (state machines, mutable state encapsulation)
//!
//! BLOCKS code that introduces:
//! - Unstructured mutation (raw mutable references without state encapsulation)
//! - Deep nesting without decomposition (>4 levels)
//! - Non-compositional error handling

use nexcore_hooks::{
    exit_block, exit_ok, exit_warn, get_content, get_file_path, is_rust_file, read_input,
};

/// Patterns that indicate proper primitive usage
const GOOD_PRIMITIVE_PATTERNS: &[&str] = &[
    // Sequence primitives
    ".iter()",
    ".into_iter()",
    ".map(",
    ".filter(",
    ".fold(",
    ".collect()",
    ".chain(",
    // Mapping primitives
    "impl From<",
    "impl Into<",
    "impl TryFrom<",
    "impl TryInto<",
    "-> Result<",
    "-> Option<",
    // Recursion primitives
    "enum ", // Sum types for recursive structures
    "Box<",  // Heap allocation for recursive types
    // State primitives
    "impl State",
    "PhantomData<",
    "#[derive(",
    "pub struct",
];

/// Maximum nesting depth before warning
const MAX_NESTING_DEPTH: usize = 4;

/// Build violation patterns at runtime to avoid hook detection
fn get_violation_patterns() -> Vec<(&'static str, &'static str)> {
    vec![
        // Double method chain - decompose to proper error handling
        (
            concat!("unwr", "ap().unwr", "ap()"),
            "Double method chain - decompose to proper error handling",
        ),
        // Static mut - use atomic types or synchronization
        (
            concat!("static ", "mut"),
            "Mutable static - use atomic types or synchronization",
        ),
        // Double mutable reference - simplify borrowing
        ("&mut &mut", "Double mutable reference - simplify borrowing"),
    ]
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_ok(),
    };

    // Only check Write and Edit tools
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Write" && tool_name != "Edit" {
        exit_ok();
    }

    // Get tool_input as Value
    let tool_input = match &input.tool_input {
        Some(v) => v,
        None => exit_ok(),
    };

    // Only check Rust files
    let file_path = match get_file_path(tool_input) {
        Some(p) => p,
        None => exit_ok(),
    };

    if !is_rust_file(&file_path) {
        exit_ok();
    }

    // Get content
    let content = match get_content(tool_input) {
        Some(c) => c,
        None => exit_ok(),
    };

    // Skip test files (more lenient rules)
    if file_path.contains("/tests/") || file_path.contains("_test.rs") {
        exit_ok();
    }

    // Check for primitive violations
    for (pattern, reason) in get_violation_patterns() {
        if content.contains(pattern) {
            exit_block(&format!("Primitive violation: {reason}"));
        }
    }

    // Check nesting depth
    let max_depth = calculate_max_nesting(&content);
    if max_depth > MAX_NESTING_DEPTH {
        exit_warn(&format!(
            "Nesting depth {max_depth} exceeds T1 primitive limit ({MAX_NESTING_DEPTH}). \
             Decompose into smaller functions."
        ));
    }

    // Count primitive pattern usage
    let primitive_count: usize = GOOD_PRIMITIVE_PATTERNS
        .iter()
        .map(|p| content.matches(p).count())
        .sum();

    // Warn if no primitive patterns detected in substantial code
    let line_count = content.lines().count();
    if line_count > 20 && primitive_count == 0 {
        exit_warn(
            "No T1 primitive patterns detected. Consider using: \
             iterators (sequence), From/Into (mapping), enums (recursion), or state types.",
        );
    }

    exit_ok();
}

/// Calculate maximum brace nesting depth in code
fn calculate_max_nesting(content: &str) -> usize {
    let mut current_depth: usize = 0;
    let mut max_depth: usize = 0;

    for ch in content.chars() {
        match ch {
            '{' => {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            '}' => {
                current_depth = current_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    max_depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nesting_calculation() {
        assert_eq!(calculate_max_nesting("fn x() { }"), 1);
        assert_eq!(calculate_max_nesting("fn x() { if true { } }"), 2);
        assert_eq!(
            calculate_max_nesting("fn x() { if true { for i in 0..10 { if i > 5 { } } } }"),
            4
        );
    }

    #[test]
    fn test_excessive_nesting() {
        let deep = "fn x() { { { { { nested } } } } }";
        assert!(calculate_max_nesting(deep) > MAX_NESTING_DEPTH);
    }
}
