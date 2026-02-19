//! T2-C: Correction — Auditable state correction record.
//!
//! Codex XI (CORRECT): All state is correctable.

use serde::{Deserialize, Serialize};

/// Correction record for auditability (Codex XI compliance).
///
/// All state in the system must be correctable. This type records
/// the correction with full provenance for audit trails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction<T> {
    /// The original value before correction.
    pub original: T,
    /// The corrected value.
    pub corrected: T,
    /// Reason for the correction.
    pub reason: String,
    /// Unix timestamp of correction.
    pub timestamp: u64,
}

impl<T> Correction<T> {
    /// Create a new correction record with current timestamp.
    pub fn now(original: T, corrected: T, reason: impl Into<String>) -> Self {
        Self {
            original,
            corrected,
            reason: reason.into(),
            timestamp: Self::unix_secs(),
        }
    }

    /// Create a correction record with explicit timestamp.
    pub fn new(original: T, corrected: T, reason: impl Into<String>, timestamp: u64) -> Self {
        Self {
            original,
            corrected,
            reason: reason.into(),
            timestamp,
        }
    }

    /// Current Unix seconds. Returns 0 only if system clock is pre-epoch.
    fn unix_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    }

    /// Apply the correction, consuming self and returning the corrected value.
    pub fn apply(self) -> T {
        self.corrected
    }

    /// Get reference to corrected value without consuming.
    pub fn corrected(&self) -> &T {
        &self.corrected
    }

    /// Get reference to original value.
    pub fn original(&self) -> &T {
        &self.original
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_change() {
        let correction = Correction::now(10, 20, "Fixed off-by-one");
        assert_eq!(*correction.original(), 10);
        assert_eq!(*correction.corrected(), 20);
        assert_eq!(correction.reason, "Fixed off-by-one");
        assert_eq!(correction.apply(), 20);
    }

    #[test]
    fn timestamp_is_nonzero() {
        let correction = Correction::now("old", "new", "update");
        assert!(correction.timestamp > 0);
    }

    #[test]
    fn explicit_timestamp() {
        let c = Correction::new(1, 2, "fix", 1700000000);
        assert_eq!(c.timestamp, 1700000000);
    }

    #[test]
    fn serde_round_trip() {
        let c = Correction::now(42, 43, "increment");
        let json = serde_json::to_string(&c).unwrap();
        let back: Correction<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(*back.original(), 42);
        assert_eq!(*back.corrected(), 43);
    }
}
