//! Skill registry for runtime lookup

use super::Skill;
use std::collections::HashMap;

/// Skill registry for runtime lookup
#[derive(Default)]
pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl SkillRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a skill
    pub fn register<S: Skill + 'static>(&mut self, skill: S) {
        self.skills
            .insert(skill.name().to_string(), Box::new(skill));
    }

    /// Get skill by name
    pub fn get(&self, name: &str) -> Option<&dyn Skill> {
        self.skills.get(name).map(|s| s.as_ref())
    }

    /// Find skill matching input
    pub fn find_matching(&self, input: &str) -> Option<&dyn Skill> {
        self.skills
            .values()
            .find(|s| s.matches(input))
            .map(|s| s.as_ref())
    }

    /// List all skill names
    pub fn list(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Number of registered skills
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        ComplianceLevel, SkillContext, SkillMetadata, SkillOutput, SkillResult, Trigger,
    };

    struct TestSkill;

    #[async_trait::async_trait]
    impl Skill for TestSkill {
        fn name(&self) -> &str {
            "test-skill"
        }

        fn metadata(&self) -> SkillMetadata {
            SkillMetadata {
                name: "test-skill".into(),
                version: "1.0.0".into(),
                description: "Test".into(),
                compliance: ComplianceLevel::Bronze,
                mcp_tools: vec![],
                paired_agent: None,
                requires: vec![],
            }
        }

        fn triggers(&self) -> Vec<Trigger> {
            vec![Trigger::command("/test")]
        }

        async fn execute(&self, ctx: SkillContext) -> SkillResult {
            Ok(SkillOutput::text(format!("Executed: {}", ctx.input)))
        }
    }

    #[test]
    fn test_registry() {
        let mut reg = SkillRegistry::new();
        reg.register(TestSkill);

        assert!(reg.get("test-skill").is_some());
        assert!(reg.find_matching("/test foo").is_some());
        assert!(reg.find_matching("/other").is_none());
    }
}
