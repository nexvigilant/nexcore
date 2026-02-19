//! User Intent Detection
//!
//! Analyzes user prompts for keywords that indicate they need
//! specialized Rust agents.

use super::DetectionResult;

/// Intent pattern: keywords mapped to agent
pub struct IntentPattern {
    /// Keywords that trigger this intent (any match)
    pub keywords: &'static [&'static str],
    /// Agent to trigger
    pub agent: &'static str,
    /// Description of intent
    pub description: &'static str,
}

/// Intent patterns for user prompt analysis
pub const INTENT_PATTERNS: &[IntentPattern] = &[
    // Migration intents
    IntentPattern {
        keywords: &["migrate from python", "convert python", "python to rust"],
        agent: "rust-migrator",
        description: "Python to Rust migration",
    },
    IntentPattern {
        keywords: &["migrate from c", "c to rust", "c++ to rust", "convert c"],
        agent: "rust-c-migrator",
        description: "C/C++ to Rust migration",
    },
    IntentPattern {
        keywords: &["migrate from go", "go to rust", "convert go"],
        agent: "rust-go-migrator",
        description: "Go to Rust migration",
    },
    IntentPattern {
        keywords: &[
            "migrate from js",
            "javascript to rust",
            "wasm",
            "webassembly",
        ],
        agent: "rust-js-migrator",
        description: "JavaScript/WASM migration",
    },
    // Specialization intents
    IntentPattern {
        keywords: &["async", "tokio", "futures", "spawn", "runtime"],
        agent: "rust-async-expert",
        description: "Async/await programming",
    },
    IntentPattern {
        keywords: &["macro", "proc-macro", "derive macro", "macro_rules"],
        agent: "rust-macro-engineer",
        description: "Macro development",
    },
    IntentPattern {
        keywords: &["unsafe", "raw pointer", "ffi", "c interop", "extern"],
        agent: "rust-unsafe-specialist",
        description: "Unsafe code or FFI",
    },
    // Optimization intents
    IntentPattern {
        keywords: &[
            "benchmark",
            "performance",
            "optimize",
            "profil",
            "flamegraph",
        ],
        agent: "rust-optimize",
        description: "Performance optimization",
    },
    IntentPattern {
        keywords: &["binary size", "small binary", "strip", "lto", "opt-level"],
        agent: "rust-binary-optimizer",
        description: "Binary size optimization",
    },
    // DevOps intents
    IntentPattern {
        keywords: &["docker", "container", "dockerfile", "containerize"],
        agent: "rust-container-deployer",
        description: "Container deployment",
    },
    IntentPattern {
        keywords: &["release", "publish", "ci/cd", "github actions", "pipeline"],
        agent: "rust-release",
        description: "Release and CI/CD",
    },
    // Testing intents
    IntentPattern {
        keywords: &["fuzz", "property test", "mutation test", "proptest"],
        agent: "rust-fuzz-tester",
        description: "Advanced testing",
    },
];

/// Detect user intent from prompt
pub fn detect_intent(prompt: &str) -> Option<DetectionResult> {
    let prompt_lower = prompt.to_lowercase();

    for pattern in INTENT_PATTERNS {
        for keyword in pattern.keywords {
            if prompt_lower.contains(keyword) {
                return Some(DetectionResult::new(
                    pattern.agent,
                    pattern.description,
                    pattern.description,
                    format!(
                        "User requested: {}. Prompt: {}",
                        pattern.description, prompt
                    ),
                ));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_python_migration() {
        // "python to rust" is the keyword
        let result = detect_intent("I want to convert python to rust");
        assert!(result.is_some());
        assert_eq!(result.map(|r| r.agent), Some("rust-migrator"));
    }

    #[test]
    fn test_detect_async() {
        let result = detect_intent("I need help with async tokio code");
        assert!(result.is_some());
        assert_eq!(result.map(|r| r.agent), Some("rust-async-expert"));
    }

    #[test]
    fn test_no_detection() {
        let result = detect_intent("hello how are you");
        assert!(result.is_none());
    }
}
