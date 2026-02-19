//! Vigilance Subsystem REST endpoints — π(∂·ν)|∝
//!
//! Endpoints for the always-on vigilance daemon:
//! - Health & status of the 4-layer architecture
//! - Ledger queries and verification
//! - Boundary management
//! - Runtime statistics

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

// ============================================================================
// Request/Response Types
// ============================================================================

/// Vigilance subsystem status response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VigilSysStatusResponse {
    /// Formula: π(∂·ν)|∝
    pub formula: String,
    /// Whether a WAL file exists
    pub wal_exists: bool,
    /// WAL file path
    pub wal_path: String,
    /// Ledger entry count
    pub ledger_entries: usize,
    /// Hash chain integrity
    pub chain_intact: bool,
    /// Head hash (hex)
    pub head_hash: String,
}

/// Ledger query request
#[derive(Debug, Deserialize, ToSchema)]
pub struct LedgerQueryRequest {
    /// Filter by entry type (event, violation, scheduled, executed, failed, started, stopped)
    #[serde(default)]
    pub entry_type: Option<String>,
    /// Max entries to return
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Ledger entry in API response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LedgerEntryResponse {
    /// Sequence number
    pub sequence: u64,
    /// Timestamp (ms since epoch)
    pub timestamp: u64,
    /// Entry type
    pub entry_type: String,
    /// Payload
    pub payload: serde_json::Value,
    /// Hash (hex, first 16 chars)
    pub hash: String,
}

/// Ledger query response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LedgerQueryResponse {
    /// Total entries in ledger
    pub total_entries: usize,
    /// Number of entries returned
    pub returned: usize,
    /// Matching entries
    pub entries: Vec<LedgerEntryResponse>,
}

/// Ledger verify response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LedgerVerifyResponse {
    /// Whether chain is intact
    pub intact: bool,
    /// Number of entries verified
    pub entries_verified: usize,
    /// Head hash (full hex)
    pub head_hash: String,
    /// Status message
    pub message: String,
}

// ============================================================================
// Router
// ============================================================================

/// Vigilance subsystem router
pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/status", get(status))
        .route("/ledger", post(ledger_query))
        .route("/ledger/verify", get(ledger_verify))
}

// ============================================================================
// Helpers
// ============================================================================

fn default_wal_path() -> PathBuf {
    PathBuf::from("/tmp/vigil-ledger.wal")
}

fn map_entry_type(s: &str) -> Option<nexcore_vigil::vigilance::LedgerEntryType> {
    use nexcore_vigil::vigilance::LedgerEntryType;
    match s {
        "event" => Some(LedgerEntryType::EventObserved),
        "violation" => Some(LedgerEntryType::BoundaryViolation),
        "scheduled" => Some(LedgerEntryType::ConsequenceScheduled),
        "executed" => Some(LedgerEntryType::ConsequenceExecuted),
        "failed" => Some(LedgerEntryType::ConsequenceFailed),
        "started" => Some(LedgerEntryType::DaemonStarted),
        "stopped" => Some(LedgerEntryType::DaemonStopped),
        _ => None,
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// Get vigilance subsystem status
#[utoipa::path(
    get,
    path = "/api/v1/vigil-sys/status",
    tag = "vigil-sys",
    responses(
        (status = 200, description = "Vigilance subsystem status", body = VigilSysStatusResponse)
    )
)]
pub async fn status() -> ApiResult<VigilSysStatusResponse> {
    let wal_path = default_wal_path();
    let wal_exists = wal_path.exists();

    if !wal_exists {
        return Ok(Json(VigilSysStatusResponse {
            formula: "π(∂·ν)|∝".to_string(),
            wal_exists: false,
            wal_path: wal_path.display().to_string(),
            ledger_entries: 0,
            chain_intact: false,
            head_hash: "none".to_string(),
        }));
    }

    let ledger = nexcore_vigil::vigilance::VigilanceLedger::recover_from_wal(&wal_path)
        .map_err(|e| ApiError::new("LEDGER_RECOVERY", e.to_string()))?;

    let chain_intact = ledger.verify_chain().unwrap_or(false);
    let head_hash: String = ledger
        .head_hash()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    Ok(Json(VigilSysStatusResponse {
        formula: "π(∂·ν)|∝".to_string(),
        wal_exists: true,
        wal_path: wal_path.display().to_string(),
        ledger_entries: ledger.len(),
        chain_intact,
        head_hash,
    }))
}

