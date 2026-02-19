//! Reality Gradient scoring for CTVP validation.
//!
//! The Reality Gradient quantifies how close a software deliverable is to
//! being production-ready by weighting evidence quality across all validation phases.

use crate::types::*;
use serde::{Deserialize, Serialize};

/// Reality Gradient calculation result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealityGradient {
    /// Overall score (0.0 - 1.0)
    pub value: f64,

    /// Human-readable interpretation
    pub interpretation: RealityInterpretation,

    /// Breakdown by phase
    pub phase_contributions: Vec<PhaseContribution>,

    /// Highest phase with validated evidence
    pub highest_validated_phase: Option<ValidationPhase>,

    /// Limiting factor (lowest-contributing phase)
    pub limiting_factor: Option<ValidationPhase>,
}

/// Interpretation of the Reality Gradient score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealityInterpretation {
    /// Score < 0.20 - Only Phase 0 evidence
    TestingTheater,

    /// Score 0.20 - 0.50 - Through Phase 1
    SafetyValidated,

    /// Score 0.50 - 0.80 - Through Phase 2
    EfficacyDemonstrated,

    /// Score 0.80 - 0.95 - Through Phase 3
    ScaleConfirmed,

    /// Score > 0.95 - All phases validated
    ProductionReady,
}

impl RealityInterpretation {
    /// Returns the score threshold for this interpretation
    pub fn get_threshold(&self) -> f64 {
        match self {
            Self::TestingTheater => 0.0,
            Self::SafetyValidated => 0.20,
            Self::EfficacyDemonstrated => 0.50,
            Self::ScaleConfirmed => 0.80,
            Self::ProductionReady => 0.95,
        }
    }

    /// Returns the human-readable description
    pub fn get_description(&self) -> &'static str {
        match self {
            Self::TestingTheater => "Testing Theater - Only mechanism validity (Phase 0)",
            Self::SafetyValidated => "Safety Validated - Failure modes tested (through Phase 1)",
            Self::EfficacyDemonstrated => {
                "Efficacy Demonstrated - Real data validation (through Phase 2)"
            }
            Self::ScaleConfirmed => {
                "Scale Confirmed - Production-like validation (through Phase 3)"
            }
            Self::ProductionReady => "Production Ready - Full continuous validation (all phases)",
        }
    }

    /// Returns the recommended next action
    pub fn get_next_action(&self) -> &'static str {
        match self {
            Self::TestingTheater => "Add fault injection and chaos engineering tests (Phase 1)",
            Self::SafetyValidated => "Validate with real data and measure SLOs (Phase 2)",
            Self::EfficacyDemonstrated => "Implement shadow/canary deployment (Phase 3)",
            Self::ScaleConfirmed => "Add drift detection and continuous validation (Phase 4)",
            Self::ProductionReady => "Maintain observability and iterate based on feedback",
        }
    }

    /// Determines interpretation from a score
    pub fn from_score(score: f64) -> Self {
        if score >= 0.95 {
            Self::ProductionReady
        } else if score >= 0.80 {
            Self::ScaleConfirmed
        } else if score >= 0.50 {
            Self::EfficacyDemonstrated
        } else if score >= 0.20 {
            Self::SafetyValidated
        } else {
            Self::TestingTheater
        }
    }
}

impl std::fmt::Display for RealityInterpretation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_description())
    }
}

/// Contribution of a single phase to the Reality Gradient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseContribution {
    /// Which phase
    pub phase: ValidationPhase,

    /// Evidence quality for this phase
    pub evidence_quality: EvidenceQuality,

    /// Weight of this phase in the calculation
    pub weight: f64,

    /// Weighted contribution to the total score
    pub contribution: f64,

    /// Maximum possible contribution for this phase
    pub max_contribution: f64,
}

impl RealityGradient {
    /// Calculates the Reality Gradient from validation results.
    pub fn calculate(results: &[ValidationResult]) -> Self {
        let mut phase_contributions = Vec::new();
        let mut total_score = 0.0;
        let mut max_score = 0.0;
        let mut highest_validated: Option<ValidationPhase> = None;
        let mut lowest_contribution: Option<(ValidationPhase, f64)> = None;

        for phase in ValidationPhase::get_all() {
            let contribution = Self::analyze_phase(phase, results);

            max_score += contribution.max_contribution;
            total_score += contribution.contribution;

            // Track highest validated phase
            if contribution.evidence_quality >= EvidenceQuality::Weak {
                highest_validated = Some(phase);
            }

            // Track limiting factor
            Self::update_lowest_contribution(&mut lowest_contribution, &contribution);

            phase_contributions.push(contribution);
        }

        let value = if max_score > 0.0 {
            total_score / max_score
        } else {
            0.0
        };
        let interpretation = RealityInterpretation::from_score(value);
        let limiting_factor = lowest_contribution.map(|(phase, _)| phase);

        Self {
            value,
            interpretation,
            phase_contributions,
            highest_validated_phase: highest_validated,
            limiting_factor,
        }
    }

