//! PVDSL (Pharmacovigilance Domain-Specific Language) API endpoints
//!
//! Compile, execute, and evaluate PVDSL scripts for signal detection workflows.

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use super::common::ApiError;

/// PVDSL compile request
#[derive(Debug, Deserialize, ToSchema)]
pub struct PvdslCompileRequest {
    /// PVDSL source code to compile
    pub source: String,
}

/// PVDSL compile response
#[derive(Debug, Serialize, ToSchema)]
pub struct PvdslCompileResponse {
    /// Compilation succeeded
    pub success: bool,
    /// Number of bytecode instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<usize>,
    /// Number of constants
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constants: Option<usize>,
    /// Number of names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<usize>,
    /// Error message if compilation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// PVDSL execute request
#[derive(Debug, Deserialize, ToSchema)]
pub struct PvdslExecuteRequest {
    /// PVDSL source code to execute
    pub source: String,
    /// Optional variables to inject into the execution context
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
}

/// PVDSL execute response
#[derive(Debug, Serialize, ToSchema)]
pub struct PvdslExecuteResponse {
    /// Execution succeeded
    pub success: bool,
    /// Execution result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error message if execution failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// PVDSL eval request (single expression)
#[derive(Debug, Deserialize, ToSchema)]
pub struct PvdslEvalRequest {
    /// PVDSL expression to evaluate
    pub expression: String,
}

/// PVDSL function info
#[derive(Debug, Serialize, ToSchema)]
pub struct PvdslFunctionInfo {
    /// Function name
    pub name: String,
    /// Function signature/description
    pub signature: String,
}

/// PVDSL function category
#[derive(Debug, Serialize, ToSchema)]
pub struct PvdslFunctionCategory {
    /// Category name
    pub category: String,
    /// Functions in this category
    pub functions: Vec<PvdslFunctionInfo>,
}

/// PVDSL functions list response
#[derive(Debug, Serialize, ToSchema)]
pub struct PvdslFunctionsResponse {
    /// Function categories with their functions
    pub categories: Vec<PvdslFunctionCategory>,
}

/// PVDSL router
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/compile", post(compile))
        .route("/execute", post(execute))
        .route("/eval", post(eval))
        .route("/functions", get(functions))
}

/// Compile PVDSL source code (validate syntax, return bytecode stats)
#[utoipa::path(
    post,
    path = "/api/v1/pvdsl/compile",
    tag = "pvdsl",
    request_body = PvdslCompileRequest,
    responses(
        (status = 200, description = "Compilation result", body = PvdslCompileResponse),
        (status = 400, description = "Invalid request", body = super::common::ApiError)
    )
)]
pub async fn compile(
    Json(req): Json<PvdslCompileRequest>,
) -> Result<Json<PvdslCompileResponse>, ApiError> {
    use nexcore_vigilance::pvdsl::PvdslEngine;

    let engine = PvdslEngine::new();

    match engine.compile(&req.source) {
        Ok(program) => Ok(Json(PvdslCompileResponse {
            success: true,
            instructions: Some(program.instructions.len()),
            constants: Some(program.constants.len()),
            names: Some(program.names.len()),
            error: None,
        })),
        Err(e) => Ok(Json(PvdslCompileResponse {
            success: false,
            instructions: None,
            constants: None,
            names: None,
            error: Some(e.to_string()),
        })),
    }
}

/// Execute PVDSL source code with optional variables
#[utoipa::path(
    post,
    path = "/api/v1/pvdsl/execute",
    tag = "pvdsl",
    request_body = PvdslExecuteRequest,
    responses(
        (status = 200, description = "Execution result", body = PvdslExecuteResponse),
        (status = 400, description = "Invalid request", body = super::common::ApiError)
    )
)]
pub async fn execute(
    Json(req): Json<PvdslExecuteRequest>,
) -> Result<Json<PvdslExecuteResponse>, ApiError> {
    use nexcore_vigilance::pvdsl::PvdslEngine;

    let mut engine = PvdslEngine::new();

    // Set variables from request
    for (key, value) in &req.variables {
        let runtime_value = json_to_runtime(value);
        engine.set_variable(&key, runtime_value);
    }

    match engine.eval(&req.source) {
        Ok(result) => {
            let result_json = match result {
                Some(ref val) => runtime_to_json(val),
                None => serde_json::Value::Null,
            };
            Ok(Json(PvdslExecuteResponse {
                success: true,
                result: Some(result_json),
                error: None,
            }))
        }
        Err(e) => Ok(Json(PvdslExecuteResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        })),
    }
}

