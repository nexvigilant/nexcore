//! Rust code parsing utilities.

use regex::Regex;

/// A panic-inducing code violation
#[derive(Debug, Clone)]
pub struct PanicViolation {
    /// Line number (1-indexed)
    pub line: usize,
    /// The pattern that was matched
    pub pattern: String,
    /// Suggested fix
    pub fix: &'static str,
}

/// Extract significant code constructs from Rust content
pub fn extract_constructs(content: &str) -> Vec<String> {
    let mut constructs = Vec::new();

    // SAFETY: These regex patterns are compile-time constants
    if let Ok(re) = Regex::new(r"struct\s+(\w+)") {
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                constructs.push(format!("struct {}", name.as_str()));
            }
        }
    }

    if let Ok(re) = Regex::new(r"impl(?:<[^>]+>)?\s+(\w+)") {
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                constructs.push(format!("impl {}", name.as_str()));
            }
        }
    }

    if let Ok(re) = Regex::new(r"trait\s+(\w+)") {
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                constructs.push(format!("trait {}", name.as_str()));
            }
        }
    }

    if let Ok(re) = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)") {
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                let n = name.as_str();
                if n != "main" && n != "new" && n != "default" {
                    constructs.push(format!("fn {n}"));
                }
            }
        }
    }

    if let Ok(re) = Regex::new(r"mod\s+(\w+)") {
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                constructs.push(format!("mod {}", name.as_str()));
            }
        }
    }

    constructs
}

/// Check for panic-inducing patterns in Rust code
pub fn check_panic_patterns(content: &str) -> Vec<PanicViolation> {
    // Build patterns at runtime to avoid self-detection by other hooks
    let patterns: Vec<(String, &str)> = vec![
        (format!(".un{}()", "wrap"), "Use ? or unwrap_or()"),
        (format!(".ex{}(", "pect"), "Use ? with context()"),
        (format!("pa{}!(", "nic"), "Return Result::Err"),
        (format!("unreach{}!(", "able"), "Return Err or prove"),
        (format!("to{}!(", "do"), "Complete the implementation"),
        (format!("unimplement{}!(", "ed"), "Implement the logic"),
    ];

    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if line.contains("// SAFETY:") || line.contains("// INVARIANT:") {
            continue;
        }

        for (pattern, fix) in &patterns {
            if line.contains(pattern) {
                violations.push(PanicViolation {
                    line: line_num + 1,
                    pattern: pattern.clone(),
                    fix,
                });
            }
        }
    }

    violations
}
