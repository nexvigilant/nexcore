//! DNA Parameters (DNA-based Computation)
//! Tier: T2-T3 (Nucleotide Logic)
//!
//! Encoding, decoding, evaluation, tiling, voxels, and drug profiling.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for DNA encoding.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaEncodeParams {
    /// Text to encode.
    pub text: String,
}

/// Parameters for DNA decoding.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaDecodeParams {
    /// Strand to decode.
    pub strand: String,
}

/// Parameters for expression evaluation on Codon VM.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaEvalParams {
    /// Source expression.
    pub expr: String,
}

/// Parameters for tile generation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaTileParams {
    /// Source expression.
    pub expr: String,
}

/// Parameters for voxel cube generation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaVoxelParams {
    /// Source expression.
    pub expr: String,
}

/// Parameters for PV signal detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaPvSignalParams {
    /// Drug name.
    pub drug: String,
    /// Adverse event name.
    pub event: String,
}

/// Parameters for drug profiling.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaProfileDrugParams {
    /// Drug name/word.
    pub name: String,
}

/// Parameters for assembly compilation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaCompileAsmParams {
    /// Source expression.
    pub source: String,
}
