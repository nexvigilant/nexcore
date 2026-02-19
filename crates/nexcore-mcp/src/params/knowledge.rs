//! Knowledge & Guidelines Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Guidelines search (ICH, CIOMS, EMA), glossary, and principle lookups.
//! KSB article access (628 PV articles across 15 domains).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ============================================================================
// KSB (Knowledge-Skills-Behaviors) Article Parameters
// ============================================================================

/// Parameters for getting a specific KSB article by ID
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KsbGetParams {
    /// Article ID (e.g., "D01-001")
    pub article_id: String,
}

/// Parameters for searching KSB articles
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KsbSearchParams {
    /// Search query
    pub query: String,
    /// Filter by domain (e.g., "D01", "D02")
    #[serde(default)]
    pub domain: Option<String>,
    /// Maximum results (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for KSB statistics
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KsbStatsParams {}

/// Parameters for guidelines search
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuidelinesSearchParams {
    /// Search query (matches ID, title, keywords, description)
    pub query: String,
    /// Filter by source: "ich", "cioms", or "ema"
    #[serde(default)]
    pub source: Option<String>,
    /// Filter by category within source
    #[serde(default)]
    pub category: Option<String>,
    /// Maximum results to return (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for guidelines get
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuidelinesGetParams {
    /// Guideline ID (e.g., "E2B", "CIOMS-I", "GVP-Module-VI")
    pub id: String,
}

/// Parameters for guidelines URL lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuidelinesUrlParams {
    /// Guideline ID
    pub id: String,
}

// ============================================================================
// FDA Guidance Document Parameters
// ============================================================================

/// Parameters for FDA guidance search
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaGuidanceSearchParams {
    /// Search query (matches title, topics, products, centers)
    pub query: String,
    /// Filter by FDA center: "CDER", "CBER", "CDRH", "CFSAN", "CVM", "CTP", "ORA"
    #[serde(default)]
    pub center: Option<String>,
    /// Filter by product area: "Drugs", "Biologics", "Medical Devices", "Food & Beverages"
    #[serde(default)]
    pub product: Option<String>,
    /// Filter by status: "Draft" or "Final"
    #[serde(default)]
    pub status: Option<String>,
    /// Maximum results to return (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for FDA guidance get (by slug or partial title)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaGuidanceGetParams {
    /// Document slug or partial title
    pub id: String,
}

/// Parameters for FDA guidance URL lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaGuidanceUrlParams {
    /// Document slug
    pub id: String,
}

/// Parameters for FDA guidance status filter
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaGuidanceStatusParams {
    /// "Draft" or "Final"
    pub status: String,
    /// Only show documents open for public comment
    #[serde(default)]
    pub open_for_comment: Option<bool>,
}

// Re-export ICH types for qualified access (params::knowledge::IchLookupParams)
pub use super::ich::*;

// ============================================================================
// Primitive Scanner Parameters
// ============================================================================

/// Parameters for scanning a domain for primitives.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveScanParams {
    /// Domain name to analyze.
    pub domain: String,
    /// Source file paths or glob patterns.
    #[serde(default)]
    pub sources: Vec<String>,
}

/// A term with definition for batch testing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TermForTest {
    /// The term being tested.
    pub term: String,
    /// Natural language definition.
    pub definition: String,
    /// Domain terms found in definition
    #[serde(default)]
    pub domain_terms: Vec<String>,
    /// If true, use deep recurrence scanning
    #[serde(default)]
    pub deep_scan: bool,
    /// External grounding concepts
    #[serde(default)]
    pub external_grounding: Vec<String>,
    /// Number of domains where term appears.
    pub domain_count: Option<usize>,
}

/// Parameters for batch testing terms.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveBatchTestParams {
    /// Terms to test.
    pub terms: Vec<TermForTest>,
}

/// Parameters for vocab skill lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VocabSkillLookupParams {
    /// Vocabulary shorthand (e.g., "build-doctrine", "ctvp-validated")
    pub vocab: String,
}

/// Parameters for primitive skill lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveSkillLookupParams {
    /// Primitive name (e.g., "sequence", "mapping", "state")
    pub primitive: String,
}

/// Parameters for skill chain lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillChainLookupParams {
    /// Chain name or trigger phrase
    pub query: String,
}
