//! Immunity & Antipattern Parameters
//! Tier: T2-C (Cross-domain composite)
//!
//! Behavioral and code-level immunity via antipattern detection.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for scanning code content for antipatterns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityScanParams {
    /// The code content to scan for antipatterns.
    pub content: String,
    /// Optional file path for context-aware pattern matching.
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Parameters for scanning error output for known patterns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityScanErrorsParams {
    /// The stderr/error output to scan.
    pub stderr: String,
}

/// Parameters for getting a specific antibody by ID.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityGetParams {
    /// The antibody ID.
    pub id: String,
}

/// Parameters for listing antibodies with optional filters.
#[derive(Debug, Deserialize, JsonSchema, Default)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityListParams {
    /// Filter by threat type: "PAMP" or "DAMP".
    #[serde(default)]
    pub threat_type: Option<String>,
    /// Filter by minimum severity: "low", "medium", "high", "critical".
    #[serde(default)]
    pub min_severity: Option<String>,
}

/// Parameters for proposing a new antibody from an observed error.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityProposeParams {
    /// The error message or pattern that triggered the issue.
    pub error_pattern: String,
    /// The fix that was applied to resolve the issue.
    pub fix_applied: String,
    /// Context about where this occurred.
    #[serde(default)]
    pub context: Option<String>,
    /// Suggested severity: "low", "medium", "high", "critical".
    #[serde(default)]
    pub severity: Option<String>,
}
