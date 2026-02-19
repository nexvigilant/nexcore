//! Laboratory Parameters (Specimen Experiments)
//!
//! Concept experiments, comparison, reaction, and batch processing.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for lab_experiment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabExperimentParams {
    /// Name of the concept/word.
    pub name: Option<String>,
    /// Array of T1 primitive names or symbols.
    pub primitives: Vec<String>,
}

/// Parameters for lab_compare.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabCompareParams {
    /// Name of concept A.
    pub name_a: Option<String>,
    /// Primitives for concept A.
    pub primitives_a: Vec<String>,
    /// Name of concept B.
    pub name_b: Option<String>,
    /// Primitives for concept B.
    pub primitives_b: Vec<String>,
}

/// Parameters for lab_react.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabReactParams {
    /// Name of concept A.
    pub name_a: Option<String>,
    /// Primitives for concept A.
    pub primitives_a: Vec<String>,
    /// Name of concept B.
    pub name_b: Option<String>,
    /// Primitives for concept B.
    pub primitives_b: Vec<String>,
}

/// Parameters for lab_batch.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabBatchParams {
    /// Array of specimens to experiment on.
    pub specimens: Vec<LabBatchSpecimen>,
}

/// A single specimen in a batch experiment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabBatchSpecimen {
    /// Name of the concept/word.
    pub name: Option<String>,
    /// Array of T1 primitive names or symbols.
    pub primitives: Vec<String>,
}
