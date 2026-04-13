//! Pipeline-wide configuration for the perception system.

use serde::{Deserialize, Serialize};

/// Top-level configuration for the `PerceptionPipeline`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionConfig {
    /// Deduplication window in hours. Records with matching fingerprints
    /// that arrive within this window are considered duplicates.
    pub dedup_window_hours: f64,

    /// Minimum confidence required to merge two records instead of routing
    /// the lower-confidence one to the hold queue.
    pub merge_threshold: f64,

    /// Conflict delta above which two records are escalated to arbitration
    /// rather than being automatically fused.
    pub arbitration_threshold: f64,

    /// Maximum age in hours before a world-model entity is considered stale
    /// and marked `uncertain = true`.
    pub max_staleness_hours: f64,

    /// Channel buffer size (records) for each connector's ingestion channel.
    pub ingestion_buffer_size: usize,

    /// Maximum number of concurrent source connectors polled in parallel.
    pub max_concurrent_connectors: usize,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            dedup_window_hours: 24.0,
            merge_threshold: 0.6,
            arbitration_threshold: 0.4,
            max_staleness_hours: 72.0,
            ingestion_buffer_size: 1_000,
            max_concurrent_connectors: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = PerceptionConfig::default();
        assert!(cfg.dedup_window_hours > 0.0);
        assert!((0.0..=1.0).contains(&cfg.merge_threshold));
        assert!((0.0..=1.0).contains(&cfg.arbitration_threshold));
        assert!(cfg.max_staleness_hours > 0.0);
        assert!(cfg.ingestion_buffer_size > 0);
    }

    #[test]
    fn config_serializes_round_trip() {
        let cfg = PerceptionConfig::default();
        let json = serde_json::to_string(&cfg).expect("serialize failed");
        let back: PerceptionConfig = serde_json::from_str(&json).expect("deserialize failed");
        // f64 equality is fine here — same bit-pattern round-trip through JSON
        assert!((back.dedup_window_hours - cfg.dedup_window_hours).abs() < f64::EPSILON);
        assert!((back.merge_threshold - cfg.merge_threshold).abs() < f64::EPSILON);
    }
}
