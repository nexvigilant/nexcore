//! Vigil orchestrator REST endpoints
//!
//! Always-on AI orchestrator with:
//! - EventBus: Multi-priority event routing
//! - MemoryLayer: Qdrant vector store for KSB knowledge
//! - DecisionEngine: Authority-based action selection

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

/// Default Vigil webhook URL
const VIGIL_URL: &str = "http://localhost:8080";

// ============================================================================
// Request/Response Types
// ============================================================================

/// Vigil status response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StatusResponse {
    /// Overall status (running/stopped)
    pub status: String,
    /// Process info
    pub process: ProcessInfo,
    /// Webhook and health endpoints
    pub endpoints: EndpointInfo,
    /// Core components
    pub components: ComponentsInfo,
    /// Event sources
    pub sources: Vec<SourceInfo>,
    /// Action executors
    pub executors: Vec<ExecutorInfo>,
}

/// Process information
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProcessInfo {
    /// Process name
    pub name: String,
    /// Whether process is running
    pub running: bool,
}

/// Endpoint URLs
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EndpointInfo {
    /// Webhook URL
    pub webhook: String,
    /// Health check URL
    pub health: String,
}

/// Component information
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ComponentsInfo {
    /// EventBus details
    pub event_bus: serde_json::Value,
    /// Memory layer details
    pub memory_layer: serde_json::Value,
    /// Decision engine details
    pub decision_engine: serde_json::Value,
}

/// Event source info
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SourceInfo {
    /// Source name
    pub name: String,
    /// Source type
    #[serde(rename = "type")]
    pub source_type: String,
    /// Description
    pub description: String,
}

/// Executor info
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ExecutorInfo {
    /// Executor name
    pub name: String,
    /// Executor type
    #[serde(rename = "type")]
    pub executor_type: String,
    /// Description
    pub description: String,
}

/// Health check response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Overall status (healthy/degraded/unhealthy)
    pub status: String,
    /// Individual service checks
    pub checks: HealthChecks,
    /// Summary counts
    pub summary: HealthSummary,
}

/// Health checks for services
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthChecks {
    /// Vigil process status
    pub vigil_process: CheckResult,
    /// Qdrant status
    pub qdrant: CheckResult,
    /// Prometheus status
    pub prometheus: CheckResult,
    /// Grafana status
    pub grafana: CheckResult,
}

/// Individual check result
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CheckResult {
    /// Status (up/down)
    pub status: String,
    /// Whether this check is required for healthy status
    pub required: bool,
    /// Service URL (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Health summary
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthSummary {
    /// Total checks performed
    pub total_checks: u32,
    /// Number of healthy checks
    pub healthy: u32,
    /// Number of required checks that are healthy
    pub required_healthy: u32,
}

/// Emit event request
#[derive(Debug, Deserialize, ToSchema)]
pub struct EmitEventRequest {
    /// Event source (e.g., "webhook", "voice", "filesystem")
    pub source: String,
    /// Event type (e.g., "file_changed", "user_command")
    pub event_type: String,
    /// Event payload (JSON object)
    pub payload: serde_json::Value,
    /// Priority (Critical/Normal)
    #[serde(default)]
    pub priority: Option<String>,
}

/// Emit event response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EmitEventResponse {
    /// Status (emitted/failed/error)
    pub status: String,
    /// Generated event ID
    pub event_id: String,
    /// HTTP status from Vigil (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Event details
    pub event: EventSummary,
}

/// Event summary
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EventSummary {
    /// Source
    pub source: String,
    /// Event type
    pub event_type: String,
    /// Priority
    pub priority: String,
}

