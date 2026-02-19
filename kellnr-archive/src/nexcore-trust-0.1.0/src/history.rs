/// Trust audit trail with ring buffer storage.
///
/// Records evidence events with before/after state for forensic analysis.
/// Addresses the lossy accumulation problem: once alpha/beta are updated,
/// the individual evidence events are normally destroyed.
///
/// Fixes: Gap #6 (Audit Trail / History).
///
/// Tier: T2-C (Sequence s + Persistence p + State c)
use crate::evidence::Evidence;
use crate::level::TrustLevel;

/// A single entry in the trust audit trail.
#[derive(Debug, Clone, Copy)]
pub struct AuditEntry {
    /// The evidence that was recorded
    pub evidence: Evidence,
    /// Timestamp (abstract time units) when the evidence was recorded
    pub timestamp: f64,
    /// Trust score before this evidence was applied
    pub score_before: f64,
    /// Trust score after this evidence was applied
    pub score_after: f64,
    /// Trust level before
    pub level_before: TrustLevel,
    /// Trust level after
    pub level_after: TrustLevel,
}

impl AuditEntry {
    /// Score delta caused by this evidence.
    pub fn delta(&self) -> f64 {
        self.score_after - self.score_before
    }

    /// Whether this evidence caused a level transition.
    pub fn caused_transition(&self) -> bool {
        self.level_before != self.level_after
    }
}

impl core::fmt::Display for AuditEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "t={:.1}: {} | {:.4} -> {:.4} ({}{})",
            self.timestamp,
            self.evidence,
            self.score_before,
            self.score_after,
            self.level_after,
            if self.caused_transition() {
                format!(" [was {}]", self.level_before)
            } else {
                String::new()
            },
        )
    }
}

/// Ring buffer of trust audit entries.
///
/// Stores the most recent `capacity` entries. When full, oldest entries
/// are overwritten. This bounds memory usage while preserving recent history.
#[derive(Debug, Clone)]
pub struct TrustHistory {
    entries: Vec<AuditEntry>,
    capacity: usize,
    /// Write position (wraps at capacity)
    write_pos: usize,
    /// Total entries ever recorded (may exceed capacity)
    total_recorded: u64,
}

