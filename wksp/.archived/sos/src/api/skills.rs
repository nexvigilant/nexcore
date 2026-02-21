/// Skills API client — σ Sequence (execution) + ∂ Boundary (compliance levels)
///
/// Calls /api/v1/skills on nexcore-api
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use super::{url, ApiError};

/// Skill summary (from list endpoint)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillSummary {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub compliance: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Skill detail (from get endpoint)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillDetail {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub compliance: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub smst_score: f64,
    #[serde(default)]
    pub has_scripts: bool,
    #[serde(default)]
    pub has_references: bool,
    #[serde(default)]
    pub has_templates: bool,
}

/// Skill execution result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillExecResult {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
    #[serde(default)]
    pub exit_code: i32,
    #[serde(default)]
    pub duration_ms: u64,
}

/// List all skills
pub async fn list_skills() -> Result<Vec<SkillSummary>, ApiError> {
    let endpoint = url("/api/v1/skills");
    let resp = Request::get(&endpoint).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: Vec<SkillSummary> = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Skills API returned {}", resp.status()),
        })
    }
}

/// Get skill detail by name
pub async fn get_skill(name: &str) -> Result<SkillDetail, ApiError> {
    let endpoint = url(&format!("/api/v1/skills/{name}"));
    let resp = Request::get(&endpoint).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: SkillDetail = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Skill detail API returned {}", resp.status()),
        })
    }
}

/// Execute a skill by name
pub async fn execute_skill(name: &str) -> Result<SkillExecResult, ApiError> {
    let endpoint = url(&format!("/api/v1/skills/{name}/execute"));
    let resp = Request::post(&endpoint).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: SkillExecResult = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Skill exec API returned {}", resp.status()),
        })
    }
}
