//! PVDSL (Pharmacovigilance Domain-Specific Language) tools
//!
//! Compile and execute PVDSL scripts for signal detection workflows.

use crate::params::{PvdslCompileParams, PvdslEvalParams, PvdslExecuteParams};
use nexcore_vigilance::pvdsl::{PvdslEngine, RuntimeValue};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Compile PVDSL source to bytecode (validates syntax)
pub fn pvdsl_compile(params: PvdslCompileParams) -> Result<CallToolResult, McpError> {
    let engine = PvdslEngine::new();

    match engine.compile(&params.source) {
        Ok(program) => {
            let json = json!({
                "success": true,
                "instructions": program.instructions.len(),
                "constants": program.constants.len(),
                "names": program.names.len(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "success": false,
                "error": e.to_string(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Execute PVDSL source code with optional variables
pub fn pvdsl_execute(params: PvdslExecuteParams) -> Result<CallToolResult, McpError> {
    let mut engine = PvdslEngine::new();

    // Set variables from params
    for (key, value) in &params.variables {
        let runtime_value = json_to_runtime(value);
        engine.set_variable(key, runtime_value);
    }

    match engine.eval(&params.source) {
        Ok(result) => {
            let result_json = match result {
                Some(ref val) => runtime_to_json(val),
                None => serde_json::Value::Null,
            };
            let json = json!({
                "success": true,
                "result": result_json,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "success": false,
                "error": e.to_string(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Evaluate a PVDSL expression and return the result
pub fn pvdsl_eval(params: PvdslEvalParams) -> Result<CallToolResult, McpError> {
    let mut engine = PvdslEngine::new();

    // Wrap expression in return statement if needed
    let source = if params.expression.starts_with("return ") {
        params.expression.clone()
    } else {
        format!("return {}", params.expression)
    };

    match engine.eval(&source) {
        Ok(result) => {
            let result_json = match result {
                Some(ref val) => runtime_to_json(val),
                None => serde_json::Value::Null,
            };
            let json = json!({
                "success": true,
                "result": result_json,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "success": false,
                "error": e.to_string(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// List available PVDSL functions
pub fn pvdsl_functions() -> Result<CallToolResult, McpError> {
    let functions = json!({
        "signal": {
            "prr": "PRR(a, b, c, d) - Proportional Reporting Ratio",
            "ror": "ROR(a, b, c, d) - Reporting Odds Ratio",
            "ic": "IC(a, b, c, d) - Information Component",
            "ebgm": "EBGM(a, b, c, d) - Empirical Bayes Geometric Mean",
            "chi_square": "Chi-square(a, b, c, d) - Chi-squared test",
            "fisher": "Fisher(a, b, c, d) - Fisher exact test",
            "sprt": "SPRT(observed, expected, null_rr, alt_rr) - Sequential Probability Ratio Test",
            "maxsprt": "MaxSPRT(observed, expected) - Maximized SPRT",
            "cusum": "CuSum(values, baseline, k, h) - Cumulative Sum control chart",
            "mgps": "MGPS(a, b, c, d) - Multi-item Gamma Poisson Shrinker (returns Dict)"
        },
        "causality": {
            "naranjo": "Naranjo(temporal, dechallenge, rechallenge, alternatives, previous)",
            "who_umc": "WHO-UMC(temporal, dechallenge, rechallenge, alternatives, plausibility)",
            "rucam": "RUCAM(time_to_onset, time_to_resolution, alt_causes, rechall, prev_info)"
        },
        "meddra": {
            "levenshtein": "Levenshtein(str1, str2) - Edit distance",
            "similarity": "Similarity(str1, str2) - Normalized similarity (0-1)"
        },
        "math": {
            "abs": "abs(x)", "sqrt": "sqrt(x)", "pow": "pow(base, exp)",
            "log": "log(x) - base 10", "ln": "ln(x) - natural log", "exp": "exp(x)",
            "min": "min(a, b)", "max": "max(a, b)",
            "floor": "floor(x)", "ceil": "ceil(x)", "round": "round(x)"
        },
        "risk": {
            "sar": "SAR(rates, confidence) - Value at Risk",
            "es": "ES(rates, confidence) - Expected Shortfall",
            "monte_carlo": "MonteCarlo(mean, std, simulations)"
        },
        "date": {
            "now": "now() - Unix timestamp",
            "diff_days": "diff_days(t1, t2) - Days between timestamps"
        },
        "classify": {
            "hartwig_siegel": "HartwigSiegel(severity, outcome, hospitalization)"
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        functions.to_string(),
    )]))
}

// Helper: Convert JSON value to PVDSL RuntimeValue
fn json_to_runtime(value: &serde_json::Value) -> RuntimeValue {
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
fn runtime_to_json(value: &RuntimeValue) -> serde_json::Value {
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
