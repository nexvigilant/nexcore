//! Params for model delegation/routing tools.

use serde::Deserialize;

/// Route a task to the optimal model.
#[derive(Debug, Deserialize)]
pub struct ModelRouteParams {
    /// Task description
    pub task: String,
    /// Task characteristics
    pub complexity: Option<String>, // "trivial", "simple", "moderate", "complex", "expert"
    /// Error tolerance: "none", "low", "medium", "high"
    pub error_tolerance: Option<String>,
    /// Latency sensitivity: "real-time", "interactive", "batch"
    pub latency: Option<String>,
    /// Token budget (input + output estimate)
    pub token_budget: Option<u64>,
}

/// Compare models for a specific task.
#[derive(Debug, Deserialize)]
pub struct ModelCompareParams {
    /// Task description
    pub task: String,
    /// Models to compare (default: all known)
    pub models: Option<Vec<String>>,
}

/// List available models and their capabilities.
#[derive(Debug, Deserialize)]
pub struct ModelListParams {}
