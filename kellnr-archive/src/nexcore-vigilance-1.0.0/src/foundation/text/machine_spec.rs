//! # SMST (Skill Machine Specification) Extraction
//!
//! Extract machine-readable specifications from SKILL.md files.

use serde::{Deserialize, Serialize};

/// Result of SMST extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmstResult {
    /// Overall SMST score (0-100)
    pub score: f64,
    /// Input specification found
    pub has_input: bool,
    /// Output specification found
    pub has_output: bool,
    /// Logic specification found
    pub has_logic: bool,
    /// Error handling specification found
    pub has_errors: bool,
    /// Examples found
    pub has_examples: bool,
    /// Number of validation rules
    pub rule_count: usize,
    /// Compliance level based on score
    pub compliance_level: String,
}

/// Extract SMST components from SKILL.md content
#[must_use]
pub fn extract_smst(content: &str) -> SmstResult {
    let content_lower = content.to_lowercase();

    let has_input = content_lower.contains("## input") || content_lower.contains("### input");
    let has_output = content_lower.contains("## output") || content_lower.contains("### output");
    let has_logic = content_lower.contains("## logic")
        || content_lower.contains("### logic")
        || content_lower.contains("## algorithm");
    let has_errors = content_lower.contains("## error") || content_lower.contains("### error");
    let has_examples =
        content_lower.contains("## example") || content_lower.contains("### example");

    // Count validation rules (lines starting with - or * in logic section)
    let rule_count = content
        .lines()
        .filter(|l| l.trim().starts_with('-') || l.trim().starts_with('*'))
        .count();

    // Calculate score
    let mut score = 0.0;
    if has_input {
        score += 20.0;
    }
    if has_output {
        score += 20.0;
    }
    if has_logic {
        score += 25.0;
    }
    if has_errors {
        score += 15.0;
    }
    if has_examples {
        score += 10.0;
    }
    if rule_count > 3 {
        score += 10.0;
    }

    let compliance_level = match score as u32 {
        85..=100 => "diamond",
        70..=84 => "platinum",
        55..=69 => "gold",
        40..=54 => "silver",
        _ => "bronze",
    }
    .to_string();

    SmstResult {
        score,
        has_input,
        has_output,
        has_logic,
        has_errors,
        has_examples,
        rule_count,
        compliance_level,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_smst_complete() {
        let content = r#"
# Test Skill

## Input
- param1: string

## Output
- result: boolean

## Logic
- Validate input
- Process data
- Return result
- Handle edge cases

## Error Handling
- Invalid input returns error

## Examples
```
example code
```
"#;
        let result = extract_smst(content);
        assert!(result.has_input);
        assert!(result.has_output);
        assert!(result.has_logic);
        assert!(result.has_errors);
        assert!(result.has_examples);
        assert!(result.score >= 85.0);
        assert_eq!(result.compliance_level, "diamond");
    }

    #[test]
    fn test_extract_smst_minimal() {
        let content = "# Minimal Skill\n\nJust some text.";
        let result = extract_smst(content);
        assert!(!result.has_input);
        assert!(!result.has_output);
        assert_eq!(result.compliance_level, "bronze");
    }
}
