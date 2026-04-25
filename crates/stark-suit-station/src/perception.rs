//! Perception sensor fusion source abstraction.
//!
//! Mirrors the v0.4 BMS pattern (Mock + Replay + Serial via pty + CAN deferral).
//! The Perception compound fuses 4+ sensors (IMU, magnetometer, barometer,
//! body proprioceptive); rather than one trait per sensor, the source emits a
//! pre-fused `PerceptionFrame` with all sensor fields flat — same NDJSON
//! discipline as `BmsFrame`, one wire-format contract spans all backends.
//!
//! Wire format: NDJSON, UTF-8, one `PerceptionFrame` per line. Every frame
//! carries `version: u32` (currently `1`) and `ts_ms: u64`. See
//! `docs/perception_frame_v1.md`.

use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Current wire-format schema version. Bump on incompatible field changes.
pub const PERCEPTION_FRAME_VERSION: u32 = 1;

/// One fused multi-sensor frame. Flat field layout for NDJSON portability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionFrame {
    /// Wire-format schema version (always 1 for this schema).
    #[serde(default = "default_version")]
    pub version: u32,
    /// Epoch milliseconds at frame ingest. Serializable, deterministic.
    pub ts_ms: u64,
    /// IMU linear acceleration, m/s² (x, y, z).
    pub accel_mps2: [f32; 3],
    /// IMU angular velocity, rad/s (x, y, z).
    pub gyro_radps: [f32; 3],
    /// Magnetometer field strength, microtesla (x, y, z).
    pub mag_ut: [f32; 3],
    /// Barometric pressure, hPa.
    pub pressure_hpa: f32,
    /// Ambient temperature, °C.
    pub temp_c: f32,
    /// Heart rate, BPM (0..=255).
    pub heart_rate_bpm: u8,
    /// Blood oxygen saturation, percent.
    pub spo2_pct: u8,
    /// Wall-clock at frame ingest. Skipped on serialize.
    #[serde(skip)]
    pub ts: Option<Instant>,
}

fn default_version() -> u32 {
    PERCEPTION_FRAME_VERSION
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Errors a `poll()` can return.
#[derive(Debug)]
pub enum PerceptionError {
    /// Frame did not arrive within the loop's tick budget.
    Timeout,
    /// Source backend is exhausted (trace ran out, connection closed).
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

impl core::fmt::Display for PerceptionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Timeout => write!(f, "perception timeout"),
            Self::Exhausted => write!(f, "perception source exhausted"),
            Self::EmptyTrace => write!(f, "replay trace was empty"),
            Self::ReplayExhausted => write!(f, "replay reached end of trace"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported frame version {v}"),
            Self::Io(s) => write!(f, "perception i/o: {s}"),
        }
    }
}

impl std::error::Error for PerceptionError {}

/// Health classification of the source itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerceptionHealth {
    /// Source is alive and last poll succeeded.
    Alive,
    /// Source reachable but last frame was stale.
    Degraded,
    /// Source is offline.
    Down,
}

/// Trait every Perception backend implements.
pub trait PerceptionSource: Send + Sync {
    /// Pull one fused frame.
    fn poll(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<PerceptionFrame, PerceptionError>> + Send + '_>>;

    /// Snapshot of the source's health.
    fn health(&self) -> PerceptionHealth;
}

fn baked(
    accel: [f32; 3],
    gyro: [f32; 3],
    mag: [f32; 3],
    pressure: f32,
    temp: f32,
    hr: u8,
    spo2: u8,
) -> PerceptionFrame {
    PerceptionFrame {
        version: PERCEPTION_FRAME_VERSION,
        ts_ms: 0,
        accel_mps2: accel,
        gyro_radps: gyro,
        mag_ut: mag,
        pressure_hpa: pressure,
        temp_c: temp,
        heart_rate_bpm: hr,
        spo2_pct: spo2,
        ts: None,
    }
}

/// Mock backend: cycles through a baked-in 5-frame trace.
pub struct MockPerceptionSource {
    frames: Mutex<TraceCursor>,
}

struct TraceCursor {
    frames: Vec<PerceptionFrame>,
    idx: usize,
}

impl MockPerceptionSource {
    /// Build with a baked-in trace approximating a "standing-still" sequence
    /// with mild head sway.
    #[must_use]
    pub fn new() -> Self {
        let frames = vec![
            baked([0.0, 0.0, 9.81], [0.0, 0.0, 0.0], [22.0, 0.0, -42.0], 1013.25, 20.0, 60, 98),
            baked([0.05, 0.0, 9.80], [0.01, 0.0, 0.0], [22.0, 0.1, -42.0], 1013.20, 20.1, 61, 98),
            baked([0.0, 0.05, 9.81], [0.0, 0.01, 0.0], [22.1, 0.0, -42.0], 1013.22, 20.0, 60, 98),
            baked([-0.05, 0.0, 9.79], [-0.01, 0.0, 0.0], [21.9, -0.1, -42.0], 1013.18, 20.1, 60, 98),
            baked([0.0, -0.05, 9.80], [0.0, -0.01, 0.0], [22.0, 0.0, -42.1], 1013.25, 20.0, 60, 98),
        ];
        Self { frames: Mutex::new(TraceCursor { frames, idx: 0 }) }
    }

