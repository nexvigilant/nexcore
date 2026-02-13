//! Credibility Evidence and Fit-for-Use Assessment
//!
//! ## T1 Grounding
//!
//! - **CredibilityEvidence**: ∃ (Existence) + κ (Comparison)
//!   - Evidence exists and can be compared for adequacy
//!
//! - **FitForUse**: ∃ (Existence) + κ (Comparison)
//!   - Data is Relevant (exists for purpose) AND Reliable (meets quality threshold)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Type of credibility evidence
///
/// T1 Grounding: σ (Sequence) — Ordered categories of evidence
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Model architecture and design rationale
    Architecture,
    /// Training data characteristics
    TrainingData,
    /// Validation metrics (ROC, AUC, etc.)
    ValidationMetrics,
    /// Independent test results
    TestResults,
    /// Bias and fairness analysis
    BiasAnalysis,
    /// Explainability/interpretability
    Explainability,
    /// Prior knowledge/literature
    PriorKnowledge,
    /// Regulatory precedent
    Precedent,
    /// Other with description
    Other(String),
}

impl fmt::Display for EvidenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Architecture => write!(f, "Architecture"),
            Self::TrainingData => write!(f, "Training Data"),
            Self::ValidationMetrics => write!(f, "Validation Metrics"),
            Self::TestResults => write!(f, "Test Results"),
            Self::BiasAnalysis => write!(f, "Bias Analysis"),
            Self::Explainability => write!(f, "Explainability"),
            Self::PriorKnowledge => write!(f, "Prior Knowledge"),
            Self::Precedent => write!(f, "Precedent"),
            Self::Other(s) => write!(f, "Other: {}", s),
        }
    }
}

/// Quality level of evidence
///
/// T1 Grounding: κ (Comparison) — Evidence strength ranking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EvidenceQuality {
    /// Weak or indirect evidence
    Low,
    /// Moderate quality, some limitations
    Medium,
    /// Strong, direct, well-documented
    High,
}

impl EvidenceQuality {
    /// Converts to numeric score
    ///
    /// T1 Grounding: N (Quantity)
    pub fn score(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }
}

/// A piece of credibility evidence supporting AI model output
///
/// T1 Grounding: ∃ (Existence) + κ (Comparison)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredibilityEvidence {
    /// Type of evidence
    evidence_type: EvidenceType,
    /// Quality assessment
    quality: EvidenceQuality,
    /// Description of the evidence
    description: String,
    /// Reference (document, URL, etc.)
    reference: Option<String>,
}

impl CredibilityEvidence {
    pub fn new(
        evidence_type: EvidenceType,
        quality: EvidenceQuality,
        description: impl Into<String>,
    ) -> Self {
        Self {
            evidence_type,
            quality,
            description: description.into(),
            reference: None,
        }
    }

    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    pub fn evidence_type(&self) -> &EvidenceType {
        &self.evidence_type
    }

    pub fn quality(&self) -> EvidenceQuality {
        self.quality
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn reference(&self) -> Option<&str> {
        self.reference.as_deref()
    }

    /// Returns true if evidence is high quality
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn is_high_quality(&self) -> bool {
        self.quality == EvidenceQuality::High
    }
}

/// Data relevance criterion
///
/// T1 Grounding: ∃ (Existence) — Required elements exist
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Relevance {
    /// Data includes key elements for COU
    has_key_elements: bool,
    /// Population is representative
    is_representative: bool,
    /// Time period is appropriate
    is_temporally_appropriate: bool,
}

impl Relevance {
    pub fn new(
        has_key_elements: bool,
        is_representative: bool,
        is_temporally_appropriate: bool,
    ) -> Self {
        Self {
            has_key_elements,
            is_representative,
            is_temporally_appropriate,
        }
    }

    /// Returns true if all relevance criteria met
    ///
    /// T1 Grounding: κ (Comparison) — AND of all criteria
    pub fn is_adequate(&self) -> bool {
        self.has_key_elements && self.is_representative && self.is_temporally_appropriate
    }

    /// Counts satisfied criteria
    ///
    /// T1 Grounding: N (Quantity)
    pub fn satisfied_count(&self) -> u8 {
        let mut count = 0;
        if self.has_key_elements {
            count += 1;
        }
        if self.is_representative {
            count += 1;
        }
        if self.is_temporally_appropriate {
            count += 1;
        }
        count
    }
}

/// Data reliability criterion
///
/// T1 Grounding: ∃ (Existence) — Quality properties exist
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Reliability {
    /// Data is accurate (correct values)
    is_accurate: bool,
    /// Data is complete (no critical gaps)
    is_complete: bool,
    /// Data is traceable (source documented)
    is_traceable: bool,
}

impl Reliability {
    pub fn new(is_accurate: bool, is_complete: bool, is_traceable: bool) -> Self {
        Self {
            is_accurate,
            is_complete,
            is_traceable,
        }
    }

    /// Returns true if all reliability criteria met
    ///
    /// T1 Grounding: κ (Comparison) — AND of all criteria
    pub fn is_adequate(&self) -> bool {
        self.is_accurate && self.is_complete && self.is_traceable
    }

