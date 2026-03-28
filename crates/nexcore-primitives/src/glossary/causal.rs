//! # Causal and Temporal Primitives
//!
//! Type-level encodings of T1 causal and temporal concepts from Lex Primitiva:
//! Causality (→), Sequence (σ), Persistence (π), and Irreversibility (∝).
//!
//! These types enforce temporal ordering and mutation semantics at compile time.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// ─── Timestamp helper ────────────────────────────────────────────────────────

/// Returns the current time as Unix milliseconds.
/// Falls back to 0 if the system clock is before the epoch.
#[inline]
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

// ─── Effect ──────────────────────────────────────────────────────────────────

/// An observed outcome — the causal result of an action (→ + ∃ + ς).
///
/// `Effect<T>` distinguishes between outcomes that were anticipated and those
/// that arrived unexpectedly. The `predicted` flag encodes whether the effect
/// was within the hypothesis at the time of observation.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::causal::Effect;
///
/// let e = Effect::expected(42u32);
/// assert!(e.was_predicted());
///
/// let u = Effect::unexpected("signal_spike");
/// assert!(!u.was_predicted());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect<T> {
    /// The observed value.
    pub value: T,
    /// Whether the effect matched the prediction at observation time.
    pub predicted: bool,
    /// Unix milliseconds at which the effect was recorded.
    pub observed_at: u64,
}

impl<T> Effect<T> {
    /// Construct an expected (predicted) effect observed now.
    #[inline]
    pub fn expected(value: T) -> Self {
        Self {
            value,
            predicted: true,
            observed_at: now_ms(),
        }
    }

    /// Construct an unexpected (unpredicted) effect observed now.
    #[inline]
    pub fn unexpected(value: T) -> Self {
        Self {
            value,
            predicted: false,
            observed_at: now_ms(),
        }
    }

    /// Returns `true` when this effect was anticipated.
    #[inline]
    pub fn was_predicted(&self) -> bool {
        self.predicted
    }
}

// ─── Sequence ────────────────────────────────────────────────────────────────

/// A temporally ordered collection (σ + ν).
///
/// Each entry is stamped with a Unix millisecond timestamp at the moment of
/// insertion. The ordering contract is append-only: there is no random access
/// by index. Callers reason about the stream, not its positions.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::causal::Sequence;
///
/// let mut seq: Sequence<&str> = Sequence::new();
/// seq.push("first");
/// seq.push("second");
/// assert_eq!(seq.len(), 2);
/// assert_eq!(*seq.latest().unwrap(), "second");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence<T> {
    entries: Vec<(u64, T)>,
}

impl<T> Sequence<T> {
    /// Create an empty sequence.
    #[inline]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append a value, auto-stamping with the current time.
    #[inline]
    pub fn push(&mut self, value: T) {
        self.entries.push((now_ms(), value));
    }

    /// Number of entries in the sequence.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when the sequence contains no entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over `(timestamp_ms, value)` pairs in insertion order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &(u64, T)> {
        self.entries.iter()
    }

    /// Reference to the most recently inserted value, or `None` if empty.
    #[inline]
    pub fn latest(&self) -> Option<&T> {
        self.entries.last().map(|(_, v)| v)
    }
}

impl<T> Default for Sequence<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ─── Persistence ─────────────────────────────────────────────────────────────

/// Tracks whether a value has been saved (π + ς).
///
/// A `Persistence<T>` wraps a value with a dirty flag. Calling `update` marks
/// the wrapper as needing a save. Calling `mark_persisted` clears the flag.
/// This pattern separates "has the data changed" from "has the change been
/// committed to storage".
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::causal::Persistence;
///
/// let mut p = Persistence::new("draft");
/// assert!(!p.is_dirty());   // fresh, not yet modified
///
/// p.update("revision 1");
/// assert!(p.is_dirty());
///
/// p.mark_persisted();
/// assert!(!p.is_dirty());
/// assert_eq!(*p.value(), "revision 1");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persistence<T: Clone> {
    value: T,
    /// Unix milliseconds at creation.
    created_at: u64,
    /// Unix milliseconds of last update.
    updated_at: u64,
    /// `true` when `value` has been changed since the last `mark_persisted`.
    dirty: bool,
}

impl<T: Clone> Persistence<T> {
    /// Wrap a value. Initially clean (not dirty).
    #[inline]
    pub fn new(value: T) -> Self {
        let now = now_ms();
        Self {
            value,
            created_at: now,
            updated_at: now,
            dirty: false,
        }
    }

    /// Replace the inner value, marking the wrapper dirty.
    #[inline]
    pub fn update(&mut self, value: T) {
        self.value = value;
        self.updated_at = now_ms();
        self.dirty = true;
    }

    /// Acknowledge that the current value has been written to storage.
    #[inline]
    pub fn mark_persisted(&mut self) {
        self.dirty = false;
    }

    /// Returns `true` when the value has unsaved changes.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Borrow the inner value.
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }
}

// ─── Irreversibility ─────────────────────────────────────────────────────────

