//! OpenFDA drug endpoint functions.
//!
//! Wraps the five drug endpoints:
//! - `/drug/event.json` — FAERS adverse events
//! - `/drug/label.json` — structured product labels
//! - `/drug/enforcement.json` — drug recalls
//! - `/drug/ndc.json` — National Drug Code directory
//! - `/drug/drugsfda.json` — Drugs@FDA applications

use crate::client::{OpenFdaClient, QueryParams};
use crate::types::common::OpenFdaResponse;
use crate::types::drug::{DrugApplication, DrugEvent, DrugLabel, DrugNdc, DrugRecall};

/// OpenFDA endpoint path for drug adverse events.
pub const DRUG_EVENT_ENDPOINT: &str = "/drug/event.json";
/// OpenFDA endpoint path for structured product labels.
pub const DRUG_LABEL_ENDPOINT: &str = "/drug/label.json";
/// OpenFDA endpoint path for drug enforcement/recalls.
pub const DRUG_RECALL_ENDPOINT: &str = "/drug/enforcement.json";
/// OpenFDA endpoint path for the NDC directory.
pub const DRUG_NDC_ENDPOINT: &str = "/drug/ndc.json";
/// OpenFDA endpoint path for Drugs@FDA applications.
pub const DRUG_DRUGSFDA_ENDPOINT: &str = "/drug/drugsfda.json";

// =============================================================================
// Endpoint Functions
// =============================================================================

/// Fetch drug adverse event reports from FAERS.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_drug_events(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DrugEvent>, anyhow::Error> {
    client.fetch::<DrugEvent>(DRUG_EVENT_ENDPOINT, params).await
}

/// Fetch structured product label (SPL) records.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_drug_labels(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DrugLabel>, anyhow::Error> {
    client.fetch::<DrugLabel>(DRUG_LABEL_ENDPOINT, params).await
}

/// Fetch drug enforcement/recall records.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_drug_recalls(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DrugRecall>, anyhow::Error> {
    client
        .fetch::<DrugRecall>(DRUG_RECALL_ENDPOINT, params)
        .await
}

/// Fetch NDC directory product records.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_drug_ndc(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DrugNdc>, anyhow::Error> {
    client.fetch::<DrugNdc>(DRUG_NDC_ENDPOINT, params).await
}

/// Fetch Drugs@FDA NDA/BLA/ANDA application records.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_drugs_at_fda(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DrugApplication>, anyhow::Error> {
    client
        .fetch::<DrugApplication>(DRUG_DRUGSFDA_ENDPOINT, params)
        .await
}

// =============================================================================
// Query Builders
// =============================================================================

/// Build an event search string matching a drug by brand or generic name.
///
/// # Example
///
/// ```rust
/// use nexcore_openfda::endpoints::drug::drug_event_search_by_drug;
/// let query = drug_event_search_by_drug("aspirin");
/// assert!(query.contains("aspirin"));
/// ```
#[must_use]
pub fn drug_event_search_by_drug(drug_name: &str) -> String {
    format!(
        "(patient.drug.openfda.brand_name:\"{name}\" OR patient.drug.openfda.generic_name:\"{name}\")",
        name = drug_name
    )
}

/// Build an event search string filtering to a specific MedDRA reaction.
#[must_use]
pub fn drug_event_search_by_reaction(reaction_meddrapt: &str) -> String {
    format!(
        "patient.reaction.reactionmeddrapt:\"{}\"",
        reaction_meddrapt
    )
}

/// Build an event search string filtering to serious events only.
#[must_use]
pub fn drug_event_search_serious() -> &'static str {
    "serious:1"
}

/// Build a label search string for a drug name.
#[must_use]
pub fn drug_label_search_by_name(name: &str) -> String {
    format!(
        "(openfda.brand_name:\"{name}\" OR openfda.generic_name:\"{name}\")",
        name = name
    )
}

/// Build an NDC search string for a drug name.
#[must_use]
pub fn drug_ndc_search_by_name(name: &str) -> String {
    format!(
        "(brand_name:\"{name}\" OR generic_name:\"{name}\")",
        name = name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drug_event_search_contains_both_names() {
        let q = drug_event_search_by_drug("aspirin");
        assert!(q.contains("brand_name"));
        assert!(q.contains("generic_name"));
        assert!(q.contains("aspirin"));
    }

    #[test]
    fn drug_event_search_reaction() {
        let q = drug_event_search_by_reaction("HEADACHE");
        assert!(q.contains("reactionmeddrapt"));
        assert!(q.contains("HEADACHE"));
    }

    #[test]
    fn drug_label_search_by_name_contains_term() {
        let q = drug_label_search_by_name("metformin");
        assert!(q.contains("brand_name"));
        assert!(q.contains("metformin"));
    }
}
