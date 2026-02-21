/// Academy API client — π Persistence (enrollments) + μ Mapping (courses)
///
/// Calls /api/v1/academy/* on nexcore-api
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use super::{url, ApiError};

/// Course summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Course {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub level: String,
}

/// Enrollment with progress
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Enrollment {
    #[serde(default)]
    pub course_code: String,
    #[serde(default)]
    pub course_title: String,
    #[serde(default)]
    pub progress: f64,
    #[serde(default)]
    pub enrolled_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// Pathway node
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathwayNode {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub level: String,
}

/// Pathway
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Pathway {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub nodes: Vec<PathwayNode>,
}

/// List all courses
pub async fn list_courses() -> Result<Vec<Course>, ApiError> {
    let endpoint = url("/api/v1/academy/courses");
    let resp = Request::get(&endpoint).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: Vec<Course> = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Courses API returned {}", resp.status()),
        })
    }
}

/// List enrollments
pub async fn list_enrollments() -> Result<Vec<Enrollment>, ApiError> {
    let endpoint = url("/api/v1/academy/enrollments");
    let resp = Request::get(&endpoint).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: Vec<Enrollment> = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Enrollments API returned {}", resp.status()),
        })
    }
}

/// List pathways
pub async fn list_pathways() -> Result<Vec<Pathway>, ApiError> {
    let endpoint = url("/api/v1/academy/pathways");
    let resp = Request::get(&endpoint).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: Vec<Pathway> = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Pathways API returned {}", resp.status()),
        })
    }
}
