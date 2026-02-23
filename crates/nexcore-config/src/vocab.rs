//! # Vocabulary-Skill Mapping
//!
//! Maps vocabulary shorthands and primitives to skills for multi-trigger invocation.
//!
//! ## Architecture
//!
//! ```text
//! Vocab/Primitive → Hook Detection → Skill Trigger → Subagent Delegation
//!                       ↓
//!               MCP Tool Exposure
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_config::vocab::{VocabSkillMap, SkillMapping};
//!
//! let map = VocabSkillMap::load_default()?;
//! if let Some(mapping) = map.get_vocab_mapping("build-doctrine") {
//!     println!("Primary skill: {}", mapping.primary);
//!     println!("Hooks: {:?}", mapping.hooks);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mapping from a vocabulary term to associated skills and hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMapping {
    /// Primary skill to invoke
    pub primary: String,
    /// Secondary skills (invoked in order if needed)
    #[serde(default)]
    pub secondary: Vec<String>,
    /// Hooks that enforce this vocabulary
    #[serde(default)]
    pub hooks: Vec<String>,
    /// T1/T2 Concepts (σ, μ, ς) associated with this vocabulary
    #[serde(default)]
    pub primitives: Vec<String>,
}

/// Skill chain definition for multi-step orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillChain {
    /// Trigger phrase or command
    pub trigger: String,
    /// Ordered list of skills in the chain
    pub chain: Vec<String>,
    /// Subagent to spawn for execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent: Option<String>,
}

/// Complete vocabulary-to-skill mapping
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VocabSkillMap {
    /// Metadata
    #[serde(default)]
    pub _meta: HashMap<String, String>,

    /// Vocabulary shorthand → skill mapping
    #[serde(default)]
    pub vocab_to_skills: HashMap<String, SkillMapping>,

    /// Primitive → skills mapping
    #[serde(default)]
    pub primitive_to_skills: HashMap<String, Vec<String>>,

    /// Named skill chains
    #[serde(default)]
    pub skill_chains: HashMap<String, SkillChain>,
}

impl VocabSkillMap {
    /// Load from JSON file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> nexcore_error::Result<Self> {
        use nexcore_error::Context;
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read vocab map: {}", path.display()))?;
        let map: Self = serde_json::from_str(&content)
            .context(format!("Failed to parse vocab map: {}", path.display()))?;
        Ok(map)
    }

    /// Load from default path (~/.claude/implicit/vocab_skill_map.json)
    pub fn load_default() -> nexcore_error::Result<Self> {
        use nexcore_error::Context;
        let home = std::env::var("HOME").context("HOME env not set")?;
        let path = format!("{}/.claude/implicit/vocab_skill_map.json", home);

        if std::path::Path::new(&path).exists() {
            Self::from_file(&path)
        } else {
            Ok(Self::default())
        }
    }

    /// Get skill mapping for a vocabulary term
    pub fn get_vocab_mapping(&self, vocab: &str) -> Option<&SkillMapping> {
        self.vocab_to_skills.get(vocab)
    }

    /// Get all skills associated with a primitive
    pub fn get_primitive_skills(&self, primitive: &str) -> Vec<&str> {
        self.primitive_to_skills
            .get(primitive)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get skill chain by name
    pub fn get_chain(&self, name: &str) -> Option<&SkillChain> {
        self.skill_chains.get(name)
    }

    /// Find all skill chains triggered by a phrase
    pub fn find_chains_by_trigger(&self, text: &str) -> Vec<(&str, &SkillChain)> {
        self.skill_chains
            .iter()
            .filter(|(_, chain)| text.contains(&chain.trigger))
            .map(|(name, chain)| (name.as_str(), chain))
            .collect()
    }

    /// Get all skills for a vocabulary term (primary + secondary)
    pub fn get_all_skills(&self, vocab: &str) -> Vec<&str> {
        self.vocab_to_skills
            .get(vocab)
            .map(|m| {
                let mut skills = vec![m.primary.as_str()];
                skills.extend(m.secondary.iter().map(|s| s.as_str()));
                skills
            })
            .unwrap_or_default()
    }

    /// Get hooks that enforce a vocabulary term
    pub fn get_hooks(&self, vocab: &str) -> Vec<&str> {
        self.vocab_to_skills
            .get(vocab)
            .map(|m| m.hooks.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// List all vocabulary terms
    pub fn list_vocab(&self) -> Vec<&str> {
        self.vocab_to_skills.keys().map(|s| s.as_str()).collect()
    }

    /// List all primitives
    pub fn list_primitives(&self) -> Vec<&str> {
        self.primitive_to_skills
            .keys()
            .map(|s| s.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_mapping_serde() {
        let json = r#"{
            "primary": "rust-dev",
            "secondary": ["forge"],
            "hooks": ["python_file_blocker"],
            "primitives": ["state", "mapping"]
        }"#;

        let mapping: SkillMapping = serde_json::from_str(json).unwrap();
        assert_eq!(mapping.primary, "rust-dev");
        assert_eq!(mapping.secondary.len(), 1);
        assert_eq!(mapping.hooks.len(), 1);
        assert_eq!(mapping.primitives.len(), 2);
    }

    #[test]
    fn test_vocab_skill_map_default() {
        let map = VocabSkillMap::default();
        assert!(map.vocab_to_skills.is_empty());
        assert!(map.primitive_to_skills.is_empty());
    }

    #[test]
    fn test_find_chains_by_trigger() {
        let mut map = VocabSkillMap::default();
        map.skill_chains.insert(
            "forge-pipeline".to_string(),
            SkillChain {
                trigger: "START FORGE".to_string(),
                chain: vec!["primitive-extractor".to_string()],
                subagent: Some("forge".to_string()),
            },
        );

        let chains = map.find_chains_by_trigger("START FORGE build a parser");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].0, "forge-pipeline");
    }
}
