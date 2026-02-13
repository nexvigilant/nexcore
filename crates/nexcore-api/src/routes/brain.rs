//! Brain/memory system endpoints

use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

/// Session create request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SessionCreateRequest {
    /// Optional session name
    pub name: Option<String>,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Session response
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    /// Session ID (UUID)
    pub id: String,
    /// Session name
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: String,
    /// Artifact count
    pub artifact_count: usize,
}

/// Artifact save request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ArtifactSaveRequest {
    /// Session ID
    pub session_id: String,
    /// Artifact name (e.g., "task.md")
    pub name: String,
    /// Artifact content
    pub content: String,
    /// Optional artifact type
    pub artifact_type: Option<String>,
}

/// Artifact response
#[derive(Debug, Serialize, ToSchema)]
pub struct ArtifactResponse {
    /// Artifact name
    pub name: String,
    /// Artifact content
    pub content: String,
    /// Version number (if resolved)
    pub version: Option<u32>,
    /// Last modified timestamp
    pub modified_at: String,
}

/// Code tracker request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CodeTrackerRequest {
    /// File path to track
    pub path: String,
}

/// Code tracker response
#[derive(Debug, Serialize, ToSchema)]
pub struct CodeTrackerResponse {
    /// File path
    pub path: String,
    /// Content hash
    pub hash: String,
    /// Has changed since tracking
    pub changed: bool,
    /// Tracked at timestamp
    pub tracked_at: String,
}

/// Brain router
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/sessions", post(session_create))
        .route("/sessions", get(session_list))
        .route("/sessions/{id}", get(session_load))
        .route("/artifacts", post(artifact_save))
        .route("/artifacts/{session_id}/{name}", get(artifact_get))
        .route(
            "/artifacts/{session_id}/{name}/resolve",
            post(artifact_resolve),
        )
        .route("/code-tracker/track", post(code_tracker_track))
        .route("/code-tracker/changed", post(code_tracker_changed))
}

/// Create a new brain session
#[utoipa::path(
    post,
    path = "/api/v1/brain/sessions",
    tag = "brain",
    request_body = SessionCreateRequest,
    responses(
        (status = 200, description = "Session created", body = SessionResponse)
    )
)]
pub async fn session_create(Json(req): Json<SessionCreateRequest>) -> ApiResult<SessionResponse> {
    let id = nexcore_id::NexId::v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Create session directory
    let brain_dir = shellexpand::tilde("~/.claude/brain/sessions").to_string();
    let session_dir = std::path::Path::new(&brain_dir).join(&id);
    std::fs::create_dir_all(&session_dir)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    // Write session metadata
    let metadata = serde_json::json!({
        "id": id,
        "name": req.name,
        "created_at": now,
        "metadata": req.metadata,
    });

    std::fs::write(
        session_dir.join("session.json"),
        serde_json::to_string_pretty(&metadata).unwrap_or_else(|_| "{}".to_string()),
    )
    .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(SessionResponse {
        id,
        name: req.name,
        created_at: now,
        artifact_count: 0,
    }))
}