/// A marker for an action that cannot be undone (∝).
///
/// Once constructed, `Irreversibility` is immutable — all fields are private
/// and accessible only through read-only getters. There is no `update`,
/// `revert`, or `cancel`. This enforces the invariant at the type level:
/// if you hold an `Irreversibility`, the action has already been committed.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::causal::Irreversibility;
///
/// let i = Irreversibility::commit("delete_patient_record", "audit-ordered cleanup");
/// assert_eq!(i.action(), "delete_patient_record");
/// assert_eq!(i.reason(), "audit-ordered cleanup");
/// assert!(i.committed_at() > 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Irreversibility {
    action: String,
    committed_at: u64,
    reason: String,
}

impl Irreversibility {
    /// Record an irreversible action with its justification.
    ///
    /// Once this value is constructed the action is considered committed.
    /// There is no mutation path.
    #[inline]
    pub fn commit(action: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            committed_at: now_ms(),
            reason: reason.into(),
        }
    }

    /// The name of the action that was committed.
    #[inline]
    pub fn action(&self) -> &str {
        &self.action
    }

    /// Unix milliseconds at which the action was committed.
    #[inline]
    pub fn committed_at(&self) -> u64 {
        self.committed_at
    }

    /// The stated justification for the irreversible action.
    #[inline]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Effect

    #[test]
    fn effect_expected_is_predicted() {
        let e = Effect::expected(100u32);
        assert!(e.was_predicted());
        assert_eq!(e.value, 100);
    }

    #[test]
    fn effect_unexpected_is_not_predicted() {
        let e = Effect::unexpected("anomaly");
        assert!(!e.was_predicted());
        assert_eq!(e.value, "anomaly");
    }

    #[test]
    fn effect_observed_at_is_nonzero() {
        let e = Effect::expected(0u8);
        // Timestamps after 2020-01-01 00:00:00 UTC = 1_577_836_800_000 ms
        assert!(e.observed_at >= 1_577_836_800_000);
    }

    #[test]
    fn effect_clone_is_independent() {
        let original = Effect::expected(42u32);
        let cloned = original.clone();
        assert_eq!(original.value, cloned.value);
        assert_eq!(original.predicted, cloned.predicted);
    }

    // Sequence

    #[test]
    fn sequence_new_is_empty() {
        let seq: Sequence<u32> = Sequence::new();
        assert!(seq.is_empty());
        assert_eq!(seq.len(), 0);
        assert!(seq.latest().is_none());
    }

    #[test]
    fn sequence_push_appends_in_order() {
        let mut seq = Sequence::new();
        seq.push(1u32);
        seq.push(2u32);
        seq.push(3u32);
        assert_eq!(seq.len(), 3);
        assert_eq!(*seq.latest().unwrap(), 3);
    }

    #[test]
    fn sequence_iter_yields_all_entries() {
        let mut seq = Sequence::new();
        seq.push("a");
        seq.push("b");
        let values: Vec<&str> = seq.iter().map(|(_, v)| *v).collect();
        assert_eq!(values, vec!["a", "b"]);
    }

    #[test]
    fn sequence_timestamps_are_nondecreasing() {
        let mut seq = Sequence::new();
        for i in 0u32..5 {
            seq.push(i);
        }
        let timestamps: Vec<u64> = seq.iter().map(|(ts, _)| *ts).collect();
        for window in timestamps.windows(2) {
            assert!(window[0] <= window[1]);
        }
    }

    // Persistence

    #[test]
    fn persistence_new_is_clean() {
        let p = Persistence::new("initial");
        assert!(!p.is_dirty());
        assert_eq!(*p.value(), "initial");
    }

    #[test]
    fn persistence_update_marks_dirty() {
        let mut p = Persistence::new("v1");
        p.update("v2");
        assert!(p.is_dirty());
        assert_eq!(*p.value(), "v2");
    }

    #[test]
    fn persistence_mark_persisted_clears_dirty() {
        let mut p = Persistence::new(0u32);
        p.update(1u32);
        assert!(p.is_dirty());
        p.mark_persisted();
        assert!(!p.is_dirty());
        assert_eq!(*p.value(), 1);
    }

    #[test]
    fn persistence_multiple_updates_stay_dirty_until_persisted() {
        let mut p = Persistence::new(vec![0u8]);
        p.update(vec![1u8]);
        p.update(vec![2u8]);
        assert!(p.is_dirty());
        p.mark_persisted();
        assert!(!p.is_dirty());
    }

    // Irreversibility

    #[test]
    fn irreversibility_commit_stores_action_and_reason() {
        let i = Irreversibility::commit("purge_cache", "scheduled maintenance");
        assert_eq!(i.action(), "purge_cache");
        assert_eq!(i.reason(), "scheduled maintenance");
    }

    #[test]
    fn irreversibility_committed_at_is_recent() {
        let i = Irreversibility::commit("drop_index", "migration v12");
        assert!(i.committed_at() >= 1_577_836_800_000);
    }

    #[test]
    fn irreversibility_clone_preserves_fields() {
        let original = Irreversibility::commit("seal_audit", "end of quarter");
        let copy = original.clone();
        assert_eq!(original.action(), copy.action());
        assert_eq!(original.reason(), copy.reason());
        assert_eq!(original.committed_at(), copy.committed_at());
    }
}
