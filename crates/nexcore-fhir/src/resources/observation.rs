//! FHIR R4 Observation resource.

use serde::{Deserialize, Serialize};

use crate::base::{CodeableConcept, Meta, Narrative, Quantity, Reference};

/// FHIR R4 Observation — measurements and simple assertions about a patient.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    #[serde(default = "observation_resource_type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Narrative>,
    /// registered | preliminary | final | amended | corrected | cancelled | entered-in-error | unknown
    pub status: String,
    /// Classification of type of observation (vital-signs, laboratory, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category: Vec<CodeableConcept>,
    /// What was observed (LOINC code).
    pub code: CodeableConcept,
    /// Who/what this is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>,
    /// Clinically relevant time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_date_time: Option<String>,
    /// Actual result — quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_quantity: Option<Quantity>,
    /// Actual result — codeable concept.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_codeable_concept: Option<CodeableConcept>,
    /// Actual result — string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_string: Option<String>,
}

fn observation_resource_type() -> String {
    "Observation".to_owned()
}
