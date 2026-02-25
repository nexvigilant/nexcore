//! FHIR R4 Healthcare Interoperability Types.
//!
//! HL7 FHIR R4 resource types for US Core compliance.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// FHIR Meta element for versioning and profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRMeta {
    #[serde(default = "default_version")]
    pub version_id: String,
    #[serde(default = "DateTime::now", rename = "lastUpdated")]
    pub last_updated: DateTime,
    #[serde(default)]
    pub profile: Vec<String>,
}

fn default_version() -> String {
    "1".to_string()
}

impl Default for FHIRMeta {
    fn default() -> Self {
        Self {
            version_id: default_version(),
            last_updated: DateTime::now(),
            profile: Vec::new(),
        }
    }
}

/// FHIR Coding element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRCoding {
    pub system: String,
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// FHIR CodeableConcept element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRCodeableConcept {
    pub coding: Vec<FHIRCoding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// FHIR Reference element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRReference {
    pub reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// FHIR Identifier element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRIdentifier {
    #[serde(default = "default_use")]
    pub r#use: String,
    pub system: String,
    pub value: String,
}

fn default_use() -> String {
    "usual".to_string()
}

/// FHIR Quantity element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRQuantity {
    pub value: f64,
    pub unit: String,
    #[serde(default = "default_unit_system")]
    pub system: String,
    pub code: String,
}

fn default_unit_system() -> String {
    "http://unitsofmeasure.org".to_string()
}

/// FHIR Period element.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FHIRPeriod {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime>,
}

/// FHIR Annotation element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRAnnotation {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "authorString"
    )]
    pub author_string: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "authorReference"
    )]
    pub author_reference: Option<FHIRReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<DateTime>,
    pub text: String,
}

/// FHIR R4 AllergyIntolerance Resource - US Core compliant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRAllergyIntolerance {
    #[serde(default = "allergy_resource_type", rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    #[serde(default)]
    pub meta: FHIRMeta,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "clinicalStatus"
    )]
    pub clinical_status: Option<FHIRCodeableConcept>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "verificationStatus"
    )]
    pub verification_status: Option<FHIRCodeableConcept>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default)]
    pub category: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criticality: Option<String>,
    pub code: FHIRCodeableConcept,
    pub patient: FHIRReference,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "recordedDate"
    )]
    pub recorded_date: Option<DateTime>,
    #[serde(default)]
    pub note: Vec<FHIRAnnotation>,
    #[serde(default)]
    pub reaction: Vec<serde_json::Value>,
}

fn allergy_resource_type() -> String {
    "AllergyIntolerance".to_string()
}

/// FHIR R4 Condition Resource - US Core compliant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRCondition {
    #[serde(default = "condition_resource_type", rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    #[serde(default)]
    pub meta: FHIRMeta,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "clinicalStatus"
    )]
    pub clinical_status: Option<FHIRCodeableConcept>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "verificationStatus"
    )]
    pub verification_status: Option<FHIRCodeableConcept>,
    pub category: Vec<FHIRCodeableConcept>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<FHIRCodeableConcept>,
    pub code: FHIRCodeableConcept,
    pub subject: FHIRReference,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "recordedDate"
    )]
    pub recorded_date: Option<DateTime>,
    #[serde(default)]
    pub note: Vec<FHIRAnnotation>,
}

fn condition_resource_type() -> String {
    "Condition".to_string()
}

/// FHIR R4 Procedure Resource - US Core compliant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRProcedure {
    #[serde(default = "procedure_resource_type", rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    #[serde(default)]
    pub meta: FHIRMeta,
    #[serde(default)]
    pub identifier: Vec<FHIRIdentifier>,
    pub status: String,
    pub code: FHIRCodeableConcept,
    pub subject: FHIRReference,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "performedDateTime"
    )]
    pub performed_date_time: Option<DateTime>,
    #[serde(default)]
    pub performer: Vec<serde_json::Value>,
    #[serde(default)]
    pub note: Vec<FHIRAnnotation>,
}

fn procedure_resource_type() -> String {
    "Procedure".to_string()
}

/// FHIR R4 DiagnosticReport Resource - US Core compliant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRDiagnosticReport {
    #[serde(default = "diagnostic_resource_type", rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    #[serde(default)]
    pub meta: FHIRMeta,
    #[serde(default)]
    pub identifier: Vec<FHIRIdentifier>,
    pub status: String,
    pub category: Vec<FHIRCodeableConcept>,
    pub code: FHIRCodeableConcept,
    pub subject: FHIRReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issued: Option<DateTime>,
    #[serde(default)]
    pub result: Vec<FHIRReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
}

fn diagnostic_resource_type() -> String {
    "DiagnosticReport".to_string()
}

/// FHIR R4 MedicationRequest Resource - US Core compliant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRMedicationRequest {
    #[serde(default = "medication_resource_type", rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    #[serde(default)]
    pub meta: FHIRMeta,
    #[serde(default)]
    pub identifier: Vec<FHIRIdentifier>,
    pub status: String,
    pub intent: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "medicationCodeableConcept"
    )]
    pub medication_codeable_concept: Option<FHIRCodeableConcept>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "medicationReference"
    )]
    pub medication_reference: Option<FHIRReference>,
    pub subject: FHIRReference,
    #[serde(rename = "authoredOn")]
    pub authored_on: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requester: Option<FHIRReference>,
    #[serde(default)]
    pub note: Vec<FHIRAnnotation>,
}

fn medication_resource_type() -> String {
    "MedicationRequest".to_string()
}

/// FHIR HumanName element.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FHIRHumanName {
    #[serde(default = "default_name_use")]
    pub r#use: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(default)]
    pub given: Vec<String>,
    #[serde(default)]
    pub prefix: Vec<String>,
    #[serde(default)]
    pub suffix: Vec<String>,
}

fn default_name_use() -> String {
    "official".to_string()
}

/// FHIR R4 Practitioner Resource - US Core compliant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FHIRPractitioner {
    #[serde(default = "practitioner_resource_type", rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    #[serde(default)]
    pub meta: FHIRMeta,
    pub identifier: Vec<FHIRIdentifier>,
    #[serde(default = "default_true")]
    pub active: bool,
    pub name: Vec<FHIRHumanName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "birthDate")]
    pub birth_date: Option<String>,
    #[serde(default)]
    pub qualification: Vec<serde_json::Value>,
}

fn practitioner_resource_type() -> String {
    "Practitioner".to_string()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fhir_coding() {
        let coding = FHIRCoding {
            system: "http://snomed.info/sct".to_string(),
            code: "123456".to_string(),
            display: Some("Test condition".to_string()),
        };

        assert_eq!(coding.code, "123456");
    }

    #[test]
    fn test_fhir_meta_default() {
        let meta = FHIRMeta::default();
        assert_eq!(meta.version_id, "1");
    }
}
