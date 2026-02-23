//! FHIR AdverseEvent → NexVigilant signal detection adapter.
//!
//! Converts FHIR R4 AdverseEvent resources into signal detection inputs
//! compatible with nexcore-vigilance's disproportionality analysis.

use crate::base::{CodeableConcept, MEDDRA_SYSTEM};
use crate::resources::AdverseEvent;

/// Signal-relevant data extracted from a single FHIR AdverseEvent.
///
/// One `SignalInput` = one ICSR (Individual Case Safety Report).
/// Aggregate many of these to build contingency tables for PRR/ROR/IC/EBGM.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignalInput {
    /// Source FHIR resource ID.
    pub fhir_id: Option<String>,
    /// actual | potential.
    pub actuality: String,
    /// MedDRA preferred term for the adverse event.
    pub meddra_term: MeddraTerm,
    /// Suspect drug name.
    pub drug: DrugInfo,
    /// Severity / seriousness classification.
    pub severity: SeverityLevel,
    /// Outcome / resolution status.
    pub outcome: OutcomeStatus,
    /// When the event occurred (ISO 8601).
    pub event_date: Option<String>,
    /// When the event was recorded (ISO 8601).
    pub recorded_date: Option<String>,
}

/// MedDRA term extracted from AdverseEvent.event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeddraTerm {
    /// Human-readable preferred term.
    pub preferred_term: String,
    /// MedDRA numeric code.
    pub code: Option<String>,
    /// Whether resolved from MedDRA coding (vs. free text fallback).
    pub is_coded: bool,
}

/// Drug information from AdverseEvent.suspectEntity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DrugInfo {
    /// Normalized drug name.
    pub name: String,
    /// Causality assessment display (Naranjo / WHO-UMC).
    pub causality: Option<String>,
}

/// Severity tier for signal prioritization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeverityTier {
    Critical,
    Serious,
    Moderate,
    Mild,
}

/// Structured severity from FHIR seriousness + severity fields.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SeverityLevel {
    pub tier: SeverityTier,
    pub is_serious: bool,
}

/// Outcome status from AdverseEvent.outcome.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutcomeStatus {
    pub code: String,
    pub is_fatal: bool,
    pub is_resolved: bool,
}

/// Extract a `SignalInput` from a FHIR R4 `AdverseEvent`.
///
/// This is the primary conversion function for the FHIR→Signal pipeline.
#[must_use]
pub fn adverse_event_to_signal(ae: &AdverseEvent) -> SignalInput {
    let meddra_term = extract_meddra_term(ae.event.as_ref());
    let drug = extract_drug_info(ae);
    let severity = extract_severity(ae);
    let outcome = extract_outcome(ae.outcome.as_ref());

    SignalInput {
        fhir_id: ae.id.clone(),
        actuality: format!("{:?}", ae.actuality).to_lowercase(),
        meddra_term,
        drug,
        severity,
        outcome,
        event_date: ae.date.clone(),
        recorded_date: ae.recorded_date.clone(),
    }
}

/// Batch convert multiple AdverseEvents, collecting successes.
#[must_use]
pub fn adverse_events_to_signals(events: &[AdverseEvent]) -> Vec<SignalInput> {
    events.iter().map(adverse_event_to_signal).collect()
}

fn extract_meddra_term(event: Option<&CodeableConcept>) -> MeddraTerm {
    let Some(concept) = event else {
        return MeddraTerm {
            preferred_term: "Unknown".to_owned(),
            code: None,
            is_coded: false,
        };
    };

    // Try MedDRA system first
    if let Some(coding) = concept.coding_for_system(MEDDRA_SYSTEM) {
        return MeddraTerm {
            preferred_term: coding
                .display
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned()),
            code: coding.code.clone(),
            is_coded: true,
        };
    }

    // Fall back to first coding or text
    let display = concept.display_text().unwrap_or("Unknown");
    let code = concept.coding.first().and_then(|c| c.code.clone());

    MeddraTerm {
        preferred_term: display.to_owned(),
        code,
        is_coded: false,
    }
}

fn extract_drug_info(ae: &AdverseEvent) -> DrugInfo {
    let Some(entity) = ae.suspect_entity.first() else {
        return DrugInfo {
            name: "Unknown".to_owned(),
            causality: None,
        };
    };

    let name = entity
        .instance
        .display
        .clone()
        .unwrap_or_else(|| "Unknown drug".to_owned());

    let causality = entity.causality.first().and_then(|c| {
        c.assessment
            .as_ref()
            .and_then(|a| a.display_text().map(|s| s.to_owned()))
    });

    DrugInfo { name, causality }
}

