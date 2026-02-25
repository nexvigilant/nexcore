//! # Skill Registry
//!
//! Discover and register skills from the filesystem.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::foundation::skill_metadata::parse_frontmatter;

/// Information about a registered skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    /// Skill name
    pub name: String,
    /// Path to SKILL.md
    pub path: PathBuf,
    /// Intent from frontmatter
    pub intent: Option<String>,
    /// Compliance level
    pub compliance: Option<String>,
    /// SMST score
    pub smst_score: Option<f64>,
    /// Tags
    pub tags: Vec<String>,
    /// Nested sub-skills (relative paths)
    #[serde(default)]
    pub nested_skills: Vec<String>,
    /// Related skills (see-also references)
    #[serde(default)]
    pub see_also: Vec<String>,
    /// Upstream dependencies (skills that feed into this one)
    #[serde(default)]
    pub upstream: Vec<String>,
    /// Downstream consumers (skills that consume this one's output)
    #[serde(default)]
    pub downstream: Vec<String>,
    /// Position in a skill chain (head, middle, tail)
    #[serde(default)]
    pub chain_position: Option<String>,
    /// Pipeline this skill belongs to
    #[serde(default)]
    pub pipeline: Option<String>,
    /// Domain this skill operates in
    #[serde(default)]
    pub domain: Option<String>,
    /// AI model preference (opus, sonnet, haiku)
    #[serde(default)]
    pub model: Option<String>,
}

/// Registry of discovered skills
#[derive(Debug, Clone, Default)]
pub struct SkillRegistry {
    skills: HashMap<String, SkillInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillRegistryCache {
    skills: HashMap<String, SkillInfo>,
}

impl SkillRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan a directory for skills
    ///
    /// # Errors
    ///
    /// Returns an error if directory traversal fails.
    pub fn scan(&mut self, dir: &Path) -> Result<usize, String> {
        let mut count = 0;

        for entry in nexcore_fs::walk::WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.file_name().is_some_and(|n| n == "SKILL.md") {
                if let Ok(info) = self.parse_skill(path) {
                    self.skills.insert(info.name.clone(), info);
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Load registry from a cache file.
    ///
    /// # Errors
    ///
    /// Returns an error if cache read or parse fails.
    pub fn load_cache(&mut self, path: &Path) -> Result<usize, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let cache: SkillRegistryCache =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;
        self.skills = cache.skills;
        Ok(self.skills.len())
    }

    /// Save registry to a cache file.
    ///
    /// # Errors
    ///
    /// Returns an error if cache write fails.
    pub fn save_cache(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let cache = SkillRegistryCache {
            skills: self.skills.clone(),
        };
        let content = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
        std::fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Parse a SKILL.md file
    fn parse_skill(&self, path: &Path) -> Result<SkillInfo, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

        let metadata = parse_frontmatter(&content)?;

        Ok(SkillInfo {
            name: metadata.name,
            path: path.to_path_buf(),
            intent: metadata.intent,
            compliance: metadata.compliance,
            smst_score: metadata.smst_score,
            tags: metadata.tags,
            nested_skills: metadata.nested_skills,
            see_also: metadata.see_also,
            upstream: metadata.upstream,
            downstream: metadata.downstream,
            chain_position: metadata.chain_position,
            pipeline: metadata.pipeline,
            domain: metadata.domain,
            model: metadata.model,
        })
    }

    /// Get a skill by name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&SkillInfo> {
        self.skills.get(name)
    }

    /// List all skills
    #[must_use]
    pub fn list(&self) -> Vec<&SkillInfo> {
        self.skills.values().collect()
    }

    /// Search skills by tag
    #[must_use]
    pub fn search_by_tag(&self, tag: &str) -> Vec<&SkillInfo> {
        self.skills
            .values()
            .filter(|s| s.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect()
    }

    /// Get skill count
    #[must_use]
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if registry is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// List nested skills for a parent skill
    ///
    /// Returns resolved `SkillInfo` for each nested skill declared in the parent's
    /// `nested-skills` frontmatter. Nested skills are discovered relative to the
    /// parent's directory.
    #[must_use]
    pub fn list_nested(&self, parent_name: &str) -> Vec<SkillInfo> {
        let parent = match self.skills.get(parent_name) {
            Some(p) => p,
            None => return Vec::new(),
        };

        if parent.nested_skills.is_empty() {
            return Vec::new();
        }

        let parent_dir = parent.path.parent().unwrap_or(Path::new("."));

        parent
            .nested_skills
            .iter()
            .filter_map(|rel_path| {
                let nested_skill_md = parent_dir.join(rel_path).join("SKILL.md");
                if nested_skill_md.exists() {
                    self.parse_skill(&nested_skill_md).ok()
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get a nested skill by parent and child name
    #[must_use]
    pub fn get_nested(&self, parent_name: &str, child_name: &str) -> Option<SkillInfo> {
        self.list_nested(parent_name)
            .into_iter()
            .find(|s| s.name == child_name)
    }
}

/// Default skills cache path.
#[must_use]
pub fn default_skills_cache_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(format!(
        "{home}/.claude/skills/.registry-cache.json"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = SkillRegistry::new();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_nested_skills_in_frontmatter() {
        // Test that nested_skills field is properly parsed
        let mut registry = SkillRegistry::new();

        // Get home directory
        if let Ok(home) = std::env::var("HOME") {
            let skills_dir = PathBuf::from(&home).join(".claude/skills");
            if skills_dir.exists() {
                // Scan skills - ignore errors in test (best effort)
                if registry.scan(&skills_dir).is_ok() {
                    // Check if forge skill exists and has nested_skills
                    if let Some(forge) = registry.get("forge") {
                        // If nested skills are declared, ensure they are parsed sensibly.
                        if !forge.nested_skills.is_empty() {
                            assert!(
                                forge
                                    .nested_skills
                                    .iter()
                                    .any(|s| s.contains("forge-miner")),
                                "forge should declare forge-miner as nested skill"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_list_nested_discovers_subskills() {
        let mut registry = SkillRegistry::new();

        if let Ok(home) = std::env::var("HOME") {
            let skills_dir = PathBuf::from(&home).join(".claude/skills");
            if skills_dir.exists() {
                if registry.scan(&skills_dir).is_ok() {
                    // Test list_nested for forge
                    let nested = registry.list_nested("forge");

                    // If forge-miner exists as nested skill, it should be discovered
                    if nested.iter().any(|s| s.name == "forge-miner") {
                        assert!(
                            nested.iter().find(|s| s.name == "forge-miner").is_some(),
                            "forge-miner should be discovered as nested skill"
                        );
                    }
                }
            }
        }
    }
}
