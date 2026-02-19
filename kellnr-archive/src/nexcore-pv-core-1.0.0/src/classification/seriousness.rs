//! ICH E2A Regulatory Seriousness Assessment
//!
//! Determines whether an adverse event meets regulatory seriousness criteria
//! according to ICH E2A guidelines and FDA/EMA requirements.
//!
//! # Safety Axioms
//!
//! This module implements **Safety Axiom S1: Harm Classification** - the systematic
//! categorization of adverse events by their impact on patient safety. The ICH E2A
//! criteria provide a hierarchical harm taxonomy that maps to the Theory of Vigilance
//! harm types (A-H).
//!
//! The seriousness assessment enforces **Conservation Law CL3: No harm shall go
//! unclassified** - every adverse event must be evaluated against all ICH E2A criteria
//! to ensure complete regulatory coverage.
//!
//! # Seriousness Criteria (ICH E2A)
//!
//! An adverse event is considered **serious** if it results in any of:
//! - Death
//! - Life-threatening condition
//! - Hospitalization (initial or prolonged)
//! - Disability or permanent damage
//! - Congenital anomaly/birth defect
//! - Other medically important condition requiring intervention
//!
//! # Regulatory Reporting
//!
//! | Criteria | Reporting Deadline |
//! |----------|-------------------|
//! | Death/Life-threatening | 7-15 calendar days |
//! | Other serious | 15 calendar days |
//! | Non-serious | Periodic reporting |
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::pv::classification::seriousness::{
//!     SeriousnessInput, assess_seriousness,
//! };
//!
//! let input = SeriousnessInput {
//!     death: false,
//!     life_threatening: false,
//!     hospitalization: true,
//!     hospitalization_type: Some(nexcore_vigilance::pv::classification::seriousness::HospitalizationType::Initial),
//!     disability: false,
//!     congenital_anomaly: false,
//!     other_medically_important: false,
//!     medical_justification: None,
//!     required_intervention: false,
//! };
//!
//! let result = assess_seriousness(&input);
//! assert!(result.is_serious);
//! assert!(result.regulatory_impact.requires_expedited);
//! ```
//!
//! # Reference
//!
//! ICH E2A: Clinical Safety Data Management - Definitions and Standards for
//! Expedited Reporting. International Council for Harmonisation (1994).

use serde::{Deserialize, Serialize};

/// Type of hospitalization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HospitalizationType {
    /// Initial hospital admission
    Initial,
    /// Prolonged existing hospital stay
    Prolonged,
    /// Both initial and prolonged
    Both,
}

/// ICH E2A seriousness criterion identifier
///
/// These criteria align with Safety Axiom S1 harm categories and ensure
/// regulatory compliance through exhaustive classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeriousnessCriterion {
    /// Patient died as a result of the event
    Death,
    /// Event placed patient at immediate risk of death
    LifeThreatening,
    /// Event required inpatient hospitalization
    Hospitalization,
    /// Event resulted in persistent/significant disability
    Disability,
    /// Event is a congenital anomaly/birth defect
    CongenitalAnomaly,
    /// Other medically significant event
    OtherMedicallyImportant,
}

impl std::fmt::Display for SeriousnessCriterion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Death => write!(f, "Death"),
            Self::LifeThreatening => write!(f, "Life-threatening"),
            Self::Hospitalization => write!(f, "Hospitalization"),
            Self::Disability => write!(f, "Disability/Incapacity"),
            Self::CongenitalAnomaly => write!(f, "Congenital Anomaly"),
            Self::OtherMedicallyImportant => write!(f, "Other Medically Important"),
        }
    }
}

impl SeriousnessCriterion {
    /// Get severity priority (higher = more severe)
    ///
    /// Priority ordering follows Safety Axiom S1 harm hierarchy.
    #[must_use]
    pub const fn priority(&self) -> u8 {
        match self {
            Self::Death => 6,
            Self::LifeThreatening => 5,
            Self::Disability => 4,
            Self::CongenitalAnomaly => 3,
            Self::Hospitalization => 2,
            Self::OtherMedicallyImportant => 1,
        }
    }
}

/// Regulatory category for reporting purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RegulatoryCategory {
    /// Event meets ICH E2A serious criteria
    Serious,
    /// Event does not meet serious criteria
    #[default]
    NonSerious,
}

