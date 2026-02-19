//! File Content Pattern Detection
//!
//! Detects Rust code patterns that warrant specialized agent attention.

use regex::Regex;

use super::DetectionResult;

/// A file pattern to detect
pub struct FilePattern {
    /// Regex pattern to match
    pub pattern: &'static str,
    /// Agent to trigger
    pub agent: &'static str,
    /// Description
    pub description: &'static str,
    /// Priority (higher = more important)
    pub priority: u8,
}

/// File patterns for Rust code detection
pub const FILE_PATTERNS: &[FilePattern] = &[
    FilePattern {
        pattern: r"unsafe\s*\{",
        agent: "rust-unsafe-specialist",
        description: "Unsafe block",
        priority: 90,
    },
    FilePattern {
        pattern: r"unsafe\s+fn",
        agent: "rust-unsafe-specialist",
        description: "Unsafe function",
        priority: 90,
    },
    FilePattern {
        pattern: r"async\s+fn",
        agent: "rust-async-expert",
        description: "Async function",
        priority: 80,
    },
    FilePattern {
        pattern: r"\.await\b",
        agent: "rust-async-expert",
        description: "Await expression",
        priority: 70,
    },
    FilePattern {
        pattern: r"macro_rules!",
        agent: "rust-macro-engineer",
        description: "Declarative macro",
        priority: 85,
    },
    FilePattern {
        pattern: r"#\[proc_macro",
        agent: "rust-macro-engineer",
        description: "Proc macro",
        priority: 90,
    },
    FilePattern {
        pattern: r#"extern\s+"C""#,
        agent: "rust-ffi-bridge",
        description: "FFI boundary",
        priority: 85,
    },
    FilePattern {
        pattern: r"#\[no_mangle\]",
        agent: "rust-ffi-bridge",
        description: "No-mangle",
        priority: 80,
    },
    FilePattern {
        pattern: r"#\[test\]",
        agent: "rust-test-architect",
        description: "Test function",
        priority: 60,
    },
];

/// Match patterns in content and return highest priority detection
pub fn detect_patterns(content: &str) -> Option<DetectionResult> {
    let mut best: Option<(&FilePattern, String)> = None;

    for pattern in FILE_PATTERNS {
        if let Ok(re) = Regex::new(pattern.pattern) {
            if let Some(m) = re.find(content) {
                let matched_text = m.as_str().to_string();
                match &best {
                    Some((current, _)) if current.priority >= pattern.priority => {}
                    _ => best = Some((pattern, matched_text)),
                }
            }
        }
    }

    best.map(|(p, matched)| {
        DetectionResult::new(
            p.agent,
            p.description,
            p.description,
            format!("Review {} code: `{}`", p.description, matched),
        )
    })
}

/// Check if content is a large refactor
pub fn is_large_refactor(content: &str) -> bool {
    content.lines().count() > 100
}
