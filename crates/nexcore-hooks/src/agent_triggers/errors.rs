//! Compiler Error Detection and Classification
//!
//! Classifies Rust compiler errors to determine which specialized
//! agent should handle them.

use super::DetectionResult;

/// Error categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Borrow checker errors (E0382, E0499, etc.)
    Borrow,
    /// Lifetime errors (E0597, E0621, etc.)
    Lifetime,
    /// Trait errors (E0277, E0599)
    Trait,
    /// Async-specific errors (E0728, E0732)
    Async,
    /// Type errors (E0308, E0412)
    Type,
    /// Module/path errors (E0432, E0433)
    Module,
    /// Generic error
    Other,
}

/// Borrow checker error codes
pub const BORROW_ERRORS: &[&str] = &["E0382", "E0499", "E0502", "E0503", "E0505", "E0507"];

/// Lifetime error codes
pub const LIFETIME_ERRORS: &[&str] = &["E0597", "E0621", "E0623", "E0700"];

/// Trait error codes
pub const TRAIT_ERRORS: &[&str] = &["E0277", "E0599"];

/// Async-specific error codes
pub const ASYNC_ERRORS: &[&str] = &["E0728", "E0732"];

/// Type error codes
pub const TYPE_ERRORS: &[&str] = &["E0308", "E0412"];

/// Module/path error codes
pub const MODULE_ERRORS: &[&str] = &["E0432", "E0433", "E0463"];

/// Classify an error code
pub fn classify_error(code: &str) -> ErrorCategory {
    if BORROW_ERRORS.contains(&code) {
        ErrorCategory::Borrow
    } else if LIFETIME_ERRORS.contains(&code) {
        ErrorCategory::Lifetime
    } else if TRAIT_ERRORS.contains(&code) {
        ErrorCategory::Trait
    } else if ASYNC_ERRORS.contains(&code) {
        ErrorCategory::Async
    } else if TYPE_ERRORS.contains(&code) {
        ErrorCategory::Type
    } else if MODULE_ERRORS.contains(&code) {
        ErrorCategory::Module
    } else {
        ErrorCategory::Other
    }
}

/// Get the appropriate agent for an error category
pub fn agent_for_category(category: ErrorCategory) -> &'static str {
    match category {
        ErrorCategory::Borrow => "rust-borrow-doctor",
        ErrorCategory::Lifetime
        | ErrorCategory::Trait
        | ErrorCategory::Type
        | ErrorCategory::Module
        | ErrorCategory::Other => "rust-compiler-doctor",
        ErrorCategory::Async => "rust-async-expert",
    }
}

/// Detect errors in cargo output and return appropriate agent
pub fn detect_errors(output: &str) -> Option<DetectionResult> {
    use regex::Regex;

    let re = Regex::new(r"error\[E(\d{4})\]").ok()?;
    let mut errors: Vec<(String, ErrorCategory)> = Vec::new();

    for cap in re.captures_iter(output) {
        let code = format!("E{}", &cap[1]);
        let category = classify_error(&code);
        errors.push((code, category));
    }

    if errors.is_empty() {
        if output.contains("error:") || output.contains("error[") {
            return Some(DetectionResult::new(
                "rust-compiler-doctor",
                "Compilation error detected",
                "Generic compilation error",
                "Diagnose and fix the compilation errors",
            ));
        }
        return None;
    }

    // Find first error and generate result
    let (code, category) = &errors[0];
    let agent = agent_for_category(*category);
    let reason = match category {
        ErrorCategory::Borrow => "Borrow checker error",
        ErrorCategory::Lifetime => "Lifetime error",
        ErrorCategory::Async => "Async error",
        _ => "Compilation error",
    };

    Some(DetectionResult::new(
        agent,
        format!("{:?}: {}", category, code),
        reason,
        format!("Diagnose error {} and provide fix", code),
    ))
}
