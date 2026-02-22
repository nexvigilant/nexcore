//! FHIR R4 Patient resource.

use serde::{Deserialize, Serialize};

use crate::base::{
    Address, CodeableConcept, ContactPoint, HumanName, Identifier, Meta, Narrative, Reference,
};

/// FHIR R4 Patient — demographics and administrative information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    #[serde(default = "patient_resource_type")]
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
    /// male | female | other | unknown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    /// YYYY-MM-DD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub address: Vec<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marital_status: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managing_organization: Option<Reference>,
}

fn patient_resource_type() -> String {
    "Patient".to_owned()
}
