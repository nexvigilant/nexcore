//! Parameter types for pharmacovigilance taxonomy MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Get the full taxonomy summary (T1/T2-P/T2-C/T3 counts).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PvTaxonomySummaryParams {}

/// Look up a T2-P primitive by name.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PvPrimitiveLookupParams {
    /// Primitive name (e.g., "threshold", "ratio", "reporting_rate", "harm", "severity").
    pub name: String,
}

/// Look up a T2-C composite by name.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PvCompositeLookupParams {
    /// Composite name (e.g., "signal", "contingency_table", "adverse_event",
    /// "benefit_risk_evaluation", "causality_assessment").
    pub name: String,
}

/// Look up a T3 concept by pillar and name.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PvConceptLookupParams {
    /// Pillar: "detection", "assessment", "understanding", "prevention",
    /// "regulatory", "infrastructure", "operations", "analytics",
    /// "safety_comms", "special_populations", "scope".
    pub pillar: String,
    /// Concept name within the pillar (e.g., "prr", "naranjo_algorithm", "rems").
    pub name: String,
}

/// Get Chomsky classification for a PV subsystem.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PvChomskyLookupParams {
    /// Subsystem name (e.g., "case_intake", "meddra_coding", "signal_management").
    pub subsystem: String,
}

/// Get WHO pillar complexity classification.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PvWhoPillarsParams {}

/// Look up cross-domain transfer confidence.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PvTransferLookupParams {
    /// Target domain: "clinical_trials", "regulatory_affairs", "epidemiology", "health_economics".
    pub domain: String,
}

/// Get the full transfer confidence matrix.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PvTransferMatrixParams {}

/// Get all T1 Lex Primitiva symbols used in grounding.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PvLexSymbolsParams {}
