//! Params for digestive system (data pipeline processing) tools.

use serde::Deserialize;

/// Process data through the full digestive pipeline.
#[derive(Debug, Deserialize)]
pub struct DigestiveProcessParams {
    /// Raw input data to digest
    pub input: String,
    /// Data kind hint (array, text, number, object, binary)
    #[serde(default)]
    pub kind: Option<String>,
}

/// Taste (quality assess) input data before processing.
#[derive(Debug, Deserialize)]
pub struct DigestiveTasteParams {
    /// Sample to taste
    pub sample: String,
}

/// Get digestive system health overview.
#[derive(Debug, Deserialize)]
pub struct DigestiveHealthParams {}
