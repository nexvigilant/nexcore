//! Type-safe disease identity.
//!
//! `DiseaseId` is a validated string newtype that prevents accidental
//! cross-domain identity confusion (e.g., passing a company ID where a
//! disease ID is expected).

use std::fmt;

use serde::{Deserialize, Serialize};

/// Opaque, type-safe disease identifier.
///
/// Constructed from any `&str` via [`DiseaseId::new`]. Serialises as a plain
/// JSON string for cross-boundary transport (π).
///
/// # Examples
///
/// ```
/// use nexcore_disease::DiseaseId;
///
/// let id = DiseaseId::new("t2dm");
/// assert_eq!(id.as_str(), "t2dm");
/// assert_eq!(id.to_string(), "t2dm");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DiseaseId(String);

impl DiseaseId {
    /// Construct a new `DiseaseId` from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DiseaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for DiseaseId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for DiseaseId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_as_str() {
        let id = DiseaseId::new("alzheimers");
        assert_eq!(id.as_str(), "alzheimers");
    }

    #[test]
    fn display_equals_inner() {
        let id = DiseaseId::new("nsclc");
        assert_eq!(id.to_string(), "nsclc");
    }

    #[test]
    fn from_str_ref() {
        let id: DiseaseId = "ra".into();
        assert_eq!(id.as_str(), "ra");
    }

    #[test]
    fn serialises_as_plain_string() {
        let id = DiseaseId::new("obesity");
        let json = serde_json::to_string(&id).expect("serialisation must not fail");
        assert_eq!(json, r#""obesity""#);
    }

    #[test]
    fn round_trip_serde() {
        let original = DiseaseId::new("hf");
        let json = serde_json::to_string(&original).expect("serialise");
        let parsed: DiseaseId = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(original, parsed);
    }
}