/// List all brain sessions
#[utoipa::path(
    get,
    path = "/api/v1/brain/sessions",
    tag = "brain",
    responses(
        (status = 200, description = "List of sessions", body = Vec<SessionResponse>)
    )
)]
pub async fn session_list() -> ApiResult<Vec<SessionResponse>> {
    let brain_dir = shellexpand::tilde("~/.claude/brain/sessions").to_string();
    let path = std::path::Path::new(&brain_dir);

    let mut sessions = Vec::new();

    if path.exists() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let session_json = entry.path().join("session.json");
                    if session_json.exists() {
                        if let Ok(content) = std::fs::read_to_string(&session_json) {
                            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                                let artifact_count = std::fs::read_dir(entry.path())
                                    .map(|e| {
                                        e.filter(|f| {
                                            f.as_ref()
                                                .ok()
                                                .map(|f| {
                                                    f.path()
                                                        .extension()
                                                        .map(|e| e == "md")
                                                        .unwrap_or(false)
                                                })
                                                .unwrap_or(false)
                                        })
                                        .count()
                                    })
                                    .unwrap_or(0);

                                sessions.push(SessionResponse {
                                    id: meta["id"].as_str().unwrap_or("").to_string(),
                                    name: meta["name"].as_str().map(String::from),
                                    created_at: meta["created_at"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string(),
                                    artifact_count,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Json(sessions))
}

/// Load a specific session
#[utoipa::path(
    get,
    path = "/api/v1/brain/sessions/{id}",
    tag = "brain",
    params(
        ("id" = String, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session details", body = SessionResponse),
        (status = 404, description = "Session not found", body = super::common::ApiError)
    )
)]
pub async fn session_load(Path(id): Path<String>) -> Result<Json<SessionResponse>, ApiError> {
    let brain_dir = shellexpand::tilde("~/.claude/brain/sessions").to_string();
    let session_dir = std::path::Path::new(&brain_dir).join(&id);

    if !session_dir.exists() {
        return Err(ApiError::new(
            "NOT_FOUND",
            format!("Session '{}' not found", id),
        ));
    }

    let session_json = session_dir.join("session.json");
    let content = std::fs::read_to_string(&session_json)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let meta: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let artifact_count = std::fs::read_dir(&session_dir)
        .map(|e| {
            e.filter(|f| {
                f.as_ref()
                    .ok()
                    .map(|f| f.path().extension().map(|e| e == "md").unwrap_or(false))
                    .unwrap_or(false)
            })
            .count()
        })
        .unwrap_or(0);

    Ok(Json(SessionResponse {
        id: meta["id"].as_str().unwrap_or(&id).to_string(),
        name: meta["name"].as_str().map(String::from),
        created_at: meta["created_at"].as_str().unwrap_or("").to_string(),
        artifact_count,
    }))
}

/// Save an artifact to a session
#[utoipa::path(
    post,
    path = "/api/v1/brain/artifacts",
    tag = "brain",
    request_body = ArtifactSaveRequest,
    responses(
        (status = 200, description = "Artifact saved", body = ArtifactResponse)
    )
)]
pub async fn artifact_save(
    Json(req): Json<ArtifactSaveRequest>,
) -> Result<Json<ArtifactResponse>, ApiError> {
    let brain_dir = shellexpand::tilde("~/.claude/brain/sessions").to_string();
    let session_dir = std::path::Path::new(&brain_dir).join(&req.session_id);

    if !session_dir.exists() {
        return Err(ApiError::new(
            "NOT_FOUND",
            format!("Session '{}' not found", req.session_id),
        ));
    }

    let artifact_path = session_dir.join(&req.name);
    let now = chrono::Utc::now().to_rfc3339();

    std::fs::write(&artifact_path, &req.content)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    // Write metadata
    let meta = serde_json::json!({
        "name": req.name,
        "artifact_type": req.artifact_type,
        "modified_at": now,
    });

    let meta_path = session_dir.join(format!("{}.metadata.json", req.name));
    std::fs::write(
        &meta_path,
        serde_json::to_string_pretty(&meta).unwrap_or_default(),
    )
    .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(ArtifactResponse {
        name: req.name,
        content: req.content,
        version: None,
        modified_at: now,
    }))
}

/// Get an artifact from a session
#[utoipa::path(
    get,
    path = "/api/v1/brain/artifacts/{session_id}/{name}",
    tag = "brain",
    params(
        ("session_id" = String, Path, description = "Session ID"),
        ("name" = String, Path, description = "Artifact name")
    ),
    responses(
        (status = 200, description = "Artifact content", body = ArtifactResponse),
        (status = 404, description = "Artifact not found", body = super::common::ApiError)
    )
)]
pub async fn artifact_get(
    Path((session_id, name)): Path<(String, String)>,
) -> Result<Json<ArtifactResponse>, ApiError> {
    let brain_dir = shellexpand::tilde("~/.claude/brain/sessions").to_string();
    let artifact_path = std::path::Path::new(&brain_dir)
        .join(&session_id)
        .join(&name);

    if !artifact_path.exists() {
        return Err(ApiError::new(
            "NOT_FOUND",
            format!("Artifact '{}' not found", name),
        ));
    }

    let content = std::fs::read_to_string(&artifact_path)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    // Get metadata
    let meta_path = artifact_path
        .parent()
        .map(|p| p.join(format!("{}.metadata.json", name)))
        .unwrap_or_default();

    let modified_at = if meta_path.exists() {
        std::fs::read_to_string(&meta_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|m| m["modified_at"].as_str().map(String::from))
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
    } else {
        chrono::Utc::now().to_rfc3339()
    };

    Ok(Json(ArtifactResponse {
        name,
        content,
        version: None,
        modified_at,
    }))
}

