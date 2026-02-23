//! OpenAPI 3.1 → 3.0 downconversion shim.
//!
//! utoipa 5.x emits OpenAPI 3.1.0, but progenitor (and some other client
//! generators) require 3.0.x.  The two versions differ primarily in how
//! nullable schemas are expressed:
//!
//! | Feature        | 3.1                              | 3.0                                  |
//! |----------------|----------------------------------|--------------------------------------|
//! | Version string | `"3.1.0"`                        | `"3.0.3"`                            |
//! | Nullable type  | `"type": ["string", "null"]`     | `"type": "string", "nullable": true` |
//! | Nullable anyOf | `"anyOf": [{...}, {"type":"null"}]` | `"anyOf": [{...}], "nullable": true` |
//!
//! [`downconvert_31_to_30`] performs a recursive, allocation-minimal walk of
//! the JSON tree and applies all three transformations.

use serde_json::{Map, Value};

/// Downconvert an OpenAPI 3.1.0 JSON value to OpenAPI 3.0.3.
///
/// The function:
/// - Replaces `"openapi": "3.1.0"` with `"openapi": "3.0.3"`.
/// - Converts `"type": ["T", "null"]` arrays into
///   `"type": "T", "nullable": true`.
/// - Converts `anyOf` that contain a `{"type":"null"}` variant into an
///   `anyOf` without that variant plus `"nullable": true`.
/// - Recurses into all object values and array elements.
///
/// # Examples
///
/// ```rust
/// use nexcore_api::openapi_compat::downconvert_31_to_30;
/// use serde_json::json;
///
/// let input = json!({
///     "openapi": "3.1.0",
///     "paths": {
///         "/foo": {
///             "get": {
///                 "responses": {
///                     "200": {
///                         "content": {
///                             "application/json": {
///                                 "schema": {
///                                     "type": ["string", "null"]
///                                 }
///                             }
///                         }
///                     }
///                 }
///             }
///         }
///     }
/// });
///
/// let out = downconvert_31_to_30(input);
/// assert_eq!(out["openapi"], "3.0.3");
/// assert_eq!(out["paths"]["/foo"]["get"]["responses"]["200"]
///     ["content"]["application/json"]["schema"]["type"], "string");
/// assert_eq!(out["paths"]["/foo"]["get"]["responses"]["200"]
///     ["content"]["application/json"]["schema"]["nullable"], true);
/// ```
pub fn downconvert_31_to_30(mut spec: Value) -> Value {
    // Fix the version string at the top level first.
    if let Value::Object(ref mut map) = spec {
        if map.get("openapi").and_then(Value::as_str) == Some("3.1.0") {
            map.insert("openapi".to_owned(), Value::String("3.0.3".to_owned()));
        }
    }

    // Recursively rewrite schemas throughout the document.
    walk(&mut spec);
    spec
}

