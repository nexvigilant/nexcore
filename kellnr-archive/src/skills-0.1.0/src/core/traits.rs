//! Core skill traits

use super::{OutputContent, SkillContext, SkillMetadata, SkillOutput, SkillResult, Trigger};
use std::collections::HashMap;

/// Core trait for all skills
#[async_trait::async_trait]
pub trait Skill: Send + Sync {
    /// Skill name (kebab-case)
    fn name(&self) -> &str;

    /// Skill metadata
    fn metadata(&self) -> SkillMetadata;

    /// Trigger patterns that activate this skill
    fn triggers(&self) -> Vec<Trigger>;

    /// Execute the skill
    async fn execute(&self, ctx: SkillContext) -> SkillResult;

    /// Validate context before execution (optional)
    fn validate(&self, _ctx: &SkillContext) -> Result<(), super::SkillError> {
        Ok(())
    }

    /// Check if skill matches input
    fn matches(&self, input: &str) -> bool {
        self.triggers().iter().any(|t| t.matches(input))
    }
}

/// Trait for composable skills
pub trait Composable: Skill {
    /// Chain this skill with another
    fn chain<S: Skill + 'static>(self, next: S) -> SkillChain
    where
        Self: Sized + 'static,
    {
        SkillChain::new(Box::new(self), Box::new(next))
    }

    /// Run in parallel with another skill
    fn parallel<S: Skill + 'static>(self, other: S) -> SkillParallel
    where
        Self: Sized + 'static,
    {
        SkillParallel::new(Box::new(self), Box::new(other))
    }
}

/// Chained skill execution (A -> B)
pub struct SkillChain {
    first: Box<dyn Skill>,
    second: Box<dyn Skill>,
}

impl SkillChain {
    /// Create new chain
    pub fn new(first: Box<dyn Skill>, second: Box<dyn Skill>) -> Self {
        Self { first, second }
    }

    fn chain_name(&self) -> String {
        format!("{}->{}", self.first.name(), self.second.name())
    }
}

#[async_trait::async_trait]
impl Skill for SkillChain {
    fn name(&self) -> &str {
        "chain"
    }

    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: self.chain_name(),
            version: "1.0.0".to_string(),
            description: "Chained skill execution".to_string(),
            compliance: super::ComplianceLevel::Bronze,
            mcp_tools: Vec::new(),
            paired_agent: None,
            requires: vec![
                self.first.name().to_string(),
                self.second.name().to_string(),
            ],
        }
    }

    fn triggers(&self) -> Vec<Trigger> {
        self.first.triggers()
    }

    async fn execute(&self, ctx: SkillContext) -> SkillResult {
        let first_result = self.first.execute(ctx.clone()).await?;
        let second_ctx = context_from_output(&first_result, ctx);
        self.second.execute(second_ctx).await
    }
}

fn context_from_output(result: &SkillOutput, fallback: SkillContext) -> SkillContext {
    match &result.content {
        OutputContent::Text(t) => SkillContext::new(t.clone()),
        OutputContent::Markdown(m) => SkillContext::new(m.clone()),
        OutputContent::Json(j) => SkillContext::new(j.to_string()),
        _ => fallback,
    }
}

/// Parallel skill execution (A | B)
pub struct SkillParallel {
    first: Box<dyn Skill>,
    second: Box<dyn Skill>,
}

impl SkillParallel {
    /// Create new parallel
    pub fn new(first: Box<dyn Skill>, second: Box<dyn Skill>) -> Self {
        Self { first, second }
    }

    fn parallel_name(&self) -> String {
        format!("{}|{}", self.first.name(), self.second.name())
    }
}

#[async_trait::async_trait]
impl Skill for SkillParallel {
    fn name(&self) -> &str {
        "parallel"
    }

    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: self.parallel_name(),
            version: "1.0.0".to_string(),
            description: "Parallel skill execution".to_string(),
            compliance: super::ComplianceLevel::Bronze,
            mcp_tools: Vec::new(),
            paired_agent: None,
            requires: vec![
                self.first.name().to_string(),
                self.second.name().to_string(),
            ],
        }
    }

    fn triggers(&self) -> Vec<Trigger> {
        let mut triggers = self.first.triggers();
        triggers.extend(self.second.triggers());
        triggers
    }

    async fn execute(&self, ctx: SkillContext) -> SkillResult {
        let (r1, r2) = tokio::join!(self.first.execute(ctx.clone()), self.second.execute(ctx));

        let outputs = collect_outputs(r1, r2);
        Ok(SkillOutput {
            content: OutputContent::Multi(outputs),
            metadata: HashMap::new(),
            suggestions: Vec::new(),
        })
    }
}

fn collect_outputs(r1: SkillResult, r2: SkillResult) -> Vec<OutputContent> {
    let o1 = r1
        .map(|o| o.content)
        .unwrap_or(OutputContent::Text("Error".into()));
    let o2 = r2
        .map(|o| o.content)
        .unwrap_or(OutputContent::Text("Error".into()));
    vec![o1, o2]
}
