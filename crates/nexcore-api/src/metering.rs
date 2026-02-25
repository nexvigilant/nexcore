//! Per-customer metering middleware for the NexVigilant Commercial API.
//!
//! Logs every authenticated API request to a local CSV file with:
//! - Timestamp (ISO 8601)
//! - API key (masked: first 8 chars + `***`)
//! - Endpoint (method + URI)
//! - Response time (milliseconds)
//! - HTTP status code
//!
//! Non-blocking: writes happen asynchronously via a channel (same pattern as
//! `audit.rs`). The CSV file lives at `~/.nexvigilant/metering.csv`.
//!
//! ## Configuration
//!
//! - `METERING_LOG_PATH` env var overrides the default file location.
//!
//! Tier: T2-C (∂ Boundary + π Persistence + ν Frequency)

use axum::{body::Body, extract::Request, http::HeaderMap, middleware::Next, response::Response};
use nexcore_fs::dirs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

/// A single metering record.
#[derive(Debug)]
struct MeteringRecord {
    /// ISO 8601 timestamp
    timestamp: String,
    /// Masked API key (first 8 chars + ***)
    api_key: String,
    /// HTTP method + URI
    endpoint: String,
    /// Response time in milliseconds
    response_time_ms: u64,
    /// HTTP status code
    status_code: u16,
}

/// Global sender for metering records.
static METERING_SENDER: OnceLock<mpsc::UnboundedSender<MeteringRecord>> = OnceLock::new();

/// Initialize the metering writer background task.
///
/// Safe to call multiple times — only initializes once.
fn init_metering_writer() {
    METERING_SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(metering_writer_task(rx));
        tx
    });
}

/// Get the metering log file path.
fn metering_path() -> PathBuf {
    if let Ok(custom) = std::env::var("METERING_LOG_PATH") {
        return PathBuf::from(custom);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".nexvigilant/metering.csv")
}

/// Background task that writes metering records to disk as CSV.
async fn metering_writer_task(mut rx: mpsc::UnboundedReceiver<MeteringRecord>) {
    let path = metering_path();

    // Ensure directory exists (best-effort — non-critical infrastructure)
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent).await {
            tracing::warn!("Failed to create metering directory: {e}");
        }
    }

    // Write CSV header if file doesn't exist or is empty
    let needs_header = !path.exists()
        || fs::metadata(&path)
            .await
            .map(|m| m.len() == 0)
            .unwrap_or(true);

    if needs_header {
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
        {
            Ok(mut file) => {
                if let Err(e) = file
                    .write_all(b"timestamp,api_key,endpoint,response_time_ms,status_code\n")
                    .await
                {
                    tracing::warn!("Failed to write metering CSV header: {e}");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to open metering file for header: {e}");
            }
        }
    }

    while let Some(record) = rx.recv().await {
        let line = format!(
            "{},{},{},{},{}\n",
            record.timestamp,
            record.api_key,
            record.endpoint,
            record.response_time_ms,
            record.status_code,
        );

        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(line.as_bytes()).await {
                    tracing::warn!("Failed to write metering record: {e}");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to open metering file: {e}");
            }
        }
    }
}

/// Fire-and-forget metering record.
fn record_metering(record: MeteringRecord) {
    if let Some(tx) = METERING_SENDER.get() {
        // send returns Err only if receiver is dropped — benign in shutdown
        #[allow(unused_results)]
        {
            tx.send(record);
        }
    }
}

/// Extract and mask API key from request headers.
///
/// Returns first 8 characters + `***` for privacy. Falls back to `"anonymous"`
/// when no key is present (should not happen behind auth middleware).
fn extract_masked_key(headers: &HeaderMap) -> String {
    let raw = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
        })
        .unwrap_or("anonymous");

    if raw.len() > 8 {
        format!("{}***", &raw[..8])
    } else {
        raw.to_string()
    }
}

/// Axum middleware that meters each API request.
///
/// Position in middleware stack: inside auth (so only authed requests are
/// metered), wrapping the route handlers.
pub async fn metering_layer(req: Request<Body>, next: Next) -> Response {
    init_metering_writer();

    let api_key = extract_masked_key(req.headers());
    let endpoint = format!("{} {}", req.method(), req.uri().path());
    let start = Instant::now();

    let response = next.run(req).await;

    let record = MeteringRecord {
        timestamp: nexcore_chrono::DateTime::now().to_rfc3339(),
        api_key,
        endpoint,
        response_time_ms: start.elapsed().as_millis() as u64,
        status_code: response.status().as_u16(),
    };
    record_metering(record);

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_masked_key_x_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("grd_abc123456789"));
        assert_eq!(extract_masked_key(&headers), "grd_abc1***");
    }

    #[test]
    fn test_extract_masked_key_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer eyJhbGciOiJSUzI1NiJ9.test"),
        );
        assert_eq!(extract_masked_key(&headers), "eyJhbGci***");
    }

    #[test]
    fn test_extract_masked_key_short() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("short"));
        assert_eq!(extract_masked_key(&headers), "short");
    }

    #[test]
    fn test_extract_masked_key_missing() {
        let headers = HeaderMap::new();
        // "anonymous" is 9 chars → masked to "anonymou***"
        assert_eq!(extract_masked_key(&headers), "anonymou***");
    }

    #[test]
    fn test_metering_path_default() {
        let path = metering_path();
        let s = path.to_string_lossy();
        assert!(
            s.contains("metering.csv"),
            "Expected metering.csv in path: {s}"
        );
    }
}
