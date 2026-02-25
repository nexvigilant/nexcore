//! SKILL.md frontmatter parsing

use nexcore_error::Error;
use serde::Deserialize;

/// Frontmatter parsing errors
#[derive(Debug, Error)]
pub enum FrontmatterError {
    /// No frontmatter found
    #[error("no frontmatter found")]
    NotFound,
    /// Invalid YAML
    #[error("invalid yaml: {0}")]
    InvalidYaml(String),
}

/// Parsed SKILL.md frontmatter (Diamond v2)
#[derive(Debug, Clone, Deserialize)]
pub struct SkillFrontmatter {
    /// Skill name
    pub name: String,
    /// Version
    #[serde(default = "default_version")]
    pub version: String,
    /// Short description
    #[serde(default)]
    pub description: String,
    /// Trigger patterns
    #[serde(default)]
    pub triggers: Vec<String>,
    /// Compliance level
    #[serde(default = "default_compliance")]
    pub compliance: String,
    /// Required MCP tools
    #[serde(default)]
    pub mcp_tools: Vec<String>,
    /// Paired agent
    #[serde(default)]
    pub paired_agent: Option<String>,
    /// Skill dependencies
    #[serde(default)]
    pub requires: Vec<String>,
    /// Whether skill has Rust crate implementation
    #[serde(default)]
    pub has_crate: bool,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_compliance() -> String {
    "Bronze".to_string()
}

/// Parse frontmatter from SKILL.md content
pub fn parse_frontmatter(content: &str) -> Result<SkillFrontmatter, FrontmatterError> {
    let yaml = extract_yaml(content)?;
    serde_yml::from_str(&yaml).map_err(|e| FrontmatterError::InvalidYaml(e.to_string()))
}

fn extract_yaml(content: &str) -> Result<String, FrontmatterError> {
    let lines: Vec<_> = content.lines().collect();

    if lines.first().map(|l| *l != "---").unwrap_or(true) {
        return Err(FrontmatterError::NotFound);
    }

    let end = find_closing_delimiter(&lines)?;
    Ok(lines[1..end].join("\n"))
}

fn find_closing_delimiter(lines: &[&str]) -> Result<usize, FrontmatterError> {
    lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, l)| **l == "---")
        .map(|(i, _)| i)
        .ok_or(FrontmatterError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
version: 1.0.0
description: A test skill
triggers:
  - /test
  - test keyword
compliance: Gold
---

# Test Skill

Content here.
"#;

        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm.name, "test-skill");
        assert_eq!(fm.compliance, "Gold");
        assert_eq!(fm.triggers.len(), 2);
    }
}
