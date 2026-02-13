//! # JSON Processing Utilities
//!
//! JSON manipulation, path access, and merging.

use serde_json::Value as JsonValue;

/// Get a value from a JSON object using dot notation path
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::foundation::data::json::json_get;
/// use serde_json::json;
///
/// let data = json!({"a": {"b": {"c": 42}}});
/// assert_eq!(json_get(&data, "a.b.c"), Some(&json!(42)));
/// ```
#[must_use]
pub fn json_get<'a>(value: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let mut current = value;

    for part in path.split('.') {
        match current {
            JsonValue::Object(map) => {
                current = map.get(part)?;
            }
            JsonValue::Array(arr) => {
                let idx: usize = part.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

/// Set a value in a JSON object using dot notation path
///
/// Creates intermediate objects as needed.
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::foundation::data::json::json_set;
/// use serde_json::json;
///
/// let mut data = json!({});
/// json_set(&mut data, "a.b.c", json!(42));
/// assert_eq!(data, json!({"a": {"b": {"c": 42}}}));
/// ```
pub fn json_set(value: &mut JsonValue, path: &str, new_value: JsonValue) {
    let mut parts = path.split('.').peekable();
    let mut current = value;

    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            // Last segment — move value instead of cloning
            if let JsonValue::Object(map) = current {
                map.insert(part.to_string(), new_value);
            }
            return;
        }
        // Intermediate segment — ensure object exists
        if let JsonValue::Object(map) = current {
            if !map.contains_key(part) {
                map.insert(part.to_string(), JsonValue::Object(serde_json::Map::new()));
            }
            if let Some(next) = map.get_mut(part) {
                current = next;
            } else {
                return;
            }
        }
    }
}

/// Deep merge two JSON objects
///
/// Values from `b` override values from `a` at the same path.
/// Arrays are replaced, not concatenated.
#[must_use]
pub fn json_merge(a: &JsonValue, b: &JsonValue) -> JsonValue {
    match (a, b) {
        (JsonValue::Object(a_map), JsonValue::Object(b_map)) => {
            let mut result = a_map.clone();
            for (key, b_val) in b_map {
                let merged = if let Some(a_val) = a_map.get(key) {
                    json_merge(a_val, b_val)
                } else {
                    b_val.clone()
                };
                result.insert(key.clone(), merged);
            }
            JsonValue::Object(result)
        }
        // For non-objects, b takes precedence
        (_, b) => b.clone(),
    }
}

/// Extract all keys from a JSON object (flattened with dot notation)
#[must_use]
pub fn json_keys(value: &JsonValue) -> Vec<String> {
    let mut keys = Vec::new();
    collect_keys(value, "", &mut keys);
    keys
}

fn collect_keys(value: &JsonValue, prefix: &str, keys: &mut Vec<String>) {
    if let JsonValue::Object(map) = value {
        for (k, v) in map {
            let path = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{prefix}.{k}")
            };
            keys.push(path.clone());
            collect_keys(v, &path, keys);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_get_simple() {
        let data = json!({"a": 1, "b": 2});
        assert_eq!(json_get(&data, "a"), Some(&json!(1)));
        assert_eq!(json_get(&data, "b"), Some(&json!(2)));
        assert_eq!(json_get(&data, "c"), None);
    }

    #[test]
    fn test_json_get_nested() {
        let data = json!({"a": {"b": {"c": 42}}});
        assert_eq!(json_get(&data, "a.b.c"), Some(&json!(42)));
        assert_eq!(json_get(&data, "a.b"), Some(&json!({"c": 42})));
    }

    #[test]
    fn test_json_get_array() {
        let data = json!({"arr": [1, 2, 3]});
        assert_eq!(json_get(&data, "arr.0"), Some(&json!(1)));
        assert_eq!(json_get(&data, "arr.2"), Some(&json!(3)));
    }

    #[test]
    fn test_json_set_simple() {
        let mut data = json!({});
        json_set(&mut data, "a", json!(1));
        assert_eq!(data, json!({"a": 1}));
    }

    #[test]
    fn test_json_set_nested() {
        let mut data = json!({});
        json_set(&mut data, "a.b.c", json!(42));
        assert_eq!(data, json!({"a": {"b": {"c": 42}}}));
    }

    #[test]
    fn test_json_merge() {
        let a = json!({"a": 1, "b": {"c": 2}});
        let b = json!({"b": {"d": 3}, "e": 4});
        let result = json_merge(&a, &b);
        assert_eq!(result, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_json_keys() {
        let data = json!({"a": 1, "b": {"c": 2}});
        let keys = json_keys(&data);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        assert!(keys.contains(&"b.c".to_string()));
    }
}