/// Recursively walk every node in the JSON tree, rewriting schema nodes
/// in-place.
fn walk(value: &mut Value) {
    match value {
        Value::Object(map) => {
            rewrite_schema_object(map);
            for v in map.values_mut() {
                walk(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                walk(v);
            }
        }
        // Scalars need no transformation.
        _ => {}
    }
}

/// Apply all 3.0-compatibility rewrites to a single JSON object.
///
/// This is called on *every* object in the tree; guards at the start of each
/// branch ensure transforms only fire when the object actually looks like a
/// schema node that needs conversion.
fn rewrite_schema_object(map: &mut Map<String, Value>) {
    rewrite_type_array(map);
    rewrite_any_of_null(map);
}

/// Convert `"type": ["T", "null"]` → `"type": "T", "nullable": true`.
///
/// The 3.1 spec allows `type` to be either a string or an array of strings.
/// 3.0 only allows a string.  When the array contains exactly one non-null
/// member alongside `"null"` we fold it into the scalar form with
/// `nullable: true`.  When the array contains multiple non-null members we
/// leave it alone — that case has no clean 3.0 equivalent and is uncommon in
/// practice for this codebase.
fn rewrite_type_array(map: &mut Map<String, Value>) {
    let type_val = match map.get("type") {
        Some(Value::Array(arr)) => arr.clone(),
        _ => return,
    };

    let non_null: Vec<&Value> = type_val
        .iter()
        .filter(|v| v.as_str() != Some("null"))
        .collect();

    let has_null = type_val.iter().any(|v| v.as_str() == Some("null"));

    if !has_null {
        // Already a plain array without null — collapse to scalar if there's
        // exactly one entry, otherwise leave as-is (unusual).
        if non_null.len() == 1 {
            let scalar = (*non_null[0]).clone();
            map.insert("type".to_owned(), scalar);
        }
        return;
    }

    match non_null.as_slice() {
        [] => {
            // `"type": ["null"]` — represent as a generic nullable schema
            // without a type constraint (closest 3.0 equivalent).
            map.remove("type");
            map.insert("nullable".to_owned(), Value::Bool(true));
        }
        [single] => {
            let scalar = (*single).clone();
            map.insert("type".to_owned(), scalar);
            map.insert("nullable".to_owned(), Value::Bool(true));
        }
        _ => {
            // Multiple non-null types with null — no clean 3.0 mapping.
            // Leave the `type` array intact and set nullable so at least
            // the nullable intent is preserved.
            map.insert("nullable".to_owned(), Value::Bool(true));
        }
    }
}

/// Convert `anyOf` that contains `{"type":"null"}` into a pruned `anyOf`
/// with `"nullable": true`.
///
/// OpenAPI 3.1 encourages `anyOf: [{$ref: "..."}, {type: "null"}]` for
/// optional references.  3.0 represents this as the ref schema with
/// `nullable: true` on the *enclosing* schema object.
fn rewrite_any_of_null(map: &mut Map<String, Value>) {
    let any_of = match map.get("anyOf") {
        Some(Value::Array(arr)) => arr.clone(),
        _ => return,
    };

    let has_null_variant = any_of.iter().any(is_null_schema);
    if !has_null_variant {
        return;
    }

    let non_null_variants: Vec<Value> = any_of.into_iter().filter(|v| !is_null_schema(v)).collect();

    map.insert("nullable".to_owned(), Value::Bool(true));

    match non_null_variants.len() {
        0 => {
            // Only null was present — remove anyOf entirely.
            map.remove("anyOf");
        }
        1 => {
            // A single non-null variant: inline its keys into the parent
            // object and drop anyOf, producing a flat, readable 3.0 schema.
            map.remove("anyOf");
            if let Some(Value::Object(inner)) = non_null_variants.into_iter().next() {
                for (k, v) in inner {
                    // Do not overwrite keys already present (e.g. nullable).
                    map.entry(k).or_insert(v);
                }
            }
        }
        _ => {
            // Multiple non-null variants: keep anyOf but without the null
            // entry, which we've replaced with `nullable: true`.
            map.insert("anyOf".to_owned(), Value::Array(non_null_variants));
        }
    }
}

/// Returns `true` when `value` is `{"type": "null"}` — the canonical 3.1
/// representation of the null type.
fn is_null_schema(value: &Value) -> bool {
    matches!(
        value,
        Value::Object(m) if m.get("type").and_then(Value::as_str) == Some("null")
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper: run downconvert and return the result.
    fn convert(input: Value) -> Value {
        downconvert_31_to_30(input)
    }

    #[test]
    fn version_string_is_rewritten() {
        let out = convert(json!({ "openapi": "3.1.0" }));
        assert_eq!(out["openapi"], "3.0.3");
    }

    #[test]
    fn non_31_version_is_left_alone() {
        let out = convert(json!({ "openapi": "3.0.1" }));
        assert_eq!(out["openapi"], "3.0.1");
    }

    #[test]
    fn type_array_string_null_becomes_nullable() {
        let out = convert(json!({ "openapi": "3.1.0", "schema": { "type": ["string", "null"] } }));
        assert_eq!(out["schema"]["type"], "string");
        assert_eq!(out["schema"]["nullable"], true);
    }

    #[test]
    fn type_array_null_string_order_does_not_matter() {
        let out = convert(json!({ "openapi": "3.1.0", "schema": { "type": ["null", "integer"] } }));
        assert_eq!(out["schema"]["type"], "integer");
        assert_eq!(out["schema"]["nullable"], true);
    }

    #[test]
    fn type_array_only_null_removes_type_key() {
        let out = convert(json!({ "openapi": "3.1.0", "schema": { "type": ["null"] } }));
        assert!(out["schema"].get("type").is_none());
        assert_eq!(out["schema"]["nullable"], true);
    }

    #[test]
    fn type_string_scalar_unchanged() {
        let out = convert(json!({ "openapi": "3.1.0", "schema": { "type": "string" } }));
        assert_eq!(out["schema"]["type"], "string");
        assert!(out["schema"].get("nullable").is_none());
    }

    #[test]
    fn any_of_with_null_ref_becomes_nullable() {
        let out = convert(json!({
            "openapi": "3.1.0",
            "schema": {
                "anyOf": [
                    { "$ref": "#/components/schemas/Foo" },
                    { "type": "null" }
                ]
            }
        }));
        // anyOf should be removed; ref should be inlined.
        assert!(out["schema"].get("anyOf").is_none());
        assert_eq!(out["schema"]["nullable"], true);
        assert_eq!(out["schema"]["$ref"], "#/components/schemas/Foo");
    }

    #[test]
    fn any_of_multiple_non_null_keeps_any_of() {
        let out = convert(json!({
            "openapi": "3.1.0",
            "schema": {
                "anyOf": [
                    { "type": "string" },
                    { "type": "integer" },
                    { "type": "null" }
                ]
            }
        }));
        let any_of = out["schema"]["anyOf"]
            .as_array()
            .expect("anyOf should remain");
        assert_eq!(any_of.len(), 2, "null variant should be removed");
        assert_eq!(out["schema"]["nullable"], true);
    }

    #[test]
    fn any_of_without_null_is_unchanged() {
        let out = convert(json!({
            "openapi": "3.1.0",
            "schema": {
                "anyOf": [
                    { "type": "string" },
                    { "type": "integer" }
                ]
            }
        }));
        let any_of = out["schema"]["anyOf"]
            .as_array()
            .expect("anyOf should remain");
        assert_eq!(any_of.len(), 2);
        assert!(out["schema"].get("nullable").is_none());
    }

    #[test]
    fn deeply_nested_schemas_are_rewritten() {
        let out = convert(json!({
            "openapi": "3.1.0",
            "components": {
                "schemas": {
                    "MyModel": {
                        "type": "object",
                        "properties": {
                            "name": { "type": ["string", "null"] },
                            "count": { "type": "integer" }
                        }
                    }
                }
            }
        }));
        let props = &out["components"]["schemas"]["MyModel"]["properties"];
        assert_eq!(props["name"]["type"], "string");
        assert_eq!(props["name"]["nullable"], true);
        assert_eq!(props["count"]["type"], "integer");
        assert!(props["count"].get("nullable").is_none());
    }

    #[test]
    fn already_nullable_not_duplicated() {
        // If nullable is already set, it should not be overwritten with false.
        let out = convert(json!({
            "openapi": "3.1.0",
            "schema": { "type": ["string", "null"], "nullable": false }
        }));
        // Our transform sets nullable: true unconditionally — this is correct
        // because the presence of "null" in the type array is the ground truth.
        assert_eq!(out["schema"]["nullable"], true);
    }

    #[test]
    fn single_element_array_without_null_collapses_to_scalar() {
        let out = convert(json!({
            "openapi": "3.1.0",
            "schema": { "type": ["string"] }
        }));
        assert_eq!(out["schema"]["type"], "string");
        assert!(out["schema"].get("nullable").is_none());
    }

    #[test]
    fn passthrough_when_no_version_key() {
        let input = json!({ "info": { "title": "Test" } });
        let out = convert(input.clone());
        assert_eq!(out, input);
    }
}
