//! # Intervention Vigilance Framework Axioms (ToV §35)
//!
//! The five foundational axioms of Intervention Vigilance that generalize
//! pharmacovigilance methodology to all intervention domains.
//!
//! > **Intervention Vigilance:** The science and activities relating to detection,
//! > assessment, understanding, and prevention of harms arising from any intervention
//! > deployed at scale, throughout the intervention's lifecycle.
//!
//! # The Five IVF Axioms
//!
//! | Axiom | Statement | ToV Mapping |
//! |-------|-----------|-------------|
//! | Pharmakon | Benefit and harm are inseparable | A3 (Conservation) |
//! | Emergence | Harms emerge unpredictably | A5 (Propagation) |
//! | Vulnerability | Harms affect vulnerable disproportionately | A5 + social θ |
//! | Scale | Harms visible only at deployment scale | A2 (Hierarchy) |
//! | Vigilance | Continuous monitoring is necessary | Definition 1.1 (ℳ) |
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::pv::ivf::{
//!     IvfAxiom, IvfAssessment, InterventionCharacteristics,
//!     assess_ivf_axioms,
//! };
//!
//! let characteristics = InterventionCharacteristics::new()
//!     .with_potency(0.8)
//!     .with_emergence_uncertainty(0.6)
//!     .with_vulnerability_exposure(0.7);
//!
//! let assessment = assess_ivf_axioms(&characteristics);
//! assert!(assessment.requires_vigilance());
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// IVF AXIOMS (T2-P)
// ============================================================================

/// The five IVF Axioms (ToV §35.2).
///
/// # Tier: T2-P
///
/// Cross-domain primitive enum representing the foundational axioms
/// of Intervention Vigilance.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IvfAxiom {
    /// Benefit and harm are inseparable properties of potency.
    ///
    /// ToV Mapping: Axiom 3 (Conservation) - same gᵢ defines safe/unsafe states.
    ///
    /// > "Any intervention powerful enough to create intended change is
    /// > powerful enough to create unintended change."
    Pharmakon = 1,

    /// Harms emerge unpredictably, often in unanticipated populations.
    ///
    /// ToV Mapping: Axiom 5 (Hierarchical Propagation) - Pᵢ→ᵢ₊₁ depends on θ.
    ///
    /// > "Harms from interventions emerge unpredictably, often in populations
    /// > and through mechanisms not anticipated by intervention designers."
    Emergence = 2,

    /// Harms disproportionately affect vulnerable populations.
    ///
    /// ToV Mapping: Axiom 5 extended - θ includes social vulnerability.
    ///
    /// > "Intervention harms disproportionately affect those with least power
    /// > to avoid, report, or remedy them."
    Vulnerability = 3,

    /// Harms become visible only after deployment at scale.
    ///
    /// ToV Mapping: Axiom 2 (Hierarchy) - Levels 6-8 require population exposure.
    ///
    /// > "Harms become visible only after deployment at scale, making
    /// > pre-deployment testing inherently incomplete."
    Scale = 4,

    /// Systematic, continuous monitoring is a necessary response.
    ///
    /// ToV Mapping: Definition 1.1 - justifies monitoring apparatus ℳ.
    ///
    /// > "Systematic, continuous monitoring is a rational and necessary
    /// > response—not a bureaucratic burden but an ethical obligation."
    Vigilance = 5,
}

impl IvfAxiom {
    /// Get all five axioms in order.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Pharmakon,
            Self::Emergence,
            Self::Vulnerability,
            Self::Scale,
            Self::Vigilance,
        ]
    }

    /// Get axiom by number (1-5).
    #[must_use]
    pub const fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::Pharmakon),
            2 => Some(Self::Emergence),
            3 => Some(Self::Vulnerability),
            4 => Some(Self::Scale),
            5 => Some(Self::Vigilance),
            _ => None,
        }
    }

    /// Get axiom number (1-5).
    #[must_use]
    pub const fn number(self) -> u8 {
        self as u8
    }

    /// Get the ToV axiom mapping.
    #[must_use]
    pub const fn tov_mapping(self) -> &'static str {
        match self {
            Self::Pharmakon => "A3 (Conservation)",
            Self::Emergence => "A5 (Propagation)",
            Self::Vulnerability => "A5 + social θ",
            Self::Scale => "A2 (Hierarchy)",
            Self::Vigilance => "Definition 1.1 (ℳ)",
        }
    }

    /// Get the formal statement of the axiom.
    #[must_use]
    pub const fn statement(self) -> &'static str {
        match self {
            Self::Pharmakon => "Benefit and harm are inseparable properties of potency",
            Self::Emergence => "Harms emerge unpredictably in unanticipated populations",
            Self::Vulnerability => "Harms disproportionately affect vulnerable populations",
            Self::Scale => "Harms become visible only after deployment at scale",
            Self::Vigilance => "Continuous monitoring is a necessary ethical obligation",
        }
    }
}

