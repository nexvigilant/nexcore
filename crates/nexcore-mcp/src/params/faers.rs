//! FAERS Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! FDA Adverse Event Reporting System (OpenFDA) parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for FAERS search
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSearchParams {
    /// Drug name to search (generic or brand)
    #[serde(default)]
    pub drug_name: Option<String>,
    /// Adverse reaction to search (MedDRA PT)
    #[serde(default)]
    pub reaction: Option<String>,
    /// Filter to serious events only
    #[serde(default)]
    pub serious: Option<bool>,
    /// Max results (1-100)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for FAERS drug events
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersDrugEventsParams {
    /// Drug name to analyze
    pub drug_name: String,
    /// Number of top events to return (default: 20)
    #[serde(default)]
    pub top_n: Option<usize>,
    /// Minimum disproportionality threshold
    pub min_dispro: f64,
}

/// Parameters for FAERS signal check and disproportionality
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSignalParams {
    /// Drug name
    pub drug_name: String,
    /// Target event term (MedDRA PT)
    pub event_name: String,
}

// ============================================================================
// FAERS A79 — Reporter-Weighted Disproportionality
// ============================================================================

/// Parameters for Algorithm A79: Reporter-Weighted Disproportionality.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersReporterWeightedParams {
    /// Array of cases with reporter qualification
    pub cases: Vec<FaersReporterCase>,
    /// Minimum raw case count
    #[serde(default)]
    pub min_cases: Option<u32>,
    /// Diversity threshold
    #[serde(default)]
    pub diversity_threshold: Option<f64>,
}

/// A single case for reporter-weighted analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersReporterCase {
    /// Drug name
    pub drug: String,
    /// Event name
    pub event: String,
    /// Reporter qualification code
    pub qualification_code: String,
}

// ============================================================================
// FAERS A81 — Geographic Signal Divergence
// ============================================================================

/// Parameters for Algorithm A81: Geographic Signal Divergence.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersGeographicDivergenceParams {
    /// Array of cases with country data
    pub cases: Vec<FaersGeographicCase>,
    /// Minimum total cases
    #[serde(default)]
    pub min_cases: Option<u32>,
    /// Minimum countries required
    #[serde(default)]
    pub min_countries: Option<usize>,
    /// Divergence ratio threshold
    #[serde(default)]
    pub divergence_threshold: Option<f64>,
    /// P-value threshold
    #[serde(default)]
    pub p_value_threshold: Option<f64>,
    /// Minimum cases per country
    #[serde(default)]
    pub min_country_cases: Option<u32>,
}

/// A single case for geographic analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersGeographicCase {
    /// Drug name
    pub drug: String,
    /// Event name
    pub event: String,
    /// Occurrence country (ISO 2-letter code)
    pub country: String,
}

// Re-export ETL and analytics types for qualified access (params::faers::FaersEtlRunParams)
pub use super::faers_analytics::{
    FaersOutcomeCase, FaersOutcomeConditionedParams, FaersPolypharmacyCase, FaersPolypharmacyDrug,
    FaersPolypharmacyParams, FaersSeriousnessCascadeParams, FaersSeriousnessCase,
    FaersSignalVelocityParams, FaersStandardPrr, FaersTemporalCase,
};
pub use super::faers_etl::*;

/// Parameters for FAERS drug comparison
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersCompareDrugsParams {
    /// First drug name
    pub drug1: String,
    /// Second drug name
    pub drug2: String,
    /// Number of events per drug (default: 15)
    #[serde(
        default,
        deserialize_with = "crate::params::serde_lenient::deserialize_option_usize_lenient"
    )]
    pub top_n: Option<usize>,
}
