//! BMS telemetry source abstraction.
//!
//! The power compound used to read constants. v0.2 introduces a `BmsSource`
//! trait so the loop can swap among:
//!
//! - `MockBmsSource`   — recorded JSON-trace replay (default, smoke-stable)
//! - `SerialBmsSource` — JSON-over-serial bench rig (TODO v0.3)
//! - `CanBmsSource`    — CAN 11-bit identifier frames (TODO v0.4)
//!
//! Public contract: `poll()` returns one frame per call, never blocks beyond
//! its own timeout, and never panics.

use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::sync::Mutex;

/// One BMS sample. `tier` is computed at the source from cell-level data;
/// the loop trusts it as authoritative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmsFrame {
    /// Pack voltage, V.
    pub pack_voltage_v: f32,
    /// Pack current, A. Positive = draw, negative = charge.
    pub pack_current_a: f32,
    /// Cell temperature, °C (hottest cell).
    pub cell_temp_c: f32,
    /// State of charge, 0..=100.
    pub soc_pct: f32,
    /// State of health, 0..=100.
    pub soh_pct: f32,
    /// Active load tier, source-of-truth from BMS.
    pub tier: PowerTier,
    /// Wall clock at frame ingest. Skipped on serialize for replay determinism.
    #[serde(skip)]
    pub ts: Option<Instant>,
}

/// Power tier — what the BMS thinks the current load classification is.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PowerTier {
    /// Communications-only (lowest draw).
    Comms,
    /// Compute-class workload.
    Compute,
    /// Actuation engaged.
    Actuation,
    /// Critical maneuver — full draw allowed.
    Critical,
}

impl PowerTier {
    /// Stable label for snapshot serialization.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Comms => "Comms",
            Self::Compute => "Compute",
            Self::Actuation => "Actuation",
            Self::Critical => "Critical",
        }
    }
}

/// Health classification of the source itself (not the battery).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceHealth {
    /// Source is alive and last poll succeeded.
    Alive,
    /// Source is reachable but last frame was stale.
    Degraded,
    /// Source is offline.
    Down,
}

/// Errors a `poll()` can return.
#[derive(Debug)]
pub enum BmsError {
    /// Frame did not arrive within the loop's tick budget.
    Timeout,
    /// BMS reported a fault code; degrade the snapshot.
    Fault(u16),
    /// Source backend is exhausted (e.g. trace ran out, connection closed).
    Exhausted,
}

impl core::fmt::Display for BmsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Timeout => write!(f, "bms timeout"),
            Self::Fault(c) => write!(f, "bms fault code {c}"),
            Self::Exhausted => write!(f, "bms source exhausted"),
        }
    }
}

impl std::error::Error for BmsError {}

/// Trait every BMS backend implements.
pub trait BmsSource: Send + Sync {
    /// Pull one frame. Awaitable so backends can do real I/O without blocking
    /// the tokio runtime.
    fn poll(
        &self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<BmsFrame, BmsError>> + Send + '_>,
    >;

    /// Snapshot of the source's own health (not the battery's).
    fn health(&self) -> SourceHealth;
}

/// Mock backend: cycles through a pre-loaded trace, looping when exhausted.
/// Default for `--bms-source mock`. Used by smoke tests to keep behavior
/// deterministic across runs.
pub struct MockBmsSource {
    frames: Mutex<TraceCursor>,
}

struct TraceCursor {
    frames: Vec<BmsFrame>,
    idx: usize,
}

impl MockBmsSource {
    /// Build with a baked-in 5-frame trace approximating a steady drain.
    /// SoC slopes from 100→96 across the trace; the loop sees this as live data.
    #[must_use]
    pub fn new() -> Self {
        let frames = vec![
            BmsFrame {
                pack_voltage_v: 400.0,
                pack_current_a: 10.0,
                cell_temp_c: 25.0,
                soc_pct: 100.0,
                soh_pct: 100.0,
                tier: PowerTier::Comms,
                ts: None,
            },
            BmsFrame {
                pack_voltage_v: 399.5,
                pack_current_a: 10.5,
                cell_temp_c: 25.2,
                soc_pct: 99.0,
                soh_pct: 100.0,
                tier: PowerTier::Comms,
                ts: None,
            },
            BmsFrame {
                pack_voltage_v: 398.8,
                pack_current_a: 11.0,
                cell_temp_c: 25.6,
                soc_pct: 98.0,
                soh_pct: 99.9,
                tier: PowerTier::Compute,
                ts: None,
            },
            BmsFrame {
                pack_voltage_v: 398.0,
                pack_current_a: 11.5,
                cell_temp_c: 26.1,
                soc_pct: 97.0,
                soh_pct: 99.9,
                tier: PowerTier::Compute,
                ts: None,
            },
            BmsFrame {
                pack_voltage_v: 397.2,
                pack_current_a: 12.0,
                cell_temp_c: 26.5,
                soc_pct: 96.0,
                soh_pct: 99.8,
                tier: PowerTier::Actuation,
                ts: None,
            },
        ];
        Self {
            frames: Mutex::new(TraceCursor { frames, idx: 0 }),
        }
    }

    /// Construct from an explicit trace (used by integration tests).
    #[must_use]
    pub fn with_trace(frames: Vec<BmsFrame>) -> Self {
        Self {
            frames: Mutex::new(TraceCursor { frames, idx: 0 }),
        }
    }
}

impl Default for MockBmsSource {
    fn default() -> Self {
        Self::new()
    }
}

impl BmsSource for MockBmsSource {
    fn poll(
        &self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<BmsFrame, BmsError>> + Send + '_>,
    > {
        Box::pin(async move {
            let mut cursor = self.frames.lock().await;
            if cursor.frames.is_empty() {
                return Err(BmsError::Exhausted);
            }
            let mut frame = cursor.frames[cursor.idx].clone();
            frame.ts = Some(Instant::now());
            cursor.idx = (cursor.idx + 1) % cursor.frames.len();
            Ok(frame)
        })
    }

    fn health(&self) -> SourceHealth {
        SourceHealth::Alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_source_yields_decreasing_soc_across_first_pass() {
        let src = MockBmsSource::new();
        let mut last = f32::INFINITY;
        for _ in 0..5 {
            let f = src.poll().await.expect("poll");
            assert!(f.soc_pct <= last, "soc should not increase: {} vs {last}", f.soc_pct);
            last = f.soc_pct;
        }
    }

    #[tokio::test]
    async fn mock_source_loops_after_exhaustion() {
        let src = MockBmsSource::new();
        for _ in 0..5 {
            src.poll().await.expect("first pass");
        }
        // Sixth call should wrap around to frame 0.
        let frame_six = src.poll().await.expect("wraparound");
        assert_eq!(frame_six.soc_pct, 100.0);
    }

    #[test]
    fn power_tier_labels_are_stable() {
        assert_eq!(PowerTier::Comms.label(), "Comms");
        assert_eq!(PowerTier::Compute.label(), "Compute");
        assert_eq!(PowerTier::Actuation.label(), "Actuation");
        assert_eq!(PowerTier::Critical.label(), "Critical");
    }
}