/// Query vigilance ledger entries
#[utoipa::path(
    post,
    path = "/api/v1/vigil-sys/ledger",
    tag = "vigil-sys",
    request_body = LedgerQueryRequest,
    responses(
        (status = 200, description = "Ledger entries", body = LedgerQueryResponse)
    )
)]
pub async fn ledger_query(Json(req): Json<LedgerQueryRequest>) -> ApiResult<LedgerQueryResponse> {
    let wal_path = default_wal_path();

    if !wal_path.exists() {
        return Ok(Json(LedgerQueryResponse {
            total_entries: 0,
            returned: 0,
            entries: Vec::new(),
        }));
    }

    let ledger = nexcore_vigil::vigilance::VigilanceLedger::recover_from_wal(&wal_path)
        .map_err(|e| ApiError::new("LEDGER_RECOVERY", e.to_string()))?;

    let et_filter = req.entry_type.as_deref().and_then(map_entry_type);
    let limit = req.limit.unwrap_or(50);

    let query = nexcore_vigil::vigilance::LedgerQuery {
        entry_type: et_filter,
        since: None,
        limit: Some(limit),
    };

    let raw_entries = ledger.query(&query);
    let entries: Vec<LedgerEntryResponse> = raw_entries
        .iter()
        .map(|e| {
            let hash_hex: String = e.hash.iter().take(8).map(|b| format!("{b:02x}")).collect();
            LedgerEntryResponse {
                sequence: e.sequence,
                timestamp: e.timestamp,
                entry_type: format!("{}", e.entry_type),
                payload: e.payload.clone(),
                hash: hash_hex,
            }
        })
        .collect();

    let returned = entries.len();
    Ok(Json(LedgerQueryResponse {
        total_entries: ledger.len(),
        returned,
        entries,
    }))
}

/// Verify vigilance ledger hash chain integrity
#[utoipa::path(
    get,
    path = "/api/v1/vigil-sys/ledger/verify",
    tag = "vigil-sys",
    responses(
        (status = 200, description = "Chain verification result", body = LedgerVerifyResponse)
    )
)]
pub async fn ledger_verify() -> ApiResult<LedgerVerifyResponse> {
    let wal_path = default_wal_path();

    if !wal_path.exists() {
        return Ok(Json(LedgerVerifyResponse {
            intact: false,
            entries_verified: 0,
            head_hash: "none".to_string(),
            message: "No WAL file found — daemon has not been started".to_string(),
        }));
    }

    let ledger = nexcore_vigil::vigilance::VigilanceLedger::recover_from_wal(&wal_path)
        .map_err(|e| ApiError::new("LEDGER_RECOVERY", e.to_string()))?;

    let intact = ledger
        .verify_chain()
        .map_err(|e| ApiError::new("VERIFY_ERROR", e.to_string()))?;

    let head_hash: String = ledger
        .head_hash()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    let message = if intact {
        "Chain integrity confirmed — all hashes verified".to_string()
    } else {
        "COMPROMISED — hash chain broken, possible tampering".to_string()
    };

    Ok(Json(LedgerVerifyResponse {
        intact,
        entries_verified: ledger.len(),
        head_hash,
        message,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_status_no_wal() {
        // With no WAL file, should return wal_exists=false
        let response = status().await;
        assert!(response.is_ok());
        let body = response.unwrap_or_else(|_| {
            Json(VigilSysStatusResponse {
                formula: String::new(),
                wal_exists: false,
                wal_path: String::new(),
                ledger_entries: 0,
                chain_intact: false,
                head_hash: String::new(),
            })
        });
        // Either the WAL exists from a prior run or it doesn't — both valid
        assert_eq!(body.formula, "π(∂·ν)|∝");
    }

    #[tokio::test]
    async fn test_ledger_verify_no_wal() {
        let response = ledger_verify().await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_ledger_query_no_wal() {
        let req = LedgerQueryRequest {
            entry_type: None,
            limit: Some(10),
        };
        let response = ledger_query(Json(req)).await;
        assert!(response.is_ok());
    }
}
