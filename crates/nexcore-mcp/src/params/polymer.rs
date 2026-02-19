//! Params for hook pipeline (polymer) composition tools.

use serde::Deserialize;

/// Compose a hook pipeline from named hooks.
#[derive(Debug, Deserialize)]
pub struct PolymerComposeParams {
    /// Ordered list of hook names in the pipeline
    pub hooks: Vec<String>,
    /// Pipeline type: "linear" or "cyclic" (default: "linear")
    pub topology: Option<String>,
    /// Required success ratio per stage (0.0-1.0, default: 1.0)
    pub stoichiometry: Option<f64>,
}

/// Validate a pipeline definition.
#[derive(Debug, Deserialize)]
pub struct PolymerValidateParams {
    /// Ordered list of hook names
    pub hooks: Vec<String>,
    /// Check for event compatibility between stages
    pub check_events: Option<bool>,
}

/// Analyze a pipeline's theoretical properties.
#[derive(Debug, Deserialize)]
pub struct PolymerAnalyzeParams {
    /// Ordered list of hook names
    pub hooks: Vec<String>,
}
