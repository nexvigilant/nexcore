//! # Clearance Audit Trail
//!
//! Append-only audit log for classification access and changes.
//!
//! ## Primitive Grounding
//! - **ClearanceEntry**: T2-P, Dominant: π Persistence (π + σ)
//! - **ClearanceAudit**: T3, Dominant: π Persistence (π + σ + N + κ)

use crate::level::ClassificationLevel;
use crate::tag::TagTarget;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

/// What action was performed on a classified asset.
///
/// ## Tier: T1
/// ## Dominant: σ Sequence (action in a sequence of events)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditAction {
    /// Asset was read/accessed.
    Access,
    /// Asset was written/modified.
    Write,
    /// Classification was assigned or changed.
    Classify,
    /// Classification was upgraded (more restrictive).
    Upgrade,
    /// Classification was downgraded (less restrictive).
    Downgrade,
    /// Access was denied by enforcement.
    Denied,
    /// A cross-boundary operation was attempted.
    CrossBoundary,
    /// External tool call involving classified data.
    ExternalCall,
}

impl fmt::Display for AuditAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Access => write!(f, "ACCESS"),
            Self::Write => write!(f, "WRITE"),
            Self::Classify => write!(f, "CLASSIFY"),
            Self::Upgrade => write!(f, "UPGRADE"),
            Self::Downgrade => write!(f, "DOWNGRADE"),
            Self::Denied => write!(f, "DENIED"),
            Self::CrossBoundary => write!(f, "CROSS_BOUNDARY"),
            Self::ExternalCall => write!(f, "EXTERNAL_CALL"),
        }
    }
}

/// A single audit trail entry.
///
/// ## Tier: T2-P
/// ## Dominant: π Persistence
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearanceEntry {
    /// What was accessed/modified.
    pub target: TagTarget,
    /// What action was performed.
    pub action: AuditAction,
    /// Classification level at time of action.
    pub level: ClassificationLevel,
    /// Who performed the action.
    pub actor: String,
    /// When the action occurred.
    pub timestamp: DateTime,
    /// Additional context.
    pub context: String,
}

impl ClearanceEntry {
    /// Create a new audit entry with current timestamp.
    #[must_use]
    pub fn new(
        target: TagTarget,
        action: AuditAction,
        level: ClassificationLevel,
        actor: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            target,
            action,
            level,
            actor: actor.into(),
            timestamp: DateTime::now(),
            context: context.into(),
        }
    }
}

impl fmt::Display for ClearanceEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} {} on {} ({})",
            self.timestamp
                .format("%Y-%m-%dT%H:%M:%S")
                .unwrap_or_default(),
            self.actor,
            self.action,
            self.target,
            self.level,
        )
    }
}

/// Append-only audit trail for the classification system.
///
/// No entries can be removed or modified — regulatory compliance.
///
/// ## Tier: T3
/// ## Dominant: π Persistence
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClearanceAudit {
    entries: Vec<ClearanceEntry>,
}

impl ClearanceAudit {
    /// Create an empty audit trail.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append an entry to the audit trail. Cannot be undone.
    pub fn append(&mut self, entry: ClearanceEntry) {
        self.entries.push(entry);
    }

    /// Total number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the audit trail is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Read-only access to all entries.
    #[must_use]
    pub fn entries(&self) -> &[ClearanceEntry] {
        &self.entries
    }

    /// Filter entries by target.
    #[must_use]
    pub fn entries_for_target(&self, target: &TagTarget) -> Vec<&ClearanceEntry> {
        self.entries
            .iter()
            .filter(|e| &e.target == target)
            .collect()
    }

    /// Filter entries by action.
    #[must_use]
    pub fn entries_by_action(&self, action: &AuditAction) -> Vec<&ClearanceEntry> {
        self.entries
            .iter()
            .filter(|e| &e.action == action)
            .collect()
    }

    /// Filter entries by classification level.
    #[must_use]
    pub fn entries_at_level(&self, level: ClassificationLevel) -> Vec<&ClearanceEntry> {
        self.entries.iter().filter(|e| e.level == level).collect()
    }

