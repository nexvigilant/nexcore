//! Parameter structs for pharma company MCP tools.
//!
//! Four tools, all read-only:
//! - `pharma_company_profile`: Full company aggregate
//! - `pharma_signal_portfolio`: All safety signals across products
//! - `pharma_pipeline`: Pipeline candidates filtered by phase
//! - `pharma_boxed_warnings`: Products carrying FDA boxed warnings

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for `pharma_company_profile`.
///
/// Returns the full company aggregate (products, pipeline, safety communications)
/// for the named pharmaceutical company.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PharmaCompanyProfileParams {
    /// Company identifier. Case-insensitive. Accepted values:
    /// abbvie, astrazeneca, bms, gsk, jnj, lilly, merck,
    /// novartis, novo-nordisk, pfizer, roche, takeda.
    pub company: String,
}

/// Parameters for `pharma_signal_portfolio`.
///
/// Returns all safety signals flattened across every product for the company.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PharmaSignalPortfolioParams {
    /// Company identifier (same accepted values as `pharma_company_profile`).
    pub company: String,
}

/// Parameters for `pharma_pipeline`.
///
/// Returns pipeline candidates, optionally filtered to a single development phase.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PharmaPipelineParams {
    /// Company identifier (same accepted values as `pharma_company_profile`).
    pub company: String,
    /// Optional phase filter. Accepted values: phase1, phase2, phase3, filed, approved.
    /// Omit to return all phases.
    pub phase: Option<String>,
}

/// Parameters for `pharma_boxed_warnings`.
///
/// Returns products that carry an FDA boxed warning for the company.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PharmaBoxedWarningsParams {
    /// Company identifier (same accepted values as `pharma_company_profile`).
    pub company: String,
}
