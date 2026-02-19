//! Rust-specific code patterns for static analysis.
//!
//! Patterns for detecting antipatterns, error handling issues, and code smells.

use once_cell::sync::Lazy;
use regex::Regex;

use super::safe_regex;

// =============================================================================
// TYPE ANTIPATTERNS
// =============================================================================

/// Reference to owned String in function parameters (should be &str)
pub static REF_STRING: Lazy<Regex> = Lazy::new(|| safe_regex(r"\([^)]*:\s*&(?:mut\s+)?String\b"));

/// Reference to Vec in function parameters (should be &[T])
pub static REF_VEC: Lazy<Regex> = Lazy::new(|| safe_regex(r"\([^)]*:\s*&(?:mut\s+)?Vec<"));

/// Reference to Box in function parameters (should be &T)
pub static REF_BOX: Lazy<Regex> = Lazy::new(|| safe_regex(r"\([^)]*:\s*&(?:mut\s+)?Box<"));

/// .clone() calls (potential unnecessary allocation)
pub static CLONE_CALL: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.clone\(\)"));

/// .to_string() on string literals
pub static TO_STRING_LITERAL: Lazy<Regex> = Lazy::new(|| safe_regex(r#""[^"]*"\.to_string\(\)"#));

/// .to_owned() on string literals
pub static TO_OWNED_LITERAL: Lazy<Regex> = Lazy::new(|| safe_regex(r#""[^"]*"\.to_owned\(\)"#));

// =============================================================================
// ERROR HANDLING
// =============================================================================

/// .unwrap() calls (panic risk)
pub static UNWRAP: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.unwrap\(\)"));

/// .expect() calls (panic risk)
pub static EXPECT: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.expect\("));

/// panic!() macro
pub static PANIC_MACRO: Lazy<Regex> = Lazy::new(|| safe_regex(r"panic!\s*\("));

/// todo!() macro
pub static TODO_MACRO: Lazy<Regex> = Lazy::new(|| safe_regex(r"todo!\s*\("));

/// unimplemented!() macro
pub static UNIMPLEMENTED_MACRO: Lazy<Regex> = Lazy::new(|| safe_regex(r"unimplemented!\s*\("));

/// unreachable!() macro (legitimate in some cases)
pub static UNREACHABLE_MACRO: Lazy<Regex> = Lazy::new(|| safe_regex(r"unreachable!\s*\("));

// =============================================================================
// UNSAFE CODE
// =============================================================================

/// unsafe block or function
pub static UNSAFE_BLOCK: Lazy<Regex> =
    Lazy::new(|| safe_regex(r"\bunsafe\s*\{|\bunsafe\s+fn\b|\bunsafe\s+impl\b"));

/// Raw pointer dereference
pub static RAW_DEREF: Lazy<Regex> = Lazy::new(|| safe_regex(r"\*(?:mut|const)\s+\w+"));

// =============================================================================
// PERFORMANCE PATTERNS
// =============================================================================

/// Box::new in loop (potential allocation hotspot)
pub static BOX_IN_LOOP: Lazy<Regex> =
    Lazy::new(|| safe_regex(r"(?:for|while|loop)\s*\{[^}]*Box::new"));

/// Vec allocation in loop
pub static VEC_IN_LOOP: Lazy<Regex> =
    Lazy::new(|| safe_regex(r"(?:for|while|loop)\s*\{[^}]*(?:Vec::new|vec!\[)"));

/// String concatenation with +
pub static STRING_CONCAT: Lazy<Regex> = Lazy::new(|| safe_regex(r#"String\s*\+\s*["&]"#));

// =============================================================================
// CONCURRENCY PATTERNS
// =============================================================================

/// Mutex without proper error handling
pub static MUTEX_UNWRAP: Lazy<Regex> = Lazy::new(|| safe_regex(r"\.lock\(\)\s*\.unwrap\(\)"));

/// RwLock without proper error handling
pub static RWLOCK_UNWRAP: Lazy<Regex> =
    Lazy::new(|| safe_regex(r"\.(?:read|write)\(\)\s*\.unwrap\(\)"));

// =============================================================================
// SAFETY COMMENTS
// =============================================================================

/// SAFETY comment (documents unsafe code justification)
pub static SAFETY_COMMENT: Lazy<Regex> = Lazy::new(|| safe_regex(r"//\s*SAFETY:"));

/// INVARIANT comment (documents assumptions)
pub static INVARIANT_COMMENT: Lazy<Regex> = Lazy::new(|| safe_regex(r"//\s*INVARIANT:"));

/// Check if line or previous line has safety documentation
pub fn has_safety_doc(line: &str, prev_line: Option<&str>) -> bool {
    let check = |s: &str| SAFETY_COMMENT.is_match(s) || INVARIANT_COMMENT.is_match(s);
    check(line) || prev_line.is_some_and(check)
}

// =============================================================================
// ANTIPATTERN DETECTION
// =============================================================================

/// Detected antipattern with fix suggestion
#[derive(Debug, Clone)]
pub struct Antipattern {
    pub pattern_type: AntipatternType,
    pub line: usize,
    pub matched: String,
    pub suggestion: String,
}

/// Types of antipatterns detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntipatternType {
    RefString,
    RefVec,
    RefBox,
    Unwrap,
    Expect,
    Panic,
    Todo,
    UnsafeWithoutSafety,
    MutexUnwrap,
}

impl AntipatternType {
    /// Get the fix suggestion for this antipattern
    pub fn suggestion(&self) -> &'static str {
        match self {
            Self::RefString => "Use &str instead of &String",
            Self::RefVec => "Use &[T] instead of &Vec<T>",
            Self::RefBox => "Use &T instead of &Box<T>",
            Self::Unwrap => "Use ? operator or match/if-let for proper error handling",
            Self::Expect => "Use ? operator with context via .context() from anyhow",
            Self::Panic => "Return Result or Option instead of panicking",
            Self::Todo => "Implement the functionality or add issue tracking",
            Self::UnsafeWithoutSafety => "Add // SAFETY: comment explaining why this is safe",
            Self::MutexUnwrap => {
                "Handle poisoned mutex: .lock().unwrap_or_else(|e| e.into_inner())"
            }
        }
    }

    /// Whether this antipattern can be auto-fixed
    pub fn auto_fixable(&self) -> bool {
        matches!(self, Self::RefString | Self::RefVec | Self::RefBox)
    }
}

/// Check content for type antipatterns
pub fn check_type_antipatterns(content: &str) -> Vec<Antipattern> {
    let mut results = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;

        // Skip test code
        if is_test_context(&lines, idx) {
            continue;
        }

        // Check for &String
        if REF_STRING.is_match(line) {
            results.push(Antipattern {
                pattern_type: AntipatternType::RefString,
                line: line_num,
                matched: line.trim().to_string(),
                suggestion: AntipatternType::RefString.suggestion().to_string(),
            });
        }

        // Check for &Vec<T>
        if REF_VEC.is_match(line) {
            results.push(Antipattern {
                pattern_type: AntipatternType::RefVec,
                line: line_num,
                matched: line.trim().to_string(),
                suggestion: AntipatternType::RefVec.suggestion().to_string(),
            });
        }

        // Check for &Box<T>
        if REF_BOX.is_match(line) {
            results.push(Antipattern {
                pattern_type: AntipatternType::RefBox,
                line: line_num,
                matched: line.trim().to_string(),
                suggestion: AntipatternType::RefBox.suggestion().to_string(),
            });
        }
    }

    results
}

/// Check content for error handling issues
pub fn check_error_handling(content: &str) -> Vec<Antipattern> {
    let mut results = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;

        // Skip test code
        if is_test_context(&lines, idx) {
            continue;
        }

        // Skip if has safety comment
        let prev_line = if idx > 0 { Some(lines[idx - 1]) } else { None };
        if has_safety_doc(line, prev_line) {
            continue;
        }

        // Check for .unwrap()
        if UNWRAP.is_match(line) {
            results.push(Antipattern {
                pattern_type: AntipatternType::Unwrap,
                line: line_num,
                matched: line.trim().to_string(),
                suggestion: AntipatternType::Unwrap.suggestion().to_string(),
            });
        }

        // Check for .expect()
        if EXPECT.is_match(line) {
            results.push(Antipattern {
                pattern_type: AntipatternType::Expect,
                line: line_num,
                matched: line.trim().to_string(),
                suggestion: AntipatternType::Expect.suggestion().to_string(),
            });
        }

        // Check for panic!()
        if PANIC_MACRO.is_match(line) {
            results.push(Antipattern {
                pattern_type: AntipatternType::Panic,
                line: line_num,
                matched: line.trim().to_string(),
                suggestion: AntipatternType::Panic.suggestion().to_string(),
            });
        }

        // Check for todo!()
        if TODO_MACRO.is_match(line) {
            results.push(Antipattern {
                pattern_type: AntipatternType::Todo,
                line: line_num,
                matched: line.trim().to_string(),
                suggestion: AntipatternType::Todo.suggestion().to_string(),
            });
        }
    }

    results
}

/// Check if we're in a test context
fn is_test_context(lines: &[&str], idx: usize) -> bool {
    // Look backwards for #[test] or #[cfg(test)]
    let mut found_fn = false;
    for i in (0..=idx).rev() {
        let line = lines[i].trim();
        if line.starts_with("#[test]") || line.starts_with("#[cfg(test)]") {
            return true;
        }
        // Track if we've passed a function definition
        if line.starts_with("fn ") || line.starts_with("pub fn ") {
            found_fn = true;
        }
        // Stop searching after we've seen the fn AND moved past its attributes
        if found_fn
            && !line.starts_with('#')
            && !line.starts_with("fn ")
            && !line.starts_with("pub fn ")
            && !line.is_empty()
        {
            break;
        }
    }
    false
}

/// Auto-fix type antipatterns in content
pub fn auto_fix_type_antipatterns(content: &str) -> String {
    let mut result = content.to_string();

    // Fix &String -> &str
    result = Regex::new(r":\s*&String\b")
        .map(|re| re.replace_all(&result, ": &str").to_string())
        .unwrap_or(result);

    // Fix &mut String -> &mut str (careful: this is less common)
    // Usually you want &str for reading, String for owning

    // Fix &Vec<T> -> &[T]
    result = Regex::new(r":\s*&Vec<([^>]+)>")
        .map(|re| re.replace_all(&result, ": &[$1]").to_string())
        .unwrap_or(result);

    // Fix &Box<T> -> &T
    result = Regex::new(r":\s*&Box<([^>]+)>")
        .map(|re| re.replace_all(&result, ": &$1").to_string())
        .unwrap_or(result);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_string_detection() {
        assert!(REF_STRING.is_match("fn foo(s: &String)"));
        assert!(REF_STRING.is_match("fn foo(s: &mut String)"));
        assert!(!REF_STRING.is_match("fn foo(s: &str)"));
    }

    #[test]
    fn test_ref_vec_detection() {
        assert!(REF_VEC.is_match("fn foo(v: &Vec<i32>)"));
        assert!(REF_VEC.is_match("fn foo(v: &mut Vec<String>)"));
        assert!(!REF_VEC.is_match("fn foo(v: &[i32])"));
    }

    #[test]
    fn test_unwrap_detection() {
        assert!(UNWRAP.is_match("let x = foo.unwrap();"));
        assert!(!UNWRAP.is_match("let x = foo?;"));
    }

    #[test]
    fn test_unsafe_detection() {
        assert!(UNSAFE_BLOCK.is_match("unsafe { ptr.read() }"));
        assert!(UNSAFE_BLOCK.is_match("unsafe fn dangerous() {}"));
        assert!(UNSAFE_BLOCK.is_match("unsafe impl Send for Foo {}"));
    }

    #[test]
    fn test_safety_comment() {
        assert!(has_safety_doc("// SAFETY: This is valid because...", None));
        assert!(has_safety_doc("code", Some("// SAFETY: documented")));
        assert!(has_safety_doc("// INVARIANT: Always non-null", None));
        assert!(!has_safety_doc("// just a comment", None));
    }

    #[test]
    fn test_auto_fix() {
        let input = "fn foo(s: &String, v: &Vec<i32>, b: &Box<Foo>)";
        let fixed = auto_fix_type_antipatterns(input);
        assert!(fixed.contains("&str"));
        assert!(fixed.contains("&[i32]"));
        assert!(fixed.contains("&Foo"));
    }

    #[test]
    fn test_check_type_antipatterns() {
        let content = r#"
fn process(data: &String) {
    println!("{}", data);
}
        "#;
        let issues = check_type_antipatterns(content);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].pattern_type, AntipatternType::RefString);
    }

    #[test]
    fn test_check_error_handling() {
        let content = r#"
fn main() {
    let x = foo().unwrap();
    let y = bar().expect("failed");
}
        "#;
        let issues = check_error_handling(content);
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_safety_comment_skips_detection() {
        let content = r#"
// SAFETY: This is always Some in this context
let x = foo().unwrap();
        "#;
        let issues = check_error_handling(content);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_test_context_skips() {
        let content = r#"
#[test]
fn test_something() {
    let x = foo().unwrap(); // OK in tests
}
        "#;
        let issues = check_error_handling(content);
        assert!(issues.is_empty());
    }
}
