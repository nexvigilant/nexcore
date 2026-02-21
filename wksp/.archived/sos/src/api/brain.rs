/// Brain API client — π Persistence (sessions/artifacts) + λ Location (browse)
///
/// Calls /api/v1/brain/* on nexcore-api
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use super::{url, ApiError};

/// Brain session summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub artifact_count: u32,
    #[serde(default)]
    pub status: String,
}

/// Brain artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub artifact_type: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub version: u32,
}

/// List all brain sessions
pub async fn list_sessions() -> Result<Vec<SessionSummary>, ApiError> {
    let resp = Request::get(&url("/api/v1/brain/sessions")).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: Vec<SessionSummary> = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Brain sessions API returned {}", resp.status()),
        })
    }
}

/// Get artifacts for a session
pub async fn get_session_artifacts(session_id: &str) -> Result<Vec<Artifact>, ApiError> {
    let resp = Request::get(&url(&format!("/api/v1/brain/sessions/{session_id}/artifacts")))
        .send()
        .await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: Vec<Artifact> = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Brain artifacts API returned {}", resp.status()),
        })
    }
}

/// Get a specific artifact
pub async fn get_artifact(artifact_id: &str) -> Result<Artifact, ApiError> {
    let resp = Request::get(&url(&format!("/api/v1/brain/artifacts/{artifact_id}")))
        .send()
        .await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: Artifact = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Brain artifact API returned {}", resp.status()),
        })
    }
}
