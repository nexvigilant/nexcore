//! # Anti-Pattern Detection
//!
//! Detect and score anti-patterns in skill implementations.

use serde::{Deserialize, Serialize};

/// An anti-pattern found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    /// Pattern name
    pub name: String,
    /// Severity (1-10)
    pub severity: u8,
    /// Description
    pub description: String,
    /// Location in code
    pub location: Option<String>,
    /// Suggested fix
    pub fix: String,
}

/// Result of anti-pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternAnalysis {
    /// Total patterns found
    pub count: usize,
    /// Total severity score
    pub total_severity: u32,
    /// Patterns detected
    pub patterns: Vec<AntiPattern>,
    /// Overall health score (100 - severity)
    pub health_score: f64,
}

/// Detect anti-patterns in code content
#[must_use]
pub fn detect_anti_patterns(content: &str) -> AntiPatternAnalysis {
    let mut patterns = Vec::new();

    // Check for common anti-patterns
    if content.contains("unwrap()") {
        patterns.push(AntiPattern {
            name: "Unhandled Error".to_string(),
            severity: 7,
            description: "Using unwrap() can cause panics".to_string(),
            location: None,
            fix: "Use proper error handling with ? or match".to_string(),
        });
    }

    if content.contains("clone()") && content.matches("clone()").count() > 5 {
        patterns.push(AntiPattern {
            name: "Excessive Cloning".to_string(),
            severity: 4,
            description: "Many clone() calls may indicate ownership issues".to_string(),
            location: None,
            fix: "Consider using references or Rc/Arc".to_string(),
        });
    }

    if content.contains("// TODO") || content.contains("// FIXME") {
        patterns.push(AntiPattern {
            name: "Incomplete Implementation".to_string(),
            severity: 3,
            description: "Code contains TODO/FIXME comments".to_string(),
            location: None,
            fix: "Complete pending implementations".to_string(),
        });
    }

    let total_severity: u32 = patterns.iter().map(|p| u32::from(p.severity)).sum();
    let health_score = (100.0 - f64::from(total_severity)).max(0.0);

    AntiPatternAnalysis {
        count: patterns.len(),
        total_severity,
        patterns,
        health_score,
    }
}
