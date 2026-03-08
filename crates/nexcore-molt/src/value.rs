//! Bidirectional conversion between `serde_json::Value` and `molt::Value`.
//!
//! ## Conversion Rules
//!
//! | JSON            | → Molt              | ← Molt (heuristic)     |
//! |-----------------|----------------------|------------------------|
//! | `null`          | `Value::empty()`     | empty string → `null`  |
//! | `bool(true)`    | `Value::from("1")`   | "1"/"true" → `true`    |
//! | `bool(false)`   | `Value::from("0")`   | "0"/"false" → `false`  |
//! | `Number(i64)`   | `Value::from(i)`     | `as_int()` → Number    |
//! | `Number(f64)`   | `Value::from(f)`     | `as_float()` → Number  |
//! | `String(s)`     | `Value::from(s)`     | default → String       |
//! | `Array([..])`   | `Value::from(&[..])` | `as_list()` → Array    |
//! | `Object({..})`  | key-value list       | not round-tripped       |
//!
//! Round-tripping is lossy by design. Tcl is stringly-typed; JSON is not.
//! For precise type control, use typed access methods instead of round-tripping.

use crate::error::MoltError;
use molt::types::{MoltFloat, MoltInt, Value};

/// Convert a `serde_json::Value` to a `molt::Value`.
pub fn to_molt(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::empty(),
        serde_json::Value::Bool(true) => Value::from(1 as MoltInt),
        serde_json::Value::Bool(false) => Value::from(0 as MoltInt),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::from(i as MoltInt)
            } else if let Some(f) = n.as_f64() {
                Value::from(f as MoltFloat)
            } else {
                // u64 that doesn't fit i64 — convert as string
                Value::from(n.to_string().as_str())
            }
        }
        serde_json::Value::String(s) => Value::from(s.as_str()),
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr.iter().map(to_molt).collect();
            Value::from(items.as_slice())
        }
        serde_json::Value::Object(map) => {
            // Flatten object to Tcl dict-like key-value list
            let mut items: Vec<Value> = Vec::with_capacity(map.len() * 2);
            for (k, v) in map {
                items.push(Value::from(k.as_str()));
                items.push(to_molt(v));
            }
            Value::from(items.as_slice())
        }
    }
}

/// Convert a `molt::Value` to a `serde_json::Value` using heuristic type detection.
///
/// The detection order is: empty → int → float → list (len > 1) → string.
/// This is inherently lossy — a Tcl string "42" will become JSON number 42.
/// For precise control, use typed accessors on the engine instead.
pub fn from_molt(value: &Value) -> serde_json::Value {
    let s = value.as_str();

    // Empty string → null
    if s.is_empty() {
        return serde_json::Value::Null;
    }

    // Try integer
    if let Ok(i) = value.as_int() {
        return serde_json::Value::Number(serde_json::Number::from(i));
    }

    // Try float (only if it contains a decimal point or exponent to avoid int→float)
    if (s.contains('.') || s.contains('e') || s.contains('E')) && value.as_float().is_ok() {
        if let Ok(i) = value.as_float() {
            if let Some(n) = serde_json::Number::from_f64(i) {
                return serde_json::Value::Number(n);
            }
        }
    }

    // Try list — but only when the string was genuinely structured as a list.
    // Tcl treats any whitespace-separated string as a list, so "hello world" becomes
    // ["hello", "world"]. We guard against this by requiring EITHER:
    //   1. Tcl list delimiters (braces) are present, OR
    //   2. Every element of the list individually parses as a non-string JSON type
    //      (number, bool, null) — meaning the original was likely a typed array.
    if let Ok(list) = value.as_list() {
        if list.len() > 1 {
            let has_braces = s.starts_with('{') || s.contains(" {") || s.contains("} ");
            let all_typed = list.iter().all(|v| {
                let vs = v.as_str();
                vs.is_empty() || v.as_int().is_ok() || v.as_float().is_ok()
            });
            if has_braces || all_typed {
                return serde_json::Value::Array(list.iter().map(from_molt).collect());
            }
        }
    }

    // Boolean detection
    if s == "true" {
        return serde_json::Value::Bool(true);
    }
    if s == "false" {
        return serde_json::Value::Bool(false);
    }

    // Default: string
    serde_json::Value::String(s.to_string())
}

/// Convert a `molt::Value` to a specific JSON type, returning an error on mismatch.
pub fn from_molt_as_string(value: &Value) -> String {
    value.as_str().to_string()
}

/// Try to convert a `molt::Value` to a JSON integer.
pub fn from_molt_as_int(value: &Value) -> crate::error::Result<i64> {
    value
        .as_int()
        .map_err(|e| MoltError::Type(e.value().to_string()))
}

/// Try to convert a `molt::Value` to a JSON float.
pub fn from_molt_as_float(value: &Value) -> crate::error::Result<f64> {
    value
        .as_float()
        .map_err(|e| MoltError::Type(e.value().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn null_round_trip() {
        let j = json!(null);
        let m = to_molt(&j);
        assert_eq!(m.as_str(), "");
        let back = from_molt(&m);
        assert_eq!(back, json!(null));
    }

    #[test]
    fn bool_round_trip() {
        let t = to_molt(&json!(true));
        assert_eq!(t.as_int().unwrap(), 1);

        let f = to_molt(&json!(false));
        assert_eq!(f.as_int().unwrap(), 0);
    }

    #[test]
    fn int_round_trip() {
        let j = json!(42);
        let m = to_molt(&j);
        assert_eq!(m.as_int().unwrap(), 42);
        let back = from_molt(&m);
        assert_eq!(back, json!(42));
    }

    #[test]
    fn float_round_trip() {
        let j = json!(3.14);
        let m = to_molt(&j);
        let f = m.as_float().unwrap();
        assert!((f - 3.14).abs() < 1e-10);
        let back = from_molt(&m);
        assert_eq!(back, json!(3.14));
    }

    #[test]
    fn string_round_trip() {
        let j = json!("hello world");
        let m = to_molt(&j);
        assert_eq!(m.as_str(), "hello world");
        let back = from_molt(&m);
        assert_eq!(back, json!("hello world"));
    }

    #[test]
    fn array_round_trip() {
        let j = json!([1, 2, 3]);
        let m = to_molt(&j);
        let list = m.as_list().unwrap();
        assert_eq!(list.len(), 3);
        let back = from_molt(&m);
        assert_eq!(back, json!([1, 2, 3]));
    }

    #[test]
    fn object_to_molt_list() {
        let j = json!({"a": 1, "b": "two"});
        let m = to_molt(&j);
        let list = m.as_list().unwrap();
        // Object becomes key-value list: ["a", 1, "b", "two"]
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn typed_accessors() {
        let m = Value::from(42 as MoltInt);
        assert_eq!(from_molt_as_int(&m).unwrap(), 42);
        assert_eq!(from_molt_as_string(&m), "42");

        let m = Value::from(3.14 as MoltFloat);
        let f = from_molt_as_float(&m).unwrap();
        assert!((f - 3.14).abs() < 1e-10);
    }

    #[test]
    fn type_error_on_bad_int() {
        let m = Value::from("not_a_number");
        assert!(from_molt_as_int(&m).is_err());
    }
}
