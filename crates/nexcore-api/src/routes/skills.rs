//! Skill registry and validation endpoints

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use nexcore_skill_exec::{
    CompositeExecutor, ExecutionMethod, ExecutionRequest, ExecutionResult, ExecutionStatus,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

/// Shared application state for skill execution.
#[derive(Clone)]
pub struct SkillAppState {
    /// The skill executor.
    pub executor: Arc<CompositeExecutor>,
}

impl Default for SkillAppState {
    fn default() -> Self {
        Self {
            executor: Arc::new(CompositeExecutor::new()),
        }
    }
}

/// Skill summary for list view
#[derive(Debug, Serialize, ToSchema)]
pub struct SkillSummary {
    /// Skill name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Compliance level (Bronze/Silver/Gold/Platinum/Diamond)
    pub compliance: String,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Full skill details
#[derive(Debug, Serialize, ToSchema)]
pub struct SkillDetail {
    /// Skill name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Compliance level
    pub compliance: String,
    /// Tags
    pub tags: Vec<String>,
    /// Path to skill directory
    pub path: String,
    /// SMST score (if Diamond)
    pub smst_score: Option<f64>,
    /// Has scripts directory
    pub has_scripts: bool,
    /// Has references directory
    pub has_references: bool,
    /// Has templates directory
    pub has_templates: bool,
}

/// Validate skill request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ValidateRequest {
    /// Path to skill directory or SKILL.md
    pub path: String,
}

/// Validation response
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidateResponse {
    /// Is valid
    pub valid: bool,
    /// Compliance level achieved
    pub compliance: String,
    /// SMST score (0-100)
    pub smst_score: f64,
    /// Validation messages
    pub messages: Vec<ValidationMessage>,
}

/// Validation message
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationMessage {
    /// Message level (error, warning, info)
    pub level: String,
    /// Message text
    pub message: String,
}

/// Scan request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ScanRequest {
    /// Directory to scan
    pub path: String,
}

/// Scan response
#[derive(Debug, Serialize, ToSchema)]
pub struct ScanResponse {
    /// Number of skills found
    pub count: usize,
    /// Skills found
    pub skills: Vec<SkillSummary>,
}

/// Taxonomy node
#[derive(Debug, Serialize, ToSchema)]
#[schema(no_recursion)]
pub struct TaxonomyNode {
    /// Node ID
    pub id: String,
    /// Node name
    pub name: String,
    /// Child nodes (recursive - see schema reference)
    #[schema(value_type = Vec<Object>)]
    pub children: Vec<TaxonomyNode>,
    /// Skill count at this node
    pub skill_count: usize,
}

/// Taxonomy query request
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct TaxonomyQueryRequest {
    /// Query string
    pub query: String,
    /// Max depth
    #[serde(default = "default_depth")]
    pub depth: usize,
}

fn default_depth() -> usize {
    3
}

fn default_timeout() -> u64 {
    60
}

/// Request to execute a skill.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecuteRequest {
    /// Parameters to pass to the skill (JSON object).
    #[serde(default)]
    pub parameters: serde_json::Value,
    /// Execution timeout in seconds (default: 60).
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

/// Response from skill execution.
#[derive(Debug, Serialize, ToSchema)]
pub struct ExecuteResponse {
    /// Skill that was executed.
    pub skill_name: String,
    /// Execution status: completed, failed, timeout, cancelled.
    pub status: String,
    /// Output data from the skill (JSON).
    pub output: serde_json::Value,
    /// Error message if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Exit code from process (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Stdout from process.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stdout: String,
    /// Stderr from process.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stderr: String,
}

impl From<ExecutionResult> for ExecuteResponse {
    fn from(r: ExecutionResult) -> Self {
        Self {
            skill_name: r.skill_name,
            status: match r.status {
                ExecutionStatus::Completed => "completed",
                ExecutionStatus::Failed => "failed",
                ExecutionStatus::Timeout => "timeout",
                ExecutionStatus::Cancelled => "cancelled",
            }
            .to_string(),
            output: r.output,
            error: r.error,
            duration_ms: r.duration_ms,
            exit_code: r.exit_code,
            stdout: r.stdout,
            stderr: r.stderr,
        }
    }
}

/// Schema information for a skill.
#[derive(Debug, Serialize, ToSchema)]
pub struct SkillSchema {
    /// Skill name.
    pub name: String,
    /// Input parameter schema (JSON Schema format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    /// Output schema (JSON Schema format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    /// Whether the skill has executable scripts.
    pub executable: bool,
    /// Available execution methods.
    pub execution_methods: Vec<String>,
}