/// Memory search request
#[derive(Debug, Deserialize, ToSchema)]
pub struct MemorySearchRequest {
    /// Search query
    pub query: String,
    /// Max results (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Memory search response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MemorySearchResponse {
    /// Status
    pub status: String,
    /// Query used
    pub query: String,
    /// Number of results
    pub results: usize,
    /// Matching points
    pub points: Vec<MemoryPoint>,
    /// Note about capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Memory point
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MemoryPoint {
    /// Point ID
    pub id: serde_json::Value,
    /// File path (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Content preview
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_preview: Option<String>,
}

/// Memory stats response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MemoryStatsResponse {
    /// Status
    pub status: String,
    /// Collection name
    pub collection: String,
    /// Statistics
    pub stats: CollectionStats,
    /// Configuration
    pub config: CollectionConfig,
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CollectionStats {
    /// Number of points
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_count: Option<u64>,
    /// Number of vectors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vectors_count: Option<u64>,
    /// Number of indexed vectors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_vectors_count: Option<u64>,
    /// Number of segments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments_count: Option<u64>,
    /// Collection status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Collection configuration
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CollectionConfig {
    /// Vector size
    pub vector_size: u32,
    /// Distance metric
    pub distance: String,
    /// On-disk payload setting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_disk_payload: Option<bool>,
}

/// LLM stats response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LlmStatsResponse {
    /// Status
    pub status: String,
    /// Statistics
    pub stats: LlmStats,
}

/// LLM usage statistics
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LlmStats {
    /// Total LLM calls
    pub total_calls: u64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Input tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    /// Output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    /// Average tokens per call
    pub avg_tokens_per_call: String,
    /// Session start timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_start: Option<String>,
    /// Last call timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_call: Option<String>,
    /// LLM provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Model name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

// ============================================================================
// Router
// ============================================================================

/// Vigil router
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/status", get(status))
        .route("/health", get(health))
        .route("/emit", post(emit_event))
        .route("/memory/search", post(memory_search))
        .route("/memory/stats", get(memory_stats))
        .route("/llm/stats", get(llm_stats))
}

// ============================================================================
// Handlers
// ============================================================================

/// Get Vigil orchestrator status
#[utoipa::path(
    get,
    path = "/api/v1/vigil/status",
    tag = "vigil",
    responses(
        (status = 200, description = "Vigil status", body = StatusResponse)
    )
)]
pub async fn status() -> ApiResult<StatusResponse> {
    // Check if Vigil is running
    let vigil_running = std::process::Command::new("pgrep")
        .args(["-f", "nexcore-vigil"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    Ok(Json(StatusResponse {
        status: if vigil_running {
            "running".to_string()
        } else {
            "stopped".to_string()
        },
        process: ProcessInfo {
            name: "nexcore-vigil".to_string(),
            running: vigil_running,
        },
        endpoints: EndpointInfo {
            webhook: format!("{}/webhook", VIGIL_URL),
            health: format!("{}/health", VIGIL_URL),
        },
        components: ComponentsInfo {
            event_bus: serde_json::json!({
                "type": "Titan EventBus v2",
                "description": "Multi-priority lock-free event routing",
                "channels": ["critical", "normal"],
            }),
            memory_layer: serde_json::json!({
                "type": "Qdrant Vector Store",
                "collection": "ksb_knowledge",
                "vector_size": 1536,
                "distance": "Cosine",
            }),
            decision_engine: serde_json::json!({
                "type": "Authority-based",
                "actions": ["InvokeClaude", "QuickResponse", "SilentLog", "AutonomousAct", "Escalate"],
            }),
        },
        sources: vec![
            SourceInfo {
                name: "filesystem".to_string(),
                source_type: "FilesystemSource".to_string(),
                description: "File change detection".to_string(),
            },
            SourceInfo {
                name: "webhook".to_string(),
                source_type: "WebhookSource".to_string(),
                description: "HTTP POST triggers".to_string(),
            },
            SourceInfo {
                name: "voice".to_string(),
                source_type: "VoiceSource".to_string(),
                description: "Speech-to-text input".to_string(),
            },
            SourceInfo {
                name: "git_monitor".to_string(),
                source_type: "GitMonitor".to_string(),
                description: "Git commit/push detection".to_string(),
            },
        ],
        executors: vec![
            ExecutorInfo {
                name: "shell".to_string(),
                executor_type: "ShellExecutor".to_string(),
                description: "Run commands and scripts".to_string(),
            },
            ExecutorInfo {
                name: "notify".to_string(),
                executor_type: "NotifyExecutor".to_string(),
                description: "Desktop notifications".to_string(),
            },
            ExecutorInfo {
                name: "speech".to_string(),
                executor_type: "SpeechExecutor".to_string(),
                description: "Text-to-speech output".to_string(),
            },
        ],
    }))
}

/// Vigil health check
#[utoipa::path(
    get,
    path = "/api/v1/vigil/health",
    tag = "vigil",
    responses(
        (status = 200, description = "Health check results", body = HealthResponse)
    )
)]
pub async fn health() -> ApiResult<HealthResponse> {
    // Check process
    let process_ok = std::process::Command::new("pgrep")
        .args(["-f", "nexcore-vigil"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check Qdrant
    let qdrant_ok = reqwest::Client::new()
        .get("http://localhost:6333/collections")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    // Check Prometheus
    let prometheus_ok = reqwest::Client::new()
        .get("http://localhost:9090/-/healthy")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    // Check Grafana
    let grafana_ok = reqwest::Client::new()
        .get("http://localhost:3000/api/health")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    let overall_status = if process_ok && qdrant_ok {
        "healthy"
    } else if process_ok {
        "degraded"
    } else {
        "unhealthy"
    };

    let healthy_count = [process_ok, qdrant_ok, prometheus_ok, grafana_ok]
        .iter()
        .filter(|&&x| x)
        .count() as u32;
    let required_healthy = [process_ok, qdrant_ok].iter().filter(|&&x| x).count() as u32;

    Ok(Json(HealthResponse {
        status: overall_status.to_string(),
        checks: HealthChecks {
            vigil_process: CheckResult {
                status: if process_ok {
                    "up".to_string()
                } else {
                    "down".to_string()
                },
                required: true,
                url: None,
            },
            qdrant: CheckResult {
                status: if qdrant_ok {
                    "up".to_string()
                } else {
                    "down".to_string()
                },
                required: true,
                url: Some("http://localhost:6333".to_string()),
            },
            prometheus: CheckResult {
                status: if prometheus_ok {
                    "up".to_string()
                } else {
                    "down".to_string()
                },
                required: false,
                url: Some("http://localhost:9090".to_string()),
            },
            grafana: CheckResult {
                status: if grafana_ok {
                    "up".to_string()
                } else {
                    "down".to_string()
                },
                required: false,
                url: Some("http://localhost:3000".to_string()),
            },
        },
        summary: HealthSummary {
            total_checks: 4,
            healthy: healthy_count,
            required_healthy,
        },
    }))
}

/// Emit event to Vigil's event bus
#[utoipa::path(
    post,
    path = "/api/v1/vigil/emit",
    tag = "vigil",
    request_body = EmitEventRequest,
    responses(
        (status = 200, description = "Event emitted", body = EmitEventResponse)
    )
)]
pub async fn emit_event(Json(req): Json<EmitEventRequest>) -> ApiResult<EmitEventResponse> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let event_id = format!("{:032x}", ts);
    let timestamp = nexcore_chrono::DateTime::now().to_rfc3339();
    let priority = req.priority.unwrap_or_else(|| "Normal".to_string());

    let event_payload = serde_json::json!({
        "id": &event_id,
        "source": &req.source,
        "event_type": &req.event_type,
        "payload": req.payload,
        "priority": &priority,
        "timestamp": timestamp,
    });

    let client = reqwest::Client::new();
    let result = client
        .post(format!("{}/webhook", VIGIL_URL))
        .header("Content-Type", "application/json")
        .header("x-api-key", "secret-key")
        .json(&event_payload)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    match result {
        Ok(response) => {
            let status_code = response.status();
            Ok(Json(EmitEventResponse {
                status: if status_code.is_success() {
                    "emitted".to_string()
                } else {
                    "failed".to_string()
                },
                event_id,
                http_status: Some(status_code.as_u16()),
                error: None,
                event: EventSummary {
                    source: req.source,
                    event_type: req.event_type,
                    priority,
                },
            }))
        }
        Err(e) => Ok(Json(EmitEventResponse {
            status: "error".to_string(),
            event_id,
            http_status: None,
            error: Some(e.to_string()),
            event: EventSummary {
                source: req.source,
                event_type: req.event_type,
                priority,
            },
        })),
    }
}