    /// Construct from explicit trace (tests).
    #[must_use]
    pub fn with_trace(frames: Vec<PerceptionFrame>) -> Self {
        Self { frames: Mutex::new(TraceCursor { frames, idx: 0 }) }
    }
}

impl Default for MockPerceptionSource {
    fn default() -> Self {
        Self::new()
    }
}

impl PerceptionSource for MockPerceptionSource {
    fn poll(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<PerceptionFrame, PerceptionError>> + Send + '_>> {
        Box::pin(async move {
            let mut cursor = self.frames.lock().await;
            if cursor.frames.is_empty() {
                return Err(PerceptionError::Exhausted);
            }
            let mut frame = cursor.frames[cursor.idx].clone();
            frame.ts = Some(Instant::now());
            frame.ts_ms = now_ms();
            cursor.idx = (cursor.idx + 1) % cursor.frames.len();
            Ok(frame)
        })
    }

    fn health(&self) -> PerceptionHealth {
        PerceptionHealth::Alive
    }
}

/// Replay backend: NDJSON file, runs once at recorded cadence (or `speedup`).
#[derive(Debug)]
pub struct ReplayPerceptionSource {
    frames: Vec<PerceptionFrame>,
    cursor: AtomicUsize,
    start_clock: std::sync::Mutex<Option<Instant>>,
    speedup: f64,
}

impl ReplayPerceptionSource {
    /// Load NDJSON from file. Each line is one `PerceptionFrame`.
    pub fn from_file<P: AsRef<std::path::Path>>(path: P, speedup: f64) -> Result<Self, PerceptionError> {
        let bytes = std::fs::read(&path).map_err(|e| PerceptionError::Io(e.to_string()))?;
        let text = std::str::from_utf8(&bytes).map_err(|e| PerceptionError::Io(e.to_string()))?;
        let mut frames = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let frame: PerceptionFrame = serde_json::from_str(trimmed)
                .map_err(|e| PerceptionError::Io(format!("line {}: {e}", i + 1)))?;
            if frame.version != PERCEPTION_FRAME_VERSION {
                return Err(PerceptionError::UnsupportedVersion(frame.version));
            }
            frames.push(frame);
        }
        if frames.is_empty() {
            return Err(PerceptionError::EmptyTrace);
        }
        Ok(Self {
            frames,
            cursor: AtomicUsize::new(0),
            start_clock: std::sync::Mutex::new(None),
            speedup: if speedup > 0.0 { speedup } else { 1.0 },
        })
    }

    /// Construct from in-memory frames (tests).
    pub fn from_frames(frames: Vec<PerceptionFrame>, speedup: f64) -> Result<Self, PerceptionError> {
        if frames.is_empty() {
            return Err(PerceptionError::EmptyTrace);
        }
        Ok(Self {
            frames,
            cursor: AtomicUsize::new(0),
            start_clock: std::sync::Mutex::new(None),
            speedup: if speedup > 0.0 { speedup } else { 1.0 },
        })
    }
}

