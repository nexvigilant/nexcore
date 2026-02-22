// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! QSAR granular per-endpoint MCP tool parameters.
//!
//! These param structs support the 4 granular QSAR tools that expose individual
//! toxicity endpoints (`mutagenicity`, `hepatotoxicity`, `cardiotoxicity`) and
//! standalone applicability domain assessment.

use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;

/// Per-endpoint mutagenicity prediction from a SMILES string.
///
/// Internally parses the SMILES, computes descriptors, scans structural alerts,
/// and returns an Ames-surrogate mutagenicity prediction.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemPredictMutagenicityParams {
    /// SMILES string of the compound to evaluate
    pub smiles: String,
    /// Optional structural alert count override. When omitted the tool
    /// auto-scans for ICH M7 alerts internally.
    #[serde(default)]
    pub structural_alert_count: Option<usize>,
}

/// Per-endpoint hepatotoxicity (DILI) prediction from a SMILES string.
///
/// Internally parses the SMILES, computes descriptors, scans for reactive
/// metabolite alerts, and returns a drug-induced liver injury prediction.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemPredictHepatotoxicityParams {
    /// SMILES string of the compound to evaluate
    pub smiles: String,
    /// Optional reactive metabolite alert count override. When omitted the tool
    /// auto-counts hepatotoxicity-category alerts from the ICH M7 library.
    #[serde(default)]
    pub reactive_alert_count: Option<usize>,
}

/// Per-endpoint hERG-channel cardiotoxicity prediction from a SMILES string.
///
/// Internally parses the SMILES, computes descriptors, and returns a
/// cardiotoxicity (QT prolongation) prediction based on physicochemical rules.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemPredictCardiotoxicityParams {
    /// SMILES string of the compound to evaluate
    pub smiles: String,
}

/// Standalone applicability domain assessment from a SMILES string.
///
/// Checks whether the compound falls within the descriptor bounding-box of the
/// QSAR training set (Lipinski/Veber/Ertl boundaries) without running full
/// toxicity predictions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemAssessDomainParams {
    /// SMILES string of the compound to evaluate
    pub smiles: String,
}