/// Input for ICH E2A seriousness assessment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeriousnessInput {
    /// Patient died
    pub death: bool,
    /// Life-threatening at time of event
    pub life_threatening: bool,
    /// Required hospitalization
    pub hospitalization: bool,
    /// Type of hospitalization
    pub hospitalization_type: Option<HospitalizationType>,
    /// Resulted in disability/incapacity
    pub disability: bool,
    /// Congenital anomaly/birth defect
    pub congenital_anomaly: bool,
    /// Other medically important condition
    pub other_medically_important: bool,
    /// Medical justification for "other" category
    pub medical_justification: Option<String>,
    /// Intervention required to prevent serious outcome
    pub required_intervention: bool,
}

/// Regulatory reporting impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryImpact {
    /// Whether expedited reporting is required
    pub requires_expedited: bool,
    /// Reporting deadline description
    pub reporting_deadline: Option<String>,
    /// Regulatory category (serious/non-serious)
    pub regulatory_category: RegulatoryCategory,
}

impl Default for RegulatoryImpact {
    fn default() -> Self {
        Self {
            requires_expedited: false,
            reporting_deadline: None,
            regulatory_category: RegulatoryCategory::NonSerious,
        }
    }
}

/// Result of ICH E2A seriousness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriousnessResult {
    /// Whether the event is serious per ICH E2A
    pub is_serious: bool,
    /// All criteria that were met
    pub criteria_met: Vec<SeriousnessCriterion>,
    /// Primary (most severe) criterion
    pub primary_criterion: Option<SeriousnessCriterion>,
    /// Assessment confidence (0.0-1.0)
    pub confidence: f64,
    /// Regulatory reporting impact
    pub regulatory_impact: RegulatoryImpact,
}

/// Assess seriousness according to ICH E2A criteria
///
/// Implements Conservation Law CL3 by evaluating all criteria exhaustively.
/// Returns comprehensive assessment including all criteria met,
/// primary criterion, and regulatory reporting requirements.
#[must_use]
pub fn assess_seriousness(input: &SeriousnessInput) -> SeriousnessResult {
    let mut criteria_met = Vec::new();

    // Collect all met criteria (in severity order per Safety Axiom S1)
    if input.death {
        criteria_met.push(SeriousnessCriterion::Death);
    }
    if input.life_threatening {
        criteria_met.push(SeriousnessCriterion::LifeThreatening);
    }
    if input.disability {
        criteria_met.push(SeriousnessCriterion::Disability);
    }
    if input.congenital_anomaly {
        criteria_met.push(SeriousnessCriterion::CongenitalAnomaly);
    }
    if input.hospitalization {
        criteria_met.push(SeriousnessCriterion::Hospitalization);
    }
    if input.other_medically_important {
        criteria_met.push(SeriousnessCriterion::OtherMedicallyImportant);
    }

    let is_serious = !criteria_met.is_empty();

    // Primary criterion is the most severe one met
    let primary_criterion = criteria_met.iter().max_by_key(|c| c.priority()).copied();

    // Confidence is slightly lower for "other medically important" (subjective)
    let confidence = if criteria_met.is_empty() {
        1.0
    } else if criteria_met
        .iter()
        .any(|c| *c == SeriousnessCriterion::OtherMedicallyImportant)
        && criteria_met.len() == 1
    {
        0.85
    } else {
        1.0
    };

    let regulatory_impact = determine_regulatory_impact(&criteria_met);

    SeriousnessResult {
        is_serious,
        criteria_met,
        primary_criterion,
        confidence,
        regulatory_impact,
    }
}

/// Determine regulatory reporting requirements
fn determine_regulatory_impact(criteria: &[SeriousnessCriterion]) -> RegulatoryImpact {
    if criteria.is_empty() {
        return RegulatoryImpact::default();
    }

    // Death or life-threatening = fastest reporting
    let has_fatal_or_life_threatening = criteria.iter().any(|c| {
        matches!(
            c,
            SeriousnessCriterion::Death | SeriousnessCriterion::LifeThreatening
        )
    });

    let reporting_deadline = if has_fatal_or_life_threatening {
        "7-15 calendar days (death/life-threatening)".to_string()
    } else {
        "15 calendar days (other serious)".to_string()
    };

    RegulatoryImpact {
        requires_expedited: true,
        reporting_deadline: Some(reporting_deadline),
        regulatory_category: RegulatoryCategory::Serious,
    }
}

