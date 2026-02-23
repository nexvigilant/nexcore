//! # SMST v2 (Skill Machine Specification Template v2)
//!
//! Diamond v2 compliance scoring with 8 weighted components.

use nexcore_error::Error;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// SMST v2 validation errors.
#[derive(Debug, Error)]
pub enum SmstV2Error {
    /// Missing SKILL.md file.
    #[error("Missing SKILL.md")]
    MissingSkillMd,
    /// Failed to read file.
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),
}

/// SMST v2 component scores.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentScores {
    /// Inputs (10%).
    pub inputs: u32,
    /// Outputs (10%).
    pub outputs: u32,
    /// State (10%).
    pub state: u32,
    /// Operator Mode (15%).
    pub operator_mode: u32,
    /// Performance (10%).
    pub performance: u32,
    /// Invariants (15%).
    pub invariants: u32,
    /// Failure Modes (15%).
    pub failure_modes: u32,
    /// Telemetry (15%).
    pub telemetry: u32,
}

impl ComponentScores {
    /// Calculate weighted total score (0-100).
    #[must_use]
    pub fn total(&self) -> u32 {
        let weighted = (self.inputs * 10
            + self.outputs * 10
            + self.state * 10
            + self.operator_mode * 15
            + self.performance * 10
            + self.invariants * 15
            + self.failure_modes * 15
            + self.telemetry * 15)
            / 100;
        weighted.min(100)
    }
}

/// SMST v2 validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmstV2Result {
    /// Total weighted score (0-100).
    pub total_score: u32,
    /// Is Diamond v2 compliant (score >= 90).
    pub is_diamond: bool,
    /// Component breakdown.
    pub components: ComponentScores,
    /// Recommendations for improvement.
    pub recommendations: Vec<String>,
    /// Compliance level string.
    pub compliance_level: String,
}

/// Validate a skill for SMST v2 compliance.
pub fn validate_smst_v2(skill_path: &Path) -> Result<SmstV2Result, SmstV2Error> {
    let skill_md = skill_path.join("SKILL.md");
    if !skill_md.exists() {
        return Err(SmstV2Error::MissingSkillMd);
    }
    let content = std::fs::read_to_string(&skill_md)?;
    Ok(extract_smst_v2(&content))
}

/// Extract SMST v2 scores from content.
#[must_use]
pub fn extract_smst_v2(content: &str) -> SmstV2Result {
    let c = content.to_lowercase();
    let mut components = ComponentScores::default();
    let mut recs = Vec::new();

    components.inputs = score_section(
        &c,
        &["## input", "### input"],
        &[": string", ": number", ": bool"],
    );
    if components.inputs < 100 {
        recs.push("Add ## Input with typed parameters".into());
    }

    components.outputs = score_section(
        &c,
        &["## output", "### output"],
        &["returns", "side effect"],
    );
    if components.outputs < 100 {
        recs.push("Add ## Output with types".into());
    }

    components.state = score_section(&c, &["## state"], &["stateless", "stateful", "session"]);
    if components.state < 100 {
        recs.push("Document state strategy".into());
    }

    components.operator_mode = score_section(
        &c,
        &["## operator"],
        &["interactive", "autonomous", "hybrid"],
    );
    if components.operator_mode < 100 {
        recs.push("Specify operator mode".into());
    }

    components.performance = score_section(&c, &["## performance"], &["o(", "latency", "memory"]);
    if components.performance < 100 {
        recs.push("Add ## Performance".into());
    }

    components.invariants = score_section(
        &c,
        &["## invariant"],
        &["precondition", "postcondition", "requires"],
    );
    if components.invariants < 100 {
        recs.push("Document invariants".into());
    }

    components.failure_modes = score_section(
        &c,
        &["## error", "## failure"],
        &["timeout", "retry", "fallback"],
    );
    if components.failure_modes < 100 {
        recs.push("Add failure modes".into());
    }

    components.telemetry = score_section(
        &c,
        &["## telemetry", "## observability"],
        &["metric", "trace", "log"],
    );
    if components.telemetry < 100 {
        recs.push("Add telemetry hooks".into());
    }

    let total_score = components.total();
    let is_diamond = total_score >= 90;
    let compliance_level = match total_score {
        90..=100 => "diamond_v2",
        80..=89 => "platinum",
        70..=79 => "gold",
        50..=69 => "silver",
        _ => "bronze",
    }
    .into();

    if is_diamond {
        recs.clear();
    }

    SmstV2Result {
        total_score,
        is_diamond,
        components,
        recommendations: recs,
        compliance_level,
    }
}

fn score_section(content: &str, headers: &[&str], keywords: &[&str]) -> u32 {
    let mut score = 0;
    if headers.iter().any(|h| content.contains(h)) {
        score += 50;
    }
    for kw in keywords {
        if content.contains(kw) {
            score += 20;
        }
    }
    score.min(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_weights() {
        let scores = ComponentScores {
            inputs: 100,
            outputs: 100,
            state: 100,
            operator_mode: 100,
            performance: 100,
            invariants: 100,
            failure_modes: 100,
            telemetry: 100,
        };
        assert_eq!(scores.total(), 100);
    }
}
