// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Bidirectional JSON ↔ Prima Serialization
//!
//! Maps JSON types to Prima values and back.
//!
//! ## Tier: T2-P (μ + κ)

use prima::value::{Value, ValueData};
use serde_json::Value as JsonValue;
use thiserror::Error;

/// Serialization errors.
#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("Unsupported JSON type: object")]
    UnsupportedObject,
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },
}

/// Convert JSON value to Prima value.
///
/// ## Tier: T2-P (μ - Mapping)
pub fn json_to_prima(json: &JsonValue) -> Result<Value, SerializeError> {
    match json {
        JsonValue::Null => Ok(Value::void()),
        JsonValue::Bool(b) => Ok(Value::bool(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::float(f))
            } else {
                Ok(Value::int(0))
            }
        }
        JsonValue::String(s) => Ok(Value::string(s.clone())),
        JsonValue::Array(arr) => {
            let mut values = Vec::with_capacity(arr.len());
            for item in arr {
                values.push(json_to_prima(item)?);
            }
            Ok(Value::sequence(values))
        }
        JsonValue::Object(_) => Err(SerializeError::UnsupportedObject),
    }
}

/// Convert Prima value to JSON value.
///
/// ## Tier: T2-P (μ - Mapping)
pub fn prima_to_json(value: &Value) -> JsonValue {
    match &value.data {
        ValueData::Void => JsonValue::Null,
        ValueData::Bool(b) => JsonValue::Bool(*b),
        ValueData::Int(i) => JsonValue::Number((*i).into()),
        ValueData::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        ValueData::String(s) => JsonValue::String(s.clone()),
        ValueData::Sequence(seq) => {
            let arr: Vec<JsonValue> = seq.iter().map(prima_to_json).collect();
            JsonValue::Array(arr)
        }
        ValueData::Mapping(map) => {
            let obj: serde_json::Map<String, JsonValue> = map
                .iter()
                .map(|(k, v)| (k.clone(), prima_to_json(v)))
                .collect();
            JsonValue::Object(obj)
        }
        ValueData::Symbol(s) => JsonValue::String(format!(":{}", s)),
        ValueData::Quoted(_) => JsonValue::String("<quoted>".to_string()),
        ValueData::Function(_) => JsonValue::String("<function>".to_string()),
        ValueData::Builtin(name) => JsonValue::String(format!("<builtin:{}>", name)),
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
    fn test_json_to_prima_int() {
        let json = json!(42);
        let value = json_to_prima(&json);
        assert!(value.is_ok());
        if let Ok(v) = value {
            assert!(matches!(v.data, ValueData::Int(42)));
        }
    }

    #[test]
    fn test_json_to_prima_string() {
        let json = json!("hello");
        let value = json_to_prima(&json);
        assert!(value.is_ok());
        if let Ok(v) = value {
            assert!(matches!(v.data, ValueData::String(ref s) if s == "hello"));
        }
    }

    #[test]
    fn test_json_to_prima_bool() {
        let json = json!(true);
        let value = json_to_prima(&json);
        assert!(value.is_ok());
        if let Ok(v) = value {
            assert!(matches!(v.data, ValueData::Bool(true)));
        }
    }

    #[test]
    fn test_json_to_prima_null() {
        let json = json!(null);
        let value = json_to_prima(&json);
        assert!(value.is_ok());
        if let Ok(v) = value {
            assert!(matches!(v.data, ValueData::Void));
        }
    }

    #[test]
    fn test_json_to_prima_array() {
        let json = json!([1, 2, 3]);
        let value = json_to_prima(&json);
        assert!(value.is_ok());
        if let Ok(v) = value {
            assert!(matches!(v.data, ValueData::Sequence(_)));
        }
    }

    #[test]
    fn test_json_to_prima_object_fails() {
        let json = json!({"key": "value"});
        let value = json_to_prima(&json);
        assert!(value.is_err());
    }

    #[test]
    fn test_prima_to_json_int() {
        let value = Value::int(42);
        let json = prima_to_json(&value);
        assert_eq!(json, json!(42));
    }

    #[test]
    fn test_prima_to_json_string() {
        let value = Value::string("hello".to_string());
        let json = prima_to_json(&value);
        assert_eq!(json, json!("hello"));
    }

    #[test]
    fn test_prima_to_json_seq() {
        let value = Value::sequence(vec![Value::int(1), Value::int(2)]);
        let json = prima_to_json(&value);
        assert_eq!(json, json!([1, 2]));
    }

    #[test]
    fn test_roundtrip_int() {
        let original = json!(123);
        let prima = json_to_prima(&original);
        assert!(prima.is_ok());
        let back = prima_to_json(&prima.unwrap_or_else(|_| Value::void()));
        assert_eq!(back, original);
    }

    #[test]
    fn test_roundtrip_array() {
        let original = json!([1, 2, 3]);
        let prima = json_to_prima(&original);
        assert!(prima.is_ok());
        let back = prima_to_json(&prima.unwrap_or_else(|_| Value::void()));
        assert_eq!(back, original);
    }
}