    /// Counts satisfied criteria
    ///
    /// T1 Grounding: N (Quantity)
    pub fn satisfied_count(&self) -> u8 {
        let mut count = 0;
        if self.is_accurate {
            count += 1;
        }
        if self.is_complete {
            count += 1;
        }
        if self.is_traceable {
            count += 1;
        }
        count
    }
}

/// Fit-for-Use assessment: Data is Relevant AND Reliable
///
/// T1 Grounding: ∃ (Existence) + κ (Comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FitForUse {
    relevance: Relevance,
    reliability: Reliability,
}

impl FitForUse {
    pub fn new(relevance: Relevance, reliability: Reliability) -> Self {
        Self {
            relevance,
            reliability,
        }
    }

    pub fn relevance(&self) -> &Relevance {
        &self.relevance
    }

    pub fn reliability(&self) -> &Reliability {
        &self.reliability
    }

    /// Returns true if data is fit for use (both relevant AND reliable)
    ///
    /// T1 Grounding: κ (Comparison) — Logical AND
    pub fn is_adequate(&self) -> bool {
        self.relevance.is_adequate() && self.reliability.is_adequate()
    }

    /// Returns total score (0-6)
    ///
    /// T1 Grounding: N (Quantity)
    pub fn total_score(&self) -> u8 {
        self.relevance.satisfied_count() + self.reliability.satisfied_count()
    }

    /// Returns a passing fit-for-use assessment (all criteria met)
    pub fn passing() -> Self {
        Self {
            relevance: Relevance::new(true, true, true),
            reliability: Reliability::new(true, true, true),
        }
    }

    /// Returns a failing fit-for-use assessment (no criteria met)
    pub fn failing() -> Self {
        Self {
            relevance: Relevance::new(false, false, false),
            reliability: Reliability::new(false, false, false),
        }
    }
}

impl fmt::Display for FitForUse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Fit-for-Use: {} ({}/6 criteria)",
            if self.is_adequate() { "PASS" } else { "FAIL" },
            self.total_score()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_type_display() {
        assert_eq!(EvidenceType::Architecture.to_string(), "Architecture");
        assert_eq!(
            EvidenceType::Other("Custom".into()).to_string(),
            "Other: Custom"
        );
    }

    #[test]
    fn test_evidence_quality_ordering() {
        assert!(EvidenceQuality::Low < EvidenceQuality::Medium);
        assert!(EvidenceQuality::Medium < EvidenceQuality::High);
    }

    #[test]
    fn test_credibility_evidence_creation() {
        let evidence = CredibilityEvidence::new(
            EvidenceType::ValidationMetrics,
            EvidenceQuality::High,
            "ROC AUC = 0.92 (95% CI: 0.89-0.95)",
        )
        .with_reference("Internal Report #2025-01");

        assert_eq!(evidence.evidence_type(), &EvidenceType::ValidationMetrics);
        assert_eq!(evidence.quality(), EvidenceQuality::High);
        assert!(evidence.is_high_quality());
        assert!(evidence.reference().is_some());
    }

    #[test]
    fn test_relevance_adequate() {
        let good = Relevance::new(true, true, true);
        assert!(good.is_adequate());
        assert_eq!(good.satisfied_count(), 3);

        let partial = Relevance::new(true, false, true);
        assert!(!partial.is_adequate());
        assert_eq!(partial.satisfied_count(), 2);
    }

    #[test]
    fn test_reliability_adequate() {
        let good = Reliability::new(true, true, true);
        assert!(good.is_adequate());
        assert_eq!(good.satisfied_count(), 3);

        let partial = Reliability::new(true, true, false);
        assert!(!partial.is_adequate());
        assert_eq!(partial.satisfied_count(), 2);
    }

    #[test]
    fn test_fit_for_use_adequate() {
        let good_rel = Relevance::new(true, true, true);
        let good_reliab = Reliability::new(true, true, true);
        let fit = FitForUse::new(good_rel, good_reliab);

        assert!(fit.is_adequate());
        assert_eq!(fit.total_score(), 6);
    }

    #[test]
    fn test_fit_for_use_partial() {
        let good_rel = Relevance::new(true, true, true);
        let bad_reliab = Reliability::new(false, false, false);
        let fit = FitForUse::new(good_rel, bad_reliab);

        assert!(!fit.is_adequate());
        assert_eq!(fit.total_score(), 3);
    }

    #[test]
    fn test_fit_for_use_passing() {
        let fit = FitForUse::passing();
        assert!(fit.is_adequate());
        assert_eq!(fit.total_score(), 6);
    }

    #[test]
    fn test_fit_for_use_failing() {
        let fit = FitForUse::failing();
        assert!(!fit.is_adequate());
        assert_eq!(fit.total_score(), 0);
    }

    #[test]
    fn test_fit_for_use_display() {
        let fit = FitForUse::passing();
        let s = fit.to_string();
        assert!(s.contains("PASS"));
        assert!(s.contains("6/6"));

        let bad = FitForUse::failing();
        let s2 = bad.to_string();
        assert!(s2.contains("FAIL"));
        assert!(s2.contains("0/6"));
    }
}
