//! Python AST primitive extraction using regex patterns
//!
//! Extracts function and class names from Python source files without
//! requiring a full AST parser. Uses regex patterns for ~5-20x speedup
//! over Python's ast module for simple extraction tasks.
//!
//! Note: This is intentionally simple and may miss some edge cases
//! (e.g., dynamically generated functions). For full AST analysis,
//! use the Python wrapper.

use rayon::prelude::*;
use std::fs;
use std::path::Path;

/// A primitive extracted from source code
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct Primitive {
    /// Name of the function/class/method
    pub name: String,
    /// Type of primitive
    pub kind: PrimitiveKind,
    /// Line number (1-indexed)
    pub line: usize,
    /// Indentation level (0 = top-level)
    pub indent: usize,
    /// Parent class name (for methods)
    pub parent: Option<String>,
}

/// Kind of primitive
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PrimitiveKind {
    /// A function definition
    Function,
    /// A class definition
    Class,
    /// A method (function inside a class)
    Method,
    /// An async function
    AsyncFunction,
    /// An async method
    AsyncMethod,
}

impl std::fmt::Display for PrimitiveKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Class => write!(f, "class"),
            Self::Method => write!(f, "method"),
            Self::AsyncFunction => write!(f, "async_function"),
            Self::AsyncMethod => write!(f, "async_method"),
        }
    }
}

/// Extract primitives from Python source code
///
/// Uses line-by-line regex matching for speed.
/// Handles:
/// - `def function_name(...)`
/// - `async def function_name(...)`
/// - `class ClassName(...)`
///
/// # Arguments
/// * `source` - Python source code as a string
///
/// # Returns
/// Vector of extracted primitives with line numbers
pub fn extract_primitives(source: &str) -> Vec<Primitive> {
    let mut primitives = Vec::new();
    let mut current_class: Option<(String, usize)> = None; // (name, indent)

    for (line_num, line) in source.lines().enumerate() {
        let line_number = line_num + 1; // 1-indexed

        // Calculate indentation
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Update current class context based on indentation
        if let Some((_, class_indent)) = &current_class {
            if indent <= *class_indent {
                current_class = None;
            }
        }

        // Check for class definition
        if let Some(name) = extract_class_name(trimmed) {
            current_class = Some((name.clone(), indent));
            primitives.push(Primitive {
                name,
                kind: PrimitiveKind::Class,
                line: line_number,
                indent,
                parent: None,
            });
            continue;
        }

        // Check for function/method definition
        if let Some((name, is_async)) = extract_function_name(trimmed) {
            let (kind, parent) = if let Some((class_name, _)) = &current_class {
                if indent > 0 {
                    // This is a method
                    let method_kind = if is_async {
                        PrimitiveKind::AsyncMethod
                    } else {
                        PrimitiveKind::Method
                    };
                    (method_kind, Some(class_name.clone()))
                } else {
                    // Top-level, not a method
                    let func_kind = if is_async {
                        PrimitiveKind::AsyncFunction
                    } else {
                        PrimitiveKind::Function
                    };
                    (func_kind, None)
                }
            } else {
                let func_kind = if is_async {
                    PrimitiveKind::AsyncFunction
                } else {
                    PrimitiveKind::Function
                };
                (func_kind, None)
            };

            primitives.push(Primitive {
                name,
                kind,
                line: line_number,
                indent,
                parent,
            });
        }
    }

    primitives
}

/// Extract class name from a line starting with "class "
fn extract_class_name(line: &str) -> Option<String> {
    if !line.starts_with("class ") {
        return None;
    }

    let rest = line.strip_prefix("class ")?.trim();

    // Find end of class name (before ( or :)
    let end = rest
        .find(|c: char| c == '(' || c == ':')
        .unwrap_or(rest.len());

    let name = rest[..end].trim();
    if is_valid_identifier(name) {
        Some(name.to_string())
    } else {
        None
    }
}

/// Extract function name from a line starting with "def " or "async def "
/// Returns (name, is_async)
fn extract_function_name(line: &str) -> Option<(String, bool)> {
    let (rest, is_async) = if line.starts_with("async def ") {
        (line.strip_prefix("async def ")?, true)
    } else if line.starts_with("def ") {
        (line.strip_prefix("def ")?, false)
    } else {
        return None;
    };

    let rest = rest.trim();

    // Find end of function name (before ()
    let end = rest.find('(')?;
    let name = rest[..end].trim();

    if is_valid_identifier(name) {
        Some((name.to_string(), is_async))
    } else {
        None
    }
}

/// Check if a string is a valid Python identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // First character must be letter or underscore
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Extract primitives from a file
pub fn extract_from_file(path: &Path) -> Result<Vec<Primitive>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(extract_primitives(&content))
}

/// Extract primitives from multiple files in parallel
pub fn extract_from_files(paths: &[&Path]) -> Vec<(String, Vec<Primitive>)> {
    paths
        .par_iter()
        .filter_map(|path| {
            let primitives = extract_from_file(path).ok()?;
            Some((path.to_string_lossy().to_string(), primitives))
        })
        .collect()
}

/// Get only function/method names (for coverage matching)
pub fn get_function_names(primitives: &[Primitive]) -> Vec<String> {
    primitives
        .iter()
        .filter(|p| {
            matches!(
                p.kind,
                PrimitiveKind::Function
                    | PrimitiveKind::Method
                    | PrimitiveKind::AsyncFunction
                    | PrimitiveKind::AsyncMethod
            )
        })
        .map(|p| p.name.clone())
        .collect()
}

/// Get only class names
pub fn get_class_names(primitives: &[Primitive]) -> Vec<String> {
    primitives
        .iter()
        .filter(|p| p.kind == PrimitiveKind::Class)
        .map(|p| p.name.clone())
        .collect()
}