    fn analyze_phase(phase: ValidationPhase, results: &[ValidationResult]) -> PhaseContribution {
        let weight = phase.get_weight();
        let max_contribution = weight * 1.0;

        let evidence_quality = results
            .iter()
            .find(|r| r.phase == phase)
            .map(|r| r.evidence_quality)
            .unwrap_or(EvidenceQuality::None);

        let contribution = weight * evidence_quality.get_value();

        PhaseContribution {
            phase,
            evidence_quality,
            weight,
            contribution,
            max_contribution,
        }
    }

    fn update_lowest_contribution(
        lowest: &mut Option<(ValidationPhase, f64)>,
        pc: &PhaseContribution,
    ) {
        let relative = if pc.max_contribution > 0.0 {
            pc.contribution / pc.max_contribution
        } else {
            0.0
        };

        match lowest {
            None => *lowest = Some((pc.phase, relative)),
            Some((_, lowest_val)) if relative < *lowest_val => *lowest = Some((pc.phase, relative)),
            _ => {}
        }
    }

    /// Calculates from evidence quality map (simpler interface)
    pub fn from_evidence_map(
        evidence: std::collections::HashMap<ValidationPhase, EvidenceQuality>,
    ) -> Self {
        let results: Vec<ValidationResult> = evidence
            .into_iter()
            .map(|(phase, quality)| {
                ValidationResult::new("", phase, ValidationOutcome::Validated, quality)
            })
            .collect();

        Self::calculate(&results)
    }

    /// Returns the gap to the next interpretation level
    pub fn get_gap_to_next_level(&self) -> Option<f64> {
        let next_threshold = match self.interpretation {
            RealityInterpretation::TestingTheater => Some(0.20),
            RealityInterpretation::SafetyValidated => Some(0.50),
            RealityInterpretation::EfficacyDemonstrated => Some(0.80),
            RealityInterpretation::ScaleConfirmed => Some(0.95),
            RealityInterpretation::ProductionReady => None,
        };

        next_threshold.map(|t| t - self.value)
    }

    /// Returns phases that need improvement (sorted by impact)
    pub fn get_improvement_priorities(&self) -> Vec<(ValidationPhase, f64)> {
        let mut priorities: Vec<(ValidationPhase, f64)> = self
            .phase_contributions
            .iter()
            .map(|pc| {
                // Impact = potential gain if improved to Strong
                let current = pc.contribution;
                let potential = pc.max_contribution;
                (pc.phase, potential - current)
            })
            .filter(|(_, gain)| *gain > 0.0)
            .collect();

        // Sort by potential gain (highest first)
        priorities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        priorities
    }

    /// Generates a summary report
    pub fn generate_summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "Reality Score: {:.2} ({})",
            self.value, self.interpretation
        ));
        lines.push(String::new());
        self.add_phase_breakdown(&mut lines);
        lines.push(String::new());
        self.add_summary_footer(&mut lines);
        lines.join("\n")
    }

    fn add_phase_breakdown(&self, lines: &mut Vec<String>) {
        lines.push("Phase Breakdown:".to_string());
        for pc in &self.phase_contributions {
            let bar_len = (pc.contribution / pc.max_contribution * 20.0) as usize;
            let bar: String = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
            lines.push(format!(
                "  {} [{}] {:?} ({:.0}%)",
                pc.phase,
                bar,
                pc.evidence_quality,
                (pc.contribution / pc.max_contribution) * 100.0
            ));
        }
    }

    fn add_summary_footer(&self, lines: &mut Vec<String>) {
        if let Some(gap) = self.get_gap_to_next_level() {
            lines.push(format!("Gap to next level: {:.2}", gap));
        }
        lines.push(format!(
            "Next action: {}",
            self.interpretation.get_next_action()
        ));
        if let Some(limiting) = self.limiting_factor {
            lines.push(format!("Limiting factor: {}", limiting));
        }
    }
}

/// Builder for creating Reality Gradient from manual inputs.
#[derive(Debug, Default)]
pub struct RealityGradientBuilder {
    evidence: std::collections::HashMap<ValidationPhase, EvidenceQuality>,
}

