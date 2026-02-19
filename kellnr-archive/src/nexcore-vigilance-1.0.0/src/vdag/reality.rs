//! Reality Gradient Calculation
//!
//! Computes the Reality Gradient score based on CTVP phase evidence.

use crate::ctvp::phases::EvidenceQuality;
use serde::{Deserialize, Serialize};

/// Phase weights for Reality Gradient calculation
pub const PHASE_WEIGHTS: [f64; 5] = [0.05, 0.15, 0.30, 0.30, 0.20];

/// Quality scores for calculation
pub const QUALITY_SCORES: [f64; 4] = [0.0, 0.33, 0.66, 1.0];

/// Interpretation of Reality Gradient score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interpretation {
    /// < 0.20: Testing Theater - BLOCKED
    TestingTheater,
    /// 0.20-0.50: Safety Validated - WARN
    SafetyValidated,
    /// 0.50-0.80: Efficacy Demonstrated - APPROVE
    EfficacyDemonstrated,
    /// 0.80-0.95: Scale Confirmed - APPROVE+
    ScaleConfirmed,
    /// > 0.95: Production Ready - APPROVE++
    ProductionReady,
}

impl Interpretation {
    /// Returns the interpretation for a given score
    pub fn from_score(score: f64) -> Self {
        if score < 0.20 {
            Self::TestingTheater
        } else if score < 0.50 {
            Self::SafetyValidated
        } else if score < 0.80 {
            Self::EfficacyDemonstrated
        } else if score < 0.95 {
            Self::ScaleConfirmed
        } else {
            Self::ProductionReady
        }
    }

    /// Returns emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::TestingTheater => "🚫",
            Self::SafetyValidated => "⚠️",
            Self::EfficacyDemonstrated => "✅",
            Self::ScaleConfirmed => "✅✅",
            Self::ProductionReady => "🚀",
        }
    }

    /// Returns action recommendation
    pub fn action(&self) -> &'static str {
        match self {
            Self::TestingTheater => "BLOCKED - Cannot approve",
            Self::SafetyValidated => "WARN - Override available",
            Self::EfficacyDemonstrated => "APPROVE",
            Self::ScaleConfirmed => "APPROVE+",
            Self::ProductionReady => "APPROVE++",
        }
    }

    /// Returns true if this interpretation blocks execution
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::TestingTheater)
    }

    /// Returns true if this interpretation requires warning
    pub fn requires_warning(&self) -> bool {
        matches!(self, Self::SafetyValidated)
    }
}

impl std::fmt::Display for Interpretation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TestingTheater => write!(f, "Testing Theater"),
            Self::SafetyValidated => write!(f, "Safety Validated"),
            Self::EfficacyDemonstrated => write!(f, "Efficacy Demonstrated"),
            Self::ScaleConfirmed => write!(f, "Scale Confirmed"),
            Self::ProductionReady => write!(f, "Production Ready"),
        }
    }
}

/// Evidence for a single CTVP phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseEvidence {
    /// Phase number (0-4)
    pub phase: u8,
    /// Evidence quality
    pub quality: EvidenceQuality,
    /// Evidence artifacts (descriptions or paths)
    pub artifacts: Vec<String>,
}

impl PhaseEvidence {
    /// Creates new phase evidence
    pub fn new(phase: u8, quality: EvidenceQuality) -> Self {
        Self {
            phase,
            quality,
            artifacts: Vec::new(),
        }
    }

    /// Adds an artifact
    pub fn with_artifact(mut self, artifact: &str) -> Self {
        self.artifacts.push(artifact.to_string());
        self
    }

    /// Returns the weighted score for this phase
    pub fn weighted_score(&self) -> f64 {
        let weight = PHASE_WEIGHTS
            .get(self.phase as usize)
            .copied()
            .unwrap_or(0.0);
        let quality_score = QUALITY_SCORES[self.quality as usize];
        weight * quality_score
    }
}

/// Reality Gradient result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityGradient {
    /// The computed score (0.0 - 1.0)
    pub score: f64,
    /// Evidence for each phase
    pub phases: Vec<PhaseEvidence>,
    /// Maximum possible score
    pub max_possible: f64,
    /// Interpretation of the score
    pub interpretation: Interpretation,
}

impl RealityGradient {
    /// Creates a new Reality Gradient from phase evidence
    pub fn calculate(phases: Vec<PhaseEvidence>) -> Self {
        let mut total_score = 0.0;
        let max_possible = PHASE_WEIGHTS.iter().sum::<f64>();

        for evidence in &phases {
            total_score += evidence.weighted_score();
        }

        // Score is the raw sum (weights already sum to 1.0)
        let score = total_score;
        let interpretation = Interpretation::from_score(score);

        Self {
            score,
            phases,
            max_possible,
            interpretation,
        }
    }

    /// Creates a new Reality Gradient from quality values
    pub fn from_qualities(qualities: [EvidenceQuality; 5]) -> Self {
        let phases: Vec<PhaseEvidence> = qualities
            .iter()
            .enumerate()
            .map(|(i, q)| PhaseEvidence::new(i as u8, *q))
            .collect();

        Self::calculate(phases)
    }

