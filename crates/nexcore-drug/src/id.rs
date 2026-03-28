//! Drug identity newtype.
//!
//! `DrugId` is a thin wrapper over `String` that provides type-safe
//! identity for drug entities. Follows the cartouche pattern used across
//! NexCore (mirrors `CompanyId` in `nexcore-pharma`).

use std::fmt;

use serde::{Deserialize, Serialize};

/// Type-safe identifier for a drug entity.
///
/// # Examples
///
/// ```
/// use nexcore_drug::DrugId;
///
/// let id = DrugId::new("semaglutide");
/// assert_eq!(id.as_str(), "semaglutide");
/// assert_eq!(id.to_string(), "semaglutide");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DrugId(String);

impl DrugId {
    /// Create a new `DrugId` from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return a reference to the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DrugId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for DrugId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DrugId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for DrugId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drug_id_new_and_display() {
        let id = DrugId::new("tirzepatide");
        assert_eq!(id.as_str(), "tirzepatide");
        assert_eq!(id.to_string(), "tirzepatide");
    }

    #[test]
    fn drug_id_from_string() {
        let id = DrugId::from("semaglutide".to_string());
        assert_eq!(id.as_str(), "semaglutide");
    }

    #[test]
    fn drug_id_from_str() {
        let id = DrugId::from("donanemab");
        assert_eq!(id.as_str(), "donanemab");
    }

    #[test]
    fn drug_id_eq_and_hash() {
        use std::collections::HashSet;
        let a = DrugId::new("pembrolizumab");
        let b = DrugId::new("pembrolizumab");
        let c = DrugId::new("adalimumab");
        assert_eq!(a, b);
        assert_ne!(a, c);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b); // duplicate
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn drug_id_serializes_round_trip() {
        let id = DrugId::new("apixaban");
        let json = serde_json::to_string(&id).expect("serialization cannot fail on valid UTF-8");
        let parsed: DrugId =
            serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
        assert_eq!(id, parsed);
    }
}
