//! Compliance & Regulatory Assessment Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! SAM.gov exclusion checks, SEC EDGAR lookups, and ICH control assessments.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for SAM.gov exclusion check
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceCheckExclusionParams {
    /// Unique Entity Identifier (UEI)
    #[serde(default)]
    pub uei: Option<String>,
    /// CAGE Code
    #[serde(default)]
    pub cage_code: Option<String>,
    /// Entity name for fuzzy search
    #[serde(default)]
    pub entity_name: Option<String>,
}

/// Control input for compliance assessment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ControlInput {
    /// Control identifier (e.g., "ICH-E2A-1")
    pub id: String,
    /// Control title
    pub title: String,
    /// Control description
    #[serde(default)]
    pub description: Option<String>,
    /// Source catalog
    #[serde(default)]
    pub catalog: Option<String>,
    /// Implementation status: "implemented", "partial", "not_implemented", "na"
    pub status: String,
}

/// Finding input for compliance assessment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FindingInput {
    /// Related control ID
    pub control_id: String,
    /// Severity: "critical", "high", "medium", "low", "info"
    pub severity: String,
    /// Finding title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Recommended remediation
    #[serde(default)]
    pub remediation: Option<String>,
}

/// Parameters for compliance assessment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceAssessParams {
    /// Assessment identifier
    pub assessment_id: String,
    /// Controls to assess
    pub controls: Vec<ControlInput>,
    /// Findings to record
    #[serde(default)]
    pub findings: Vec<FindingInput>,
}

/// Parameters for ICH control catalog retrieval
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceCatalogParams {
    /// Optional filter by guideline
    #[serde(default)]
    pub guideline_filter: Option<String>,
}

/// Parameters for SEC EDGAR company filings lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceSecFilingsParams {
    /// Company CIK (Central Index Key)
    pub cik: String,
    /// Optional form type filter
    #[serde(default)]
    pub form_filter: Option<String>,
    /// Maximum filings to return
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for SEC EDGAR pharma company lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceSecPharmaParams {
    /// Pharma company name: pfizer, jnj, merck, abbvie, bms, lilly, amgen, gilead, regeneron, moderna
    pub company: String,
}
