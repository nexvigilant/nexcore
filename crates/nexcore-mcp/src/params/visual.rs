//! Visual primitives params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VisualShapeClassifyParams {
    /// Shape name: circle, triangle, line, rectangle, point, polygon
    pub shape: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VisualColorAnalyzeParams {
    /// Color as hex (#ff0000) or named (red, blue, green)
    pub color: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VisualShapeListParams {}
