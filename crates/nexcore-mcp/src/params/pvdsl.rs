//! PVDSL (Pharmacovigilance Domain-Specific Language) Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! PVDSL compilation, execution, and evaluation parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for compiling PVDSL source code
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvdslCompileParams {
    /// PVDSL source code to compile
    pub source: String,
}

/// Parameters for executing compiled PVDSL bytecode
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvdslExecuteParams {
    /// PVDSL source code to execute
    pub source: String,
    /// Optional variables to set before execution
    #[serde(default)]
    pub variables: std::collections::HashMap<String, serde_json::Value>,
}

/// Parameters for evaluating a PVDSL expression
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvdslEvalParams {
    /// PVDSL expression to evaluate
    pub expression: String,
}