/// Search Vigil's memory layer
#[utoipa::path(
    post,
    path = "/api/v1/vigil/memory/search",
    tag = "vigil",
    request_body = MemorySearchRequest,
    responses(
        (status = 200, description = "Search results", body = MemorySearchResponse)
    )
)]
pub async fn memory_search(
    Json(req): Json<MemorySearchRequest>,
) -> ApiResult<MemorySearchResponse> {
    let client = reqwest::Client::new();
    let limit = req.limit.unwrap_or(10);

    let scroll_request = serde_json::json!({
        "limit": limit,
        "with_payload": true,
        "with_vector": false,
    });

    let result = client
        .post("http://localhost:6333/collections/ksb_knowledge/points/scroll")
        .json(&scroll_request)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    match result {
        Ok(response) => {
            if response.status().is_success() {
                let body: serde_json::Value =
                    response.json().await.unwrap_or(serde_json::json!({}));

                let points: Vec<MemoryPoint> = body
                    .get("result")
                    .and_then(|r| r.get("points"))
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter(|p| {
                                p.get("payload")
                                    .and_then(|pl| pl.get("content"))
                                    .and_then(|c| c.as_str())
                                    .map(|content| {
                                        content.to_lowercase().contains(&req.query.to_lowercase())
                                    })
                                    .unwrap_or(false)
                            })
                            .take(limit)
                            .map(|p| MemoryPoint {
                                id: p.get("id").cloned().unwrap_or(serde_json::json!(null)),
                                path: p
                                    .get("payload")
                                    .and_then(|pl| pl.get("path"))
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                                content_preview: p
                                    .get("payload")
                                    .and_then(|pl| pl.get("content"))
                                    .and_then(|c| c.as_str())
                                    .map(|s| {
                                        if s.len() > 200 {
                                            format!("{}...", &s[..200])
                                        } else {
                                            s.to_string()
                                        }
                                    }),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let results_count = points.len();
                Ok(Json(MemorySearchResponse {
                    status: "success".to_string(),
                    query: req.query,
                    results: results_count,
                    points,
                    note: Some(
                        "Full semantic search requires embedding API integration".to_string(),
                    ),
                }))
            } else {
                Err(ApiError::new(
                    "QDRANT_ERROR",
                    format!("Qdrant returned status {}", response.status()),
                ))
            }
        }
        Err(e) => Err(ApiError::new("QDRANT_CONNECTION", e.to_string())),
    }
}

/// Get Vigil memory statistics
#[utoipa::path(
    get,
    path = "/api/v1/vigil/memory/stats",
    tag = "vigil",
    responses(
        (status = 200, description = "Memory statistics", body = MemoryStatsResponse)
    )
)]
pub async fn memory_stats() -> ApiResult<MemoryStatsResponse> {
    let client = reqwest::Client::new();

    let result = client
        .get("http://localhost:6333/collections/ksb_knowledge")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    match result {
        Ok(response) => {
            if response.status().is_success() {
                let body: serde_json::Value =
                    response.json().await.unwrap_or(serde_json::json!({}));
                let collection = body.get("result").cloned().unwrap_or(serde_json::json!({}));

                Ok(Json(MemoryStatsResponse {
                    status: "success".to_string(),
                    collection: "ksb_knowledge".to_string(),
                    stats: CollectionStats {
                        points_count: collection.get("points_count").and_then(|v| v.as_u64()),
                        vectors_count: collection.get("vectors_count").and_then(|v| v.as_u64()),
                        indexed_vectors_count: collection
                            .get("indexed_vectors_count")
                            .and_then(|v| v.as_u64()),
                        segments_count: collection.get("segments_count").and_then(|v| v.as_u64()),
                        status: collection
                            .get("status")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    },
                    config: CollectionConfig {
                        vector_size: 1536,
                        distance: "Cosine".to_string(),
                        on_disk_payload: collection
                            .get("config")
                            .and_then(|c| c.get("params"))
                            .and_then(|p| p.get("on_disk_payload"))
                            .and_then(|v| v.as_bool()),
                    },
                }))
            } else {
                Err(ApiError::new(
                    "QDRANT_ERROR",
                    format!("Qdrant returned status {}", response.status()),
                ))
            }
        }
        Err(e) => Err(ApiError::new("QDRANT_CONNECTION", e.to_string())),
    }
}

