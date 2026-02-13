// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Tombstone
//!
//! **Tier**: T2-C (void + pi + exists + irrev)
//! **Fills pair gap**: Void x Persistence (previously unexplored)
//!
//! Persistent marker of deletion — records that something USED to exist
//! but was intentionally removed. The void persists across time.
//!
//! Common in distributed systems (Cassandra tombstones, soft deletes),
//! but novel as a typed primitive composition.

use core::fmt;
use std::collections::BTreeMap;

/// Unique tombstone identifier.
pub type TombstoneId = u64;

/// Reason for deletion.
///
/// ## Tier: T2-P (void + causality)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeletionReason {
    /// Explicitly deleted by user action.
    UserAction(String),
    /// Expired by TTL.
    Expired,
    /// Superseded by a newer version.
    Superseded,
    /// Compacted/merged into another record.
    Compacted,
    /// Regulatory requirement (right to be forgotten).
    RegulatoryErasure,
}

impl fmt::Display for DeletionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserAction(who) => write!(f, "user:{who}"),
            Self::Expired => write!(f, "expired"),
            Self::Superseded => write!(f, "superseded"),
            Self::Compacted => write!(f, "compacted"),
            Self::RegulatoryErasure => write!(f, "regulatory_erasure"),
        }
    }
}

/// A persistent marker of deletion.
///
/// ## Tier: T2-C (void + pi + exists + irrev)
/// Dominant: void (this is fundamentally about recorded absence)
///
/// Innovation: fills the Void x Persistence gap.
/// Void CAN persist — as proof that something was intentionally removed.
#[derive(Debug, Clone)]
pub struct Tombstone {
    /// Unique tombstone ID.
    pub id: TombstoneId,
    /// Key of the deleted entity.
    pub entity_key: String,
    /// When the entity was deleted (monotonic timestamp).
    pub deleted_at: u64,
    /// Why it was deleted.
    pub reason: DeletionReason,
    /// When this tombstone itself should be garbage collected (0 = never).
    pub gc_after: u64,
    /// Hash of the deleted content (proof of prior existence without storing it).
    pub content_hash: Option<u64>,
}

impl Tombstone {
    /// Create a new tombstone.
    #[must_use]
    pub fn new(
        id: TombstoneId,
        entity_key: impl Into<String>,
        deleted_at: u64,
        reason: DeletionReason,
    ) -> Self {
        Self {
            id,
            entity_key: entity_key.into(),
            deleted_at,
            reason,
            gc_after: 0,
            content_hash: None,
        }
    }

    /// Set GC expiry for this tombstone.
    #[must_use]
    pub fn with_gc_after(mut self, gc_after: u64) -> Self {
        self.gc_after = gc_after;
        self
    }

    /// Set content hash (proof of prior existence).
    #[must_use]
    pub fn with_content_hash(mut self, hash: u64) -> Self {
        self.content_hash = Some(hash);
        self
    }

    /// Whether this tombstone itself has expired and can be GC'd.
    #[must_use]
    pub fn is_gc_eligible(&self, now: u64) -> bool {
        self.gc_after > 0 && now >= self.gc_after
    }
}

impl fmt::Display for Tombstone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tombstone[{}] '{}' deleted@{} ({})",
            self.id, self.entity_key, self.deleted_at, self.reason,
        )
    }
}

/// A registry of tombstones for tracking deletions.
///
/// ## Tier: T2-C (void + pi + mu + exists)
#[derive(Debug, Clone)]
pub struct TombstoneRegistry {
    /// Tombstones by ID.
    tombstones: BTreeMap<TombstoneId, Tombstone>,
    /// Entity key -> tombstone ID lookup.
    key_index: BTreeMap<String, TombstoneId>,
    /// Next tombstone ID.
    next_id: TombstoneId,
}

