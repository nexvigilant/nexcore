//! Lenient serde deserializers for MCP tool parameters.
//!
//! MCP clients (including Claude Code) sometimes serialize numeric values as
//! strings (e.g., `"10"` instead of `10`, `"1.5"` instead of `1.5`). These
//! deserializers accept both native types and string representations.
//!
//! Usage: `#[serde(default, deserialize_with = "deserialize_option_usize_lenient")]`

use rmcp::serde::{self, Deserialize, Deserializer};

// ---------------------------------------------------------------------------
// Option<usize>
// ---------------------------------------------------------------------------

/// Accepts `null`, `10`, or `"10"` → `Option<usize>`.
pub fn deserialize_option_usize_lenient<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde", untagged)]
    enum Val {
        Num(usize),
        Str(String),
        Null,
    }

    match Option::<Val>::deserialize(deserializer)? {
        None | Some(Val::Null) => Ok(None),
        Some(Val::Num(n)) => Ok(Some(n)),
        Some(Val::Str(s)) if s.is_empty() => Ok(None),
        Some(Val::Str(s)) => s
            .parse::<usize>()
            .map(Some)
            .map_err(|_| serde::de::Error::custom(format!("expected usize, got: {s}"))),
    }
}

// ---------------------------------------------------------------------------
// Option<u32>
// ---------------------------------------------------------------------------

/// Accepts `null`, `10`, or `"10"` → `Option<u32>`.
pub fn deserialize_option_u32_lenient<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde", untagged)]
    enum Val {
        Num(u32),
        Str(String),
        Null,
    }

    match Option::<Val>::deserialize(deserializer)? {
        None | Some(Val::Null) => Ok(None),
        Some(Val::Num(n)) => Ok(Some(n)),
        Some(Val::Str(s)) if s.is_empty() => Ok(None),
        Some(Val::Str(s)) => s
            .parse::<u32>()
            .map(Some)
            .map_err(|_| serde::de::Error::custom(format!("expected u32, got: {s}"))),
    }
}

// ---------------------------------------------------------------------------
// Option<u64>
// ---------------------------------------------------------------------------

/// Accepts `null`, `10`, or `"10"` → `Option<u64>`.
pub fn deserialize_option_u64_lenient<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde", untagged)]
    enum Val {
        Num(u64),
        Str(String),
        Null,
    }

    match Option::<Val>::deserialize(deserializer)? {
        None | Some(Val::Null) => Ok(None),
        Some(Val::Num(n)) => Ok(Some(n)),
        Some(Val::Str(s)) if s.is_empty() => Ok(None),
        Some(Val::Str(s)) => s
            .parse::<u64>()
            .map(Some)
            .map_err(|_| serde::de::Error::custom(format!("expected u64, got: {s}"))),
    }
}

// ---------------------------------------------------------------------------
// Option<f64>
// ---------------------------------------------------------------------------

/// Accepts `null`, `1.5`, or `"1.5"` → `Option<f64>`.
pub fn deserialize_option_f64_lenient<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde", untagged)]
    enum Val {
        Num(f64),
        Str(String),
        Null,
    }

    match Option::<Val>::deserialize(deserializer)? {
        None | Some(Val::Null) => Ok(None),
        Some(Val::Num(n)) => Ok(Some(n)),
        Some(Val::Str(s)) if s.is_empty() => Ok(None),
        Some(Val::Str(s)) => s
            .parse::<f64>()
            .map(Some)
            .map_err(|_| serde::de::Error::custom(format!("expected f64, got: {s}"))),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde")]
    struct TestUsize {
        #[serde(default, deserialize_with = "deserialize_option_usize_lenient")]
        val: Option<usize>,
    }

    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde")]
    struct TestF64 {
        #[serde(default, deserialize_with = "deserialize_option_f64_lenient")]
        val: Option<f64>,
    }

    #[test]
    fn usize_from_number() {
        let t: TestUsize = serde_json::from_str(r#"{"val": 42}"#).unwrap();
        assert_eq!(t.val, Some(42));
    }

    #[test]
    fn usize_from_string() {
        let t: TestUsize = serde_json::from_str(r#"{"val": "42"}"#).unwrap();
        assert_eq!(t.val, Some(42));
    }

    #[test]
    fn usize_from_null() {
        let t: TestUsize = serde_json::from_str(r#"{"val": null}"#).unwrap();
        assert_eq!(t.val, None);
    }

    #[test]
    fn usize_missing() {
        let t: TestUsize = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(t.val, None);
    }

    #[test]
    fn f64_from_number() {
        let t: TestF64 = serde_json::from_str(r#"{"val": 3.14}"#).unwrap();
        assert!((t.val.unwrap() - 3.14).abs() < 1e-10);
    }

    #[test]
    fn f64_from_string() {
        let t: TestF64 = serde_json::from_str(r#"{"val": "3.14"}"#).unwrap();
        assert!((t.val.unwrap() - 3.14).abs() < 1e-10);
    }

    #[test]
    fn f64_from_integer_string() {
        let t: TestF64 = serde_json::from_str(r#"{"val": "1"}"#).unwrap();
        assert!((t.val.unwrap() - 1.0).abs() < 1e-10);
    }
}
