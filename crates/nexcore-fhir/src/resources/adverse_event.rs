//! FHIR R4 AdverseEvent resource.
//!
//! The core PV resource — represents an untoward medical occurrence.
//! Spec: <https://www.hl7.org/fhir/R4/adverseevent.html>

use serde::{Deserialize, Serialize};

use crate::base::{CodeableConcept, Meta, Narrative, Reference};

/// FHIR R4 AdverseEvent actuality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdverseEventActuality {
    /// The adverse event actually happened.
    Actual,
    /// A potential adverse event.
    Potential,
}

/// Causality assessment for a suspect entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdverseEventCausality {
    /// Assessment of likelihood (Naranjo, WHO-UMC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessment: Option<CodeableConcept>,
    /// Description of product relatedness.
    #[serde(rename = "productRelatedness", skip_serializing_if = "Option::is_none")]
    pub product_relatedness: Option<String>,
    /// Who authored the assessment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Reference>,
    /// Method of assessment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<CodeableConcept>,
}

/// Suspect entity — the drug or substance suspected of causing the event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdverseEventSuspectEntity {
    /// Refers to the specific entity (Substance, Medication, etc.).
    pub instance: Reference,
    /// Causality information for this entity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causality: Vec<AdverseEventCausality>,
}

/// FHIR R4 AdverseEvent — an untoward medical occurrence in a patient.
///
/// This is the primary pharmacovigilance resource in FHIR.
/// Maps to ICH E2B(R3) Individual Case Safety Report (ICSR).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdverseEvent {
    /// Resource type discriminator — always "AdverseEvent".
    #[serde(default = "adverse_event_resource_type")]
    pub resource_type: String,
    /// Logical id of this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Metadata about the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    /// Text summary (narrative).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Narrative>,
    /// actual | potential.
    pub actuality: AdverseEventActuality,
    /// product-problem | product-quality | product-use-error | ...
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category: Vec<CodeableConcept>,
    /// The specific event that occurred (MedDRA coded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<CodeableConcept>,
    /// Subject of the event (Patient).
    pub subject: Reference,
    /// Encounter during which event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encounter: Option<Reference>,
    /// When the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// When the event was detected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected: Option<String>,
    /// When the event was recorded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_date: Option<String>,
    /// Effect on the subject — resolved | recovering | ongoing | fatal | unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<CodeableConcept>,
    /// Seriousness of the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seriousness: Option<CodeableConcept>,
    /// Severity of the event (mild | moderate | severe).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<CodeableConcept>,
    /// The suspected cause(s).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suspect_entity: Vec<AdverseEventSuspectEntity>,
    /// Who recorded the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorder: Option<Reference>,
    /// Contributing factors.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributing_factor: Vec<Reference>,
}

fn adverse_event_resource_type() -> String {
    "AdverseEvent".to_owned()
}
