//! Primitive Trace Parameters
//!
//! Tracing the T1/T2 primitives of a concept.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for primitive_trace.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveTraceParams {
    /// Name of the concept being traced
    pub concept: Option<String>,
    /// Array of T1 primitive names or symbols
    pub primitives: Vec<String>,
}
