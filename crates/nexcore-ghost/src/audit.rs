//! # Redaction Audit Trail
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | π Persistence | Append-only audit log of all redaction actions |
//! | σ Sequence | Ordered sequence of redaction entries |
//!
//! ## Tier: T2-C (RedactionEntry), T3 (RedactionAudit)

use serde::{Deserialize, Serialize};
use std::fmt;

/// A single audit record of a redaction/pseudonymization action.
///
/// ## Tier: T2-C (π Persistence + σ Sequence)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionEntry {
    /// Field name that was acted upon (e.g., "patient_name").
    pub field: String,
    /// Action taken: "pseudonymize", "redact", "generalize", "suppress", "retain".
    pub action: String,
    /// Reason for the action (e.g., "GhostMode::Strict policy").
    pub reason: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Data category of the field.
    pub category: String,
}

impl RedactionEntry {
    /// Create a new entry with current timestamp.
    #[must_use]
    pub fn new(
        field: impl Into<String>,
        action: impl Into<String>,
        reason: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            action: action.into(),
            reason: reason.into(),
            timestamp: nexcore_chrono::DateTime::now().to_rfc3339(),
            category: category.into(),
        }
    }
}

impl fmt::Display for RedactionEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} → {} ({})",
            self.timestamp, self.field, self.action, self.reason
        )
    }
}

/// Append-only audit trail for all redaction operations.
///
/// Entries can be added but never removed or modified.
/// This ensures regulatory auditability.
///
/// ## Tier: T3 (π Persistence + σ Sequence + N Quantity)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RedactionAudit {
    entries: Vec<RedactionEntry>,
}

impl RedactionAudit {
    /// Create an empty audit trail.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append an entry to the trail. Append-only — no removal.
    pub fn append(&mut self, entry: RedactionEntry) {
        self.entries.push(entry);
    }

    /// Number of entries in the trail.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the trail is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Immutable view of all entries.
    #[must_use]
    pub fn entries(&self) -> &[RedactionEntry] {
        &self.entries
    }

    /// Filter entries by field name.
    #[must_use]
    pub fn entries_for_field(&self, field: &str) -> Vec<&RedactionEntry> {
        self.entries.iter().filter(|e| e.field == field).collect()
    }

    /// Filter entries by action type.
    #[must_use]
    pub fn entries_by_action(&self, action: &str) -> Vec<&RedactionEntry> {
        self.entries.iter().filter(|e| e.action == action).collect()
    }

    /// Summary: count of each action type.
    #[must_use]
    pub fn summary(&self) -> std::collections::HashMap<String, usize> {
        let mut map = std::collections::HashMap::new();
        for entry in &self.entries {
            *map.entry(entry.action.clone()).or_insert(0) += 1;
        }
        map
    }

    /// Merge another audit trail into this one (append all entries).
    pub fn merge(&mut self, other: &RedactionAudit) {
        self.entries.extend(other.entries.iter().cloned());
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_audit_is_empty() {
        let audit = RedactionAudit::new();
        assert!(audit.is_empty());
        assert_eq!(audit.len(), 0);
    }

    #[test]
    fn append_increases_length() {
        let mut audit = RedactionAudit::new();
        audit.append(RedactionEntry::new(
            "name",
            "pseudonymize",
            "policy",
            "basic_identity",
        ));
        assert_eq!(audit.len(), 1);
    }

    #[test]
    fn entries_are_ordered() {
        let mut audit = RedactionAudit::new();
        audit.append(RedactionEntry::new("field_a", "redact", "r1", "cat1"));
        audit.append(RedactionEntry::new("field_b", "pseudonymize", "r2", "cat2"));
        assert_eq!(audit.entries()[0].field, "field_a");
        assert_eq!(audit.entries()[1].field, "field_b");
    }

    #[test]
    fn filter_by_field() {
        let mut audit = RedactionAudit::new();
        audit.append(RedactionEntry::new("name", "pseudonymize", "r", "c"));
        audit.append(RedactionEntry::new("email", "redact", "r", "c"));
        audit.append(RedactionEntry::new("name", "generalize", "r", "c"));
        let name_entries = audit.entries_for_field("name");
        assert_eq!(name_entries.len(), 2);
    }

    #[test]
    fn filter_by_action() {
        let mut audit = RedactionAudit::new();
        audit.append(RedactionEntry::new("a", "redact", "r", "c"));
        audit.append(RedactionEntry::new("b", "redact", "r", "c"));
        audit.append(RedactionEntry::new("c", "pseudonymize", "r", "c"));
        let redacted = audit.entries_by_action("redact");
        assert_eq!(redacted.len(), 2);
    }

    #[test]
    fn summary_counts_actions() {
        let mut audit = RedactionAudit::new();
        audit.append(RedactionEntry::new("a", "redact", "r", "c"));
        audit.append(RedactionEntry::new("b", "redact", "r", "c"));
        audit.append(RedactionEntry::new("c", "pseudonymize", "r", "c"));
        let summary = audit.summary();
        assert_eq!(summary.get("redact").copied().unwrap_or(0), 2);
        assert_eq!(summary.get("pseudonymize").copied().unwrap_or(0), 1);
    }

    #[test]
    fn merge_combines_audits() {
        let mut a = RedactionAudit::new();
        a.append(RedactionEntry::new("x", "redact", "r", "c"));
        let mut b = RedactionAudit::new();
        b.append(RedactionEntry::new("y", "pseudonymize", "r", "c"));
        a.merge(&b);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn entry_display() {
        let entry = RedactionEntry::new("patient_name", "pseudonymize", "strict policy", "basic");
        let s = format!("{entry}");
        assert!(s.contains("patient_name"));
        assert!(s.contains("pseudonymize"));
    }

    #[test]
    fn serde_roundtrip() {
        let mut audit = RedactionAudit::new();
        audit.append(RedactionEntry::new("f", "redact", "r", "c"));
        let json = serde_json::to_string(&audit).unwrap_or_default();
        let back: std::result::Result<RedactionAudit, _> = serde_json::from_str(&json);
        assert!(back.is_ok());
    }

    #[test]
    fn default_is_empty() {
        let audit = RedactionAudit::default();
        assert!(audit.is_empty());
    }

    #[test]
    fn entry_has_timestamp() {
        let entry = RedactionEntry::new("f", "a", "r", "c");
        assert!(!entry.timestamp.is_empty());
    }
}
