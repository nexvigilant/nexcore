// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Chemivigilance MCP tool parameters (15 typed param structs).

use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;

/// Parse a SMILES string into a molecular graph.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemParseSmilesParams {
    /// SMILES string to parse
    pub smiles: String,
}

/// Calculate molecular descriptors (MW, LogP, TPSA, HBA, HBD, etc.).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemDescriptorParams {
    /// SMILES string
    pub smiles: String,
}

/// Generate Morgan/ECFP circular fingerprint.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemFingerprintParams {
    /// SMILES string
    pub smiles: String,
    /// Fingerprint radius (default 2 for ECFP4)
    pub radius: Option<usize>,
    /// Number of bits (default 2048)
    pub nbits: Option<usize>,
}

/// Compute Tanimoto or Dice similarity between two molecules.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemSimilarityParams {
    /// SMILES of first molecule
    pub smiles_a: String,
    /// SMILES of second molecule
    pub smiles_b: String,
    /// Similarity metric: "tanimoto" (default) or "dice"
    pub metric: Option<String>,
}

/// Scan a molecule for ICH M7 structural alerts.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemStructuralAlertsParams {
    /// SMILES string to scan for alerts
    pub smiles: String,
}

/// QSAR toxicity prediction (mutagenicity, hepatotoxicity, cardiotoxicity).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemPredictToxicityParams {
    /// SMILES string
    pub smiles: String,
}

/// Phase I/II metabolite prediction.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemPredictMetabolitesParams {
    /// SMILES string
    pub smiles: String,
}

/// Degradation product prediction.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemPredictDegradantsParams {
    /// SMILES string
    pub smiles: String,
}

/// Generate full chemivigilance SafetyBrief.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemSafetyBriefParams {
    /// SMILES string
    pub smiles: String,
}

/// VF2 substructure match.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemSubstructureParams {
    /// SMILES of the target molecule
    pub molecule: String,
    /// SMILES of the substructure pattern
    pub pattern: String,
}

/// Regulatory watchlist screening.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemWatchlistParams {
    /// SMILES string
    pub smiles: String,
}

/// Query the ICH M7 alert library.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemAlertLibraryParams {
    /// Optional category filter (e.g. "mutagenicity", "hepatotoxicity")
    pub category: Option<String>,
}

/// SSSR ring perception.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemRingScanParams {
    /// SMILES string
    pub smiles: String,
}

/// Hückel aromaticity detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemAromaticityParams {
    /// SMILES string
    pub smiles: String,
}

/// Hill-notation molecular formula.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemMolecularFormulaParams {
    /// SMILES string
    pub smiles: String,
}
