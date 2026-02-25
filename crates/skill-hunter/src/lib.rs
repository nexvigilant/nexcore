//! Skill Hunter Library
//!
//! Tier: T3 (Domain-Specific Logic)

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod grounding;

use console::Emoji;

use serde::Deserialize;

use std::path::Path;

// Game emojis

pub static SKULL: Emoji = Emoji("💀", "X");

pub static TROPHY: Emoji = Emoji("🏆", "*");

pub static BUG: Emoji = Emoji("🐛", "!");

pub static SHIELD: Emoji = Emoji("🛡️", "#");

pub static STAR: Emoji = Emoji("⭐", "*");

pub static SWORD: Emoji = Emoji("⚔️", ">");

/// Skill frontmatter structure
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
    pub triggers: Option<Vec<String>>,
    #[serde(rename = "user-invocable")]
    pub user_invocable: Option<bool>,
    pub compliance: Option<String>,
    pub model: Option<String>,
    pub max_turns: Option<u32>,
}

/// Classification of validation/schema issue severity.
///
/// Tier: T2-P (κ + ∂ — comparison with boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Critical, // Missing required field
    Warning,  // Missing recommended field
    Info,     // Suggestion
}

/// Backward-compatible alias.
#[deprecated(note = "use DiagnosticLevel — F2 equivocation fix")]
pub type Severity = DiagnosticLevel;

/// A detected issue
#[derive(Debug, PartialEq)]
pub struct Issue {
    pub severity: DiagnosticLevel,
    pub message: String,
    pub fix_hint: String,
}

/// Skill scan result
#[derive(Debug)]
pub struct SkillResult {
    pub name: String,
    pub issues: Vec<Issue>,
    pub score: u32,
}

/// Game state
pub struct GameState {
    pub skills_scanned: u32,
    pub total_issues: u32,
    pub critical_count: u32,
    pub warning_count: u32,
    pub perfect_skills: u32,
    pub total_score: u32,
}

