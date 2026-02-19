//! # Epistemic Rigor Validation
//!
//! Validate epistemic quality of claims and analyses.

use serde::{Deserialize, Serialize};

/// Epistemic quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicAssessment {
    /// Overall quality score (0-100)
    pub score: f64,
    /// Evidence strength
    pub evidence_strength: String,
    /// Confidence level
    pub confidence: String,
    /// Issues found
    pub issues: Vec<String>,
}

/// Assess epistemic quality of a claim
#[must_use]
pub fn assess_claim(
    claim: &str,
    evidence_count: usize,
    has_citations: bool,
) -> EpistemicAssessment {
    let mut score: f64 = 50.0;
    let mut issues = Vec::new();

    if evidence_count >= 3 {
        score += 20.0;
    } else {
        issues.push(format!(
            "Insufficient evidence (only {} sources)",
            evidence_count
        ));
    }

    if has_citations {
        score += 15.0;
    } else {
        issues.push("Missing citations".to_string());
    }

    if claim.contains("may") || claim.contains("might") || claim.contains("possibly") {
        score += 10.0; // Appropriate hedging
    }

    if claim.contains("always") || claim.contains("never") || claim.contains("definitely") {
        score -= 15.0;
        issues.push("Overly certain language".to_string());
    }

    let evidence_strength = match evidence_count {
        0..=1 => "Weak",
        2..=4 => "Moderate",
        _ => "Strong",
    }
    .to_string();

    let confidence = match score as u32 {
        80..=100 => "High",
        60..=79 => "Moderate",
        40..=59 => "Low",
        _ => "Very Low",
    }
    .to_string();

    EpistemicAssessment {
        score: score.clamp(0.0, 100.0),
        evidence_strength,
        confidence,
        issues,
    }
}
