// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Function Executor
//!
//! Loads Prima source, finds functions, executes with parameters.
//!
//! ## Tier: T2-C (μ + σ + → + ∂)

use crate::serialize::{SerializeError, json_to_prima, prima_to_json};
use nexcore_error::Error;
use prima::value::{Value, ValueData};
use prima_mcp::{FunctionSig, extract_functions};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Executor errors.
#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    #[error("Parameter missing: {0}")]
    ParameterMissing(String),
    #[error("Serialization error: {0}")]
    Serialize(#[from] SerializeError),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

/// Prima function executor.
///
/// ## Tier: T2-C (μ + σ + →)
pub struct Executor {
    /// Source code.
    source: String,
    /// Extracted function signatures.
    functions: Vec<FunctionSig>,
}

impl Executor {
    /// Create executor from Prima source.
    pub fn new(source: &str) -> Self {
        let functions = extract_functions(source);
        Self {
            source: source.to_string(),
            functions,
        }
    }

    /// List available functions.
    pub fn list_functions(&self) -> &[FunctionSig] {
        &self.functions
    }

    /// Find function by name.
    pub fn find_function(&self, name: &str) -> Option<&FunctionSig> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Execute a function with JSON parameters.
    ///
    /// ## Tier: T2-C (μ + → + ∂)
    pub fn execute(
        &self,
        function_name: &str,
        params: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, ExecutorError> {
        // Find function signature
        let func = self
            .find_function(function_name)
            .ok_or_else(|| ExecutorError::FunctionNotFound(function_name.to_string()))?;

        // Build argument list in order
        let mut args: Vec<Value> = Vec::with_capacity(func.params.len());
        for param in &func.params {
            let json_val = params
                .get(&param.name)
                .ok_or_else(|| ExecutorError::ParameterMissing(param.name.clone()))?;
            let prima_val = json_to_prima(json_val)?;
            args.push(prima_val);
        }

        // Build function call expression
        let args_source: Vec<String> = args.iter().map(value_to_source).collect();
        let call_source = format!(
            "{}\n{}({})",
            self.source,
            function_name,
            args_source.join(", ")
        );

        // Use Prima's eval() function
        let result =
            prima::eval(&call_source).map_err(|e| ExecutorError::Runtime(format!("{:?}", e)))?;

        Ok(prima_to_json(&result))
    }
}

/// Convert Prima value back to source representation for calling.
fn value_to_source(value: &Value) -> String {
    match &value.data {
        ValueData::Void => "∅".to_string(),
        ValueData::Bool(b) => b.to_string(),
        ValueData::Int(i) => i.to_string(),
        ValueData::Float(f) => f.to_string(),
        ValueData::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
        ValueData::Sequence(seq) => {
            let items: Vec<String> = seq.iter().map(value_to_source).collect();
            format!("σ[{}]", items.join(", "))
        }
        ValueData::Mapping(map) => {
            let pairs: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, value_to_source(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        ValueData::Symbol(s) => format!(":{}", s),
        ValueData::Quoted(_) => "'...".to_string(),
        ValueData::Function(_) => "<function>".to_string(),
        ValueData::Builtin(name) => name.clone(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_executor_list_functions() {
        let source = r#"
μ add(a: N, b: N) → N { a + b }
μ double(x: N) → N { x * 2 }
"#;
        let executor = Executor::new(source);
        assert_eq!(executor.list_functions().len(), 2);
    }

    #[test]
    fn test_executor_find_function() {
        let source = "μ greet(name: String) → String { name }";
        let executor = Executor::new(source);
        let func = executor.find_function("greet");
        assert!(func.is_some());
        assert_eq!(func.map(|f| f.name.as_str()), Some("greet"));
    }

    #[test]
    fn test_executor_execute_add() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let executor = Executor::new(source);

        let mut params = HashMap::new();
        params.insert("a".to_string(), json!(3));
        params.insert("b".to_string(), json!(4));

        let result = executor.execute("add", &params);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(json!(7)));
    }

    #[test]
    fn test_executor_execute_double() {
        let source = "μ double(x: N) → N { x * 2 }";
        let executor = Executor::new(source);

        let mut params = HashMap::new();
        params.insert("x".to_string(), json!(21));

        let result = executor.execute("double", &params);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(json!(42)));
    }

    #[test]
    fn test_executor_function_not_found() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let executor = Executor::new(source);

        let params = HashMap::new();
        let result = executor.execute("nonexistent", &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_executor_missing_param() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let executor = Executor::new(source);

        let mut params = HashMap::new();
        params.insert("a".to_string(), json!(3));
        // Missing "b"

        let result = executor.execute("add", &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_value_to_source_int() {
        assert_eq!(value_to_source(&Value::int(42)), "42");
    }

    #[test]
    fn test_value_to_source_string() {
        assert_eq!(
            value_to_source(&Value::string("hello".to_string())),
            "\"hello\""
        );
    }

    #[test]
    fn test_value_to_source_seq() {
        let seq = Value::sequence(vec![Value::int(1), Value::int(2), Value::int(3)]);
        assert_eq!(value_to_source(&seq), "σ[1, 2, 3]");
    }
}
