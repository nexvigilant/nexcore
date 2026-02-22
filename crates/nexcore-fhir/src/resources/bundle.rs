//! FHIR R4 Bundle resource.
//!
//! Container for a collection of resources — used for batch operations,
//! search results, and transaction processing.

use serde::{Deserialize, Serialize};

use crate::base::Meta;

/// FHIR R4 Bundle entry — one resource within a Bundle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BundleEntry {
    /// URL for the entry (absolute or relative).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_url: Option<String>,
    /// The resource content (stored as raw JSON value for polymorphism).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<serde_json::Value>,
}

/// FHIR R4 Bundle — a container for a collection of resources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    #[serde(default = "bundle_resource_type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    /// document | message | transaction | transaction-response | batch | batch-response |
    /// history | searchset | collection
    pub r#type: String,
    /// Total number of matches (for searchset bundles).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
    /// Entry in the bundle.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry: Vec<BundleEntry>,
}

fn bundle_resource_type() -> String {
    "Bundle".to_owned()
}