impl TombstoneRegistry {
    /// Create a new tombstone registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tombstones: BTreeMap::new(),
            key_index: BTreeMap::new(),
            next_id: 0,
        }
    }

    /// Record a deletion (create a tombstone).
    pub fn delete(
        &mut self,
        entity_key: impl Into<String>,
        timestamp: u64,
        reason: DeletionReason,
    ) -> TombstoneId {
        let key = entity_key.into();
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let tombstone = Tombstone::new(id, key.clone(), timestamp, reason);
        self.tombstones.insert(id, tombstone);
        self.key_index.insert(key, id);

        id
    }

    /// Check if an entity has been deleted.
    #[must_use]
    pub fn is_deleted(&self, entity_key: &str) -> bool {
        self.key_index.contains_key(entity_key)
    }

    /// Get the tombstone for a deleted entity.
    #[must_use]
    pub fn get_tombstone(&self, entity_key: &str) -> Option<&Tombstone> {
        self.key_index
            .get(entity_key)
            .and_then(|id| self.tombstones.get(id))
    }

    /// Garbage-collect expired tombstones.
    /// Returns the number of tombstones removed.
    pub fn gc(&mut self, now: u64) -> usize {
        let expired: Vec<TombstoneId> = self
            .tombstones
            .iter()
            .filter(|(_, t)| t.is_gc_eligible(now))
            .map(|(id, _)| *id)
            .collect();

        let count = expired.len();

        for id in expired {
            if let Some(tombstone) = self.tombstones.remove(&id) {
                self.key_index.remove(&tombstone.entity_key);
            }
        }

        count
    }

    /// Total tombstones.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tombstones.len()
    }

    /// Whether empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tombstones.is_empty()
    }

    /// All deletion reasons with counts.
    #[must_use]
    pub fn reason_summary(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for t in self.tombstones.values() {
            let key = format!("{}", t.reason);
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for TombstoneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tombstone() {
        let mut reg = TombstoneRegistry::new();

        let id = reg.delete("user:42", 1000, DeletionReason::UserAction("admin".into()));
        assert_eq!(id, 0);
        assert!(reg.is_deleted("user:42"));
        assert!(!reg.is_deleted("user:43"));
    }

    #[test]
    fn test_get_tombstone_details() {
        let mut reg = TombstoneRegistry::new();
        reg.delete("session:abc", 5000, DeletionReason::Expired);

        let tombstone = reg.get_tombstone("session:abc");
        assert!(tombstone.is_some());
        let t = tombstone.unwrap_or_else(|| {
            static EMPTY: Tombstone = Tombstone {
                id: 0,
                entity_key: String::new(),
                deleted_at: 0,
                reason: DeletionReason::Expired,
                gc_after: 0,
                content_hash: None,
            };
            &EMPTY
        });
        assert_eq!(t.reason, DeletionReason::Expired);
        assert_eq!(t.deleted_at, 5000);
    }

    #[test]
    fn test_gc_expired_tombstones() {
        let mut reg = TombstoneRegistry::new();

        // Create tombstone that GC's at time 10000
        let id = reg.delete("old", 1000, DeletionReason::Compacted);
        if let Some(t) = reg.tombstones.get_mut(&id) {
            t.gc_after = 10000;
        }

        // Create tombstone that GC's never
        reg.delete("permanent", 2000, DeletionReason::RegulatoryErasure);

        assert_eq!(reg.len(), 2);

        // GC at time 15000
        let removed = reg.gc(15000);
        assert_eq!(removed, 1);
        assert_eq!(reg.len(), 1);
        assert!(!reg.is_deleted("old"));
        assert!(reg.is_deleted("permanent"));
    }

    #[test]
    fn test_reason_summary() {
        let mut reg = TombstoneRegistry::new();
        reg.delete("a", 1, DeletionReason::Expired);
        reg.delete("b", 2, DeletionReason::Expired);
        reg.delete("c", 3, DeletionReason::Compacted);

        let summary = reg.reason_summary();
        assert_eq!(summary.get("expired"), Some(&2));
        assert_eq!(summary.get("compacted"), Some(&1));
    }
}