impl std::fmt::Display for IvfAxiom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pharmakon => write!(f, "IVF-A1: Pharmakon"),
            Self::Emergence => write!(f, "IVF-A2: Emergence"),
            Self::Vulnerability => write!(f, "IVF-A3: Vulnerability"),
            Self::Scale => write!(f, "IVF-A4: Scale"),
            Self::Vigilance => write!(f, "IVF-A5: Vigilance"),
        }
    }
}

// ============================================================================
// AXIOM SATISFACTION LEVEL (T2-P)
// ============================================================================

/// Satisfaction level for an IVF axiom assessment.
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AxiomSatisfactionLevel {
    /// Axiom conditions not met (low risk from this axiom).
    NotApplicable = 0,
    /// Axiom conditions partially met (moderate risk).
    Partial = 1,
    /// Axiom conditions fully met (high risk, vigilance required).
    Full = 2,
}

impl AxiomSatisfactionLevel {
    /// Check if vigilance is recommended based on this level.
    #[must_use]
    pub const fn requires_vigilance(self) -> bool {
        matches!(self, Self::Partial | Self::Full)
    }
}

impl std::fmt::Display for AxiomSatisfactionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotApplicable => write!(f, "Not Applicable"),
            Self::Partial => write!(f, "Partial"),
            Self::Full => write!(f, "Full"),
        }
    }
}

// ============================================================================
// INTERVENTION CHARACTERISTICS (T2-C)
// ============================================================================

/// Characteristics of an intervention for IVF assessment.
///
/// # Tier: T2-C
///
/// These characteristics determine which IVF axioms apply and to what degree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionCharacteristics {
    /// Potency/power of intervention (0.0-1.0).
    ///
    /// Higher potency → Pharmakon axiom more relevant.
    pub potency: f64,

    /// Uncertainty about emergence patterns (0.0-1.0).
    ///
    /// Higher uncertainty → Emergence axiom more relevant.
    pub emergence_uncertainty: f64,

    /// Exposure to vulnerable populations (0.0-1.0).
    ///
    /// Higher exposure → Vulnerability axiom more relevant.
    pub vulnerability_exposure: f64,

    /// Deployment scale (0.0-1.0).
    ///
    /// Higher scale → Scale axiom more relevant.
    pub deployment_scale: f64,

    /// Completeness of pre-deployment testing (0.0-1.0).
    ///
    /// Lower completeness → all axioms more relevant.
    pub testing_completeness: f64,
}

impl InterventionCharacteristics {
    /// Create with default (neutral) values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            potency: 0.5,
            emergence_uncertainty: 0.5,
            vulnerability_exposure: 0.5,
            deployment_scale: 0.5,
            testing_completeness: 0.5,
        }
    }

    /// Set potency.
    #[must_use]
    pub fn with_potency(mut self, potency: f64) -> Self {
        self.potency = potency.clamp(0.0, 1.0);
        self
    }

    /// Set emergence uncertainty.
    #[must_use]
    pub fn with_emergence_uncertainty(mut self, uncertainty: f64) -> Self {
        self.emergence_uncertainty = uncertainty.clamp(0.0, 1.0);
        self
    }

    /// Set vulnerability exposure.
    #[must_use]
    pub fn with_vulnerability_exposure(mut self, exposure: f64) -> Self {
        self.vulnerability_exposure = exposure.clamp(0.0, 1.0);
        self
    }

    /// Set deployment scale.
    #[must_use]
    pub fn with_deployment_scale(mut self, scale: f64) -> Self {
        self.deployment_scale = scale.clamp(0.0, 1.0);
        self
    }

    /// Set testing completeness.
    #[must_use]
    pub fn with_testing_completeness(mut self, completeness: f64) -> Self {
        self.testing_completeness = completeness.clamp(0.0, 1.0);
        self
    }

    /// Calculate overall risk level (0.0-1.0).
    #[must_use]
    pub fn overall_risk(&self) -> f64 {
        // Weight factors
        let potency_weight = 0.25;
        let emergence_weight = 0.20;
        let vulnerability_weight = 0.20;
        let scale_weight = 0.20;
        let testing_gap_weight = 0.15;

        let testing_gap = 1.0 - self.testing_completeness;

        self.potency * potency_weight
            + self.emergence_uncertainty * emergence_weight
            + self.vulnerability_exposure * vulnerability_weight
            + self.deployment_scale * scale_weight
            + testing_gap * testing_gap_weight
    }
}

