//! # Transfer Confidence Skill
//!
//! Compute cross-domain mapping confidence using the formula:
//! `confidence = (structural × 0.4) + (functional × 0.4) + (contextual × 0.2)`

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]
mod compute;
pub mod grounding;

pub use compute::{TransferConfidence, TransferScore, compute_confidence};

use skill_core::{
    ComplianceLevel, Skill, SkillContext, SkillMetadata, SkillOutput, SkillResult, Trigger,
};

/// Transfer confidence skill implementation
pub struct TransferConfidenceSkill;

impl TransferConfidenceSkill {
    /// Create new skill
    pub fn new() -> Self {
        Self
    }
}

impl Default for TransferConfidenceSkill {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Skill for TransferConfidenceSkill {
    fn name(&self) -> &str {
        "transfer-confidence"
    }

    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: "transfer-confidence".to_string(),
            version: "1.0.0".to_string(),
            description: "Compute cross-domain transfer confidence".to_string(),
            compliance: ComplianceLevel::Gold,
            mcp_tools: vec![],
            paired_agent: None,
            requires: vec![],
        }
    }

    fn triggers(&self) -> Vec<Trigger> {
        vec![
            Trigger::command("/transfer-confidence"),
            Trigger::command("/tc"),
            Trigger::keyword("transfer confidence"),
        ]
    }

    async fn execute(&self, ctx: SkillContext) -> SkillResult {
        let scores = parse_scores(&ctx.args);
        let result = compute_confidence(scores.0, scores.1, scores.2);
        Ok(format_result(&result))
    }
}

fn parse_scores(args: &[String]) -> (f64, f64, f64) {
    let s = args.first().and_then(|a| a.parse().ok()).unwrap_or(0.5);
    let f = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(0.5);
    let c = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(0.5);
    (s, f, c)
}

fn format_result(result: &TransferScore) -> SkillOutput {
    let md = format!(
        "**Transfer Confidence: {:.2}**\n\n\
        | Dimension | Score | Weight |\n\
        |-----------|-------|--------|\n\
        | Structural | {:.2} | 0.40 |\n\
        | Functional | {:.2} | 0.40 |\n\
        | Contextual | {:.2} | 0.20 |\n\n\
        **Tier:** {:?}",
        result.confidence, result.structural, result.functional, result.contextual, result.tier
    );
    SkillOutput::markdown(md)
}
