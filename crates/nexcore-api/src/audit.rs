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
use nexcore_chrono::DateTime;
use nexcore_codec::hex;
use nexcore_fs::dirs;
use nexcore_hash::hmac::HmacSha256;
use nexcore_hash::sha256::Sha256;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
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
    pub ts: DateTime,
    /// HTTP method
    pub method: String,
    /// Request URI
    pub uri: String,
    /// HTTP status code
    pub status: u16,
    /// Request duration in milliseconds
    pub duration_ms: u64,
    /// HMAC-SHA256 signature of this record (hex-encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hmac_sig: Option<String>,
    /// Hash of the previous record for chain integrity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_hash: Option<String>,
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
        // send returns Err only if receiver is dropped — benign
        #[allow(unused_results)]
        {
            tx.send(record);
        }
    }
}

/// Last record hash for chain integrity
static LAST_HASH: OnceLock<Mutex<String>> = OnceLock::new();

fn get_last_hash() -> &'static Mutex<String> {
    LAST_HASH.get_or_init(|| Mutex::new("genesis".to_string()))
}

/// Sign an audit record with HMAC-SHA256 and chain to previous record
fn sign_audit_record(record: &mut ApiAuditRecord) {
    // Get previous hash for chaining
    let prev = {
        let lock = get_last_hash();
        match lock.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => "error".to_string(),
        }
    };
    record.prev_hash = Some(prev);

    // Build payload to sign: ts|method|uri|status|duration|prev_hash
    let payload = format!(
        "{}|{}|{}|{}|{}|{}",
        record.ts.to_rfc3339(),
        record.method,
        record.uri,
        record.status,
        record.duration_ms,
        record.prev_hash.as_deref().unwrap_or(""),
    );

    let secret = std::env::var("AUDIT_SECRET")
        .unwrap_or_else(|_| "audit-dev-secret-change-in-prod".to_string());

    let sig = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mut hmac) => {
            hmac.update(payload.as_bytes());
            hex::encode(hmac.finalize())
        }
        Err(_) => hex::encode(Sha256::digest(payload.as_bytes())),
    };

    record.hmac_sig = Some(sig.clone());

    // Update chain hash
    let lock = get_last_hash();
    if let Ok(mut guard) = lock.lock() {
        *guard = sig;
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

    let mut record = ApiAuditRecord {
        ts: DateTime::now(),
        method,
        uri,
        status: response.status().as_u16(),
        duration_ms: start.elapsed().as_millis() as u64,
        hmac_sig: None,
        prev_hash: None,
    };
    sign_audit_record(&mut record);
    record_api_audit(record);

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_audit_record_serialization() {
        let record = ApiAuditRecord {
            ts: DateTime::now(),
            method: "POST".to_string(),
            uri: "/api/v1/pv/signal/prr".to_string(),
            status: 200,
            duration_ms: 3,
            hmac_sig: None,
            prev_hash: None,
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