/// Evaluate a single PVDSL expression
#[utoipa::path(
    post,
    path = "/api/v1/pvdsl/eval",
    tag = "pvdsl",
    request_body = PvdslEvalRequest,
    responses(
        (status = 200, description = "Evaluation result", body = PvdslExecuteResponse),
        (status = 400, description = "Invalid request", body = super::common::ApiError)
    )
)]
pub async fn eval(
    Json(req): Json<PvdslEvalRequest>,
) -> Result<Json<PvdslExecuteResponse>, ApiError> {
    use nexcore_vigilance::pvdsl::PvdslEngine;

    let mut engine = PvdslEngine::new();

    // Wrap expression in return statement if needed
    let source = if req.expression.starts_with("return ") {
        req.expression.clone()
    } else {
        format!("return {}", req.expression)
    };

    match engine.eval(&source) {
        Ok(result) => {
            let result_json = match result {
                Some(ref val) => runtime_to_json(val),
                None => serde_json::Value::Null,
            };
            Ok(Json(PvdslExecuteResponse {
                success: true,
                result: Some(result_json),
                error: None,
            }))
        }
        Err(e) => Ok(Json(PvdslExecuteResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        })),
    }
}

/// List all available PVDSL functions
#[utoipa::path(
    get,
    path = "/api/v1/pvdsl/functions",
    tag = "pvdsl",
    responses(
        (status = 200, description = "List of available functions", body = PvdslFunctionsResponse)
    )
)]
pub async fn functions() -> Json<PvdslFunctionsResponse> {
    let categories = vec![
        PvdslFunctionCategory {
            category: "signal".to_string(),
            functions: vec![
                PvdslFunctionInfo { name: "prr".to_string(), signature: "PRR(a, b, c, d) - Proportional Reporting Ratio".to_string() },
                PvdslFunctionInfo { name: "ror".to_string(), signature: "ROR(a, b, c, d) - Reporting Odds Ratio".to_string() },
                PvdslFunctionInfo { name: "ic".to_string(), signature: "IC(a, b, c, d) - Information Component".to_string() },
                PvdslFunctionInfo { name: "ebgm".to_string(), signature: "EBGM(a, b, c, d) - Empirical Bayes Geometric Mean".to_string() },
                PvdslFunctionInfo { name: "chi_square".to_string(), signature: "Chi-square(a, b, c, d) - Chi-squared test".to_string() },
                PvdslFunctionInfo { name: "fisher".to_string(), signature: "Fisher(a, b, c, d) - Fisher exact test".to_string() },
                PvdslFunctionInfo { name: "sprt".to_string(), signature: "SPRT(observed, expected, null_rr, alt_rr) - Sequential Probability Ratio Test".to_string() },
                PvdslFunctionInfo { name: "maxsprt".to_string(), signature: "MaxSPRT(observed, expected) - Maximized SPRT".to_string() },
                PvdslFunctionInfo { name: "cusum".to_string(), signature: "CuSum(values, baseline, k, h) - Cumulative Sum control chart".to_string() },
                PvdslFunctionInfo { name: "mgps".to_string(), signature: "MGPS(a, b, c, d) - Multi-item Gamma Poisson Shrinker (returns Dict)".to_string() },
            ],
        },
        PvdslFunctionCategory {
            category: "causality".to_string(),
            functions: vec![
                PvdslFunctionInfo { name: "naranjo".to_string(), signature: "Naranjo(temporal, dechallenge, rechallenge, alternatives, previous)".to_string() },
                PvdslFunctionInfo { name: "who_umc".to_string(), signature: "WHO-UMC(temporal, dechallenge, rechallenge, alternatives, plausibility)".to_string() },
                PvdslFunctionInfo { name: "rucam".to_string(), signature: "RUCAM(time_to_onset, time_to_resolution, alt_causes, rechall, prev_info)".to_string() },
            ],
        },
        PvdslFunctionCategory {
            category: "meddra".to_string(),
            functions: vec![
                PvdslFunctionInfo { name: "levenshtein".to_string(), signature: "Levenshtein(str1, str2) - Edit distance".to_string() },
                PvdslFunctionInfo { name: "similarity".to_string(), signature: "Similarity(str1, str2) - Normalized similarity (0-1)".to_string() },
            ],
        },
        PvdslFunctionCategory {
            category: "math".to_string(),
            functions: vec![
                PvdslFunctionInfo { name: "abs".to_string(), signature: "abs(x)".to_string() },
                PvdslFunctionInfo { name: "sqrt".to_string(), signature: "sqrt(x)".to_string() },
                PvdslFunctionInfo { name: "pow".to_string(), signature: "pow(base, exp)".to_string() },
                PvdslFunctionInfo { name: "log".to_string(), signature: "log(x) - base 10".to_string() },
                PvdslFunctionInfo { name: "ln".to_string(), signature: "ln(x) - natural log".to_string() },
                PvdslFunctionInfo { name: "exp".to_string(), signature: "exp(x)".to_string() },
                PvdslFunctionInfo { name: "min".to_string(), signature: "min(a, b)".to_string() },
                PvdslFunctionInfo { name: "max".to_string(), signature: "max(a, b)".to_string() },
                PvdslFunctionInfo { name: "floor".to_string(), signature: "floor(x)".to_string() },
                PvdslFunctionInfo { name: "ceil".to_string(), signature: "ceil(x)".to_string() },
                PvdslFunctionInfo { name: "round".to_string(), signature: "round(x)".to_string() },
            ],
        },
        PvdslFunctionCategory {
            category: "risk".to_string(),
            functions: vec![
                PvdslFunctionInfo { name: "sar".to_string(), signature: "SAR(rates, confidence) - Value at Risk".to_string() },
                PvdslFunctionInfo { name: "es".to_string(), signature: "ES(rates, confidence) - Expected Shortfall".to_string() },
                PvdslFunctionInfo { name: "monte_carlo".to_string(), signature: "MonteCarlo(mean, std, simulations)".to_string() },
            ],
        },
        PvdslFunctionCategory {
            category: "date".to_string(),
            functions: vec![
                PvdslFunctionInfo { name: "now".to_string(), signature: "now() - Unix timestamp".to_string() },
                PvdslFunctionInfo { name: "diff_days".to_string(), signature: "diff_days(t1, t2) - Days between timestamps".to_string() },
            ],
        },
        PvdslFunctionCategory {
            category: "classify".to_string(),
            functions: vec![
                PvdslFunctionInfo { name: "hartwig_siegel".to_string(), signature: "HartwigSiegel(severity, outcome, hospitalization)".to_string() },
            ],
        },
    ];

    Json(PvdslFunctionsResponse { categories })
}

