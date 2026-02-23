//! Skill loader from filesystem

use crate::frontmatter::{SkillFrontmatter, parse_frontmatter};
use nexcore_error::Error;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Loader errors
#[derive(Debug, Error)]
pub enum LoaderError {
    /// IO error
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Frontmatter error
    #[error("frontmatter error: {0}")]
    Frontmatter(#[from] crate::frontmatter::FrontmatterError),
}

/// A loaded skill from SKILL.md
#[derive(Debug, Clone)]
pub struct LoadedSkill {
    /// Parsed frontmatter
    pub frontmatter: SkillFrontmatter,
    /// Full prompt content (after frontmatter)
    pub prompt: String,
    /// Source path
    pub path: PathBuf,
}

/// Skill loader
#[derive(Default)]
pub struct SkillLoader {
    skills: HashMap<String, LoadedSkill>,
}

impl SkillLoader {
    /// Create new loader
    pub fn new() -> Self {
        Self::default()
    }

    /// Load single SKILL.md file
    pub fn load_file(&mut self, path: &Path) -> Result<(), LoaderError> {
        let content = std::fs::read_to_string(path)?;
        let frontmatter = parse_frontmatter(&content)?;
        let prompt = extract_prompt(&content);
        let name = frontmatter.name.clone();

        self.skills.insert(
            name,
            LoadedSkill {
                frontmatter,
                prompt,
                path: path.to_path_buf(),
            },
        );

        Ok(())
    }

    /// Get skill by name
    pub fn get(&self, name: &str) -> Option<&LoadedSkill> {
        self.skills.get(name)
    }

    /// List all loaded skill names
    pub fn list(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Number of loaded skills
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

fn extract_prompt(content: &str) -> String {
    let mut in_fm = false;
    let mut start = 0;

    for (i, line) in content.lines().enumerate() {
        if line == "---" {
            if !in_fm {
                in_fm = true;
            } else {
                start = i + 1;
                break;
            }
        }
    }

    content
        .lines()
        .skip(start)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prompt() {
        let content = "---\nname: test\n---\n\n# Header\n\nContent";
        let prompt = extract_prompt(content);
        assert!(prompt.starts_with("# Header"));
    }
}
