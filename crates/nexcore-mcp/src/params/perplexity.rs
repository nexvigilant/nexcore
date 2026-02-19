//! Perplexity Search Parameters (AI-driven Research)
//! Tier: T2-C (Search-Grounded Mapping)
//!
//! Search, research, competitive intelligence, and regulatory search.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for Perplexity AI search query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PerplexitySearchParams {
    /// Search query.
    pub query: String,
    /// Model: "sonar", "sonar-pro", etc.
    #[serde(default)]
    pub model: Option<String>,
    /// Recency filter: "hour", "day", etc.
    #[serde(default)]
    pub recency: Option<String>,
    /// Domain filter.
    #[serde(default)]
    pub domains: Option<Vec<String>>,
}

/// Parameters for high-level research.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PerplexityResearchParams {
    /// Research query.
    pub query: String,
    /// Use case: "general", "competitive", "regulatory".
    pub use_case: String,
}

/// Parameters for competitive intelligence.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PerplexityCompetitiveParams {
    /// Query.
    pub query: String,
    /// Competitor domains.
    pub competitors: Vec<String>,
}

/// Parameters for regulatory search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PerplexityRegulatoryParams {
    /// Query.
    pub query: String,
    /// Recency filter.
    #[serde(default)]
    pub recency: Option<String>,
}
