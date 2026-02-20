//! OpenFDA Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Typed parameter structs for OpenFDA API tools (drug events, labels, recalls, NDC).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for openfda_drug_events — search drug adverse events via openFDA.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OpenFdaDrugEventsParams {
    /// Drug name to search (brand or generic, e.g. "aspirin")
    pub drug_name: String,
    /// Optional MedDRA reaction term filter (e.g. "HEADACHE")
    #[serde(default)]
    pub reaction: Option<String>,
    /// Filter to serious events only
    #[serde(default)]
    pub serious: Option<bool>,
    /// Max results to return (1-100, default 25)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Parameters for openfda_drug_labels — search structured product labels.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OpenFdaDrugLabelsParams {
    /// Drug name to search (brand or generic)
    pub drug_name: String,
    /// Specific label section to extract (e.g. "warnings", "indications_and_usage", "boxed_warning")
    #[serde(default)]
    pub section: Option<String>,
    /// Max results to return (1-100, default 5)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Parameters for openfda_recalls — search drug enforcement/recall actions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OpenFdaRecallsParams {
    /// Search term (drug name, recalling firm, or free text)
    pub search_term: String,
    /// Filter by recall classification ("Class I", "Class II", "Class III")
    #[serde(default)]
    pub classification: Option<String>,
    /// Filter by recall status ("Ongoing", "Completed", "Terminated")
    #[serde(default)]
    pub status: Option<String>,
    /// Max results to return (1-100, default 10)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Parameters for openfda_ndc — search National Drug Code directory.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OpenFdaNdcParams {
    /// Drug name to search (brand or generic)
    pub drug_name: String,
    /// Max results to return (1-100, default 10)
    #[serde(default)]
    pub limit: Option<u32>,
}