impl Default for InterventionCharacteristics {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// AXIOM ASSESSMENT RESULT (T2-C)
// ============================================================================

/// Result of assessing a single IVF axiom.
///
/// # Tier: T2-C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxiomAssessmentResult {
    /// Which axiom was assessed.
    pub axiom: IvfAxiom,
    /// Satisfaction level.
    pub level: AxiomSatisfactionLevel,
    /// Risk score from this axiom (0.0-1.0).
    pub risk_score: f64,
    /// Explanation of the assessment.
    pub rationale: String,
}

impl AxiomAssessmentResult {
    /// Create a new assessment result.
    #[must_use]
    pub fn new(axiom: IvfAxiom, level: AxiomSatisfactionLevel, risk_score: f64) -> Self {
        Self {
            axiom,
            level,
            risk_score: risk_score.clamp(0.0, 1.0),
            rationale: String::new(),
        }
    }

    /// Add rationale.
    #[must_use]
    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = rationale.into();
        self
    }
}

// ============================================================================
// FULL IVF ASSESSMENT (T3)
// ============================================================================

/// Complete IVF axiom assessment for an intervention.
///
/// # Tier: T3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IvfAssessment {
    /// Assessment results for each axiom.
    pub axiom_results: Vec<AxiomAssessmentResult>,
    /// Overall risk score (0.0-1.0).
    pub overall_risk: f64,
    /// Whether vigilance is required.
    pub vigilance_required: bool,
    /// Recommended monitoring intensity.
    pub monitoring_intensity: MonitoringIntensity,
}

impl IvfAssessment {
    /// Check if vigilance is required.
    #[must_use]
    pub fn requires_vigilance(&self) -> bool {
        self.vigilance_required
    }

