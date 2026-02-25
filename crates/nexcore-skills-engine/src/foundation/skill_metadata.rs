//! # SKILL.md Metadata Parsing
//!
//! Parse frontmatter from SKILL.md files.

use serde::{Deserialize, Serialize};

use serde_json::Value as JsonValue;

/// Metadata from a SKILL.md frontmatter
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillMetadata {
    /// Skill name
    pub name: String,
    /// Human-readable intent
    pub intent: Option<String>,
    /// Diamond compliance level (bronze, silver, gold, platinum, diamond)
    pub compliance: Option<String>,
    /// SMST score (0-100)
    pub smst_score: Option<f64>,
    /// Skill version
    pub version: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Nested sub-skills (relative paths like "skills/sub-skill-name")
    #[serde(default)]
    pub nested_skills: Vec<String>,
    /// Trigger phrases that activate this skill
    #[serde(default)]
    pub triggers: Vec<String>,
    /// Related skills (see-also references)
    #[serde(default)]
    pub see_also: Vec<String>,
    /// Upstream dependencies (skills that feed into this one)
    #[serde(default)]
    pub upstream: Vec<String>,
    /// Downstream consumers (skills that consume this one's output)
    #[serde(default)]
    pub downstream: Vec<String>,
    /// MCP tools this skill uses
    #[serde(default)]
    pub mcp_tools: Vec<String>,
    /// Position in a skill chain (head, middle, tail)
    pub chain_position: Option<String>,
    /// Pipeline this skill belongs to
    pub pipeline: Option<String>,
    /// Domain this skill operates in
    pub domain: Option<String>,
    /// AI model preference (opus, sonnet, haiku)
    pub model: Option<String>,
}

/// Parse frontmatter from SKILL.md content
///
/// # Errors
///
/// Returns an error if frontmatter is missing or invalid.
pub fn parse_frontmatter(content: &str) -> Result<SkillMetadata, String> {
    // Extract frontmatter
    if !content.starts_with("---") {
        return Err("Missing frontmatter delimiter".to_string());
    }

    let rest = &content[3..];
    let end_idx = rest
        .find("\n---")
        .ok_or("Missing closing frontmatter delimiter")?;
    let frontmatter = &rest[..end_idx];

    // Parse YAML directly (self-contained, no foundation dependency)
    let data: JsonValue =
        serde_yml::from_str(frontmatter).map_err(|e| format!("YAML parse error: {e}"))?;

    // Helper to get string from multiple possible keys
    let get_str = |keys: &[&str]| -> Option<String> {
        for key in keys {
            if let Some(v) = data.get(*key).and_then(|v| v.as_str()) {
                return Some(v.to_string());
            }
        }
        None
    };

    // Helper to get array from multiple possible keys
    let get_array = |keys: &[&str]| -> Vec<String> {
        for key in keys {
            if let Some(arr) = data.get(*key).and_then(|v| v.as_array()) {
                return arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }
        Vec::new()
    };

    Ok(SkillMetadata {
        name: data
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        // Support both "intent" and "description" keys
        intent: get_str(&["intent", "description"]),
        // Support both "compliance" and "compliance-level" keys
        compliance: get_str(&["compliance", "compliance-level"]),
        smst_score: data.get("smst_score").and_then(|v| v.as_f64()),
        version: data
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from),
        author: data
            .get("author")
            .and_then(|v| v.as_str())
            .map(String::from),
        // Support "tags", "keywords", and "categories" keys
        tags: {
            let mut all_tags = get_array(&["tags", "keywords"]);
            all_tags.extend(get_array(&["categories"]));
            all_tags
        },
        // Support "nested-skills" for compound skills
        nested_skills: get_array(&["nested-skills", "nested_skills"]),
        triggers: get_array(&["triggers"]),
        see_also: get_array(&["see-also", "see_also"]),
        upstream: get_array(&["upstream"]),
        downstream: get_array(&["downstream"]),
        mcp_tools: get_array(&["mcp-tools", "mcp_tools"]),
        chain_position: get_str(&["chain-position", "chain_position"]),
        pipeline: get_str(&["pipeline"]),
        domain: get_str(&["domain"]),
        model: get_str(&["model"]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
intent: Do something useful
compliance: gold
smst_score: 85.5
tags:
  - validation
  - testing
---
# Test Skill
"#;
        // INVARIANT: test uses statically defined valid YAML
        let meta = parse_frontmatter(content).unwrap();
        assert_eq!(meta.name, "test-skill");
        assert_eq!(meta.intent, Some("Do something useful".to_string()));
        assert_eq!(meta.compliance, Some("gold".to_string()));
        assert_eq!(meta.smst_score, Some(85.5));
        assert!(meta.tags.contains(&"validation".to_string()));
    }

    #[test]
    fn test_missing_frontmatter() {
        let content = "# No frontmatter";
        assert!(parse_frontmatter(content).is_err());
    }

    #[test]
    fn test_parse_alternative_keys() {
        // Test "description" instead of "intent"
        // Test "compliance-level" instead of "compliance"
        // Test "keywords" and "categories" instead of "tags"
        let content = r#"---
name: levenshtein
description: Calculate edit distance between strings
compliance-level: silver
keywords:
  - algorithm
  - string
categories:
  - foundation
---
# Levenshtein
"#;
        // INVARIANT: test uses statically defined valid YAML
        let meta = parse_frontmatter(content).unwrap();
        assert_eq!(meta.name, "levenshtein");
        assert_eq!(
            meta.intent,
            Some("Calculate edit distance between strings".to_string())
        );
        assert_eq!(meta.compliance, Some("silver".to_string()));
        // Should have keywords + categories merged
        assert!(meta.tags.contains(&"algorithm".to_string()));
        assert!(meta.tags.contains(&"foundation".to_string()));
        assert_eq!(meta.tags.len(), 3); // algorithm, string, foundation
    }

    #[test]
    fn test_parse_nested_skills() {
        let content = r#"---
name: forge
description: Autonomous Rust development orchestrator
compliance: diamond
nested-skills:
  - skills/rust-anatomy
  - skills/primitive-extractor
  - skills/strat-dev
---
# FORGE
"#;
        // INVARIANT: test uses statically defined valid YAML
        let meta = parse_frontmatter(content).unwrap();
        assert_eq!(meta.name, "forge");
        assert_eq!(meta.nested_skills.len(), 3);
        assert!(
            meta.nested_skills
                .contains(&"skills/rust-anatomy".to_string())
        );
        assert!(
            meta.nested_skills
                .contains(&"skills/primitive-extractor".to_string())
        );
        assert!(meta.nested_skills.contains(&"skills/strat-dev".to_string()));
    }
}
