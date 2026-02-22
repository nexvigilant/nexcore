//! FHIR R4 base data types.
//!
//! Focused subset of FHIR primitive and complex types used by PV resources.
//! Spec: <https://www.hl7.org/fhir/R4/datatypes.html>

use serde::{Deserialize, Serialize};

/// FHIR Coding — a reference to a code in a code system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Coding {
    /// Identity of the terminology system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Version of the system (if relevant).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Symbol in syntax defined by the system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Representation defined by the system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    /// If this coding was chosen directly by the user.
    #[serde(skip_serializing_if = "Option::is_none", rename = "userSelected")]
    pub user_selected: Option<bool>,
}

/// FHIR CodeableConcept — a concept that may be defined by one or more coding systems.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeableConcept {
    /// Code defined by a terminology system.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coding: Vec<Coding>,
    /// Plain text representation of the concept.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl CodeableConcept {
    /// Find the first coding matching a given system URI.
    #[must_use]
    pub fn coding_for_system(&self, system: &str) -> Option<&Coding> {
        self.coding.iter().find(|c| {
            c.system.as_deref() == Some(system)
        })
    }

    /// Get display text: first coding display, or text fallback.
    #[must_use]
    pub fn display_text(&self) -> Option<&str> {
        self.coding
            .first()
            .and_then(|c| c.display.as_deref())
            .or(self.text.as_deref())
    }
}

/// FHIR Reference — a reference from one resource to another.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reference {
    /// Literal reference (relative, internal, or absolute URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// Type the reference refers to (e.g. "Patient").
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub type_: Option<String>,
    /// Text alternative for the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

impl Reference {
    /// Extract the resource type from the reference string (e.g. "Patient/123" → "Patient").
    #[must_use]
    pub fn resource_type(&self) -> Option<&str> {
        self.type_.as_deref().or_else(|| {
            self.reference.as_deref().and_then(|r| r.split('/').next())
        })
    }
}

/// FHIR Identifier — an identifier for a resource.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Identifier {
    /// usual | official | temp | secondary | old
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#use: Option<String>,
    /// The namespace for the identifier value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The value that is unique.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// FHIR Period — a time period defined by a start and end date/time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Period {
    /// Starting time with inclusive boundary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// End time with inclusive boundary (if not ongoing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

/// FHIR Quantity — a measured amount with unit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quantity {
    /// Numerical value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// Unit representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// System that defines coded unit form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Coded form of the unit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// FHIR HumanName — a human's name with parts and use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HumanName {
    /// usual | official | temp | nickname | anonymous | old | maiden
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#use: Option<String>,
    /// Text representation of the full name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Family name (surname).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// Given names (first name, middle name).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub given: Vec<String>,
}

impl HumanName {
    /// Format as "Family, Given" or fallback to text.
    #[must_use]
    pub fn formatted(&self) -> String {
        if let Some(family) = &self.family {
            if self.given.is_empty() {
                family.clone()
            } else {
                format!("{}, {}", family, self.given.join(" "))
            }
        } else {
            self.text.clone().unwrap_or_default()
        }
    }
}

/// FHIR ContactPoint — details of a contact point (phone, email, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactPoint {
    /// phone | fax | email | pager | url | sms | other
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The actual contact point details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// home | work | temp | old | mobile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#use: Option<String>,
}

/// FHIR Address.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Address {
    /// home | work | temp | old | billing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#use: Option<String>,
    /// Text representation of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Street name, number, direction & P.O. Box.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub line: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(rename = "postalCode", skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

/// FHIR Narrative — human-readable summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Narrative {
    /// generated | extensions | additional | empty
    pub status: String,
    /// Limited xhtml content.
    pub div: String,
}

/// FHIR Meta — metadata about a resource.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    /// Version specific identifier.
    #[serde(rename = "versionId", skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
    /// When the resource version last changed.
    #[serde(rename = "lastUpdated", skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    /// Profiles this resource claims to conform to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profile: Vec<String>,
}

/// MedDRA system URI for FHIR coding lookups.
pub const MEDDRA_SYSTEM: &str = "http://terminology.hl7.org/CodeSystem/meddra";

/// RxNorm system URI for drug coding lookups.
pub const RXNORM_SYSTEM: &str = "http://www.nlm.nih.gov/research/umls/rxnorm";

/// SNOMED CT system URI.
pub const SNOMED_SYSTEM: &str = "http://snomed.info/sct";
