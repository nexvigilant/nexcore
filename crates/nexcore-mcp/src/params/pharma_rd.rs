//! Pharma R&D taxonomy MCP tool parameters.
//!
//! Typed parameter structs for pharmaceutical R&D concept taxonomy,
//! cross-domain transfer analysis, Chomsky classification, and
//! pipeline stage queries.

use schemars::JsonSchema;
use serde::Deserialize;

/// Get the full taxonomy summary (concept counts by tier).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PharmaTaxonomySummaryParams {}

/// Look up transfer confidence for a pharma primitive to a target domain.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PharmaLookupTransferParams {
    /// Pharma primitive name (e.g. "BindingAffinity", "Absorption", "Toxicity").
    pub primitive: String,
    /// Target domain: "Biotechnology", "MedicalDevices", or "Agrochemical".
    pub domain: String,
}

/// Get the full transfer confidence matrix (all primitives × all domains).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PharmaTransferMatrixParams {}

/// Get the top N strongest cross-domain transfer corridors.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PharmaStrongestTransfersParams {
    /// Number of top results. Default: 10.
    pub top_n: Option<usize>,
}

/// Get the bottom N weakest cross-domain transfer corridors.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PharmaWeakestTransfersParams {
    /// Number of bottom results. Default: 10.
    pub bottom_n: Option<usize>,
}

/// Get Lex Primitiva symbol coverage across the R&D pipeline.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PharmaSymbolCoverageParams {}

/// Get info about a specific R&D pipeline stage.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PharmaPipelineStageParams {
    /// Pipeline stage name: "TargetIdentification", "LeadDiscovery",
    /// "LeadOptimization", "PreclinicalDevelopment", "Phase1",
    /// "Phase2", "Phase3", "RegulatoryReview", "PostMarket".
    pub stage: String,
}

/// Classify a set of generators into a Chomsky hierarchy level.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PharmaClassifyGeneratorsParams {
    /// List of generator names: "LexPrimitiva", "ProductionRule",
    /// "ContextSensitivity", "TuringCompleteness".
    pub generators: Vec<String>,
}