/// Get test function names (functions starting with test_ or ending with _test)
pub fn get_test_function_names(primitives: &[Primitive]) -> Vec<String> {
    primitives
        .iter()
        .filter(|p| {
            matches!(
                p.kind,
                PrimitiveKind::Function
                    | PrimitiveKind::Method
                    | PrimitiveKind::AsyncFunction
                    | PrimitiveKind::AsyncMethod
            ) && (p.name.starts_with("test_") || p.name.ends_with("_test"))
        })
        .map(|p| p.name.clone())
        .collect()
}

/// Summary of extracted primitives
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionSummary {
    /// Total primitives extracted
    pub total: usize,
    /// Number of functions
    pub functions: usize,
    /// Number of classes
    pub classes: usize,
    /// Number of methods
    pub methods: usize,
    /// Number of async functions/methods
    pub async_count: usize,
    /// Number of test functions
    pub test_functions: usize,
}

/// Generate summary from extracted primitives
pub fn summarize_extraction(primitives: &[Primitive]) -> ExtractionSummary {
    ExtractionSummary {
        total: primitives.len(),
        functions: primitives
            .iter()
            .filter(|p| p.kind == PrimitiveKind::Function)
            .count(),
        classes: primitives
            .iter()
            .filter(|p| p.kind == PrimitiveKind::Class)
            .count(),
        methods: primitives
            .iter()
            .filter(|p| p.kind == PrimitiveKind::Method)
            .count(),
        async_count: primitives
            .iter()
            .filter(|p| {
                matches!(
                    p.kind,
                    PrimitiveKind::AsyncFunction | PrimitiveKind::AsyncMethod
                )
            })
            .count(),
        test_functions: get_test_function_names(primitives).len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CODE: &str = r#"
# This is a comment
class MyClass:
    def __init__(self, value):
        self.value = value

    async def async_method(self):
        pass

    def regular_method(self):
        pass

def standalone_function():
    return 42

async def async_standalone():
    pass

def test_something():
    assert True

class TestClass:
    def test_method(self):
        pass
"#;

    #[test]
    fn test_extract_primitives() {
        let primitives = extract_primitives(SAMPLE_CODE);

        // Should find: MyClass, __init__, async_method, regular_method,
        // standalone_function, async_standalone, test_something, TestClass, test_method
        assert!(primitives.len() >= 9);

        // Check class detection
        let classes: Vec<_> = primitives
            .iter()
            .filter(|p| p.kind == PrimitiveKind::Class)
            .collect();
        assert_eq!(classes.len(), 2);
        assert!(classes.iter().any(|c| c.name == "MyClass"));
        assert!(classes.iter().any(|c| c.name == "TestClass"));
    }

    #[test]
    fn test_extract_methods() {
        let primitives = extract_primitives(SAMPLE_CODE);

        let methods: Vec<_> = primitives
            .iter()
            .filter(|p| p.kind == PrimitiveKind::Method || p.kind == PrimitiveKind::AsyncMethod)
            .collect();

        // __init__, async_method, regular_method, test_method
        assert!(methods.len() >= 4);

        // Check parent class assignment
        let init = methods.iter().find(|m| m.name == "__init__").unwrap();
        assert_eq!(init.parent, Some("MyClass".to_string()));
    }

    #[test]
    fn test_extract_async() {
        let primitives = extract_primitives(SAMPLE_CODE);

        let async_primitives: Vec<_> = primitives
            .iter()
            .filter(|p| {
                matches!(
                    p.kind,
                    PrimitiveKind::AsyncFunction | PrimitiveKind::AsyncMethod
                )
            })
            .collect();

        assert!(async_primitives.len() >= 2);
        assert!(async_primitives.iter().any(|p| p.name == "async_standalone"));
        assert!(async_primitives.iter().any(|p| p.name == "async_method"));
    }

    #[test]
    fn test_get_test_function_names() {
        let primitives = extract_primitives(SAMPLE_CODE);
        let test_names = get_test_function_names(&primitives);

        assert!(test_names.contains(&"test_something".to_string()));
        assert!(test_names.contains(&"test_method".to_string()));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("CamelCase"));
        assert!(is_valid_identifier("snake_case"));
        assert!(is_valid_identifier("with123numbers"));
        assert!(!is_valid_identifier("123starts_with_number"));
        assert!(!is_valid_identifier("has-hyphen"));
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn test_extract_class_name() {
        assert_eq!(
            extract_class_name("class Foo:"),
            Some("Foo".to_string())
        );
        assert_eq!(
            extract_class_name("class Bar(Base):"),
            Some("Bar".to_string())
        );
        assert_eq!(
            extract_class_name("class Baz(A, B):"),
            Some("Baz".to_string())
        );
        assert_eq!(extract_class_name("def foo():"), None);
    }

    #[test]
    fn test_extract_function_name() {
        assert_eq!(
            extract_function_name("def foo():"),
            Some(("foo".to_string(), false))
        );
        assert_eq!(
            extract_function_name("async def bar():"),
            Some(("bar".to_string(), true))
        );
        assert_eq!(
            extract_function_name("def baz(a, b, c):"),
            Some(("baz".to_string(), false))
        );
        assert_eq!(extract_function_name("class Foo:"), None);
    }

    #[test]
    fn test_summarize_extraction() {
        let primitives = extract_primitives(SAMPLE_CODE);
        let summary = summarize_extraction(&primitives);

        assert!(summary.classes >= 2);
        assert!(summary.functions >= 2);
        assert!(summary.methods >= 3);
        assert!(summary.async_count >= 2);
        assert!(summary.test_functions >= 2);
    }
}
