//! FHIR R4 Medication resource.

use serde::{Deserialize, Serialize};

use crate::base::{CodeableConcept, Meta, Narrative, Quantity, Reference};

/// Ingredient of a medication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MedicationIngredient {
    /// The ingredient (substance or medication).
    pub item_codeable_concept: Option<CodeableConcept>,
    pub item_reference: Option<Reference>,
    /// Is this ingredient active?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// Quantity of ingredient present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<MedicationIngredientStrength>,
}

/// Strength ratio for an ingredient.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MedicationIngredientStrength {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numerator: Option<Quantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denominator: Option<Quantity>,
}

/// FHIR R4 Medication — definition of a medication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Medication {
    #[serde(default = "medication_resource_type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Narrative>,
    /// Codes that identify this medication (RxNorm, NDC, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<CodeableConcept>,
    /// active | inactive | entered-in-error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Manufacturer of the medication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<Reference>,
    /// Dose form (tablet, capsule, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<CodeableConcept>,
    /// Active or inactive ingredients.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ingredient: Vec<MedicationIngredient>,
}

fn medication_resource_type() -> String {
    "Medication".to_owned()
}
