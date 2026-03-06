//! Diagram rendering parameters.

use schemars::JsonSchema;
use serde::Deserialize;

/// Render a DOT/Graphviz diagram to an image file.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiagramRenderParams {
    /// DOT language source string (e.g., "digraph G { a -> b }").
    pub source: String,
    /// Output format: "svg" (default), "png", or "pdf".
    pub format: Option<String>,
    /// Layout engine: "dot" (default), "neato", "circo", "fdp", or "twopi".
    pub engine: Option<String>,
}
