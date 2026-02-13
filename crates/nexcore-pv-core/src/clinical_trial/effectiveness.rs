//! FDA Substantial Evidence of Effectiveness
//!
//! 21 CFR 314.126: Adequate and well-controlled studies
//!
//! FDA requires "substantial evidence" consisting of adequate and well-controlled
//! investigations demonstrating the drug will have its claimed effect.

use super::endpoints::EndpointHierarchy;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Study design characteristics for FDA assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyDesign {
    /// Randomized controlled trial
    pub randomized: bool,
    /// Double-blind
    pub blinded: BlindingLevel,
    /// Placebo or active control
    pub control_type: ControlType,
    /// Multi-center
    pub multi_center: bool,
    /// Pre-specified statistical analysis plan
    pub pre_specified_sap: bool,
    /// Intent-to-treat analysis
    pub itt_analysis: bool,
}

impl StudyDesign {
    /// FDA design quality score (0-1)
    #[must_use]
    pub fn quality_score(&self) -> f64 {
        let mut score = 0.0;
        if self.randomized {
            score += 0.25;
        }
        if matches!(self.blinded, BlindingLevel::Double) {
            score += 0.20;
        } else if matches!(self.blinded, BlindingLevel::Single) {
            score += 0.10;
        }
        if matches!(
            self.control_type,
            ControlType::Placebo | ControlType::Active
        ) {
            score += 0.20;
        }
        if self.multi_center {
            score += 0.10;
        }
        if self.pre_specified_sap {
            score += 0.15;
        }
        if self.itt_analysis {
            score += 0.10;
        }
        score
    }

    /// Meets FDA adequate and well-controlled criteria
    #[must_use]
    pub fn is_adequate_well_controlled(&self) -> bool {
        self.randomized
            && !matches!(self.blinded, BlindingLevel::Open)
            && !matches!(
                self.control_type,
                ControlType::None | ControlType::Historical
            )
            && self.pre_specified_sap
    }
}

impl Default for StudyDesign {
    fn default() -> Self {
        Self {
            randomized: true,
            blinded: BlindingLevel::Double,
            control_type: ControlType::Placebo,
            multi_center: true,
            pre_specified_sap: true,
            itt_analysis: true,
        }
    }
}

/// Blinding level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlindingLevel {
    Open,
    Single,
    Double,
    Triple,
}

/// Control type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlType {
    None,
    Historical,
    Placebo,
    Active,
    DoseResponse,
}

/// Substantial evidence assessment
/// T3: Domain-specific FDA determination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstantialEvidence {
    /// Number of adequate studies
    pub adequate_studies: usize,
    /// Study designs
    pub study_designs: Vec<StudyDesign>,
    /// Endpoint hierarchy results
    pub endpoint_results: EndpointHierarchy,
    /// Replication requirement met (typically 2 studies)
    pub replication_met: bool,
    /// Consistency across studies
    pub consistent_results: bool,
}

impl SubstantialEvidence {
    /// Create a new substantial evidence assessment
    #[must_use]
    pub fn new(study_designs: Vec<StudyDesign>, endpoint_results: EndpointHierarchy) -> Self {
        let adequate_studies = study_designs
            .iter()
            .filter(|s| s.is_adequate_well_controlled())
            .count();

        Self {
            adequate_studies,
            replication_met: adequate_studies >= 2,
            consistent_results: true, // Would need cross-study analysis
            study_designs,
            endpoint_results,
        }
    }

    /// FDA substantial evidence determination
    #[must_use]
    pub fn demonstrates_effectiveness(&self) -> EffectivenessConclusion {
        let primary_met = self.endpoint_results.primary_success();
        let adequate = self.adequate_studies >= 1;
        let replicated = self.replication_met;

        match (primary_met, adequate, replicated) {
            (true, true, true) => EffectivenessConclusion::SubstantialEvidence,
            (true, true, false) => EffectivenessConclusion::SingleStudyEvidence,
            (true, false, _) => EffectivenessConclusion::DesignLimitations,
            (false, _, _) => EffectivenessConclusion::InsufficientEvidence,
        }
    }