    /// Get the highest-risk axiom.
    #[must_use]
    pub fn highest_risk_axiom(&self) -> Option<&AxiomAssessmentResult> {
        self.axiom_results.iter().max_by(|a, b| {
            a.risk_score
                .partial_cmp(&b.risk_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get all axioms that require vigilance.
    #[must_use]
    pub fn vigilance_axioms(&self) -> Vec<&AxiomAssessmentResult> {
        self.axiom_results
            .iter()
            .filter(|r| r.level.requires_vigilance())
            .collect()
    }
}

/// Recommended monitoring intensity based on IVF assessment.
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MonitoringIntensity {
    /// Minimal monitoring (low risk).
    Minimal = 0,
    /// Standard monitoring.
    Standard = 1,
    /// Enhanced monitoring.
    Enhanced = 2,
    /// Intensive monitoring (high risk).
    Intensive = 3,
}

impl std::fmt::Display for MonitoringIntensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimal => write!(f, "Minimal"),
            Self::Standard => write!(f, "Standard"),
            Self::Enhanced => write!(f, "Enhanced"),
            Self::Intensive => write!(f, "Intensive"),
        }
    }
}

// ============================================================================
// ASSESSMENT FUNCTION
// ============================================================================

/// Assess all IVF axioms for an intervention.
///
/// Returns a complete assessment with risk scores and recommendations.
#[must_use]
pub fn assess_ivf_axioms(characteristics: &InterventionCharacteristics) -> IvfAssessment {
    let mut axiom_results = Vec::with_capacity(5);

    // Assess each axiom
    axiom_results.push(assess_pharmakon(characteristics));
    axiom_results.push(assess_emergence(characteristics));
    axiom_results.push(assess_vulnerability(characteristics));
    axiom_results.push(assess_scale(characteristics));
    axiom_results.push(assess_vigilance(characteristics));

    // Calculate overall risk
    let overall_risk = characteristics.overall_risk();

    // Determine if vigilance required
    let vigilance_required =
        axiom_results.iter().any(|r| r.level.requires_vigilance()) || overall_risk > 0.3;

    // Determine monitoring intensity
    let monitoring_intensity = if overall_risk > 0.7 {
        MonitoringIntensity::Intensive
    } else if overall_risk > 0.5 {
        MonitoringIntensity::Enhanced
    } else if overall_risk > 0.3 {
        MonitoringIntensity::Standard
    } else {
        MonitoringIntensity::Minimal
    };

    IvfAssessment {
        axiom_results,
        overall_risk,
        vigilance_required,
        monitoring_intensity,
    }
}

fn assess_pharmakon(c: &InterventionCharacteristics) -> AxiomAssessmentResult {
    let risk = c.potency;
    let level = if risk > 0.7 {
        AxiomSatisfactionLevel::Full
    } else if risk > 0.3 {
        AxiomSatisfactionLevel::Partial
    } else {
        AxiomSatisfactionLevel::NotApplicable
    };

    AxiomAssessmentResult::new(IvfAxiom::Pharmakon, level, risk).with_rationale(format!(
        "Potency {:.0}%: {}",
        risk * 100.0,
        if risk > 0.7 {
            "High potency implies inseparable benefit/harm"
        } else if risk > 0.3 {
            "Moderate potency, some harm potential"
        } else {
            "Low potency, limited harm capacity"
        }
    ))
}

fn assess_emergence(c: &InterventionCharacteristics) -> AxiomAssessmentResult {
    // Emergence risk increases with uncertainty AND incomplete testing
    let testing_gap = 1.0 - c.testing_completeness;
    let risk = (c.emergence_uncertainty * 0.6 + testing_gap * 0.4).min(1.0);

    let level = if risk > 0.6 {
        AxiomSatisfactionLevel::Full
    } else if risk > 0.3 {
        AxiomSatisfactionLevel::Partial
    } else {
        AxiomSatisfactionLevel::NotApplicable
    };

    AxiomAssessmentResult::new(IvfAxiom::Emergence, level, risk).with_rationale(format!(
        "Uncertainty {:.0}%, Testing gap {:.0}%: {}",
        c.emergence_uncertainty * 100.0,
        testing_gap * 100.0,
        if risk > 0.6 {
            "High emergence uncertainty"
        } else if risk > 0.3 {
            "Moderate emergence risk"
        } else {
            "Well-characterized emergence patterns"
        }
    ))
}

fn assess_vulnerability(c: &InterventionCharacteristics) -> AxiomAssessmentResult {
    let risk = c.vulnerability_exposure;

    let level = if risk > 0.7 {
        AxiomSatisfactionLevel::Full
    } else if risk > 0.4 {
        AxiomSatisfactionLevel::Partial
    } else {
        AxiomSatisfactionLevel::NotApplicable
    };

    AxiomAssessmentResult::new(IvfAxiom::Vulnerability, level, risk).with_rationale(format!(
        "Vulnerability exposure {:.0}%: {}",
        risk * 100.0,
        if risk > 0.7 {
            "High exposure to vulnerable populations"
        } else if risk > 0.4 {
            "Moderate vulnerability exposure"
        } else {
            "Limited vulnerable population exposure"
        }
    ))
}

fn assess_scale(c: &InterventionCharacteristics) -> AxiomAssessmentResult {
    // Scale risk increases with deployment AND decreases with testing
    let testing_factor = 1.0 - (c.testing_completeness * 0.5);
    let risk = (c.deployment_scale * testing_factor).min(1.0);

    let level = if risk > 0.6 {
        AxiomSatisfactionLevel::Full
    } else if risk > 0.3 {
        AxiomSatisfactionLevel::Partial
    } else {
        AxiomSatisfactionLevel::NotApplicable
    };

    AxiomAssessmentResult::new(IvfAxiom::Scale, level, risk).with_rationale(format!(
        "Deployment scale {:.0}%: {}",
        c.deployment_scale * 100.0,
        if risk > 0.6 {
            "Large-scale deployment may reveal hidden harms"
        } else if risk > 0.3 {
            "Moderate scale, some hidden harm potential"
        } else {
            "Limited scale, pre-deployment testing adequate"
        }
    ))
}

fn assess_vigilance(c: &InterventionCharacteristics) -> AxiomAssessmentResult {
    // Vigilance axiom is a consequence of the other four
    let other_risk = c.overall_risk();

    let level = if other_risk > 0.5 {
        AxiomSatisfactionLevel::Full
    } else if other_risk > 0.2 {
        AxiomSatisfactionLevel::Partial
    } else {
        AxiomSatisfactionLevel::NotApplicable
    };

    AxiomAssessmentResult::new(IvfAxiom::Vigilance, level, other_risk).with_rationale(format!(
        "Overall risk {:.0}%: {}",
        other_risk * 100.0,
        if other_risk > 0.5 {
            "Continuous monitoring is ethically required"
        } else if other_risk > 0.2 {
            "Standard monitoring recommended"
        } else {
            "Minimal monitoring sufficient"
        }
    ))
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ivf_axioms() {
        assert_eq!(IvfAxiom::all().len(), 5);
        assert_eq!(IvfAxiom::Pharmakon.number(), 1);
        assert_eq!(IvfAxiom::Vigilance.number(), 5);
        assert_eq!(IvfAxiom::from_number(3), Some(IvfAxiom::Vulnerability));
        assert_eq!(IvfAxiom::from_number(6), None);
    }

    #[test]
    fn test_axiom_tov_mappings() {
        assert_eq!(IvfAxiom::Pharmakon.tov_mapping(), "A3 (Conservation)");
        assert_eq!(IvfAxiom::Emergence.tov_mapping(), "A5 (Propagation)");
        assert_eq!(IvfAxiom::Vulnerability.tov_mapping(), "A5 + social θ");
        assert_eq!(IvfAxiom::Scale.tov_mapping(), "A2 (Hierarchy)");
        assert_eq!(IvfAxiom::Vigilance.tov_mapping(), "Definition 1.1 (ℳ)");
    }

    #[test]
    fn test_satisfaction_levels() {
        assert!(!AxiomSatisfactionLevel::NotApplicable.requires_vigilance());
        assert!(AxiomSatisfactionLevel::Partial.requires_vigilance());
        assert!(AxiomSatisfactionLevel::Full.requires_vigilance());
    }

    #[test]
    fn test_intervention_characteristics() {
        let c = InterventionCharacteristics::new()
            .with_potency(0.8)
            .with_emergence_uncertainty(0.6)
            .with_vulnerability_exposure(0.7)
            .with_deployment_scale(0.9)
            .with_testing_completeness(0.3);

        assert!(c.overall_risk() > 0.5);
    }

    #[test]
    fn test_low_risk_assessment() {
        let c = InterventionCharacteristics::new()
            .with_potency(0.2)
            .with_emergence_uncertainty(0.2)
            .with_vulnerability_exposure(0.1)
            .with_deployment_scale(0.1)
            .with_testing_completeness(0.9);

        let assessment = assess_ivf_axioms(&c);
        assert!(assessment.overall_risk < 0.3);
        assert_eq!(
            assessment.monitoring_intensity,
            MonitoringIntensity::Minimal
        );
    }

    #[test]
    fn test_high_risk_assessment() {
        let c = InterventionCharacteristics::new()
            .with_potency(0.9)
            .with_emergence_uncertainty(0.8)
            .with_vulnerability_exposure(0.8)
            .with_deployment_scale(0.9)
            .with_testing_completeness(0.2);

        let assessment = assess_ivf_axioms(&c);
        assert!(assessment.overall_risk > 0.6);
        assert!(assessment.requires_vigilance());
        assert!(matches!(
            assessment.monitoring_intensity,
            MonitoringIntensity::Enhanced | MonitoringIntensity::Intensive
        ));
    }

    #[test]
    fn test_pharmakon_axiom_assessment() {
        let high_potency = InterventionCharacteristics::new().with_potency(0.9);
        let result = assess_pharmakon(&high_potency);
        assert_eq!(result.level, AxiomSatisfactionLevel::Full);
        assert!(result.risk_score > 0.8);

        let low_potency = InterventionCharacteristics::new().with_potency(0.2);
        let result = assess_pharmakon(&low_potency);
        assert_eq!(result.level, AxiomSatisfactionLevel::NotApplicable);
    }

    #[test]
    fn test_vulnerability_axiom_assessment() {
        let high_vuln = InterventionCharacteristics::new().with_vulnerability_exposure(0.9);
        let result = assess_vulnerability(&high_vuln);
        assert_eq!(result.level, AxiomSatisfactionLevel::Full);

        let low_vuln = InterventionCharacteristics::new().with_vulnerability_exposure(0.2);
        let result = assess_vulnerability(&low_vuln);
        assert_eq!(result.level, AxiomSatisfactionLevel::NotApplicable);
    }

    #[test]
    fn test_highest_risk_axiom() {
        let c = InterventionCharacteristics::new()
            .with_potency(0.9) // High
            .with_emergence_uncertainty(0.3) // Low
            .with_vulnerability_exposure(0.2); // Low

        let assessment = assess_ivf_axioms(&c);
        let highest = assessment.highest_risk_axiom().unwrap();
        assert_eq!(highest.axiom, IvfAxiom::Pharmakon);
    }
}
