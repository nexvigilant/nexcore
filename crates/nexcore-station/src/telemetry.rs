//! Real-time telemetry for NexVigilant Station.
//!
//! Enable streaming by setting:
//! `NEXCORE_STATION_EVENT_LOG=/absolute/path/station-events.ndjson`
//!
//! Events are newline-delimited JSON (NDJSON) so downstream systems (including
//! Jupyter notebooks) can tail the file and process events incrementally.

use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::types::TrustTier;

/// Structured station telemetry event written as one NDJSON line.
#[derive(Debug, Clone, Serialize)]
pub struct StationTelemetryEvent {
    /// Unix timestamp in milliseconds.
    pub ts_ms: u128,
    /// Event kind (e.g. resolve_start, resolve_finish, feed_http_start).
    pub event_type: &'static str,
    /// Correlation ID to connect feed + resolve events.
    pub trace_id: String,
    /// Target domain.
    pub domain: String,
    /// Resolution matrix case, when available (1..4).
    pub case: Option<u8>,
    /// Confidence score, when available.
    pub confidence: Option<f64>,
    /// Trust tier, when available.
    pub trust_tier: Option<&'static str>,
    /// Operation latency in milliseconds, when available.
    pub latency_ms: Option<u128>,
    /// HTTP status code for feed events, when available.
    pub http_status: Option<u16>,
    /// Free-form status text.
    pub status: Option<&'static str>,
    /// Error detail (if any).
    pub error: Option<String>,
}

/// Optional global sink for live station telemetry.
struct StationTelemetrySink {
    writer: Mutex<BufWriter<File>>,
}

impl StationTelemetrySink {
    fn from_env() -> Option<Self> {
        let path = std::env::var("NEXCORE_STATION_EVENT_LOG").ok()?;
        if path.trim().is_empty() {
            return None;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()?;

        Some(Self {
            writer: Mutex::new(BufWriter::new(file)),
        })
    }

    fn emit(&self, event: &StationTelemetryEvent) {
        if let Ok(line) = serde_json::to_string(event) {
            let mut guard = match self.writer.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            if guard.write_all(line.as_bytes()).is_err() {
                return;
            }
            if guard.write_all(b"\n").is_err() {
                return;
            }
            let _ = guard.flush();
        }
    }
}

fn now_ms() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis(),
        Err(_) => 0,
    }
}

fn sink() -> Option<&'static StationTelemetrySink> {
    static SINK: OnceLock<Option<StationTelemetrySink>> = OnceLock::new();
    SINK.get_or_init(StationTelemetrySink::from_env).as_ref()
}

thread_local! {
    static TRACE_CONTEXT: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn trust_tier_label(tier: &TrustTier) -> &'static str {
    match tier {
        TrustTier::Verified => "verified",
        TrustTier::Experimental => "experimental",
        TrustTier::Unavailable => "unavailable",
    }
}

/// Generate a trace ID for event correlation.
#[must_use]
pub fn new_trace_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("station-{}-{}", now_ms(), seq)
}

/// Run a closure with a trace id bound in thread-local context.
pub fn with_trace_id<T, F>(trace_id: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    TRACE_CONTEXT.with(|ctx| {
        let previous = ctx.replace(Some(trace_id.to_string()));
        let out = f();
        let _ = ctx.replace(previous);
        out
    })
}

/// Return the current thread-local trace id, if one is bound.
#[must_use]
pub fn current_trace_id() -> Option<String> {
    TRACE_CONTEXT.with(|ctx| ctx.borrow().clone())
}

/// Emit a telemetry event if streaming is enabled.
pub fn emit(event: StationTelemetryEvent) {
    if let Some(s) = sink() {
        s.emit(&event);
    }
}

/// Emit a resolution start event.
pub fn emit_resolve_start(trace_id: &str, domain: &str) {
    emit(StationTelemetryEvent {
        ts_ms: now_ms(),
        event_type: "resolve_start",
        trace_id: trace_id.to_string(),
        domain: domain.to_string(),
        case: None,
        confidence: None,
        trust_tier: None,
        latency_ms: None,
        http_status: None,
        status: Some("started"),
        error: None,
    });
}

/// Metadata for a resolution finish event.
#[derive(Debug, Clone)]
pub struct ResolutionMetadata<'a> {
    pub case: u8,
    pub confidence: Option<f64>,
    pub trust_tier: Option<&'a TrustTier>,
    pub latency_ms: u128,
    pub error: Option<String>,
}

/// Emit a resolution finish event.
pub fn emit_resolve_finish(trace_id: &str, domain: &str, meta: ResolutionMetadata<'_>) {
    emit(StationTelemetryEvent {
        ts_ms: now_ms(),
        event_type: "resolve_finish",
        trace_id: trace_id.to_string(),
        domain: domain.to_string(),
        case: Some(meta.case),
        confidence: meta.confidence,
        trust_tier: meta.trust_tier.map(trust_tier_label),
        latency_ms: Some(meta.latency_ms),
        http_status: None,
        status: Some(if meta.error.is_some() { "error" } else { "ok" }),
        error: meta.error,
    });
}

/// Metadata for an HTTP feed event.
#[derive(Debug, Clone, Default)]
pub struct HttpFeedMetadata {
    pub latency_ms: Option<u128>,
    pub http_status: Option<u16>,
    pub error: Option<String>,
}

/// Emit an HTTP feed event.
pub fn emit_feed_http(
    event_type: &'static str,
    trace_id: &str,
    domain: &str,
    meta: HttpFeedMetadata,
) {
    emit(StationTelemetryEvent {
        ts_ms: now_ms(),
        event_type,
        trace_id: trace_id.to_string(),
        domain: domain.to_string(),
        case: None,
        confidence: None,
        trust_tier: None,
        latency_ms: meta.latency_ms,
        http_status: meta.http_status,
        status: Some(if meta.error.is_some() { "error" } else { "ok" }),
        error: meta.error,
    });
}
