//! Primitive Decomposition Checker - PostToolUse:Edit|Write
//!
//! After Rust code is written, validates it follows T1 primitive decomposition:
//! - Functions should be composable (single responsibility)
//! - Data structures should map to primitives (sequence, mapping, recursion, state)
//! - Code should show evidence of primitive thinking
//!
//! WARNS when code lacks primitive structure.
//! This is a teaching hook - it suggests improvements rather than blocking.

use nexcore_hooks::{exit_ok, exit_warn, get_content, get_file_path, is_rust_file, read_input};

/// T1 Primitive indicators in Rust code
struct PrimitiveAnalysis {
    sequence_score: usize,  // Iterator usage, chaining
    mapping_score: usize,   // From/Into, transformations
    recursion_score: usize, // Enums, recursive types
    state_score: usize,     // State machines, PhantomData
}

impl PrimitiveAnalysis {
    fn total(&self) -> usize {
        self.sequence_score + self.mapping_score + self.recursion_score + self.state_score
    }

    fn dominant_primitive(&self) -> &'static str {
        let max = self
            .sequence_score
            .max(self.mapping_score)
            .max(self.recursion_score)
            .max(self.state_score);

        if max == 0 {
            return "none";
        }

        if self.sequence_score == max {
            "sequence"
        } else if self.mapping_score == max {
            "mapping"
        } else if self.recursion_score == max {
            "recursion"
        } else {
            "state"
        }
    }
}

/// Analyze code for T1 primitive patterns
fn analyze_primitives(content: &str) -> PrimitiveAnalysis {
    let mut analysis = PrimitiveAnalysis {
        sequence_score: 0,
        mapping_score: 0,
        recursion_score: 0,
        state_score: 0,
    };

    // Sequence primitives (ordered operations)
    let sequence_patterns = [
        ".iter()",
        ".into_iter()",
        ".map(",
        ".filter(",
        ".fold(",
        ".collect()",
        ".chain()",
        ".zip(",
        ".enumerate(",
        ".take(",
        ".skip(",
        ".flatten(",
        ".flat_map(",
        "for ",
        "while ",
    ];
    for pattern in sequence_patterns {
        analysis.sequence_score += content.matches(pattern).count();
    }

    // Mapping primitives (transformations)
    let mapping_patterns = [
        "impl From<",
        "impl Into<",
        "impl TryFrom<",
        "impl TryInto<",
        "-> Result<",
        "-> Option<",
        ".into()",
        ".from(",
        "as ",
        "impl AsRef<",
        "impl AsMut<",
        "impl Deref",
    ];
    for pattern in mapping_patterns {
        analysis.mapping_score += content.matches(pattern).count();
    }

    // Recursion primitives (self-reference, sum types)
    let recursion_patterns = [
        "enum ",
        "Box<",
        "Rc<",
        "Arc<",
        "match ",
        "if let ",
        "while let ",
        "Self",
        "recursive",
        "tree",
        "node",
    ];
    for pattern in recursion_patterns {
        analysis.recursion_score += content.matches(pattern).count();
    }

    // State primitives (state encapsulation)
    let state_patterns = [
        "PhantomData<",
        "impl State",
        "State {",
        "state:",
        "mut self",
        "&mut self",
        "Cell<",
        "RefCell<",
        "Mutex<",
        "RwLock<",
        "AtomicBool",
        "AtomicUsize",
        "transition",
        "next_state",
    ];
    for pattern in state_patterns {
        analysis.state_score += content.matches(pattern).count();
    }

    analysis
}

/// Check if code has good function decomposition
fn check_function_decomposition(content: &str) -> (usize, usize) {
    let fn_count = content.matches("fn ").count();

    // Count lines per function (rough heuristic)
    let total_lines = content.lines().count();
    let avg_lines_per_fn = if fn_count > 0 {
        total_lines / fn_count
    } else {
        total_lines
    };

    (fn_count, avg_lines_per_fn)
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

    // Get tool_input
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

    // Skip small edits (less than 10 lines)
    let line_count = content.lines().count();
    if line_count < 10 {
        exit_ok();
    }

    // Skip test files
    if file_path.contains("/tests/") || file_path.contains("_test.rs") {
        exit_ok();
    }

    // Analyze primitive patterns
    let analysis = analyze_primitives(&content);
    let (fn_count, avg_lines) = check_function_decomposition(&content);

    // Build feedback
    let mut feedback = Vec::new();

    // Check for primitive diversity
    if analysis.total() == 0 && line_count > 30 {
        feedback.push(
            "No T1 primitives detected. Consider: iterators (sequence), \
             From/Into (mapping), enums (recursion), or state types.",
        );
    }

    // Check function size
    if avg_lines > 50 && fn_count > 0 {
        feedback
            .push("Functions average >50 lines. Decompose into smaller, composable primitives.");
    }

    // Check for monolithic code (one big function)
    if fn_count == 1 && line_count > 100 {
        feedback.push(
            "Single large function detected. Apply T1 decomposition: \
             extract sequence, mapping, and state operations.",
        );
    }

    // Emit warning if issues found
    if !feedback.is_empty() {
        let dominant = analysis.dominant_primitive();
        let msg = format!(
            "Primitive analysis: dominant={}, scores=[seq:{}, map:{}, rec:{}, state:{}]. {}",
            dominant,
            analysis.sequence_score,
            analysis.mapping_score,
            analysis.recursion_score,
            analysis.state_score,
            feedback.join(" ")
        );
        exit_warn(&msg);
    }

    exit_ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_sequence_heavy() {
        let code = r#"
            fn process(items: Vec<i32>) -> Vec<i32> {
                items.iter()
                    .map(|x| x * 2)
                    .filter(|x| *x > 10)
                    .collect()
            }
        "#;
        let analysis = analyze_primitives(code);
        assert!(analysis.sequence_score > analysis.mapping_score);
        assert_eq!(analysis.dominant_primitive(), "sequence");
    }

    #[test]
    fn test_analyze_mapping_heavy() {
        let code = r#"
            impl From<OldType> for NewType {
                fn from(old: OldType) -> Self {
                    Self { value: old.value.into() }
                }
            }
            impl Into<String> for NewType {
                fn into(self) -> String {
                    self.value.to_string()
                }
            }
        "#;
        let analysis = analyze_primitives(code);
        assert!(analysis.mapping_score > 0);
    }

    #[test]
    fn test_analyze_recursion_heavy() {
        let code = r#"
            enum Tree<T> {
                Leaf(T),
                Node(Box<Tree<T>>, Box<Tree<T>>),
            }

            fn traverse(tree: &Tree<i32>) {
                match tree {
                    Tree::Leaf(v) => println!("{}", v),
                    Tree::Node(l, r) => {
                        traverse(l);
                        traverse(r);
                    }
                }
            }
        "#;
        let analysis = analyze_primitives(code);
        assert!(analysis.recursion_score > 0);
    }
}