/// Generate human-readable rationale for seriousness determination
#[must_use]
pub fn generate_seriousness_rationale(
    result: &SeriousnessResult,
    input: &SeriousnessInput,
) -> String {
    let mut lines = Vec::new();

    if result.is_serious {
        lines.push("This adverse event is classified as **SERIOUS** (ICH E2A).".to_string());
        lines.push(String::new());
        lines.push("Criteria met:".to_string());

        for criterion in &result.criteria_met {
            match criterion {
                SeriousnessCriterion::Death => {
                    lines.push(
                        "• **DEATH**: The adverse event resulted in the death of the patient."
                            .to_string(),
                    );
                }
                SeriousnessCriterion::LifeThreatening => {
                    lines.push("• **LIFE-THREATENING**: The patient was at immediate risk of death at the time of the event.".to_string());
                }
                SeriousnessCriterion::Hospitalization => {
                    let hosp_type = input
                        .hospitalization_type
                        .map(|t| match t {
                            HospitalizationType::Initial => "initial admission",
                            HospitalizationType::Prolonged => "prolonged stay",
                            HospitalizationType::Both => "initial admission and prolonged stay",
                        })
                        .unwrap_or("unspecified");
                    lines.push(format!(
                        "• **HOSPITALIZATION**: Required inpatient hospitalization ({hosp_type})."
                    ));
                }
                SeriousnessCriterion::Disability => {
                    lines.push("• **DISABILITY**: Resulted in persistent or significant disability/incapacity.".to_string());
                }
                SeriousnessCriterion::CongenitalAnomaly => {
                    lines.push("• **CONGENITAL ANOMALY**: The event is a congenital anomaly or birth defect.".to_string());
                }
                SeriousnessCriterion::OtherMedicallyImportant => {
                    let justification = input
                        .medical_justification
                        .as_deref()
                        .unwrap_or("No justification provided");
                    lines.push(format!("• **OTHER MEDICALLY IMPORTANT**: {justification}"));
                }
            }
        }

        if let Some(deadline) = &result.regulatory_impact.reporting_deadline {
            lines.push(String::new());
            lines.push(format!("Reporting deadline: {deadline}"));
        }
    } else {
        lines.push("This adverse event is classified as **NON-SERIOUS** (ICH E2A).".to_string());
        lines.push(String::new());
        lines.push("No ICH E2A seriousness criteria were met.".to_string());
        lines.push("Standard periodic reporting applies.".to_string());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_death_is_serious() {
        let input = SeriousnessInput {
            death: true,
            ..Default::default()
        };
        let result = assess_seriousness(&input);

        assert!(result.is_serious);
        assert_eq!(result.primary_criterion, Some(SeriousnessCriterion::Death));
        assert!(result.regulatory_impact.requires_expedited);
        assert!(
            result
                .regulatory_impact
                .reporting_deadline
                .as_ref()
                .map_or(false, |d| d.contains("7-15"))
        );
    }

    #[test]
    fn test_hospitalization_is_serious() {
        let input = SeriousnessInput {
            hospitalization: true,
            hospitalization_type: Some(HospitalizationType::Initial),
            ..Default::default()
        };
        let result = assess_seriousness(&input);

        assert!(result.is_serious);
        assert_eq!(
            result.primary_criterion,
            Some(SeriousnessCriterion::Hospitalization)
        );
        assert!(
            result
                .regulatory_impact
                .reporting_deadline
                .as_ref()
                .map_or(false, |d| d.contains("15"))
        );
    }

    #[test]
    fn test_multiple_criteria() {
        let input = SeriousnessInput {
            hospitalization: true,
            disability: true,
            ..Default::default()
        };
        let result = assess_seriousness(&input);

        assert!(result.is_serious);
        assert_eq!(result.criteria_met.len(), 2);
        // Disability has higher priority than hospitalization
        assert_eq!(
            result.primary_criterion,
            Some(SeriousnessCriterion::Disability)
        );
    }

    #[test]
    fn test_non_serious() {
        let input = SeriousnessInput::default();
        let result = assess_seriousness(&input);

        assert!(!result.is_serious);
        assert!(result.criteria_met.is_empty());
        assert!(result.primary_criterion.is_none());
        assert!(!result.regulatory_impact.requires_expedited);
    }

    #[test]
    fn test_other_medically_important_lower_confidence() {
        let input = SeriousnessInput {
            other_medically_important: true,
            medical_justification: Some("Required intervention".to_string()),
            ..Default::default()
        };
        let result = assess_seriousness(&input);

        assert!(result.is_serious);
        assert!((result.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rationale_generation() {
        let input = SeriousnessInput {
            hospitalization: true,
            hospitalization_type: Some(HospitalizationType::Prolonged),
            ..Default::default()
        };
        let result = assess_seriousness(&input);
        let rationale = generate_seriousness_rationale(&result, &input);

        assert!(rationale.contains("SERIOUS"));
        assert!(rationale.contains("HOSPITALIZATION"));
        assert!(rationale.contains("prolonged stay"));
    }
}