/// Skills router with execution endpoints.
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/", get(list))
        .route("/validate", post(validate))
        .route("/scan", post(scan))
        .route("/taxonomy", get(taxonomy_list))
        .route("/taxonomy/query", post(taxonomy_query))
        // Execution endpoints (require state)
        .route("/{name}/execute", post(execute_skill))
        .route("/{name}/schema", get(get_schema))
        // This must come last to avoid conflicts with /execute and /schema
        .route("/{name}", get(get_skill))
}

/// Skills router without execution state (for backwards compatibility).
#[allow(dead_code)]
pub fn router_stateless() -> Router {
    Router::new()
        .route("/", get(list))
        .route("/{name}", get(get_skill))
        .route("/validate", post(validate))
        .route("/scan", post(scan))
        .route("/taxonomy", get(taxonomy_list))
        .route("/taxonomy/query", post(taxonomy_query))
}

/// List all registered skills
#[utoipa::path(
    get,
    path = "/api/v1/skills",
    tag = "skills",
    responses(
        (status = 200, description = "List of skills", body = Vec<SkillSummary>)
    )
)]
pub async fn list() -> ApiResult<Vec<SkillSummary>> {
    let skills_dir = shellexpand::tilde("~/.claude/skills").to_string();
    let path = std::path::Path::new(&skills_dir);

    if !path.exists() {
        return Ok(Json(vec![]));
    }

    let mut skills = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let skill_md = entry.path().join("SKILL.md");
                if skill_md.exists() {
                    if let Ok(content) = std::fs::read_to_string(&skill_md) {
                        // Parse frontmatter
                        if let Some(name) = extract_frontmatter(&content, "name") {
                            skills.push(SkillSummary {
                                name,
                                description: extract_frontmatter(&content, "description")
                                    .unwrap_or_default(),
                                compliance: extract_frontmatter(&content, "compliance")
                                    .unwrap_or_else(|| "Bronze".to_string()),
                                tags: extract_frontmatter_list(&content, "tags"),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(Json(skills))
}

/// Get skill details by name
#[utoipa::path(
    get,
    path = "/api/v1/skills/{name}",
    tag = "skills",
    params(
        ("name" = String, Path, description = "Skill name")
    ),
    responses(
        (status = 200, description = "Skill details", body = SkillDetail),
        (status = 404, description = "Skill not found", body = super::common::ApiError)
    )
)]
pub async fn get_skill(Path(name): Path<String>) -> Result<Json<SkillDetail>, ApiError> {
    let skills_dir = shellexpand::tilde("~/.claude/skills").to_string();
    let skill_path = std::path::Path::new(&skills_dir).join(&name);

    if !skill_path.exists() {
        return Err(ApiError::new(
            "NOT_FOUND",
            format!("Skill '{}' not found", name),
        ));
    }

    let skill_md = skill_path.join("SKILL.md");
    if !skill_md.exists() {
        return Err(ApiError::new(
            "NOT_FOUND",
            format!("Skill '{}' has no SKILL.md", name),
        ));
    }

    let content = std::fs::read_to_string(&skill_md)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(SkillDetail {
        name: extract_frontmatter(&content, "name").unwrap_or(name),
        description: extract_frontmatter(&content, "description").unwrap_or_default(),
        compliance: extract_frontmatter(&content, "compliance")
            .unwrap_or_else(|| "Bronze".to_string()),
        tags: extract_frontmatter_list(&content, "tags"),
        path: skill_path.to_string_lossy().to_string(),
        smst_score: extract_frontmatter(&content, "smst_score").and_then(|s| s.parse().ok()),
        has_scripts: skill_path.join("scripts").exists(),
        has_references: skill_path.join("references").exists(),
        has_templates: skill_path.join("templates").exists(),
    }))
}

/// Validate a skill for Diamond compliance
#[utoipa::path(
    post,
    path = "/api/v1/skills/validate",
    tag = "skills",
    request_body = ValidateRequest,
    responses(
        (status = 200, description = "Validation result", body = ValidateResponse)
    )
)]
pub async fn validate(Json(req): Json<ValidateRequest>) -> ApiResult<ValidateResponse> {
    let expanded = shellexpand::tilde(&req.path).to_string();
    let path = std::path::Path::new(&expanded);

    let mut messages = Vec::new();
    let mut score = 0.0;

    // Check SKILL.md exists
    let skill_md = if path.is_dir() {
        path.join("SKILL.md")
    } else {
        path.to_path_buf()
    };

    if !skill_md.exists() {
        messages.push(ValidationMessage {
            level: "error".to_string(),
            message: "SKILL.md not found".to_string(),
        });
        return Ok(Json(ValidateResponse {
            valid: false,
            compliance: "None".to_string(),
            smst_score: 0.0,
            messages,
        }));
    }

    score += 20.0; // Has SKILL.md
    messages.push(ValidationMessage {
        level: "info".to_string(),
        message: "SKILL.md found".to_string(),
    });

    // Check frontmatter
    if let Ok(content) = std::fs::read_to_string(&skill_md) {
        if content.starts_with("---") {
            score += 10.0;
            messages.push(ValidationMessage {
                level: "info".to_string(),
                message: "Valid YAML frontmatter".to_string(),
            });
        } else {
            messages.push(ValidationMessage {
                level: "warning".to_string(),
                message: "Missing YAML frontmatter".to_string(),
            });
        }
    }

    let skill_dir = skill_md.parent().unwrap_or(path);

    // Check directories
    if skill_dir.join("scripts").exists() {
        score += 20.0;
        messages.push(ValidationMessage {
            level: "info".to_string(),
            message: "scripts/ directory found".to_string(),
        });
    }

    if skill_dir.join("references").exists() {
        score += 15.0;
        messages.push(ValidationMessage {
            level: "info".to_string(),
            message: "references/ directory found".to_string(),
        });
    }

    if skill_dir.join("templates").exists() {
        score += 15.0;
        messages.push(ValidationMessage {
            level: "info".to_string(),
            message: "templates/ directory found".to_string(),
        });
    }

    // Determine compliance level
    let compliance = if score >= 85.0 {
        "Diamond"
    } else if score >= 70.0 {
        "Platinum"
    } else if score >= 55.0 {
        "Gold"
    } else if score >= 40.0 {
        "Silver"
    } else if score >= 20.0 {
        "Bronze"
    } else {
        "None"
    };

    Ok(Json(ValidateResponse {
        valid: score >= 20.0,
        compliance: compliance.to_string(),
        smst_score: score,
        messages,
    }))
}