/// Get Vigil LLM usage statistics
#[utoipa::path(
    get,
    path = "/api/v1/vigil/llm/stats",
    tag = "vigil",
    responses(
        (status = 200, description = "LLM usage statistics", body = LlmStatsResponse)
    )
)]
pub async fn llm_stats() -> ApiResult<LlmStatsResponse> {
    let client = reqwest::Client::new();

    let result = client
        .get(format!("{}/stats", VIGIL_URL))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    match result {
        Ok(response) => {
            if response.status().is_success() {
                let stats: serde_json::Value =
                    response.json().await.unwrap_or(serde_json::json!({}));

                let total_calls = stats
                    .get("total_calls")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let total_tokens = stats
                    .get("total_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let avg_tokens = if total_calls > 0 {
                    total_tokens as f64 / total_calls as f64
                } else {
                    0.0
                };

                Ok(Json(LlmStatsResponse {
                    status: "success".to_string(),
                    stats: LlmStats {
                        total_calls,
                        total_tokens,
                        input_tokens: stats.get("input_tokens").and_then(|v| v.as_u64()),
                        output_tokens: stats.get("output_tokens").and_then(|v| v.as_u64()),
                        avg_tokens_per_call: format!("{:.1}", avg_tokens),
                        session_start: stats
                            .get("session_start")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        last_call: stats
                            .get("last_call")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        provider: stats
                            .get("provider")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        model: stats
                            .get("model")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    },
                }))
            } else {
                // Return empty stats if Vigil not running
                Ok(Json(LlmStatsResponse {
                    status: "unavailable".to_string(),
                    stats: LlmStats {
                        total_calls: 0,
                        total_tokens: 0,
                        input_tokens: None,
                        output_tokens: None,
                        avg_tokens_per_call: "0.0".to_string(),
                        session_start: None,
                        last_call: None,
                        provider: None,
                        model: None,
                    },
                }))
            }
        }
        Err(_) => {
            // Return empty stats if Vigil not reachable
            Ok(Json(LlmStatsResponse {
                status: "unavailable".to_string(),
                stats: LlmStats {
                    total_calls: 0,
                    total_tokens: 0,
                    input_tokens: None,
                    output_tokens: None,
                    avg_tokens_per_call: "0.0".to_string(),
                    session_start: None,
                    last_call: None,
                    provider: None,
                    model: None,
                },
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_status() -> Result<(), ApiError> {
        let response = status().await?.0;
        // Process may or may not be running
        assert!(response.status == "running" || response.status == "stopped");
        Ok(())
    }

    #[tokio::test]
    async fn test_health() -> Result<(), ApiError> {
        health().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_llm_stats() -> Result<(), ApiError> {
        llm_stats().await?;
        Ok(())
    }
}