// Helper: Convert JSON value to PVDSL RuntimeValue
fn json_to_runtime(value: &serde_json::Value) -> nexcore_vigilance::pvdsl::RuntimeValue {
    use nexcore_vigilance::pvdsl::RuntimeValue;

    match value {
        serde_json::Value::Null => RuntimeValue::Null,
        serde_json::Value::Bool(b) => RuntimeValue::Boolean(*b),
        serde_json::Value::Number(n) => RuntimeValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => RuntimeValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            RuntimeValue::List(arr.iter().map(json_to_runtime).collect())
        }
        serde_json::Value::Object(obj) => {
            let map = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_runtime(v)))
                .collect();
            RuntimeValue::Dict(map)
        }
    }
}

// Helper: Convert PVDSL RuntimeValue to JSON
fn runtime_to_json(value: &nexcore_vigilance::pvdsl::RuntimeValue) -> serde_json::Value {
    use nexcore_vigilance::pvdsl::RuntimeValue;

    match value {
        RuntimeValue::Null => serde_json::Value::Null,
        RuntimeValue::Boolean(b) => serde_json::Value::Bool(*b),
        RuntimeValue::Number(n) => serde_json::Value::Number(
            serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        RuntimeValue::String(s) => serde_json::Value::String(s.clone()),
        RuntimeValue::List(list) => {
            serde_json::Value::Array(list.iter().map(runtime_to_json).collect())
        }
        RuntimeValue::Dict(dict) => {
            let map: serde_json::Map<String, serde_json::Value> = dict
                .iter()
                .map(|(k, v)| (k.clone(), runtime_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}
