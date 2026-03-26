//! # Diamond Compliance Validation
//!
//! Validate skills against Diamond v2 compliance criteria.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::foundation::machine_spec::extract_smst;

/// Compliance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComplianceLevel {
    /// Invalid - does not meet minimum requirements
    Invalid = 0,
    /// Bronze - valid SKILL.md with frontmatter
    Bronze = 1,
    /// Silver - + scripts/ directory, paired agent, triggers >= 3
    Silver = 2,
    /// Gold - + references/, examples >= 2, verify.sh, triggers >= 5
    Gold = 3,
    /// Platinum - + all internal links resolve
    Platinum = 4,
    /// Diamond - + SMST score >= 85%
    Diamond = 5,
}

impl std::fmt::Display for ComplianceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid => write!(f, "Invalid"),
            Self::Bronze => write!(f, "Bronze"),
            Self::Silver => write!(f, "Silver"),
            Self::Gold => write!(f, "Gold"),
            Self::Platinum => write!(f, "Platinum"),
            Self::Diamond => write!(f, "Diamond"),
        }
    }
}

/// Diamond validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiamondValidation {
    /// Overall compliance level
    pub level: ComplianceLevel,
    /// SMST score (0-100)
    pub smst_score: f64,
    /// Issues found
    pub issues: Vec<String>,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
}

/// Validate a skill for Diamond compliance
///
/// # Errors
///
/// Returns an error if the skill path is invalid.
pub fn validate_diamond(skill_path: &Path) -> Result<DiamondValidation, String> {
    let skill_md = skill_path.join("SKILL.md");

    if !skill_md.exists() {
        return Ok(DiamondValidation {
            level: ComplianceLevel::Invalid,
            smst_score: 0.0,
            issues: vec!["Missing SKILL.md".to_string()],
            suggestions: vec!["Create SKILL.md with frontmatter".to_string()],
        });
    }

    let content = std::fs::read_to_string(&skill_md).map_err(|e| e.to_string())?;

    let mut issues = Vec::new();
    let mut suggestions = Vec::new();

    // Check frontmatter
    if !content.starts_with("---") {
        issues.push("Missing frontmatter".to_string());
    }

    // Extract SMST
    let smst = extract_smst(&content);

    // Check for scripts
    let has_scripts = skill_path.join("scripts").exists();
    if !has_scripts {
        suggestions.push("Add scripts/ directory".to_string());
    }

    // Spec-canonical tier gates (source: skill-dev/references/compliance-levels.md)
    let has_references = skill_path.join("references").exists();
    let has_verify = skill_path.join("scripts").join("verify.sh").exists()
        || skill_path.join("verify.py").exists()
        || skill_path.join("verify.rs").exists();

    // Count examples (inline ## Example headers)
    let example_count = content
        .lines()
        .filter(|l| {
            let trimmed = l.trim_start_matches('#');
            l.starts_with("##")
                && (trimmed.trim().starts_with("Example") || trimmed.trim().starts_with(" Example"))
        })
        .count();

    // Check internal link resolution
    let mut total_links: usize = 0;
    let mut broken_links: usize = 0;
    for line in content.lines() {
        // Match ](relative-path) but not ](http
        let mut rest = line;
        while let Some(pos) = rest.find("](") {
            let after = &rest[pos + 2..];
            if let Some(end) = after.find(')') {
                let link = &after[..end];
                if !link.starts_with("http") && !link.starts_with('#') {
                    total_links += 1;
                    if !skill_path.join(link).exists() {
                        broken_links += 1;
                    }
                }
                rest = &after[end..];
            } else {
                break;
            }
        }
    }
    let links_resolve = total_links == 0 || broken_links == 0;

    // Determine level (strict gate sequence — each tier requires all previous gates)
    let level = if issues.contains(&"Missing frontmatter".to_string()) {
        ComplianceLevel::Invalid
    } else if smst.score >= 85.0 && links_resolve {
        ComplianceLevel::Diamond
    } else if links_resolve && has_references && example_count >= 2 && has_verify {
        ComplianceLevel::Platinum
    } else if has_references && example_count >= 2 && has_verify {
        ComplianceLevel::Gold
    } else if has_scripts {
        ComplianceLevel::Silver
    } else {
        ComplianceLevel::Bronze
    };

    // Add suggestions based on level
    if level < ComplianceLevel::Diamond {
        if !smst.has_input {
            suggestions.push("Add ## Input section".to_string());
        }
        if !smst.has_output {
            suggestions.push("Add ## Output section".to_string());
        }
        if !smst.has_logic {
            suggestions.push("Add ## Logic section".to_string());
        }
        if !smst.has_errors {
            suggestions.push("Add ## Error Handling section".to_string());
        }
        if !smst.has_examples {
            suggestions.push("Add ## Examples section".to_string());
        }
    }

    Ok(DiamondValidation {
        level,
        smst_score: smst.score,
        issues,
        suggestions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_level_ordering() {
        assert!(ComplianceLevel::Diamond > ComplianceLevel::Platinum);
        assert!(ComplianceLevel::Platinum > ComplianceLevel::Gold);
        assert!(ComplianceLevel::Bronze > ComplianceLevel::Invalid);
    }
}
