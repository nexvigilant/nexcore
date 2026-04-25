//! BMS telemetry source abstraction.
//!
//! v0.3 backends:
//!
//! - `MockBmsSource`   — baked-in 5-frame trace, smoke-stable default
//! - `ReplayBmsSource` — NDJSON file replay at recorded cadence (or `--speedup N`)
//! - `SerialBmsSource` — JSON-over-serial via `tokio-serial`; loopback-validated
//!                       through `stark-suit-test-pty`. Hardware fidelity
//!                       (real USB-serial adapter) is a follow-on layer —
//!                       same code path, different device argument.
//!
//! Wire format: NDJSON, UTF-8, one `BmsFrame` JSON object per line. Every frame
//! carries a `version: u32` (currently `1`) and `ts_ms: u64` (epoch millis).
//! See `docs/bms_frame_v1.md`.

use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Current wire-format schema version. Bump when fields change incompatibly.
pub const FRAME_VERSION: u32 = 1;

/// One BMS sample. `tier` is computed at the source from cell-level data;
/// the loop trusts it as authoritative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmsFrame {
    /// Wire-format schema version. Mandatory from day 1 — see `docs/bms_frame_v1.md`.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Epoch millis at frame ingest. Serializable, deterministic.
    pub ts_ms: u64,
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
    /// Wall-clock at frame ingest. Skipped on serialize (use `ts_ms` instead).
    #[serde(skip)]
    pub ts: Option<Instant>,
}

fn default_version() -> u32 {
    FRAME_VERSION
}

/// Now in epoch millis. Saturates at u64::MAX if clock is before 1970.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
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
    /// Replay trace was empty.
    EmptyTrace,
    /// Replay reached end of recorded frames.
    ReplayExhausted,
    /// Frame schema version is not supported by this build.
    UnsupportedVersion(u32),
    /// I/O failure on the source backend.
    Io(String),
}

impl core::fmt::Display for BmsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Timeout => write!(f, "bms timeout"),
            Self::Fault(c) => write!(f, "bms fault code {c}"),
            Self::Exhausted => write!(f, "bms source exhausted"),
            Self::EmptyTrace => write!(f, "replay trace was empty"),
            Self::ReplayExhausted => write!(f, "replay reached end of trace"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported frame version {v}"),
            Self::Io(s) => write!(f, "bms i/o: {s}"),
        }
    }
}

impl std::error::Error for BmsError {}

/// Trait every BMS backend implements.
pub trait BmsSource: Send + Sync {
    /// Pull one frame. Awaitable so backends can do real I/O without blocking
    /// the tokio runtime.
    fn poll(&self) -> Pin<Box<dyn std::future::Future<Output = Result<BmsFrame, BmsError>> + Send + '_>>;

    /// Snapshot of the source's own health (not the battery's).
    fn health(&self) -> SourceHealth;
}

/// Mock backend: cycles through a baked-in trace, looping when exhausted.
pub struct MockBmsSource {
    frames: Mutex<TraceCursor>,
}

struct TraceCursor {
    frames: Vec<BmsFrame>,
    idx: usize,
}

fn baked_frame(
    pack_voltage_v: f32,
    pack_current_a: f32,
    cell_temp_c: f32,
    soc_pct: f32,
    soh_pct: f32,
    tier: PowerTier,
) -> BmsFrame {
    BmsFrame {
        version: FRAME_VERSION,
        ts_ms: 0,
        pack_voltage_v,
        pack_current_a,
        cell_temp_c,
        soc_pct,
        soh_pct,
        tier,
        ts: None,
    }
}

impl MockBmsSource {
    /// Build with a baked-in 5-frame trace approximating a steady drain.
    #[must_use]
    pub fn new() -> Self {
        let frames = vec![
            baked_frame(400.0, 10.0, 25.0, 100.0, 100.0, PowerTier::Comms),
            baked_frame(399.5, 10.5, 25.2, 99.0, 100.0, PowerTier::Comms),
            baked_frame(398.8, 11.0, 25.6, 98.0, 99.9, PowerTier::Compute),
            baked_frame(398.0, 11.5, 26.1, 97.0, 99.9, PowerTier::Compute),
            baked_frame(397.2, 12.0, 26.5, 96.0, 99.8, PowerTier::Actuation),
        ];
        Self { frames: Mutex::new(TraceCursor { frames, idx: 0 }) }
    }

    /// Construct from an explicit trace (used by integration tests).
    #[must_use]
    pub fn with_trace(frames: Vec<BmsFrame>) -> Self {
        Self { frames: Mutex::new(TraceCursor { frames, idx: 0 }) }
    }
}