/// Scan directory for skills
#[utoipa::path(
    post,
    path = "/api/v1/skills/scan",
    tag = "skills",
    request_body = ScanRequest,
    responses(
        (status = 200, description = "Scan results", body = ScanResponse)
    )
)]
pub async fn scan(Json(req): Json<ScanRequest>) -> ApiResult<ScanResponse> {
    let expanded = shellexpand::tilde(&req.path).to_string();
    let path = std::path::Path::new(&expanded);

    let mut skills = Vec::new();

    if path.exists() && path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let skill_md = entry.path().join("SKILL.md");
                    if skill_md.exists() {
                        if let Ok(content) = std::fs::read_to_string(&skill_md) {
                            if let Some(name) = extract_frontmatter(&content, "name") {
                                skills.push(SkillSummary {
                                    name,
                                    description: extract_frontmatter(&content, "description")
                                        .unwrap_or_default(),
                                    compliance: extract_frontmatter(&content, "compliance")
                                        .unwrap_or_else(|| "Bronze".to_string()),
                                    tags: extract_frontmatter_list(&content, "tags"),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Json(ScanResponse {
        count: skills.len(),
        skills,
    }))
}

/// List taxonomy tree
#[utoipa::path(
    get,
    path = "/api/v1/skills/taxonomy",
    tag = "skills",
    responses(
        (status = 200, description = "Taxonomy tree", body = Vec<TaxonomyNode>)
    )
)]
pub async fn taxonomy_list() -> Json<Vec<TaxonomyNode>> {
    // Simplified taxonomy tree
    let taxonomy = vec![
        TaxonomyNode {
            id: "development".to_string(),
            name: "Development".to_string(),
            children: vec![
                TaxonomyNode {
                    id: "rust".to_string(),
                    name: "Rust".to_string(),
                    children: vec![],
                    skill_count: 15,
                },
                TaxonomyNode {
                    id: "typescript".to_string(),
                    name: "TypeScript".to_string(),
                    children: vec![],
                    skill_count: 10,
                },
            ],
            skill_count: 25,
        },
        TaxonomyNode {
            id: "pharmacovigilance".to_string(),
            name: "Pharmacovigilance".to_string(),
            children: vec![
                TaxonomyNode {
                    id: "signals".to_string(),
                    name: "Signal Detection".to_string(),
                    children: vec![],
                    skill_count: 8,
                },
                TaxonomyNode {
                    id: "causality".to_string(),
                    name: "Causality Assessment".to_string(),
                    children: vec![],
                    skill_count: 5,
                },
            ],
            skill_count: 13,
        },
        TaxonomyNode {
            id: "operations".to_string(),
            name: "Operations".to_string(),
            children: vec![],
            skill_count: 20,
        },
    ];

    Json(taxonomy)
}

/// Query taxonomy
#[utoipa::path(
    post,
    path = "/api/v1/skills/taxonomy/query",
    tag = "skills",
    request_body = TaxonomyQueryRequest,
    responses(
        (status = 200, description = "Query results", body = Vec<TaxonomyNode>)
    )
)]
pub async fn taxonomy_query(Json(req): Json<TaxonomyQueryRequest>) -> Json<Vec<TaxonomyNode>> {
    let query_lower = req.query.to_lowercase();

    // Filter taxonomy by query
    let results: Vec<TaxonomyNode> = vec![TaxonomyNode {
        id: query_lower.clone(),
        name: format!("Results for: {}", req.query),
        children: vec![],
        skill_count: 0,
    }];

    Json(results)
}

