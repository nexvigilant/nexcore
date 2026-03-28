//! Company identity newtype.
//!
//! `CompanyId` is a thin wrapper over `String` that provides type-safe
//! identity for pharmaceutical companies without the overhead of UUID
//! generation. Follows the cartouche pattern used across NexCore.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Type-safe identifier for a pharmaceutical company.
///
/// # Examples
///
/// ```
/// use nexcore_pharma::CompanyId;
///
/// let id = CompanyId::new("pfizer-inc");
/// assert_eq!(id.as_str(), "pfizer-inc");
/// assert_eq!(id.to_string(), "pfizer-inc");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CompanyId(String);

impl CompanyId {
    /// Create a new `CompanyId` from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return a reference to the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CompanyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for CompanyId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CompanyId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for CompanyId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn company_id_new_and_display() {
        let id = CompanyId::new("pfizer-inc");
        assert_eq!(id.as_str(), "pfizer-inc");
        assert_eq!(id.to_string(), "pfizer-inc");
    }

    #[test]
    fn company_id_from_string() {
        let id = CompanyId::from("pfizer-inc".to_string());
        assert_eq!(id.as_str(), "pfizer-inc");
    }

    #[test]
    fn company_id_from_str() {
        let id = CompanyId::from("novartis");
        assert_eq!(id.as_str(), "novartis");
    }

    #[test]
    fn company_id_eq_and_hash() {
        use std::collections::HashSet;
        let a = CompanyId::new("roche");
        let b = CompanyId::new("roche");
        let c = CompanyId::new("astrazeneca");
        assert_eq!(a, b);
        assert_ne!(a, c);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b); // duplicate
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn company_id_serializes_round_trip() {
        let id = CompanyId::new("bristol-myers-squibb");
        let json = serde_json::to_string(&id).expect("serialization cannot fail on valid UTF-8");
        let parsed: CompanyId =
            serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
        assert_eq!(id, parsed);
    }
}