    /// Count denied actions.
    #[must_use]
    pub fn denied_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.action == AuditAction::Denied)
            .count()
    }

    /// Summary: count per action type.
    #[must_use]
    pub fn summary(&self) -> Vec<(String, usize)> {
        let actions = [
            AuditAction::Access,
            AuditAction::Write,
            AuditAction::Classify,
            AuditAction::Upgrade,
            AuditAction::Downgrade,
            AuditAction::Denied,
            AuditAction::CrossBoundary,
            AuditAction::ExternalCall,
        ];
        actions
            .iter()
            .map(|a| {
                let count = self.entries.iter().filter(|e| &e.action == a).count();
                (a.to_string(), count)
            })
            .filter(|(_, c)| *c > 0)
            .collect()
    }

    /// Merge another audit trail into this one (append-only).
    pub fn merge(&mut self, other: &Self) {
        for entry in &other.entries {
            self.entries.push(entry.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(action: AuditAction, level: ClassificationLevel) -> ClearanceEntry {
        ClearanceEntry::new(
            TagTarget::File("test.rs".into()),
            action,
            level,
            "test-actor",
            "test context",
        )
    }

    #[test]
    fn append_only() {
        let mut audit = ClearanceAudit::new();
        assert!(audit.is_empty());
        audit.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Internal,
        ));
        assert_eq!(audit.len(), 1);
        audit.append(sample_entry(
            AuditAction::Write,
            ClassificationLevel::Secret,
        ));
        assert_eq!(audit.len(), 2);
    }

    #[test]
    fn filter_by_target() {
        let mut audit = ClearanceAudit::new();
        audit.append(ClearanceEntry::new(
            TagTarget::File("a.rs".into()),
            AuditAction::Access,
            ClassificationLevel::Internal,
            "actor",
            "",
        ));
        audit.append(ClearanceEntry::new(
            TagTarget::File("b.rs".into()),
            AuditAction::Access,
            ClassificationLevel::Internal,
            "actor",
            "",
        ));
        let target = TagTarget::File("a.rs".into());
        assert_eq!(audit.entries_for_target(&target).len(), 1);
    }

    #[test]
    fn filter_by_action() {
        let mut audit = ClearanceAudit::new();
        audit.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Internal,
        ));
        audit.append(sample_entry(
            AuditAction::Denied,
            ClassificationLevel::Secret,
        ));
        audit.append(sample_entry(
            AuditAction::Denied,
            ClassificationLevel::TopSecret,
        ));
        assert_eq!(audit.entries_by_action(&AuditAction::Denied).len(), 2);
    }

    #[test]
    fn filter_by_level() {
        let mut audit = ClearanceAudit::new();
        audit.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Internal,
        ));
        audit.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Secret,
        ));
        assert_eq!(audit.entries_at_level(ClassificationLevel::Secret).len(), 1);
    }

    #[test]
    fn denied_count() {
        let mut audit = ClearanceAudit::new();
        audit.append(sample_entry(
            AuditAction::Denied,
            ClassificationLevel::Secret,
        ));
        audit.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Internal,
        ));
        audit.append(sample_entry(
            AuditAction::Denied,
            ClassificationLevel::TopSecret,
        ));
        assert_eq!(audit.denied_count(), 2);
    }

    #[test]
    fn summary_counts() {
        let mut audit = ClearanceAudit::new();
        audit.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Internal,
        ));
        audit.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Internal,
        ));
        audit.append(sample_entry(
            AuditAction::Denied,
            ClassificationLevel::Secret,
        ));
        let summary = audit.summary();
        let access_count = summary.iter().find(|(a, _)| a == "ACCESS").map(|(_, c)| *c);
        assert_eq!(access_count, Some(2));
    }

    #[test]
    fn merge_appends() {
        let mut a = ClearanceAudit::new();
        a.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Internal,
        ));
        let mut b = ClearanceAudit::new();
        b.append(sample_entry(
            AuditAction::Write,
            ClassificationLevel::Secret,
        ));
        a.merge(&b);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn serde_roundtrip() {
        let mut audit = ClearanceAudit::new();
        audit.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Confidential,
        ));
        let json = serde_json::to_string(&audit).unwrap_or_default();
        let parsed: Result<ClearanceAudit, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
        if let Ok(p) = parsed {
            assert_eq!(p.len(), 1);
        }
    }

    #[test]
    fn entry_display() {
        let entry = sample_entry(AuditAction::Access, ClassificationLevel::Secret);
        let display = entry.to_string();
        assert!(display.contains("ACCESS"));
        assert!(display.contains("Secret"));
    }

    #[test]
    fn entry_timestamps_are_recent() {
        let entry = sample_entry(AuditAction::Access, ClassificationLevel::Internal);
        let now = DateTime::now();
        let diff = now.signed_duration_since(entry.timestamp);
        assert!(diff.num_seconds() < 5);
    }

    #[test]
    fn audit_action_display() {
        assert_eq!(AuditAction::Access.to_string(), "ACCESS");
        assert_eq!(AuditAction::Denied.to_string(), "DENIED");
        assert_eq!(AuditAction::CrossBoundary.to_string(), "CROSS_BOUNDARY");
    }

    #[test]
    fn empty_audit_summary() {
        let audit = ClearanceAudit::new();
        assert!(audit.summary().is_empty());
    }

    #[test]
    fn entries_returns_all() {
        let mut audit = ClearanceAudit::new();
        audit.append(sample_entry(
            AuditAction::Access,
            ClassificationLevel::Internal,
        ));
        audit.append(sample_entry(
            AuditAction::Write,
            ClassificationLevel::Secret,
        ));
        assert_eq!(audit.entries().len(), 2);
    }
}
