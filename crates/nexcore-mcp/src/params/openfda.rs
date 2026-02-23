//! Parameter structs for OpenFDA MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Search drug adverse events from FAERS.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaDrugEventsParams {
    /// Search query (drug name, reaction, etc.)
    pub search: String,
    /// Max results to return (1-1000, default 10)
    pub limit: Option<u32>,
    /// Number of records to skip for pagination
    pub skip: Option<u32>,
    /// Sort order (e.g. "receiptdate:desc")
    pub sort: Option<String>,
}

/// Search drug product labels (SPL).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaDrugLabelsParams {
    /// Search query (drug name)
    pub search: String,
    /// Max results (1-1000, default 10)
    pub limit: Option<u32>,
}

/// Search drug recall enforcement actions.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaDrugRecallsParams {
    /// Search query
    pub search: String,
    /// Max results (1-1000, default 10)
    pub limit: Option<u32>,
}

/// Search National Drug Code directory.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaDrugNdcParams {
    /// Drug name to search
    pub name: String,
    /// Max results (1-1000, default 10)
    pub limit: Option<u32>,
}

/// Search Drugs@FDA applications (NDA/BLA/ANDA).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaDrugsAtFdaParams {
    /// Search query (drug name, application number)
    pub search: String,
    /// Max results (1-1000, default 10)
    pub limit: Option<u32>,
}

/// Search medical device adverse event reports.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaDeviceEventsParams {
    /// Search query (device name, event type)
    pub search: String,
    /// Max results (1-1000, default 10)
    pub limit: Option<u32>,
}

/// Search device recall enforcement actions.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaDeviceRecallsParams {
    /// Search query
    pub search: String,
    /// Max results (1-1000, default 10)
    pub limit: Option<u32>,
}

/// Search food recall enforcement actions.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaFoodRecallsParams {
    /// Search query (reason, firm name)
    pub search: String,
    /// Max results (1-1000, default 10)
    pub limit: Option<u32>,
}

/// Search food adverse event reports (CAERS).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaFoodEventsParams {
    /// Search query (product name, outcome)
    pub search: String,
    /// Max results (1-1000, default 10)
    pub limit: Option<u32>,
}

/// Search FDA substance registry.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaSubstancesParams {
    /// Search query (substance name, UNII, CAS)
    pub search: String,
    /// Max results (1-1000, default 10)
    pub limit: Option<u32>,
}

/// Fan-out search across all major OpenFDA endpoints.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenfdaFanOutParams {
    /// Search term to query across all endpoints
    pub term: String,
    /// Max results per endpoint (1-100, default 5)
    pub limit: Option<u32>,
}
