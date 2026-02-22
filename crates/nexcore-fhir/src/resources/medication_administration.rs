//! FHIR R4 MedicationAdministration resource.

use serde::{Deserialize, Serialize};

use crate::base::{CodeableConcept, Meta, Narrative, Quantity, Reference};

/// Dosage information for the administration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MedicationAdministrationDosage {
    /// Free text dosage instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Body site administered to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<CodeableConcept>,
    /// Path of substance into body (oral, IV, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<CodeableConcept>,
    /// Amount administered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dose: Option<Quantity>,
}

/// FHIR R4 MedicationAdministration — record of a medication being given.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MedicationAdministration {
    #[serde(default = "med_admin_resource_type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Narrative>,
    /// in-progress | not-done | on-hold | completed | entered-in-error | stopped | unknown
    pub status: String,
    /// Medication code or reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medication_codeable_concept: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medication_reference: Option<Reference>,
    /// Who received the medication.
    pub subject: Reference,
    /// Start and end time of administration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_date_time: Option<String>,
    /// Dosage details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosage: Option<MedicationAdministrationDosage>,
}

fn med_admin_resource_type() -> String {
    "MedicationAdministration".to_owned()
}
