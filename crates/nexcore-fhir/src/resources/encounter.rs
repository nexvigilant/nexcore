//! FHIR R4 Encounter resource.

use serde::{Deserialize, Serialize};

use crate::base::{CodeableConcept, Meta, Narrative, Period, Reference};

/// FHIR R4 Encounter — an interaction between a patient and healthcare provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    #[serde(default = "encounter_resource_type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Narrative>,
    /// planned | arrived | triaged | in-progress | onleave | finished | cancelled | entered-in-error | unknown
    pub status: String,
    /// Classification of encounter (AMB, EMER, IMP, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<CodeableConcept>,
    /// Specific type of encounter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub r#type: Vec<CodeableConcept>,
    /// The patient present at the encounter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>,
    /// The start and end time of the encounter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    /// Reason the encounter takes place.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_code: Vec<CodeableConcept>,
    /// The organization responsible for the encounter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_provider: Option<Reference>,
}

fn encounter_resource_type() -> String {
    "Encounter".to_owned()
}