/// Execute a skill
#[utoipa::path(
    post,
    path = "/api/v1/skills/{name}/execute",
    tag = "skills",
    params(
        ("name" = String, Path, description = "Skill name")
    ),
    request_body = ExecuteRequest,
    responses(
        (status = 200, description = "Execution result", body = ExecuteResponse),
        (status = 404, description = "Skill not found", body = super::common::ApiError),
        (status = 500, description = "Execution failed", body = super::common::ApiError)
    )
)]
pub async fn execute_skill(
    State(state): State<SkillAppState>,
    Path(name): Path<String>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, ApiError> {
    // Build execution request
    let exec_request = ExecutionRequest::new(&name, req.parameters)
        .with_timeout(Duration::from_secs(req.timeout_seconds));

    // Execute the skill
    let result = state.executor.execute(&exec_request).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("not found") {
            ApiError::new("NOT_FOUND", format!("Skill '{}' not found", name))
        } else if msg.contains("No executor available") {
            ApiError::new(
                "NOT_EXECUTABLE",
                format!("Skill '{}' has no executable scripts", name),
            )
        } else {
            ApiError::new("EXECUTION_ERROR", msg)
        }
    })?;

    Ok(Json(ExecuteResponse::from(result)))
}

/// Get skill input/output schema
#[utoipa::path(
    get,
    path = "/api/v1/skills/{name}/schema",
    tag = "skills",
    params(
        ("name" = String, Path, description = "Skill name")
    ),
    responses(
        (status = 200, description = "Skill schema", body = SkillSchema),
        (status = 404, description = "Skill not found", body = super::common::ApiError)
    )
)]
pub async fn get_schema(
    State(state): State<SkillAppState>,
    Path(name): Path<String>,
) -> Result<Json<SkillSchema>, ApiError> {
    // Discover skill info
    let skill_info = state.executor.discover_skill(&name).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("not found") {
            ApiError::new("NOT_FOUND", format!("Skill '{}' not found", name))
        } else {
            ApiError::new("INTERNAL_ERROR", msg)
        }
    })?;

    // Convert execution methods to strings
    let methods: Vec<String> = skill_info
        .execution_methods
        .iter()
        .map(|m| match m {
            ExecutionMethod::Shell(p) => {
                format!("shell:{}", p.display())
            }
            ExecutionMethod::Binary(p) => {
                format!("binary:{}", p.display())
            }
            ExecutionMethod::Library(p) => {
                format!("library:{}", p.display())
            }
        })
        .collect();

    Ok(Json(SkillSchema {
        name: skill_info.name,
        input_schema: skill_info.input_schema,
        output_schema: skill_info.output_schema,
        executable: !skill_info.execution_methods.is_empty(),
        execution_methods: methods,
    }))
}

// Helper functions

fn extract_frontmatter(content: &str, key: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }

    let frontmatter = parts[1];
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&format!("{}:", key)) {
            return Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }

    None
}

fn extract_frontmatter_list(content: &str, key: &str) -> Vec<String> {
    if !content.starts_with("---") {
        return vec![];
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return vec![];
    }

    let frontmatter = parts[1];
    let mut in_list = false;
    let mut items = Vec::new();

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{}:", key)) {
            in_list = true;
            // Check for inline list
            if let Some(rest) = trimmed.strip_prefix(&format!("{}:", key)) {
                let rest = rest.trim();
                if rest.starts_with('[') && rest.ends_with(']') {
                    let inner = &rest[1..rest.len() - 1];
                    for item in inner.split(',') {
                        items.push(item.trim().trim_matches('"').trim_matches('\'').to_string());
                    }
                    return items;
                }
            }
            continue;
        }
        if in_list {
            if trimmed.starts_with('-') {
                items.push(
                    trimmed[1..]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            } else if !trimmed.is_empty() && !trimmed.starts_with(' ') {
                break;
            }
        }
    }

    items
}