impl TrustHistory {
    /// Create a new history with the given maximum capacity.
    ///
    /// Capacity of 0 is treated as 1 (minimum).
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity.max(1).min(10_000)),
            capacity: capacity.max(1),
            write_pos: 0,
            total_recorded: 0,
        }
    }

    /// Record a new audit entry.
    ///
    /// If the buffer is full, the oldest entry is overwritten.
    pub fn record(&mut self, entry: AuditEntry) {
        if self.entries.len() < self.capacity {
            self.entries.push(entry);
        } else {
            self.entries[self.write_pos] = entry;
        }
        self.write_pos = (self.write_pos + 1) % self.capacity;
        self.total_recorded += 1;
    }

    /// Get all entries in chronological order.
    ///
    /// Returns a Vec because ring buffer entries may wrap around.
    pub fn entries(&self) -> Vec<AuditEntry> {
        if self.entries.len() < self.capacity {
            // Not yet wrapped — entries are already in order
            self.entries.clone()
        } else {
            // Wrapped — reorder from write_pos
            let mut result = Vec::with_capacity(self.entries.len());
            result.extend_from_slice(&self.entries[self.write_pos..]);
            result.extend_from_slice(&self.entries[..self.write_pos]);
            result
        }
    }

    /// Number of entries currently stored (up to capacity).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total entries ever recorded (may exceed current len if wrapped).
    pub fn total_recorded(&self) -> u64 {
        self.total_recorded
    }

    /// Whether entries have been lost due to ring buffer overflow.
    pub fn has_overflow(&self) -> bool {
        self.total_recorded > self.capacity as u64
    }

    /// Find all level transitions in the history.
    ///
    /// Returns (from_level, to_level, timestamp) for each transition.
    pub fn level_transitions(&self) -> Vec<(TrustLevel, TrustLevel, f64)> {
        self.entries()
            .iter()
            .filter(|e| e.caused_transition())
            .map(|e| (e.level_before, e.level_after, e.timestamp))
            .collect()
    }

    /// Most recent entry, if any.
    pub fn latest(&self) -> Option<&AuditEntry> {
        if self.entries.is_empty() {
            return None;
        }
        let idx = if self.write_pos == 0 {
            self.entries.len() - 1
        } else {
            self.write_pos - 1
        };
        self.entries.get(idx)
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.write_pos = 0;
        self.total_recorded = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(evidence: Evidence, t: f64, before: f64, after: f64) -> AuditEntry {
        AuditEntry {
            evidence,
            timestamp: t,
            score_before: before,
            score_after: after,
            level_before: TrustLevel::from_score(before),
            level_after: TrustLevel::from_score(after),
        }
    }

    #[test]
    fn empty_history() {
        let h = TrustHistory::new(10);
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
        assert_eq!(h.total_recorded(), 0);
        assert!(h.latest().is_none());
        assert!(h.level_transitions().is_empty());
    }

    #[test]
    fn records_and_retrieves() {
        let mut h = TrustHistory::new(10);
        h.record(make_entry(Evidence::positive(), 1.0, 0.5, 0.55));
        h.record(make_entry(Evidence::negative(), 2.0, 0.55, 0.4));

        assert_eq!(h.len(), 2);
        assert_eq!(h.total_recorded(), 2);

        let entries = h.entries();
        assert_eq!(entries.len(), 2);
        assert!((entries[0].timestamp - 1.0).abs() < f64::EPSILON);
        assert!((entries[1].timestamp - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ring_buffer_wraps() {
        let mut h = TrustHistory::new(3);

        for i in 0..5 {
            h.record(make_entry(
                Evidence::positive(),
                i as f64,
                0.5,
                0.5 + 0.01 * (i as f64),
            ));
        }

        // Should only have last 3 entries
        assert_eq!(h.len(), 3);
        assert_eq!(h.total_recorded(), 5);
        assert!(h.has_overflow());

        let entries = h.entries();
        assert!((entries[0].timestamp - 2.0).abs() < f64::EPSILON);
        assert!((entries[1].timestamp - 3.0).abs() < f64::EPSILON);
        assert!((entries[2].timestamp - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn latest_returns_most_recent() {
        let mut h = TrustHistory::new(10);
        h.record(make_entry(Evidence::positive(), 1.0, 0.5, 0.55));
        h.record(make_entry(Evidence::negative(), 2.0, 0.55, 0.4));

        let latest = h.latest();
        assert!(latest.is_some());
        if let Some(e) = latest {
            assert!((e.timestamp - 2.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn detects_level_transitions() {
        let mut h = TrustHistory::new(10);
        // No transition: stays Neutral
        h.record(make_entry(Evidence::positive(), 1.0, 0.5, 0.55));
        // Transition: Neutral -> Trusted
        h.record(make_entry(Evidence::positive(), 2.0, 0.55, 0.65));
        // No transition: stays Trusted
        h.record(make_entry(Evidence::positive(), 3.0, 0.65, 0.7));

        let transitions = h.level_transitions();
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].0, TrustLevel::Neutral);
        assert_eq!(transitions[0].1, TrustLevel::Trusted);
    }

    #[test]
    fn entry_delta_and_transition() {
        let e = make_entry(Evidence::negative(), 1.0, 0.65, 0.35);
        assert!((e.delta() - (-0.3)).abs() < f64::EPSILON);
        assert!(e.caused_transition()); // Trusted -> Suspicious
    }

    #[test]
    fn clear_resets_everything() {
        let mut h = TrustHistory::new(10);
        h.record(make_entry(Evidence::positive(), 1.0, 0.5, 0.55));
        h.clear();
        assert!(h.is_empty());
        assert_eq!(h.total_recorded(), 0);
        assert!(!h.has_overflow());
    }

    #[test]
    fn display_formatting() {
        let e = make_entry(Evidence::positive(), 5.0, 0.55, 0.65);
        let s = format!("{e}");
        assert!(s.contains("t=5.0"));
        assert!(s.contains("+1.00"));
    }
}