impl Default for MockBmsSource {
    fn default() -> Self {
        Self::new()
    }
}

impl BmsSource for MockBmsSource {
    fn poll(&self) -> Pin<Box<dyn std::future::Future<Output = Result<BmsFrame, BmsError>> + Send + '_>> {
        Box::pin(async move {
            let mut cursor = self.frames.lock().await;
            if cursor.frames.is_empty() {
                return Err(BmsError::Exhausted);
            }
            let mut frame = cursor.frames[cursor.idx].clone();
            frame.ts = Some(Instant::now());
            frame.ts_ms = now_ms();
            cursor.idx = (cursor.idx + 1) % cursor.frames.len();
            Ok(frame)
        })
    }

    fn health(&self) -> SourceHealth {
        SourceHealth::Alive
    }
}

/// Replay backend: reads NDJSON file at construction, emits each line in
/// order with inter-frame delay derived from `ts_ms`. Runs once, then returns
/// `BmsError::ReplayExhausted`.
#[derive(Debug)]
pub struct ReplayBmsSource {
    frames: Vec<BmsFrame>,
    cursor: AtomicUsize,
    start_clock: std::sync::Mutex<Option<Instant>>,
    speedup: f64,
}

impl ReplayBmsSource {
    /// Load and parse an NDJSON trace file. Each line is one `BmsFrame`.
    /// `speedup` compresses inter-frame delay (>1.0 plays faster).
    pub fn from_file<P: AsRef<std::path::Path>>(path: P, speedup: f64) -> Result<Self, BmsError> {
        let bytes = std::fs::read(&path).map_err(|e| BmsError::Io(e.to_string()))?;
        let text = std::str::from_utf8(&bytes).map_err(|e| BmsError::Io(e.to_string()))?;
        let mut frames = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let frame: BmsFrame = serde_json::from_str(trimmed)
                .map_err(|e| BmsError::Io(format!("line {}: {e}", i + 1)))?;
            if frame.version != FRAME_VERSION {
                return Err(BmsError::UnsupportedVersion(frame.version));
            }
            frames.push(frame);
        }
        if frames.is_empty() {
            return Err(BmsError::EmptyTrace);
        }
        Ok(Self {
            frames,
            cursor: AtomicUsize::new(0),
            start_clock: std::sync::Mutex::new(None),
            speedup: if speedup > 0.0 { speedup } else { 1.0 },
        })
    }

    /// Construct from an in-memory frame list (used by tests).
    pub fn from_frames(frames: Vec<BmsFrame>, speedup: f64) -> Result<Self, BmsError> {
        if frames.is_empty() {
            return Err(BmsError::EmptyTrace);
        }
        Ok(Self {
            frames,
            cursor: AtomicUsize::new(0),
            start_clock: std::sync::Mutex::new(None),
            speedup: if speedup > 0.0 { speedup } else { 1.0 },
        })
    }

    fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

impl BmsSource for ReplayBmsSource {
    fn poll(&self) -> Pin<Box<dyn std::future::Future<Output = Result<BmsFrame, BmsError>> + Send + '_>> {
        Box::pin(async move {
            let idx = self.cursor.fetch_add(1, Ordering::SeqCst);
            if idx >= self.frames.len() {
                return Err(BmsError::ReplayExhausted);
            }
            let frame = self.frames[idx].clone();
            let trace_origin_ms = self.frames[0].ts_ms;
            // Lazily start wall clock on first poll.
            let start = {
                let mut guard = self
                    .start_clock
                    .lock()
                    .map_err(|e| BmsError::Io(format!("start_clock poison: {e}")))?;
                *guard.get_or_insert_with(Instant::now)
            };
            let delta_ms = frame.ts_ms.saturating_sub(trace_origin_ms);
            let target_ms = ((delta_ms as f64) / self.speedup) as u64;
            let target = std::time::Duration::from_millis(target_ms);
            let actual = start.elapsed();
            if target > actual {
                tokio::time::sleep(target - actual).await;
            }
            let mut emitted = frame;
            emitted.ts = Some(Instant::now());
            Ok(emitted)
        })
    }

    fn health(&self) -> SourceHealth {
        if self.cursor.load(Ordering::SeqCst) >= self.frame_count() {
            SourceHealth::Down
        } else {
            SourceHealth::Alive
        }
    }
}

/// Serial backend: opens any path the kernel exposes as a serial-class
/// device (`/dev/ttyUSB0` for hardware, `/dev/pts/N` for pty fixtures) and
/// reads NDJSON one line per `poll()`.
///
/// Validated against `stark-suit-test-pty` loopback. Hardware fidelity
/// (USB-serial adapter) is a follow-on layer — same code, real device path.
pub struct SerialBmsSource {
    inner: tokio::sync::Mutex<SerialReader>,
}

