//! # Rationale Gate Hook
//!
//! **Event:** `PreToolUse:Edit|Write`
//! **Tier:** review, deploy
//! **Timeout:** 1000ms
//! **Exit Codes:** 0 (allow), 2 (block - requires rationale)
//!
//! ## Purpose
//!
//! Detects significant Rust code patterns that warrant documented reasoning.
//! When complex constructs are being written, this hook emits an informational
//! warning to encourage the developer to add explanatory comments or insights.
//!
//! ## Detected Patterns
//!
//! ### Significant Patterns (architectural changes)
//! - `trait` definitions
//! - `async fn` declarations
//! - Trait objects (`dyn`)
//! - Generic impl blocks (`impl<T>`)
//! - Derive macros
//! - Module definitions (`mod`)
//! - Unsafe blocks
//! - Macro definitions (`macro_rules!`)
//!
//! ### Complexity Indicators (advanced constructs)
//! - Lifetime annotations (`'a`)
//! - Where clauses
//! - Associated types
//! - `PhantomData` usage
//! - `Pin<>` usage
//! - `Future` implementations
//!
//! ## Behavior
//!
//! - **Blocking**: Exits 2 when significant patterns lack rationale
//! - **Rust-only**: Only triggers on `.rs` files
//! - **Enforcement**: Requires doc comments for complex constructs
//!
//! ## Example Output
//!
//! ```text
//! RATIONALE GATE: Significant: New trait definition, Generic impl block | Complexity: Lifetime annotation
//! ```
//!
//! ## Configuration (settings.json)
//!
//! ```json
//! {
//!   "hooks": {
//!     "PreToolUse": [{
//!       "matcher": "Edit|Write",
//!       "command": "~/nexcore/.../pretool_rationale_gate"
//!     }]
//!   }
//! }
//! ```
//!
//! ## See Also
//!
//! - `pretool_primitive_pattern_validator` - T1 primitive enforcement
//! - `posttool_incremental_verifier` - Post-write verification

use nexcore_hooks::{HookOutput, exit_success_auto, read_input};
use regex::Regex;

const SIGNIFICANT_PATTERNS: &[(&str, &str)] = &[
    (r"(?m)^\s*(?:pub\s+)?trait\s+\w+", "New trait definition"),
    (r"async\s+fn\s+\w+", "New async function"),
    (r"dyn\s+\w+", "Trait object usage"),
    (r"impl<[^>]+>\s+\w+", "Generic impl block"),
    (r"#\[derive\([^)]*\)\]", "Derive macro"),
    (
        r"(?m)^\s*(?:pub\s+)?mod\s+\w+\s*\{",
        "New module definition",
    ),
    (r"unsafe\s*\{", "Unsafe block"),
    (r"macro_rules!\s+\w+", "Macro definition"),
];

const COMPLEXITY_INDICATORS: &[(&str, &str)] = &[
    (r"'[a-z]", "Lifetime annotation"),
    (r"where\s+\w+\s*:", "Where clause"),
    (r"type\s+\w+\s*=", "Associated type"),
    (r"PhantomData", "PhantomData usage"),
    (r"Pin<", "Pin usage"),
    (r"impl\s+Future", "Future implementation"),
];

fn detect_patterns(content: &str, patterns: &[(&str, &str)]) -> Vec<String> {
    let mut found = Vec::new();
    for (pattern, desc) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(content) {
                found.push(desc.to_string());
            }
        }
    }
    found
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !file_path.ends_with(".rs") {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let significant = detect_patterns(content, SIGNIFICANT_PATTERNS);
    let complexity = detect_patterns(content, COMPLEXITY_INDICATORS);

    if significant.is_empty() && complexity.is_empty() {
        exit_success_auto();
    }

    let mut notes = Vec::new();
    if !significant.is_empty() {
        notes.push(format!("Significant: {}", significant.join(", ")));
    }
    if !complexity.is_empty() {
        notes.push(format!("Complexity: {}", complexity.join(", ")));
    }

    // BLOCKING - require rationale for complex constructs
    let msg = format!(
        "🛑 RATIONALE REQUIRED: {}\nAdd doc comments (/// or //!) explaining why this construct is needed.",
        notes.join(" | ")
    );
    HookOutput::block(&msg).emit();
    std::process::exit(2);
}