impl PerceptionSource for ReplayPerceptionSource {
    fn poll(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<PerceptionFrame, PerceptionError>> + Send + '_>> {
        Box::pin(async move {
            let idx = self.cursor.fetch_add(1, Ordering::SeqCst);
            if idx >= self.frames.len() {
                return Err(PerceptionError::ReplayExhausted);
            }
            let frame = self.frames[idx].clone();
            let trace_origin_ms = self.frames[0].ts_ms;
            let start = {
                let mut guard = self
                    .start_clock
                    .lock()
                    .map_err(|e| PerceptionError::Io(format!("start_clock poison: {e}")))?;
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

    fn health(&self) -> PerceptionHealth {
        if self.cursor.load(Ordering::SeqCst) >= self.frames.len() {
            PerceptionHealth::Down
        } else {
            PerceptionHealth::Alive
        }
    }
}

/// Serial backend: opens any path the kernel exposes as serial-class
/// (`/dev/ttyUSB0` or `/dev/pts/N`) and reads NDJSON one line per `poll()`.
pub struct SerialPerceptionSource {
    inner: tokio::sync::Mutex<SerialReader>,
}

struct SerialReader {
    reader: tokio::io::BufReader<tokio_serial::SerialStream>,
    line_buf: String,
}

impl SerialPerceptionSource {
    /// Open the named serial device at `baud` baud, 8N1.
    pub fn open(port: &str, baud: u32) -> Result<Self, PerceptionError> {
        use tokio_serial::SerialPortBuilderExt;
        let stream = tokio_serial::new(port, baud)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .open_native_async()
            .map_err(|e| PerceptionError::Io(format!("open {port}: {e}")))?;
        Ok(Self {
            inner: tokio::sync::Mutex::new(SerialReader {
                reader: tokio::io::BufReader::new(stream),
                line_buf: String::with_capacity(512),
            }),
        })
    }
}

impl PerceptionSource for SerialPerceptionSource {
    fn poll(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<PerceptionFrame, PerceptionError>> + Send + '_>> {
        Box::pin(async move {
            use tokio::io::AsyncBufReadExt;
            let mut guard = self.inner.lock().await;
            let SerialReader { reader, line_buf } = &mut *guard;
            line_buf.clear();
            let n = reader
                .read_line(line_buf)
                .await
                .map_err(|e| PerceptionError::Io(format!("read_line: {e}")))?;
            if n == 0 {
                return Err(PerceptionError::Exhausted);
            }
            let trimmed = line_buf.trim();
            let frame: PerceptionFrame = serde_json::from_str(trimmed)
                .map_err(|e| PerceptionError::Io(format!("parse: {e}")))?;
            if frame.version != PERCEPTION_FRAME_VERSION {
                return Err(PerceptionError::UnsupportedVersion(frame.version));
            }
            Ok(frame)
        })
    }

    fn health(&self) -> PerceptionHealth {
        PerceptionHealth::Alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_yields_baked_trace_in_order() {
        let src = MockPerceptionSource::new();
        let first = src.poll().await.expect("poll");
        assert_eq!(first.heart_rate_bpm, 60);
        let second = src.poll().await.expect("poll");
        assert_eq!(second.heart_rate_bpm, 61);
    }

    #[tokio::test]
    async fn mock_loops_after_exhaustion() {
        let src = MockPerceptionSource::new();
        for _ in 0..5 {
            src.poll().await.expect("first pass");
        }
        let frame_six = src.poll().await.expect("wraparound");
        assert_eq!(frame_six.heart_rate_bpm, 60);
    }

    #[tokio::test]
    async fn replay_round_trip_preserves_frame_sequence() {
        let trace: Vec<PerceptionFrame> = (0..5)
            .map(|i| {
                let mut f = baked(
                    [i as f32, 0.0, 9.81],
                    [0.0; 3],
                    [22.0, 0.0, -42.0],
                    1013.25,
                    20.0,
                    60,
                    98,
                );
                f.ts_ms = 1_000_000 + (i as u64) * 50;
                f
            })
            .collect();
        let mut ndjson = String::new();
        for f in &trace {
            ndjson.push_str(&serde_json::to_string(f).expect("serialize"));
            ndjson.push('\n');
        }
        let parsed: Vec<PerceptionFrame> = ndjson
            .lines()
            .map(|l| serde_json::from_str(l).expect("parse"))
            .collect();
        let src = ReplayPerceptionSource::from_frames(parsed, 1000.0).expect("build");
        for original in &trace {
            let replayed = src.poll().await.expect("poll");
            assert_eq!(replayed.ts_ms, original.ts_ms);
            assert_eq!(replayed.accel_mps2, original.accel_mps2);
            assert_eq!(replayed.version, PERCEPTION_FRAME_VERSION);
        }
        match src.poll().await {
            Err(PerceptionError::ReplayExhausted) => {}
            other => panic!("expected ReplayExhausted, got {other:?}"),
        }
    }

    #[test]
    fn replay_rejects_empty_trace() {
        match ReplayPerceptionSource::from_frames(vec![], 1.0) {
            Err(PerceptionError::EmptyTrace) => {}
            other => panic!("expected EmptyTrace, got {other:?}"),
        }
    }
}