fn extract_severity(ae: &AdverseEvent) -> SeverityLevel {
    let seriousness_text = ae
        .seriousness
        .as_ref()
        .and_then(|s| s.display_text())
        .unwrap_or("Non-serious");

    let is_serious = !seriousness_text.eq_ignore_ascii_case("non-serious");

    let tier = if seriousness_text.contains("Death") || seriousness_text.contains("LifeThreatening")
    {
        SeverityTier::Critical
    } else if is_serious {
        SeverityTier::Serious
    } else {
        // Check severity field for mild vs moderate
        let severity_text = ae
            .severity
            .as_ref()
            .and_then(|s| s.display_text())
            .unwrap_or("mild");
        if severity_text.eq_ignore_ascii_case("moderate") {
            SeverityTier::Moderate
        } else {
            SeverityTier::Mild
        }
    };

    SeverityLevel { tier, is_serious }
}

fn extract_outcome(outcome: Option<&CodeableConcept>) -> OutcomeStatus {
    let code = outcome
        .and_then(|o| o.display_text())
        .unwrap_or("unknown")
        .to_owned();

    let is_fatal = code.eq_ignore_ascii_case("fatal");
    let is_resolved = code.eq_ignore_ascii_case("resolved");

    OutcomeStatus {
        code,
        is_fatal,
        is_resolved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{Coding, Reference};
    use crate::resources::{AdverseEventActuality, AdverseEventSuspectEntity};

    fn make_test_ae() -> AdverseEvent {
        AdverseEvent {
            resource_type: "AdverseEvent".to_owned(),
            id: Some("ae-001".to_owned()),
            meta: None,
            text: None,
            actuality: AdverseEventActuality::Actual,
            category: vec![],
            event: Some(CodeableConcept {
                coding: vec![Coding {
                    system: Some(MEDDRA_SYSTEM.to_owned()),
                    version: None,
                    code: Some("10019211".to_owned()),
                    display: Some("Gastrointestinal haemorrhage".to_owned()),
                    user_selected: None,
                }],
                text: None,
            }),
            subject: Reference {
                reference: Some("Patient/p-001".to_owned()),
                type_: None,
                display: None,
            },
            encounter: None,
            date: Some("2026-01-15".to_owned()),
            detected: None,
            recorded_date: Some("2026-01-16".to_owned()),
            outcome: Some(CodeableConcept {
                coding: vec![Coding {
                    system: None,
                    version: None,
                    code: Some("resolved".to_owned()),
                    display: Some("resolved".to_owned()),
                    user_selected: None,
                }],
                text: None,
            }),
            seriousness: Some(CodeableConcept {
                coding: vec![Coding {
                    system: None,
                    version: None,
                    code: Some("Serious".to_owned()),
                    display: Some("Serious".to_owned()),
                    user_selected: None,
                }],
                text: None,
            }),
            severity: None,
            suspect_entity: vec![AdverseEventSuspectEntity {
                instance: Reference {
                    reference: Some("Medication/med-001".to_owned()),
                    type_: Some("Medication".to_owned()),
                    display: Some("Warfarin".to_owned()),
                },
                causality: vec![],
            }],
            recorder: None,
            contributing_factor: vec![],
        }
    }

    #[test]
    fn test_adverse_event_to_signal_extracts_meddra() {
        let ae = make_test_ae();
        let signal = adverse_event_to_signal(&ae);
        assert_eq!(
            signal.meddra_term.preferred_term,
            "Gastrointestinal haemorrhage"
        );
        assert_eq!(signal.meddra_term.code.as_deref(), Some("10019211"));
        assert!(signal.meddra_term.is_coded);
    }

    #[test]
    fn test_adverse_event_to_signal_extracts_drug() {
        let ae = make_test_ae();
        let signal = adverse_event_to_signal(&ae);
        assert_eq!(signal.drug.name, "Warfarin");
    }

    #[test]
    fn test_adverse_event_to_signal_extracts_severity() {
        let ae = make_test_ae();
        let signal = adverse_event_to_signal(&ae);
        assert_eq!(signal.severity.tier, SeverityTier::Serious);
        assert!(signal.severity.is_serious);
    }

    #[test]
    fn test_adverse_event_to_signal_extracts_outcome() {
        let ae = make_test_ae();
        let signal = adverse_event_to_signal(&ae);
        assert!(signal.outcome.is_resolved);
        assert!(!signal.outcome.is_fatal);
    }

    #[test]
    fn test_batch_conversion() {
        let events = vec![make_test_ae(), make_test_ae()];
        let signals = adverse_events_to_signals(&events);
        assert_eq!(signals.len(), 2);
    }
}
