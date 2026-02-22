//! FHIR R4 Practitioner resource.

use serde::{Deserialize, Serialize};

use crate::base::{Address, CodeableConcept, ContactPoint, HumanName, Identifier, Meta, Narrative};

/// FHIR R4 Practitioner — a person involved in healthcare.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Practitioner {
    #[serde(default = "practitioner_resource_type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Narrative>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identifier: Vec<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub name: Vec<HumanName>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub telecom: Vec<ContactPoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub address: Vec<Address>,
    /// male | female | other | unknown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    /// Qualifications.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub qualification: Vec<PractitionerQualification>,
}

/// A qualification obtained by the practitioner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PractitionerQualification {
    pub code: CodeableConcept,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<crate::base::Reference>,
}

fn practitioner_resource_type() -> String {
    "Practitioner".to_owned()
}