pub fn extract_skill_name(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

pub fn parse_frontmatter(content: &str) -> SkillFrontmatter {
    let trimmed = content.trim();

    if !trimmed.starts_with("---") {
        return SkillFrontmatter::default();
    }

    let end = trimmed[3..].find("---").map(|i| i + 3);

    let yaml = match end {
        Some(e) => &trimmed[3..e],

        None => return SkillFrontmatter::default(),
    };

    serde_yml::from_str(yaml).unwrap_or_default()
}

pub fn check_required_fields(fm: &SkillFrontmatter, issues: &mut Vec<Issue>) {
    if fm.name.is_none() {
        issues.push(Issue {
            severity: DiagnosticLevel::Critical,
            message: "Missing required field: name".into(),
            fix_hint: "Add 'name: your-skill-name' to frontmatter".into(),
        });
    }

    if fm.description.is_none() {
        issues.push(Issue {
            severity: DiagnosticLevel::Critical,
            message: "Missing required field: description".into(),
            fix_hint: "Add 'description: Brief description' to frontmatter".into(),
        });
    }

    if fm.triggers.is_none() || fm.triggers.as_ref().is_some_and(|t| t.is_empty()) {
        issues.push(Issue {
            severity: DiagnosticLevel::Critical,
            message: "Missing or empty triggers".into(),
            fix_hint: "Add 'triggers: [/cmd, keyword]' to frontmatter".into(),
        });
    }
}

pub fn check_recommended_fields(fm: &SkillFrontmatter, issues: &mut Vec<Issue>) {
    if fm.version.is_none() {
        issues.push(Issue {
            severity: DiagnosticLevel::Warning,
            message: "Missing version".into(),
            fix_hint: "Add 'version: 1.0.0' for tracking".into(),
        });
    }

    if fm.tags.is_none() || fm.tags.as_ref().is_some_and(|t| t.is_empty()) {
        issues.push(Issue {
            severity: DiagnosticLevel::Warning,
            message: "No tags defined".into(),
            fix_hint: "Add 'tags: [category, type]' for discoverability".into(),
        });
    }

    if fm.compliance.is_none() {
        issues.push(Issue {
            severity: DiagnosticLevel::Warning,
            message: "No compliance level".into(),
            fix_hint: "Add 'compliance: Bronze/Silver/Gold/Platinum'".into(),
        });
    }
}

pub fn extract_body(content: &str) -> &str {
    let trimmed = content.trim();

    if !trimmed.starts_with("---") {
        return trimmed;
    }

    if let Some(start) = trimmed[3..].find("---") {
        &trimmed[start + 6..]
    } else {
        trimmed
    }
}

pub fn calculate_score(issues: &[Issue]) -> u32 {
    let mut score = 100u32;

    for issue in issues {
        match issue.severity {
            DiagnosticLevel::Critical => score = score.saturating_sub(25),

            DiagnosticLevel::Warning => score = score.saturating_sub(10),

            DiagnosticLevel::Info => score = score.saturating_sub(2),
        }
    }

    score
}

pub fn calculate_game_state(results: &[SkillResult]) -> GameState {
    let mut state = GameState {
        skills_scanned: results.len() as u32,
        total_issues: 0,
        critical_count: 0,
        warning_count: 0,
        perfect_skills: 0,
        total_score: 0,
    };

    for result in results {
        state.total_score += result.score;

        if result.issues.is_empty() {
            state.perfect_skills += 1;
        }

        for issue in &result.issues {
            state.total_issues += 1;

            match issue.severity {
                DiagnosticLevel::Critical => state.critical_count += 1,

                DiagnosticLevel::Warning => state.warning_count += 1,

                DiagnosticLevel::Info => {}
            }
        }
    }

    state
}

#[cfg(test)]

mod tests {

    use super::*;

    use std::path::Path;

    #[test]

    fn test_extract_skill_name() {
        let path = Path::new("/skills/my-skill/SKILL.md");

        assert_eq!(extract_skill_name(path), "my-skill");
    }

    #[test]

    fn test_parse_frontmatter_valid() {
        let content = "---\nname: test\ndescription: desc\ntriggers: [a]\n---\nbody";

        let fm = parse_frontmatter(content);

        assert_eq!(fm.name, Some("test".into()));

        assert_eq!(fm.description, Some("desc".into()));
    }

    #[test]

    fn test_parse_frontmatter_invalid() {
        let content = "no frontmatter here";

        let fm = parse_frontmatter(content);

        assert_eq!(fm, SkillFrontmatter::default());
    }

    #[test]

    fn test_check_required_fields_missing() {
        let fm = SkillFrontmatter::default();

        let mut issues = Vec::new();

        check_required_fields(&fm, &mut issues);

        assert!(issues.iter().any(|i| i.message.contains("name")));

        assert!(issues.iter().any(|i| i.message.contains("description")));

        assert!(issues.iter().any(|i| i.message.contains("triggers")));
    }

    #[test]

    fn test_calculate_score_perfect() {
        assert_eq!(calculate_score(&[]), 100);
    }

    #[test]

    fn test_calculate_score_critical() {
        let issues = vec![Issue {
            severity: DiagnosticLevel::Critical,
            message: "m".into(),
            fix_hint: "h".into(),
        }];

        assert_eq!(calculate_score(&issues), 75);
    }

    #[test]

    fn test_calculate_game_state() {
        let results = vec![
            SkillResult {
                name: "s1".into(),
                issues: vec![],
                score: 100,
            },
            SkillResult {
                name: "s2".into(),
                issues: vec![Issue {
                    severity: DiagnosticLevel::Critical,
                    message: "m".into(),
                    fix_hint: "h".into(),
                }],
                score: 75,
            },
        ];

        let state = calculate_game_state(&results);

        assert_eq!(state.skills_scanned, 2);

        assert_eq!(state.perfect_skills, 1);

        assert_eq!(state.critical_count, 1);

        assert_eq!(state.total_score, 175);
    }

    #[test]

    fn test_extract_body() {
        let content = "---\nname: test\n---\nThis is the body";

        assert_eq!(extract_body(content).trim(), "This is the body");
    }
}