    /// Generate FDA-style summary
    #[must_use]
    pub fn fda_summary(&self) -> String {
        let conclusion = self.demonstrates_effectiveness();
        let primary_status = if self.endpoint_results.primary_success() {
            "met"
        } else {
            "not met"
        };

        format!(
            "Substantial Evidence Assessment:\n\
             - Adequate studies: {}\n\
             - Replication: {}\n\
             - Primary endpoint: {}\n\
             - Conclusion: {}",
            self.adequate_studies,
            if self.replication_met { "Yes" } else { "No" },
            primary_status,
            conclusion
        )
    }
}

/// FDA effectiveness conclusion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectivenessConclusion {
    /// Full substantial evidence (≥2 adequate studies, primary met)
    SubstantialEvidence,
    /// Single adequate study with compelling results
    SingleStudyEvidence,
    /// Positive results but study design limitations
    DesignLimitations,
    /// Primary endpoint not met
    InsufficientEvidence,
}

impl fmt::Display for EffectivenessConclusion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubstantialEvidence => write!(f, "Substantial Evidence Demonstrated"),
            Self::SingleStudyEvidence => {
                write!(f, "Single Study Evidence (may require confirmation)")
            }
            Self::DesignLimitations => write!(f, "Design Limitations Present"),
            Self::InsufficientEvidence => write!(f, "Insufficient Evidence of Effectiveness"),
        }
    }
}

/// Overall effectiveness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessAssessment {
    /// Drug/product name
    pub product_name: String,
    /// Indication
    pub indication: String,
    /// Substantial evidence evaluation
    pub evidence: SubstantialEvidence,
    /// Benefit-risk considerations
    pub benefit_risk_favorable: bool,
    /// Recommended action
    pub recommendation: RegulatoryRecommendation,
}

impl EffectivenessAssessment {
    /// Create assessment from evidence
    #[must_use]
    pub fn assess(
        product_name: impl Into<String>,
        indication: impl Into<String>,
        evidence: SubstantialEvidence,
        benefit_risk_favorable: bool,
    ) -> Self {
        let conclusion = evidence.demonstrates_effectiveness();
        let recommendation = match (conclusion, benefit_risk_favorable) {
            (EffectivenessConclusion::SubstantialEvidence, true) => {
                RegulatoryRecommendation::Approve
            }
            (EffectivenessConclusion::SubstantialEvidence, false) => {
                RegulatoryRecommendation::ApproveWithRems
            }
            (EffectivenessConclusion::SingleStudyEvidence, true) => {
                RegulatoryRecommendation::AcceleratedApproval
            }
            (EffectivenessConclusion::DesignLimitations, _) => {
                RegulatoryRecommendation::CompleteResponseLetter
            }
            _ => RegulatoryRecommendation::CompleteResponseLetter,
        };

        Self {
            product_name: product_name.into(),
            indication: indication.into(),
            evidence,
            benefit_risk_favorable,
            recommendation,
        }
    }
}

/// Regulatory recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegulatoryRecommendation {
    Approve,
    ApproveWithRems,
    AcceleratedApproval,
    CompleteResponseLetter,
    RefuseToFile,
}

impl fmt::Display for RegulatoryRecommendation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Approve => write!(f, "Recommend Approval"),
            Self::ApproveWithRems => write!(f, "Recommend Approval with REMS"),
            Self::AcceleratedApproval => write!(f, "Recommend Accelerated Approval"),
            Self::CompleteResponseLetter => write!(f, "Complete Response Letter"),
            Self::RefuseToFile => write!(f, "Refuse to File"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clinical_trial::endpoints::*;
    use crate::clinical_trial::types::*;

    #[test]
    fn test_substantial_evidence() {
        let design = StudyDesign::default();
        assert!(design.is_adequate_well_controlled());
        assert!(design.quality_score() >= 0.9);

        let result = EndpointResult::new(
            EffectSize::new(0.8),
            PValue::new_unchecked(0.001),
            ConfidenceInterval::ci95(1.5, 2.5),
            500,
        );

        let hierarchy = EndpointHierarchy::new()
            .with_primary(PrimaryEndpoint::primary("OS").with_result(result));

        let evidence = SubstantialEvidence::new(vec![design.clone(), design], hierarchy);
        assert!(evidence.replication_met);
        assert_eq!(
            evidence.demonstrates_effectiveness(),
            EffectivenessConclusion::SubstantialEvidence
        );
    }
}