impl RealityGradientBuilder {
    /// Creates a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the evidence quality for a phase
    pub fn set_phase(mut self, phase: ValidationPhase, quality: EvidenceQuality) -> Self {
        self.evidence.insert(phase, quality);
        self
    }

    /// Sets Phase 0 evidence
    pub fn set_preclinical(self, quality: EvidenceQuality) -> Self {
        self.set_phase(ValidationPhase::Preclinical, quality)
    }

    /// Sets Phase 1 evidence
    pub fn set_safety(self, quality: EvidenceQuality) -> Self {
        self.set_phase(ValidationPhase::Phase1Safety, quality)
    }

    /// Sets Phase 2 evidence
    pub fn set_efficacy(self, quality: EvidenceQuality) -> Self {
        self.set_phase(ValidationPhase::Phase2Efficacy, quality)
    }

    /// Sets Phase 3 evidence
    pub fn set_confirmation(self, quality: EvidenceQuality) -> Self {
        self.set_phase(ValidationPhase::Phase3Confirmation, quality)
    }

    /// Sets Phase 4 evidence
    pub fn set_surveillance(self, quality: EvidenceQuality) -> Self {
        self.set_phase(ValidationPhase::Phase4Surveillance, quality)
    }

    /// Builds the Reality Gradient
    pub fn build(self) -> RealityGradient {
        RealityGradient::from_evidence_map(self.evidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CtvpResult;

    #[test]
    fn test_testing_theater_score() -> CtvpResult<()> {
        // Only Phase 0 with weak evidence
        let gradient = RealityGradientBuilder::new()
            .set_preclinical(EvidenceQuality::Strong)
            .set_safety(EvidenceQuality::None)
            .set_efficacy(EvidenceQuality::None)
            .set_confirmation(EvidenceQuality::None)
            .set_surveillance(EvidenceQuality::None)
            .build();

        assert!(gradient.value < 0.20);
        assert_eq!(
            gradient.interpretation,
            RealityInterpretation::TestingTheater
        );
        Ok(())
    }

    #[test]
    fn test_production_ready_score() -> CtvpResult<()> {
        // All phases with strong evidence
        let gradient = RealityGradientBuilder::new()
            .set_preclinical(EvidenceQuality::Strong)
            .set_safety(EvidenceQuality::Strong)
            .set_efficacy(EvidenceQuality::Strong)
            .set_confirmation(EvidenceQuality::Strong)
            .set_surveillance(EvidenceQuality::Strong)
            .build();

        assert!((gradient.value - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            gradient.interpretation,
            RealityInterpretation::ProductionReady
        );
        Ok(())
    }

    #[test]
    fn test_phase_weights_in_calculation() -> CtvpResult<()> {
        // Only efficacy (weight 0.30) should give score around 0.30
        let gradient = RealityGradientBuilder::new()
            .set_preclinical(EvidenceQuality::None)
            .set_safety(EvidenceQuality::None)
            .set_efficacy(EvidenceQuality::Strong)
            .set_confirmation(EvidenceQuality::None)
            .set_surveillance(EvidenceQuality::None)
            .build();

        assert!((gradient.value - 0.30).abs() < 0.01);
        Ok(())
    }

    #[test]
    fn test_improvement_priorities() -> CtvpResult<()> {
        let gradient = RealityGradientBuilder::new()
            .set_preclinical(EvidenceQuality::Strong)
            .set_safety(EvidenceQuality::Weak)
            .set_efficacy(EvidenceQuality::None)
            .set_confirmation(EvidenceQuality::None)
            .set_surveillance(EvidenceQuality::None)
            .build();

        let priorities = gradient.get_improvement_priorities();

        // Efficacy and Confirmation (both 0.30 weight) should be highest priorities
        assert!(!priorities.is_empty());

        // First priority should be one of the 0.30 weight phases with None evidence
        let first = priorities
            .first()
            .ok_or_else(|| crate::error::CtvpError::Analysis("No priorities".into()))?;
        assert!(
            first.0 == ValidationPhase::Phase2Efficacy
                || first.0 == ValidationPhase::Phase3Confirmation
        );
        Ok(())
    }

    #[test]
    fn test_gap_to_next_level() -> CtvpResult<()> {
        let gradient = RealityGradientBuilder::new()
            .set_preclinical(EvidenceQuality::Strong)
            .set_safety(EvidenceQuality::None)
            .build();

        let gap = gradient.get_gap_to_next_level();
        assert!(gap.is_some());
        assert!(gap.ok_or_else(|| crate::error::CtvpError::Analysis("No gap".into()))? > 0.0);
        Ok(())
    }
}
