// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # JSON Schema Generator
//!
//! Converts Prima types to JSON Schema for MCP tool definitions.
//!
//! ## Tier: T2-C (μ + κ + σ)
//!
//! ## Type Mappings
//!
//! | Prima | JSON Schema |
//! |-------|-------------|
//! | N | `{"type": "integer"}` |
//! | String | `{"type": "string"}` |
//! | Bool | `{"type": "boolean"}` |
//! | σ | `{"type": "array"}` |
//! | σ[T] | `{"type": "array", "items": T}` |
//! | ∅ | `{"type": "null"}` |

use crate::extract::{FunctionSig, Param, PrimaType};
use serde_json::{Value, json};

/// Convert a Prima type to JSON Schema.
///
/// ## Tier: T2-P (μ + κ)
#[must_use]
pub fn type_to_schema(ty: &PrimaType) -> Value {
    match ty {
        PrimaType::Int => json!({"type": "integer"}),
        PrimaType::Float => json!({"type": "number"}),
        PrimaType::String => json!({"type": "string"}),
        PrimaType::Bool => json!({"type": "boolean"}),
        PrimaType::Void => json!({"type": "null"}),
        PrimaType::Seq(None) => json!({"type": "array"}),
        PrimaType::Seq(Some(inner)) => {
            json!({
                "type": "array",
                "items": type_to_schema(inner)
            })
        }
        PrimaType::Unknown(name) => {
            json!({
                "type": "object",
                "description": format!("Unknown Prima type: {}", name)
            })
        }
    }
}

/// Convert a parameter to JSON Schema property.
#[must_use]
pub fn param_to_schema(param: &Param) -> (std::string::String, Value) {
    let mut schema = type_to_schema(&param.ty);
    if let Value::Object(ref mut map) = schema {
        map.insert(
            "description".to_string(),
            Value::String(format!("Parameter: {}", param.name)),
        );
    }
    (param.name.clone(), schema)
}

/// Generate MCP tool schema from a Prima function signature.
///
/// ## Tier: T2-C (μ + σ + κ)
#[must_use]
pub fn function_to_mcp_schema(func: &FunctionSig, prefix: &str) -> Value {
    let tool_name = format!("{}_{}", prefix, func.name);

    // Build properties object
    let properties: serde_json::Map<std::string::String, Value> =
        func.params.iter().map(param_to_schema).collect();

    // Required params (all Prima params are required)
    let required: Vec<Value> = func
        .params
        .iter()
        .map(|p| Value::String(p.name.clone()))
        .collect();

    // Build description
    let mut description = func
        .doc
        .clone()
        .unwrap_or_else(|| format!("Prima function: {}", func.name));

    // Add grounding info
    let groundings = func.all_groundings().join(" ");
    description.push_str(&format!("\n\nGrounding: [{}]", groundings));

    json!({
        "name": tool_name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required
        },
        "returnType": type_to_schema(&func.return_type),
        "grounding": func.all_groundings()
    })
}

/// Generate MCP tool catalog from multiple Prima files.
///
/// ## Tier: T2-C (σ + μ)
#[must_use]
pub fn generate_catalog(functions: &[FunctionSig], prefix: &str) -> Value {
    let tools: Vec<Value> = functions
        .iter()
        .map(|f| function_to_mcp_schema(f, prefix))
        .collect();

    json!({
        "tools": tools,
        "count": tools.len(),
        "prefix": prefix
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_schema() {
        let schema = type_to_schema(&PrimaType::Int);
        assert_eq!(schema["type"], "integer");
    }

    #[test]
    fn test_string_schema() {
        let schema = type_to_schema(&PrimaType::String);
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn test_bool_schema() {
        let schema = type_to_schema(&PrimaType::Bool);
        assert_eq!(schema["type"], "boolean");
    }

    #[test]
    fn test_seq_schema() {
        let schema = type_to_schema(&PrimaType::Seq(None));
        assert_eq!(schema["type"], "array");
    }

    #[test]
    fn test_seq_typed_schema() {
        let schema = type_to_schema(&PrimaType::Seq(Some(Box::new(PrimaType::Int))));
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "integer");
    }

    #[test]
    fn test_function_to_mcp_schema() {
        let func = FunctionSig {
            name: "add".to_string(),
            params: vec![
                Param {
                    name: "a".to_string(),
                    ty: PrimaType::Int,
                },
                Param {
                    name: "b".to_string(),
                    ty: PrimaType::Int,
                },
            ],
            return_type: PrimaType::Int,
            doc: Some("Add two numbers".to_string()),
        };

        let schema = function_to_mcp_schema(&func, "prima");
        assert_eq!(schema["name"], "prima_add");
        assert!(
            schema["description"]
                .as_str()
                .unwrap_or("")
                .contains("Add two numbers")
        );
        assert_eq!(schema["inputSchema"]["properties"]["a"]["type"], "integer");
        let empty_vec: Vec<serde_json::Value> = vec![];
        assert_eq!(
            schema["inputSchema"]["required"]
                .as_array()
                .unwrap_or(&empty_vec)
                .len(),
            2
        );
    }

    #[test]
    fn test_generate_catalog() {
        let funcs = vec![
            FunctionSig {
                name: "foo".to_string(),
                params: vec![],
                return_type: PrimaType::Int,
                doc: None,
            },
            FunctionSig {
                name: "bar".to_string(),
                params: vec![],
                return_type: PrimaType::String,
                doc: None,
            },
        ];

        let catalog = generate_catalog(&funcs, "test");
        assert_eq!(catalog["count"], 2);
        let empty_vec2: Vec<serde_json::Value> = vec![];
        assert_eq!(catalog["tools"].as_array().unwrap_or(&empty_vec2).len(), 2);
    }
}
