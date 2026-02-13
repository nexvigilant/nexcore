//! Audit Middleware for NexCore REST API
//!
//! Logs every HTTP request to `~/.claude/audit/api_audit.jsonl` with:
//! - Method, URI, status code, duration
//!
//! Non-blocking: writes happen asynchronously via a channel.
//! Does not capture request/response bodies (TraceLayer handles span logging).
//!
//! Tier: T2-C (Cross-domain composite audit infrastructure)
//! Grounds to: T1 primitives (String, u16, u64, DateTime)

use axum::{extract::Request, middleware::Next, response::Response};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

/// API audit record written to JSONL.
///
/// Tier: T2-C (Cross-domain composite)
/// Grounds to: T1 primitives (String, u16, u64, DateTime)
#[derive(Debug, Clone, Serialize)]
pub struct ApiAuditRecord {
    /// ISO-8601 timestamp
    pub ts: DateTime<Utc>,
    /// HTTP method
    pub method: String,
    /// Request URI
    pub uri: String,
    /// HTTP status code
    pub status: u16,
    /// Request duration in milliseconds
    pub duration_ms: u64,
}

/// Global sender for API audit records.
static API_AUDIT_SENDER: OnceLock<mpsc::UnboundedSender<ApiAuditRecord>> = OnceLock::new();

/// Initialize the API audit writer background task.
///
/// Safe to call multiple times - only initializes once.
fn init_audit_writer() {
    API_AUDIT_SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(api_audit_writer_task(rx));
        tx
    });
}

/// Get the API audit log file path.
fn api_audit_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/audit/api_audit.jsonl")
}

/// Background task that writes API audit records to disk.
async fn api_audit_writer_task(mut rx: mpsc::UnboundedReceiver<ApiAuditRecord>) {
    let path = api_audit_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent).await;
    }

    while let Some(record) = rx.recv().await {
        if let Ok(mut line) = serde_json::to_string(&record) {
            line.push('\n');

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
            {
                let _ = file.write_all(line.as_bytes()).await;
            }
        }
    }
}

/// Record an API audit event (non-blocking, fire-and-forget).
fn record_api_audit(record: ApiAuditRecord) {
    if let Some(tx) = API_AUDIT_SENDER.get() {
        let _ = tx.send(record);
    }
}

/// Axum middleware that logs each request to the API audit trail.
///
/// Position in middleware stack: after TraceLayer (which handles tracing spans),
/// wrapping the route handlers.
pub async fn audit_layer(req: Request, next: Next) -> Response {
    // Ensure writer is initialized on first call
    init_audit_writer();

    let method = req.method().to_string();
    let uri = req.uri().to_string();
    let start = Instant::now();

    let response = next.run(req).await;

    let record = ApiAuditRecord {
        ts: Utc::now(),
        method,
        uri,
        status: response.status().as_u16(),
        duration_ms: start.elapsed().as_millis() as u64,
    };
    record_api_audit(record);

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_audit_record_serialization() {
        let record = ApiAuditRecord {
            ts: Utc::now(),
            method: "POST".to_string(),
            uri: "/api/v1/pv/signal/prr".to_string(),
            status: 200,
            duration_ms: 3,
        };

        let json = serde_json::to_string(&record);
        assert!(json.is_ok());
        let s = json.unwrap_or_default();
        assert!(s.contains("POST"));
        assert!(s.contains("/api/v1/pv/signal/prr"));
        assert!(s.contains("200"));
    }

    #[test]
    fn test_api_audit_path() {
        let path = api_audit_path();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains(".claude/audit/api_audit.jsonl"));
    }
}
