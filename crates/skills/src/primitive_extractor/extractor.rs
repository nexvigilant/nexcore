//! Primitive extraction logic

use super::types::{Primitive, PrimitiveTier};
use crate::core::{
    ComplianceLevel, Skill, SkillContext, SkillMetadata, SkillOutput, SkillResult, Trigger,
};

/// Primitive extractor skill implementation
pub struct PrimitiveExtractor;

impl PrimitiveExtractor {
    /// Create new extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract primitives from text
    pub fn extract(&self, text: &str) -> Vec<Primitive> {
        identify_terms(text)
            .into_iter()
            .map(|t| classify_term(&t))
            .collect()
    }
}

impl Default for PrimitiveExtractor {
    fn default() -> Self {
        Self::new()
    }
}

fn identify_terms(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| w.len() > 3)
        .filter(|w| w.chars().next().is_some_and(|c| c.is_alphabetic()))
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect()
}

fn classify_term(term: &str) -> Primitive {
    let (tier, grounding, confidence) = determine_tier(term);
    Primitive {
        term: term.to_string(),
        definition: format!("A concept representing '{}'", term.to_lowercase()),
        tier,
        grounding,
        transfer_confidence: confidence,
    }
}

fn determine_tier(term: &str) -> (PrimitiveTier, Option<String>, f64) {
    let lower = term.to_lowercase();

    if is_t1_term(&lower) {
        return (PrimitiveTier::T1, None, 0.95);
    }
    if is_t2p_term(&lower) {
        return (PrimitiveTier::T2P, Some("Quantity (N)".into()), 0.78);
    }
    if is_t2c_term(&lower) {
        return (PrimitiveTier::T2C, Some("State + Mapping".into()), 0.65);
    }
    (PrimitiveTier::T3, Some("Domain-specific".into()), 0.42)
}

fn is_t1_term(term: &str) -> bool {
    ["sequence", "mapping", "state", "recursion", "void", "exist"]
        .iter()
        .any(|t| term.contains(t))
}

fn is_t2p_term(term: &str) -> bool {
    ["score", "threshold", "rate", "count", "index"]
        .iter()
        .any(|t| term.contains(t))
}

fn is_t2c_term(term: &str) -> bool {
    ["signal", "assessment", "result", "response"]
        .iter()
        .any(|t| term.contains(t))
}

#[async_trait::async_trait]
impl Skill for PrimitiveExtractor {
    fn name(&self) -> &str {
        "primitive-extractor"
    }

    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: "primitive-extractor".to_string(),
            version: "1.0.0".to_string(),
            description: "Extract irreducible conceptual primitives".to_string(),
            compliance: ComplianceLevel::Gold,
            mcp_tools: vec!["mcp__nexcore__primitive_validate".to_string()],
            paired_agent: Some("primitive-extractor".to_string()),
            requires: vec!["transfer-confidence".to_string()],
        }
    }

    fn triggers(&self) -> Vec<Trigger> {
        vec![
            Trigger::command("/extract"),
            Trigger::command("/primitives"),
            Trigger::keyword("decompose to primitives"),
        ]
    }

    async fn execute(&self, ctx: SkillContext) -> SkillResult {
        let primitives = self.extract(&ctx.input);
        let output = format_output(primitives);
        Ok(output)
    }
}

fn format_output(primitives: Vec<Primitive>) -> SkillOutput {
    let headers = vec![
        "Term".into(),
        "Tier".into(),
        "Grounding".into(),
        "Conf".into(),
    ];
    let rows = primitives.iter().map(format_row).collect();
    SkillOutput::table(headers, rows).with_metadata("count", serde_json::json!(primitives.len()))
}

fn format_row(p: &Primitive) -> Vec<String> {
    vec![
        p.term.clone(),
        format!("{:?}", p.tier),
        p.grounding.clone().unwrap_or("-".into()),
        format!("{:.2}", p.transfer_confidence),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_classification() {
        let (tier, _, _) = determine_tier("sequence");
        assert_eq!(tier, PrimitiveTier::T1);

        let (tier, _, _) = determine_tier("threshold");
        assert_eq!(tier, PrimitiveTier::T2P);

        let (tier, _, _) = determine_tier("signal");
        assert_eq!(tier, PrimitiveTier::T2C);

        let (tier, _, _) = determine_tier("pharmacovigilance");
        assert_eq!(tier, PrimitiveTier::T3);
    }

    #[tokio::test]
    async fn test_skill_execution() {
        let skill = PrimitiveExtractor::new();
        let ctx = SkillContext::new("/extract sequence mapping state");
        let result = skill.execute(ctx).await.unwrap();
        assert!(matches!(
            result.content,
            crate::core::OutputContent::Table { .. }
        ));
    }
}
