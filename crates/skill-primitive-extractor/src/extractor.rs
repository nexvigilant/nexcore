//! Primitive extraction logic

use crate::types::{Primitive, PrimitiveTier};
use skill_core::{
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

    // ── identify_terms coverage ─────────────────────────────────────

    #[test]
    fn test_identify_terms_filters_short_words() {
        let terms = identify_terms("a an the and");
        assert!(terms.is_empty(), "words <=3 chars should be filtered");
    }

    #[test]
    fn test_identify_terms_filters_numeric_starts() {
        let terms = identify_terms("123abc valid 9rate");
        assert_eq!(terms, vec!["valid"]);
    }

    #[test]
    fn test_identify_terms_trims_trailing_punctuation() {
        // Leading non-alpha chars are filtered first, trailing gets trimmed
        let terms = identify_terms("sequence. mapping, state!");
        assert_eq!(terms, vec!["sequence", "mapping", "state"]);
    }

    #[test]
    fn test_identify_terms_empty_input() {
        let terms = identify_terms("");
        assert!(terms.is_empty());
    }

    // ── extract() coverage ──────────────────────────────────────────

    #[test]
    fn test_extract_returns_primitives() {
        let ext = PrimitiveExtractor::new();
        let prims = ext.extract("sequence threshold signal pharmacovigilance");
        assert_eq!(prims.len(), 4);
        assert_eq!(prims[0].tier, PrimitiveTier::T1);
        assert_eq!(prims[1].tier, PrimitiveTier::T2P);
        assert_eq!(prims[2].tier, PrimitiveTier::T2C);
        assert_eq!(prims[3].tier, PrimitiveTier::T3);
    }

    #[test]
    fn test_extract_empty_text() {
        let ext = PrimitiveExtractor::new();
        let prims = ext.extract("");
        assert!(prims.is_empty());
    }

    #[test]
    fn test_extract_all_short_words() {
        let ext = PrimitiveExtractor::new();
        let prims = ext.extract("a be do go");
        assert!(prims.is_empty());
    }

    // ── confidence values ───────────────────────────────────────────

    #[test]
    fn test_t1_confidence() {
        let (_, _, conf) = determine_tier("mapping");
        assert!((conf - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_t2p_confidence() {
        let (_, grounding, conf) = determine_tier("score");
        assert!((conf - 0.78).abs() < f64::EPSILON);
        assert_eq!(grounding.as_deref(), Some("Quantity (N)"));
    }

    #[test]
    fn test_t2c_confidence() {
        let (_, grounding, conf) = determine_tier("assessment");
        assert!((conf - 0.65).abs() < f64::EPSILON);
        assert_eq!(grounding.as_deref(), Some("State + Mapping"));
    }

    #[test]
    fn test_t3_confidence() {
        let (_, grounding, conf) = determine_tier("aspirin");
        assert!((conf - 0.42).abs() < f64::EPSILON);
        assert_eq!(grounding.as_deref(), Some("Domain-specific"));
    }

    // ── classify_term coverage ──────────────────────────────────────

    #[test]
    fn test_classify_term_fields() {
        let p = classify_term("recursion");
        assert_eq!(p.term, "recursion");
        assert_eq!(p.tier, PrimitiveTier::T1);
        assert!(p.grounding.is_none());
        assert!(p.definition.contains("recursion"));
    }

    // ── format_row coverage ─────────────────────────────────────────

    #[test]
    fn test_format_row_with_grounding() {
        let p = Primitive {
            term: "rate".into(),
            definition: "A concept".into(),
            tier: PrimitiveTier::T2P,
            grounding: Some("Quantity (N)".into()),
            transfer_confidence: 0.78,
        };
        let row = format_row(&p);
        assert_eq!(row.len(), 4);
        assert_eq!(row[0], "rate");
        assert_eq!(row[1], "T2P");
        assert_eq!(row[2], "Quantity (N)");
        assert_eq!(row[3], "0.78");
    }

    #[test]
    fn test_format_row_no_grounding() {
        let p = Primitive {
            term: "void".into(),
            definition: "A concept".into(),
            tier: PrimitiveTier::T1,
            grounding: None,
            transfer_confidence: 0.95,
        };
        let row = format_row(&p);
        assert_eq!(row[2], "-");
    }

    // ── Skill trait methods ─────────────────────────────────────────

    #[test]
    fn test_skill_name() {
        let ext = PrimitiveExtractor::new();
        assert_eq!(ext.name(), "primitive-extractor");
    }

    #[test]
    fn test_skill_metadata() {
        let ext = PrimitiveExtractor::new();
        let meta = ext.metadata();
        assert_eq!(meta.name, "primitive-extractor");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.compliance, ComplianceLevel::Gold);
    }

    #[test]
    fn test_skill_triggers() {
        let ext = PrimitiveExtractor::new();
        let triggers = ext.triggers();
        assert_eq!(triggers.len(), 3);
    }

    #[test]
    fn test_default_impl() {
        let ext = PrimitiveExtractor::default();
        assert_eq!(ext.name(), "primitive-extractor");
    }

    // ── T1 keyword coverage ─────────────────────────────────────────

    #[test]
    fn test_all_t1_keywords() {
        for keyword in [
            "sequence",
            "mapping",
            "state",
            "recursion",
            "void",
            "existence",
        ] {
            let (tier, _, _) = determine_tier(keyword);
            assert_eq!(tier, PrimitiveTier::T1, "'{keyword}' should be T1");
        }
    }

    #[test]
    fn test_all_t2p_keywords() {
        for keyword in ["score", "threshold", "rate", "count", "index"] {
            let (tier, _, _) = determine_tier(keyword);
            assert_eq!(tier, PrimitiveTier::T2P, "'{keyword}' should be T2P");
        }
    }

    #[test]
    fn test_all_t2c_keywords() {
        for keyword in ["signal", "assessment", "result", "response"] {
            let (tier, _, _) = determine_tier(keyword);
            assert_eq!(tier, PrimitiveTier::T2C, "'{keyword}' should be T2C");
        }
    }

    #[tokio::test]
    async fn test_skill_execution() {
        let skill = PrimitiveExtractor::new();
        let ctx = SkillContext::new("/extract sequence mapping state");
        let result = skill.execute(ctx).await.unwrap();
        assert!(matches!(
            result.content,
            skill_core::OutputContent::Table { .. }
        ));
    }
}
