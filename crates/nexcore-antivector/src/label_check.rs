//! Label check: queries whether an anti-vector is already deployed in the drug label.
//!
//! Before prescribing a new anti-vector, check if the countermeasure already exists
//! in the current labeling. If nausea is already listed with dose titration guidance,
//! the question shifts from "what to do" to "is the existing measure sufficient?"

use crate::types::{AntiVectorClass, RiskMinimizationMeasure};
use serde::{Deserialize, Serialize};

/// Result of checking whether an anti-vector is already deployed in labeling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelCheckResult {
    /// Drug name
    pub drug: String,
    /// Adverse event
    pub event: String,
    /// Whether the event appears in the Adverse Reactions section
    pub event_in_adr_section: bool,
    /// Whether the event appears in Warnings/Precautions
    pub event_in_warnings: bool,
    /// Whether the event appears in Boxed Warning
    pub event_in_boxed_warning: bool,
    /// Whether dose modification guidance exists
    pub has_dose_guidance: bool,
    /// Whether monitoring requirements exist
    pub has_monitoring: bool,
    /// Whether a contraindication exists for this event
    pub has_contraindication: bool,
    /// Whether a REMS is in place
    pub has_rems: bool,
    /// Whether a medication guide exists
    pub has_medication_guide: bool,
    /// Deployed anti-vector measures already in the label
    pub deployed_measures: Vec<RiskMinimizationMeasure>,
    /// Overall deployment status
    pub status: LabelDeploymentStatus,
    /// Recommendation: what additional anti-vector is needed (if any)
    pub recommendation: String,
}

/// Whether the anti-vector is already deployed in the label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LabelDeploymentStatus {
    /// Event not in label at all — anti-vector fully needed
    NotDeployed,
    /// Event acknowledged but no countermeasure in place
    PartiallyDeployed,
    /// Event acknowledged with active countermeasures
    Deployed,
    /// Event in boxed warning with full risk minimization
    FullyDeployed,
}

