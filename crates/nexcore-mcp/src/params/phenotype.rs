//! Params for phenotype (adversarial test generation) tools.

use serde::Deserialize;

/// Generate mutations of a JSON schema for adversarial testing.
#[derive(Debug, Deserialize)]
pub struct PhenotypeMutateParams {
    /// JSON string to mutate
    pub json_input: String,
    /// Specific mutation type (type_mismatch, add_field, remove_field, range_expand,
    /// length_change, array_resize, structure_swap). If omitted, applies all.
    #[serde(default)]
    pub mutation: Option<String>,
}

/// Verify schema compatibility between original and mutated JSON.
#[derive(Debug, Deserialize)]
pub struct PhenotypeVerifyParams {
    /// Original JSON
    pub original: String,
    /// Mutated JSON to verify against
    pub mutated: String,
    /// Drift threshold (0.0-1.0, default 0.5)
    #[serde(default)]
    pub threshold: Option<f64>,
}
