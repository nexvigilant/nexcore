//! FHIR R4 Condition resource.

use serde::{Deserialize, Serialize};

use crate::base::{CodeableConcept, Meta, Narrative, Reference};

/// FHIR R4 Condition — a clinical condition, problem, or diagnosis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    #[serde(default = "condition_resource_type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Narrative>,
    /// active | recurrence | relapse | inactive | remission | resolved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinical_status: Option<CodeableConcept>,
    /// unconfirmed | provisional | differential | confirmed | refuted | entered-in-error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<CodeableConcept>,
    /// problem-list-item | encounter-diagnosis
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category: Vec<CodeableConcept>,
    /// Severity (mild | moderate | severe).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<CodeableConcept>,
    /// Identification of the condition (SNOMED, ICD-10, MedDRA).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<CodeableConcept>,
    /// Who has the condition.
    pub subject: Reference,
    /// Date of onset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onset_date_time: Option<String>,
    /// Date of resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abatement_date_time: Option<String>,
    /// Date record was first recorded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_date: Option<String>,
    /// Who recorded the condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorder: Option<Reference>,
}

fn condition_resource_type() -> String {
    "Condition".to_owned()
}