/// Analyze label sections to determine what anti-vectors are already deployed.
///
/// Takes pre-fetched label section text (from DailyMed) and checks for
/// evidence of existing countermeasures.
#[must_use]
pub fn check_label_deployment(
    drug: &str,
    event: &str,
    adr_section: Option<&str>,
    warnings_section: Option<&str>,
    boxed_warning: Option<&str>,
) -> LabelCheckResult {
    let event_lower = event.to_lowercase();

    let event_in_adr = adr_section
        .map(|s| s.to_lowercase().contains(&event_lower))
        .unwrap_or(false);

    let event_in_warnings = warnings_section
        .map(|s| s.to_lowercase().contains(&event_lower))
        .unwrap_or(false);

    let event_in_boxed = boxed_warning
        .map(|s| s.to_lowercase().contains(&event_lower))
        .unwrap_or(false);

    // Only check for countermeasures if the event is actually mentioned
    let event_present = event_in_adr || event_in_warnings || event_in_boxed;

    let all_text = format!(
        "{} {} {}",
        adr_section.unwrap_or(""),
        warnings_section.unwrap_or(""),
        boxed_warning.unwrap_or("")
    )
    .to_lowercase();

    // Countermeasure keywords only count if the event itself is in the label
    let has_dose_guidance = event_present
        && (all_text.contains("dose adjustment")
            || all_text.contains("dose reduction")
            || all_text.contains("titrat")
            || all_text.contains("dose escalat")
            || all_text.contains("starting dose"));

    let has_monitoring = event_present
        && (all_text.contains("monitor")
            || all_text.contains("laboratory test")
            || all_text.contains("periodic"));

    let has_contraindication = event_present
        && (all_text.contains("contraindicated") || all_text.contains("contraindication"));

    let has_rems = event_present
        && (all_text.contains("rems") || all_text.contains("risk evaluation and mitigation"));

    let has_medication_guide = event_present && all_text.contains("medication guide");

    // Build deployed measures list
    let mut deployed = Vec::new();
    if has_dose_guidance {
        deployed.push(RiskMinimizationMeasure::DoseModification);
    }
    if has_monitoring {
        deployed.push(RiskMinimizationMeasure::RequiredMonitoring);
    }
    if has_contraindication {
        deployed.push(RiskMinimizationMeasure::Contraindication);
    }
    if has_rems {
        deployed.push(RiskMinimizationMeasure::Rems);
    }
    if has_medication_guide {
        deployed.push(RiskMinimizationMeasure::MedicationGuide);
    }
    if event_in_adr
        && !deployed
            .iter()
            .any(|m| matches!(m, RiskMinimizationMeasure::LabelUpdate))
    {
        deployed.push(RiskMinimizationMeasure::LabelUpdate);
    }

    // Determine status — LabelUpdate alone = acknowledged, not actively mitigated
    let has_active_measures = deployed
        .iter()
        .any(|m| !matches!(m, RiskMinimizationMeasure::LabelUpdate));

    let status = if event_in_boxed && has_active_measures {
        LabelDeploymentStatus::FullyDeployed
    } else if has_active_measures {
        LabelDeploymentStatus::Deployed
    } else if event_in_adr || event_in_warnings {
        LabelDeploymentStatus::PartiallyDeployed
    } else {
        LabelDeploymentStatus::NotDeployed
    };

    let recommendation = match status {
        LabelDeploymentStatus::NotDeployed => {
            format!(
                "{event} not in label. Full anti-vector needed: label update + appropriate risk minimization."
            )
        }
        LabelDeploymentStatus::PartiallyDeployed => {
            format!(
                "{event} acknowledged in label but no active countermeasures. Consider adding risk minimization measures."
            )
        }
        LabelDeploymentStatus::Deployed => {
            let deployed_str: Vec<String> = deployed.iter().map(|m| format!("{m:?}")).collect();
            format!(
                "{event} has deployed anti-vectors: {}. Assess sufficiency — if PRR still elevated, escalate.",
                deployed_str.join(", ")
            )
        }
        LabelDeploymentStatus::FullyDeployed => {
            format!(
                "{event} fully addressed with boxed warning + risk minimization. Monitor for residual signal — if PRR persists, the anti-vector may be insufficient."
            )
        }
    };

    LabelCheckResult {
        drug: drug.to_string(),
        event: event.to_string(),
        event_in_adr_section: event_in_adr,
        event_in_warnings: event_in_warnings,
        event_in_boxed_warning: event_in_boxed,
        has_dose_guidance,
        has_monitoring,
        has_contraindication,
        has_rems,
        has_medication_guide,
        deployed_measures: deployed,
        status,
        recommendation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_deployed_when_absent() {
        let result = check_label_deployment(
            "testdrug",
            "pancreatitis",
            Some("Common: headache, dizziness"),
            Some("Monitor liver function"),
            None,
        );
        assert_eq!(result.status, LabelDeploymentStatus::NotDeployed);
        assert!(!result.event_in_adr_section);
    }

    #[test]
    fn partially_deployed_when_listed_no_measures() {
        let result = check_label_deployment(
            "testdrug",
            "nausea",
            Some("Common adverse reactions: nausea, vomiting, headache"),
            Some("GI effects may occur"),
            None,
        );
        assert_eq!(result.status, LabelDeploymentStatus::PartiallyDeployed);
        assert!(result.event_in_adr_section);
    }

    #[test]
    fn deployed_when_dose_guidance_exists() {
        let result = check_label_deployment(
            "semaglutide",
            "nausea",
            Some("Common: nausea (44%), vomiting (24%)"),
            Some("Nausea is dose-dependent. Use dose escalation schedule: start at 0.25mg weekly."),
            None,
        );
        assert_eq!(result.status, LabelDeploymentStatus::Deployed);
        assert!(result.has_dose_guidance);
        assert!(
            result
                .deployed_measures
                .contains(&RiskMinimizationMeasure::DoseModification)
        );
    }

    #[test]
    fn fully_deployed_when_boxed_warning() {
        let result = check_label_deployment(
            "testdrug",
            "hepatotoxicity",
            Some("Serious: hepatotoxicity"),
            Some("Monitor liver function. Contraindicated in severe hepatic impairment."),
            Some("WARNING: HEPATOTOXICITY - Monitor ALT/AST. Medication Guide required."),
        );
        assert_eq!(result.status, LabelDeploymentStatus::FullyDeployed);
        assert!(result.event_in_boxed_warning);
        assert!(result.has_monitoring);
        assert!(result.has_contraindication);
    }
}