    /// Returns the interpretation string
    pub fn interpretation(&self) -> &Interpretation {
        &self.interpretation
    }

    /// Returns true if execution should be blocked
    pub fn is_blocked(&self) -> bool {
        self.interpretation.is_blocked()
    }

    /// Returns true if warning should be shown
    pub fn requires_warning(&self) -> bool {
        self.interpretation.requires_warning()
    }

    /// Generates a formatted report
    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str("\n╔══════════════════════════════════════════════════════╗\n");
        r.push_str("║  📊 REALITY GRADIENT                                  ║\n");
        r.push_str("╠══════════════════════════════════════════════════════╣\n");

        for evidence in &self.phases {
            let phase_name = match evidence.phase {
                0 => "P0 Preclinical",
                1 => "P1 Safety     ",
                2 => "P2 Efficacy   ",
                3 => "P3 Confirm    ",
                4 => "P4 Surveil    ",
                _ => "Unknown       ",
            };
            let weight = PHASE_WEIGHTS
                .get(evidence.phase as usize)
                .copied()
                .unwrap_or(0.0);
            r.push_str(&format!(
                "║  {} {} │ w={:.0}% │ {:<8?} │ {:.3}     ║\n",
                evidence.quality.emoji(),
                phase_name,
                weight * 100.0,
                evidence.quality,
                evidence.weighted_score()
            ));
        }

        r.push_str("╠══════════════════════════════════════════════════════╣\n");
        r.push_str(&format!(
            "║  SCORE: {:.2} ({})                    ║\n",
            self.score, self.interpretation
        ));
        r.push_str(&format!(
            "║  ACTION: {} {}                              ║\n",
            self.interpretation.emoji(),
            self.interpretation.action()
        ));
        r.push_str("╚══════════════════════════════════════════════════════╝\n");
        r
    }
}

impl Default for RealityGradient {
    fn default() -> Self {
        Self::from_qualities([
            EvidenceQuality::None,
            EvidenceQuality::None,
            EvidenceQuality::None,
            EvidenceQuality::None,
            EvidenceQuality::None,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testing_theater() {
        // Only Phase 0 strong = Testing Theater
        let rg = RealityGradient::from_qualities([
            EvidenceQuality::Strong,
            EvidenceQuality::None,
            EvidenceQuality::None,
            EvidenceQuality::None,
            EvidenceQuality::None,
        ]);

        assert!(rg.score < 0.20);
        assert!(rg.is_blocked());
        assert_eq!(rg.interpretation, Interpretation::TestingTheater);
    }

    #[test]
    fn test_safety_validated() {
        // Phase 0 Strong (0.05*1.0=0.05) + Phase 1 Strong (0.15*1.0=0.15) + Phase 2 Weak (0.30*0.33=0.099)
        // Total = 0.05 + 0.15 + 0.099 = 0.299, which is > 0.20
        let rg = RealityGradient::from_qualities([
            EvidenceQuality::Strong, // 0.05 * 1.0 = 0.05
            EvidenceQuality::Strong, // 0.15 * 1.0 = 0.15
            EvidenceQuality::Weak,   // 0.30 * 0.33 = 0.099
            EvidenceQuality::None,
            EvidenceQuality::None,
        ]);

        // 0.05 + 0.15 + 0.099 = 0.299
        assert!(rg.score >= 0.20, "Score {} should be >= 0.20", rg.score);
        assert!(rg.score < 0.50, "Score {} should be < 0.50", rg.score);
        assert!(rg.requires_warning());
    }

    #[test]
    fn test_efficacy_demonstrated() {
        // Need >= 0.50: Phase 0 Strong (0.05) + Phase 1 Strong (0.15) + Phase 2 Strong (0.30) = 0.50
        let rg = RealityGradient::from_qualities([
            EvidenceQuality::Strong, // 0.05 * 1.0 = 0.05
            EvidenceQuality::Strong, // 0.15 * 1.0 = 0.15
            EvidenceQuality::Strong, // 0.30 * 1.0 = 0.30
            EvidenceQuality::None,
            EvidenceQuality::None,
        ]);

        // 0.05 + 0.15 + 0.30 = 0.50
        assert!(rg.score >= 0.50, "Score {} should be >= 0.50", rg.score);
        assert!(!rg.is_blocked());
        assert!(!rg.requires_warning());
    }

    #[test]
    fn test_production_ready() {
        let rg = RealityGradient::from_qualities([
            EvidenceQuality::Strong,
            EvidenceQuality::Strong,
            EvidenceQuality::Strong,
            EvidenceQuality::Strong,
            EvidenceQuality::Strong,
        ]);

        assert!(rg.score > 0.95);
        assert_eq!(rg.interpretation, Interpretation::ProductionReady);
    }

    #[test]
    fn test_phase_weights_sum_to_one() {
        let sum: f64 = PHASE_WEIGHTS.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }
}
