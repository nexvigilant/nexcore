//! Core newtypes for the build orchestrator.
//!
//! ## Primitive Foundation
//! Each newtype wraps a single T1 primitive manifestation.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// Unique identifier for a pipeline run.
///
/// Tier: T2-P (π + ∃, dominant π)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineId(pub String);

impl PipelineId {
    /// Create a new pipeline ID from timestamp + random suffix.
    #[must_use]
    pub fn generate() -> Self {
        let ts = nexcore_chrono::DateTime::now()
            .format("%Y%m%d-%H%M%S")
            .unwrap_or_default();
        let suffix: u16 = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
            % 10000) as u16;
        Self(format!("run-{ts}-{suffix:04}"))
    }
}

impl fmt::Display for PipelineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Unique identifier for a stage within a pipeline.
///
/// Tier: T1 (σ, dominant σ — position in sequence)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StageId(pub String);

impl fmt::Display for StageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Duration wrapper with serialization support.
///
/// Tier: T1 (N, dominant N — pure quantity)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BuildDuration {
    pub millis: u64,
}

impl BuildDuration {
    /// Create from a std Duration.
    #[must_use]
    pub fn from_duration(d: Duration) -> Self {
        Self {
            millis: d.as_millis() as u64,
        }
    }

    /// Convert to std Duration.
    #[must_use]
    pub fn to_duration(self) -> Duration {
        Duration::from_millis(self.millis)
    }

    /// Human-readable format.
    #[must_use]
    pub fn display(&self) -> String {
        if self.millis < 1_000 {
            format!("{}ms", self.millis)
        } else if self.millis < 60_000 {
            format!("{:.1}s", self.millis as f64 / 1_000.0)
        } else {
            let mins = self.millis / 60_000;
            let secs = (self.millis % 60_000) / 1_000;
            format!("{mins}m {secs}s")
        }
    }
}

impl fmt::Display for BuildDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display())
    }
}

/// A chunk of log output from a build stage.
///
/// Tier: T2-P (σ + λ, dominant σ — sequential output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogChunk {
    /// Which stage produced this output.
    pub stage_id: StageId,
    /// The log text content.
    pub content: String,
    /// Whether this is stderr (true) or stdout (false).
    pub is_stderr: bool,
    /// Timestamp of capture.
    pub timestamp: nexcore_chrono::DateTime,
}

/// Stream of log chunks (type alias for collections).
pub type LogStream = Vec<LogChunk>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_id_generate_unique() {
        let a = PipelineId::generate();
        let b = PipelineId::generate();
        // While not guaranteed unique in same millisecond, format should be valid
        assert!(a.0.starts_with("run-"));
        assert!(b.0.starts_with("run-"));
    }

    #[test]
    fn pipeline_id_display() {
        let id = PipelineId("run-20260207-1234-5678".into());
        assert_eq!(format!("{id}"), "run-20260207-1234-5678");
    }

    #[test]
    fn stage_id_display() {
        let id = StageId("clippy".into());
        assert_eq!(format!("{id}"), "clippy");
    }

    #[test]
    fn build_duration_millis() {
        let d = BuildDuration { millis: 500 };
        assert_eq!(d.display(), "500ms");
    }

    #[test]
    fn build_duration_seconds() {
        let d = BuildDuration { millis: 3_500 };
        assert_eq!(d.display(), "3.5s");
    }

    #[test]
    fn build_duration_minutes() {
        let d = BuildDuration { millis: 125_000 };
        assert_eq!(d.display(), "2m 5s");
    }

    #[test]
    fn build_duration_roundtrip() {
        let original = std::time::Duration::from_millis(42_000);
        let bd = BuildDuration::from_duration(original);
        assert_eq!(bd.millis, 42_000);
        assert_eq!(bd.to_duration(), original);
    }

    #[test]
    fn pipeline_id_serde_roundtrip() {
        let id = PipelineId("run-test-123".into());
        let json = serde_json::to_string(&id);
        assert!(json.is_ok());
        let back: Result<PipelineId, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(back.is_ok());
        assert_eq!(back.unwrap_or(PipelineId("x".into())).0, "run-test-123");
    }

    #[test]
    fn build_duration_serde_roundtrip() {
        let d = BuildDuration { millis: 9999 };
        let json = serde_json::to_string(&d);
        assert!(json.is_ok());
        let back: Result<BuildDuration, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(back.is_ok());
        assert_eq!(back.unwrap_or(BuildDuration { millis: 0 }).millis, 9999);
    }

    #[test]
    fn log_chunk_serde_roundtrip() {
        let chunk = LogChunk {
            stage_id: StageId("test".into()),
            content: "hello world".into(),
            is_stderr: false,
            timestamp: nexcore_chrono::DateTime::now(),
        };
        let json = serde_json::to_string(&chunk);
        assert!(json.is_ok());
        let back: Result<LogChunk, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(back.is_ok());
    }
}
