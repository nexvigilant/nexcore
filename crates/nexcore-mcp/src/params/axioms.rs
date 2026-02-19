//! PV Axioms Database Parameters (Regulatory Grounding)
//! Tier: T2 (Knowledge Retrieval)
//!
//! KSB lookups, regulation search, traceability, and dashboarding.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for KSB lookup.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsKsbLookupParams {
    /// Exact KSB ID.
    pub ksb_id: Option<String>,
    /// Filter by domain.
    pub domain_id: Option<String>,
    /// Filter by type.
    pub ksb_type: Option<String>,
    /// Search keyword.
    pub keyword: Option<String>,
    /// Max results.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for regulation search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsRegulationSearchParams {
    /// Search query.
    pub query: Option<String>,
    /// Jurisdiction filter.
    pub jurisdiction: Option<String>,
    /// Domain ID filter.
    pub domain_id: Option<String>,
    /// Max results.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for traceability chain query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsTraceabilityParams {
    /// Axiom ID filter.
    pub axiom_id: Option<String>,
    /// Source guideline filter.
    pub source_guideline: Option<String>,
    /// Primitive symbol filter.
    pub primitive: Option<String>,
}

/// Parameters for domain dashboard query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsDomainDashboardParams {
    /// Specific domain ID.
    pub domain_id: Option<String>,
}

/// Parameters for raw read-only SQL queries.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsQueryParams {
    /// SQL SELECT query.
    pub sql: String,
}