struct SerialReader {
    reader: tokio::io::BufReader<tokio_serial::SerialStream>,
    line_buf: String,
}

impl SerialBmsSource {
    /// Open the named serial device at `baud` baud, 8N1.
    pub fn open(port: &str, baud: u32) -> Result<Self, BmsError> {
        use tokio_serial::SerialPortBuilderExt;
        let stream = tokio_serial::new(port, baud)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .open_native_async()
            .map_err(|e| BmsError::Io(format!("open {port}: {e}")))?;
        Ok(Self {
            inner: tokio::sync::Mutex::new(SerialReader {
                reader: tokio::io::BufReader::new(stream),
                line_buf: String::with_capacity(512),
            }),
        })
    }
}

impl BmsSource for SerialBmsSource {
    fn poll(&self) -> Pin<Box<dyn std::future::Future<Output = Result<BmsFrame, BmsError>> + Send + '_>> {
        Box::pin(async move {
            use tokio::io::AsyncBufReadExt;
            let mut guard = self.inner.lock().await;
            let SerialReader { reader, line_buf } = &mut *guard;
            line_buf.clear();
            let n = reader
                .read_line(line_buf)
                .await
                .map_err(|e| BmsError::Io(format!("read_line: {e}")))?;
            if n == 0 {
                return Err(BmsError::Exhausted);
            }
            let trimmed = line_buf.trim();
            let frame: BmsFrame = serde_json::from_str(trimmed)
                .map_err(|e| BmsError::Io(format!("parse: {e}")))?;
            if frame.version != FRAME_VERSION {
                return Err(BmsError::UnsupportedVersion(frame.version));
            }
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

    #[tokio::test]
    async fn replay_round_trip_preserves_frame_sequence() {
        // Build a synthetic trace with monotonic ts_ms.
        let trace: Vec<BmsFrame> = (0..5)
            .map(|i| {
                let mut f = baked_frame(400.0 - i as f32, 10.0, 25.0, 100.0 - i as f32 * 5.0, 100.0, PowerTier::Comms);
                f.ts_ms = 1_000_000 + (i as u64) * 100;
                f
            })
            .collect();

        // Serialize as NDJSON.
        let mut ndjson = String::new();
        for f in &trace {
            ndjson.push_str(&serde_json::to_string(f).expect("serialize"));
            ndjson.push('\n');
        }

        // Deserialize through ReplayBmsSource at high speedup so test runs fast.
        let frames: Vec<BmsFrame> = ndjson
            .lines()
            .map(|l| serde_json::from_str::<BmsFrame>(l).expect("parse"))
            .collect();
        let src = ReplayBmsSource::from_frames(frames, 1000.0).expect("build replay");

        // Round-trip equality.
        for original in &trace {
            let replayed = src.poll().await.expect("replay poll");
            assert_eq!(replayed.soc_pct, original.soc_pct);
            assert_eq!(replayed.ts_ms, original.ts_ms);
            assert_eq!(replayed.tier, original.tier);
            assert_eq!(replayed.version, FRAME_VERSION);
        }

        // Sixth poll exhausts.
        match src.poll().await {
            Err(BmsError::ReplayExhausted) => {}
            other => panic!("expected ReplayExhausted, got {other:?}"),
        }
    }

    #[test]
    fn replay_rejects_empty_trace() {
        match ReplayBmsSource::from_frames(vec![], 1.0) {
            Err(BmsError::EmptyTrace) => {}
            other => panic!("expected EmptyTrace, got {other:?}"),
        }
    }

    #[test]
    fn replay_rejects_unsupported_version() {
        let bad_frame = BmsFrame {
            version: 999,
            ts_ms: 0,
            pack_voltage_v: 0.0,
            pack_current_a: 0.0,
            cell_temp_c: 0.0,
            soc_pct: 0.0,
            soh_pct: 0.0,
            tier: PowerTier::Comms,
            ts: None,
        };
        let line = serde_json::to_string(&bad_frame).expect("serialize");
        let tmp = std::env::temp_dir().join("bms_v999.ndjson");
        std::fs::write(&tmp, format!("{line}\n")).expect("write");
        match ReplayBmsSource::from_file(&tmp, 1.0) {
            Err(BmsError::UnsupportedVersion(999)) => {}
            other => panic!("expected UnsupportedVersion(999), got {other:?}"),
        }
        let _ = std::fs::remove_file(&tmp);
    }
}