/// Create an immutable resolved snapshot of an artifact
#[utoipa::path(
    post,
    path = "/api/v1/brain/artifacts/{session_id}/{name}/resolve",
    tag = "brain",
    params(
        ("session_id" = String, Path, description = "Session ID"),
        ("name" = String, Path, description = "Artifact name")
    ),
    responses(
        (status = 200, description = "Resolved artifact", body = ArtifactResponse),
        (status = 404, description = "Artifact not found", body = super::common::ApiError)
    )
)]
pub async fn artifact_resolve(
    Path((session_id, name)): Path<(String, String)>,
) -> Result<Json<ArtifactResponse>, ApiError> {
    let brain_dir = shellexpand::tilde("~/.claude/brain/sessions").to_string();
    let session_dir = std::path::Path::new(&brain_dir).join(&session_id);
    let artifact_path = session_dir.join(&name);

    if !artifact_path.exists() {
        return Err(ApiError::new(
            "NOT_FOUND",
            format!("Artifact '{}' not found", name),
        ));
    }

    let content = std::fs::read_to_string(&artifact_path)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    // Find next version number
    let mut version = 1u32;
    loop {
        let resolved_path = session_dir.join(format!("{}.resolved.{}", name, version));
        if !resolved_path.exists() {
            break;
        }
        version += 1;
    }

    // Create resolved snapshot
    let resolved_path = session_dir.join(format!("{}.resolved.{}", name, version));
    std::fs::write(&resolved_path, &content)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(ArtifactResponse {
        name,
        content,
        version: Some(version),
        modified_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Track a file for change detection
#[utoipa::path(
    post,
    path = "/api/v1/brain/code-tracker/track",
    tag = "brain",
    request_body = CodeTrackerRequest,
    responses(
        (status = 200, description = "File tracked", body = CodeTrackerResponse)
    )
)]
pub async fn code_tracker_track(
    Json(req): Json<CodeTrackerRequest>,
) -> Result<Json<CodeTrackerResponse>, ApiError> {
    let expanded = shellexpand::tilde(&req.path).to_string();
    let path = std::path::Path::new(&expanded);

    if !path.exists() {
        return Err(ApiError::new(
            "NOT_FOUND",
            format!("File '{}' not found", req.path),
        ));
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    // Calculate hash
    let hash_result = nexcore_vigilance::foundation::sha256_hash(&content);
    let hash = hash_result.hex;
    let now = chrono::Utc::now().to_rfc3339();

    // Store in code_tracker directory
    let tracker_dir = shellexpand::tilde("~/.claude/code_tracker").to_string();
    std::fs::create_dir_all(&tracker_dir)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let safe_name = req.path.replace('/', "_").replace('\\', "_");
    let tracker_file = std::path::Path::new(&tracker_dir).join(format!("{}.json", safe_name));

    let tracker_data = serde_json::json!({
        "path": req.path,
        "hash": hash,
        "tracked_at": now,
        "original_content": content,
    });

    std::fs::write(
        &tracker_file,
        serde_json::to_string_pretty(&tracker_data).unwrap_or_default(),
    )
    .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(CodeTrackerResponse {
        path: req.path,
        hash,
        changed: false,
        tracked_at: now,
    }))
}

/// Check if a tracked file has changed
#[utoipa::path(
    post,
    path = "/api/v1/brain/code-tracker/changed",
    tag = "brain",
    request_body = CodeTrackerRequest,
    responses(
        (status = 200, description = "Change status", body = CodeTrackerResponse)
    )
)]
pub async fn code_tracker_changed(
    Json(req): Json<CodeTrackerRequest>,
) -> Result<Json<CodeTrackerResponse>, ApiError> {
    let expanded = shellexpand::tilde(&req.path).to_string();
    let path = std::path::Path::new(&expanded);

    // Load tracker data
    let tracker_dir = shellexpand::tilde("~/.claude/code_tracker").to_string();
    let safe_name = req.path.replace('/', "_").replace('\\', "_");
    let tracker_file = std::path::Path::new(&tracker_dir).join(format!("{}.json", safe_name));

    if !tracker_file.exists() {
        return Err(ApiError::new(
            "NOT_FOUND",
            format!("File '{}' is not being tracked", req.path),
        ));
    }

    let tracker_content = std::fs::read_to_string(&tracker_file)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let tracker_data: serde_json::Value = serde_json::from_str(&tracker_content)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let original_hash = tracker_data["hash"].as_str().unwrap_or("");
    let tracked_at = tracker_data["tracked_at"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // Calculate current hash
    let current_content = std::fs::read_to_string(path)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let current_hash_result = nexcore_vigilance::foundation::sha256_hash(&current_content);
    let current_hash = current_hash_result.hex;
    let changed = current_hash != original_hash;

    Ok(Json(CodeTrackerResponse {
        path: req.path,
        hash: current_hash,
        changed,
        tracked_at,
    }))
}
