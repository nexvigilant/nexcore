//! Projects & Deliverables module — R&D workspaces within Circles.
//!
//! Projects are Circle-scoped. Each project follows a stage pipeline
//! (Initiate → Design → Execute → Analyze → Report → Review → Publish → Closed).
//! Deliverables are the tangible outputs of projects.

use crate::ApiState;
use crate::persistence::{
    CircleRole, DeliverableRecord, DeliverableStatus, DeliverableType, FeedEntryRecord,
    FeedEntryType, MemberStatus, ProjectRecord, ProjectStage, ProjectStatus, ProjectType,
    ReviewStatus,
};
use crate::routes::common::ApiError;
use axum::extract::{Json, Path, State};
use axum::routing::{get, patch, post};
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// API Types
// ============================================================================

/// Project for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Project {
    pub id: String,
    pub circle_id: String,
    pub name: String,
    pub description: String,
    pub project_type: String,
    pub stage: String,
    pub status: String,
    pub therapeutic_area: Option<String>,
    pub drug_names: Vec<String>,
    pub indications: Vec<String>,
    pub data_sources: Vec<String>,
    pub started_at: DateTime,
    pub target_completion: Option<DateTime>,
    pub completed_at: Option<DateTime>,
    pub lead_user_id: String,
    pub created_by: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// Create project request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
    pub project_type: Option<String>,
    pub therapeutic_area: Option<String>,
    pub drug_names: Option<Vec<String>>,
    pub indications: Option<Vec<String>>,
    pub data_sources: Option<Vec<String>>,
    pub target_completion: Option<DateTime>,
    pub lead_user_id: String,
    pub created_by: String,
}

/// Update project request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub updated_by: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub therapeutic_area: Option<String>,
    pub drug_names: Option<Vec<String>>,
    pub indications: Option<Vec<String>>,
    pub data_sources: Option<Vec<String>>,
    pub target_completion: Option<DateTime>,
    pub lead_user_id: Option<String>,
}

/// Advance stage request (with optional review gate)
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AdvanceStageRequest {
    pub advanced_by: String,
}

/// Deliverable for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Deliverable {
    pub id: String,
    pub project_id: String,
    pub circle_id: String,
    pub name: String,
    pub deliverable_type: String,
    pub status: String,
    pub version: u32,
    pub file_url: Option<String>,
    pub content_hash: Option<String>,
    pub reviewed_by: Option<String>,
    pub review_status: String,
    pub review_notes: Option<String>,
    pub created_by: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// Create deliverable request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateDeliverableRequest {
    pub name: String,
    pub deliverable_type: Option<String>,
    pub file_url: Option<String>,
    pub created_by: String,
}

/// Update deliverable request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateDeliverableRequest {
    pub updated_by: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub file_url: Option<String>,
    pub content_hash: Option<String>,
}

/// Review deliverable request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ReviewDeliverableRequest {
    pub reviewed_by: String,
    pub review_status: String,
    pub review_notes: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn record_to_project(r: ProjectRecord) -> Project {
    Project {
        id: r.id,
        circle_id: r.circle_id,
        name: r.name,
        description: r.description,
        project_type: enum_to_str(&r.project_type),
        stage: enum_to_str(&r.stage),
        status: enum_to_str(&r.status),
        therapeutic_area: r.therapeutic_area,
        drug_names: r.drug_names,
        indications: r.indications,
        data_sources: r.data_sources,
        started_at: r.started_at,
        target_completion: r.target_completion,
        completed_at: r.completed_at,
        lead_user_id: r.lead_user_id,
        created_by: r.created_by,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

fn record_to_deliverable(r: DeliverableRecord) -> Deliverable {
    Deliverable {
        id: r.id,
        project_id: r.project_id,
        circle_id: r.circle_id,
        name: r.name,
        deliverable_type: enum_to_str(&r.deliverable_type),
        status: enum_to_str(&r.status),
        version: r.version,
        file_url: r.file_url,
        content_hash: r.content_hash,
        reviewed_by: r.reviewed_by,
        review_status: enum_to_str(&r.review_status),
        review_notes: r.review_notes,
        created_by: r.created_by,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

fn enum_to_str<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_string(val)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

fn parse_enum<T: serde::de::DeserializeOwned + Default>(s: &str) -> T {
    let quoted = format!("\"{s}\"");
    serde_json::from_str(&quoted).unwrap_or_default()
}

fn err(code: &str, msg: impl Into<String>) -> ApiError {
    ApiError::new(code, msg)
}

/// Best-effort feed entry save.
async fn save_feed_best_effort(state: &ApiState, entry: FeedEntryRecord) {
    if let Err(e) = state.persistence.save_feed_entry(&entry).await {
        tracing::warn!("Failed to save feed entry: {e}");
    }
}

/// Best-effort circle count update.
async fn update_project_count_best_effort(state: &ApiState, circle_id: &str, delta: i32) {
    if let Ok(Some(mut circle)) = state.persistence.get_circle(circle_id).await {
        circle.project_count = (circle.project_count as i32 + delta).max(0) as u32;
        circle.updated_at = DateTime::now();
        if let Err(e) = state.persistence.save_circle(&circle).await {
            tracing::warn!("Failed to update circle project count: {e}");
        }
    }
}

/// Check if user has at least the given role in a circle.
async fn check_role(
    state: &ApiState,
    circle_id: &str,
    user_id: &str,
    min_role: CircleRole,
) -> Result<bool, ApiError> {
    let member = state
        .persistence
        .get_circle_member(circle_id, user_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    let Some(m) = member else {
        return Ok(false);
    };
    if m.status != MemberStatus::Active {
        return Ok(false);
    }

    let role_level = |r: &CircleRole| -> u8 {
        match r {
            CircleRole::Founder => 6,
            CircleRole::Lead => 5,
            CircleRole::Researcher => 4,
            CircleRole::Reviewer => 3,
            CircleRole::Member => 2,
            CircleRole::Observer => 1,
        }
    };

    Ok(role_level(&m.role) >= role_level(&min_role))
}

// ============================================================================
// Project CRUD
// ============================================================================

/// Create a project within a circle (Researcher+ only)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{id}/projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 200, description = "Project created", body = Project),
    ),
    tag = "projects"
)]
pub async fn create_project(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, ApiError> {
    if req.name.trim().is_empty() {
        return Err(err("VALIDATION_ERROR", "Project name is required"));
    }

    // Verify circle exists
    state
        .persistence
        .get_circle(&circle_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Circle not found"))?;

    // Check role (Researcher+)
    if !check_role(&state, &circle_id, &req.created_by, CircleRole::Researcher).await? {
        return Err(err("FORBIDDEN", "Requires Researcher role or higher"));
    }

    let now = DateTime::now();
    let record = ProjectRecord {
        id: nexcore_id::NexId::v4().to_string(),
        circle_id: circle_id.clone(),
        name: req.name,
        description: req.description,
        project_type: req
            .project_type
            .map(|t| parse_enum(&t))
            .unwrap_or(ProjectType::Custom),
        stage: ProjectStage::Initiate,
        status: ProjectStatus::Active,
        therapeutic_area: req.therapeutic_area,
        drug_names: req.drug_names.unwrap_or_default(),
        indications: req.indications.unwrap_or_default(),
        data_sources: req.data_sources.unwrap_or_default(),
        started_at: now,
        target_completion: req.target_completion,
        completed_at: None,
        lead_user_id: req.lead_user_id,
        created_by: req.created_by.clone(),
        created_at: now,
        updated_at: now,
    };

    state
        .persistence
        .save_project(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Update circle project count
    update_project_count_best_effort(&state, &circle_id, 1).await;

    // Feed entry
    save_feed_best_effort(
        &state,
        FeedEntryRecord {
            id: nexcore_id::NexId::v4().to_string(),
            circle_id,
            entry_type: FeedEntryType::ProjectCreated,
            actor_user_id: req.created_by,
            content: format!("Created project: {}", record.name),
            reference_id: Some(record.id.clone()),
            reference_type: Some("project".to_string()),
            created_at: now,
        },
    )
    .await;

    Ok(Json(record_to_project(record)))
}

/// List projects within a circle (Member+ can view)
#[utoipa::path(
    get,
    path = "/api/v1/circles/{id}/projects",
    responses(
        (status = 200, description = "Project list", body = Vec<Project>),
    ),
    tag = "projects"
)]
pub async fn list_projects(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
) -> Result<Json<Vec<Project>>, ApiError> {
    let records = state
        .persistence
        .list_projects(&circle_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    let projects: Vec<Project> = records.into_iter().map(record_to_project).collect();
    Ok(Json(projects))
}

/// Get project detail
#[utoipa::path(
    get,
    path = "/api/v1/circles/{cid}/projects/{pid}",
    responses(
        (status = 200, description = "Project detail", body = Project),
    ),
    tag = "projects"
)]
pub async fn get_project(
    State(state): State<ApiState>,
    Path((circle_id, project_id)): Path<(String, String)>,
) -> Result<Json<Project>, ApiError> {
    let record = state
        .persistence
        .get_project(&project_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Project not found"))?;

    if record.circle_id != circle_id {
        return Err(err("NOT_FOUND", "Project not found in this circle"));
    }

    Ok(Json(record_to_project(record)))
}

/// Update project (Lead or project lead)
#[utoipa::path(
    patch,
    path = "/api/v1/circles/{cid}/projects/{pid}",
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Project updated", body = Project),
    ),
    tag = "projects"
)]
pub async fn update_project(
    State(state): State<ApiState>,
    Path((_circle_id, project_id)): Path<(String, String)>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, ApiError> {
    let mut record = state
        .persistence
        .get_project(&project_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Project not found"))?;

    // Lead+ or project lead required
    if let Some(ref actor) = req.updated_by {
        let is_project_lead = record.lead_user_id == *actor;
        if !is_project_lead
            && !check_role(&state, &record.circle_id, actor, CircleRole::Lead).await?
        {
            return Err(err(
                "FORBIDDEN",
                "Requires Lead role or project lead status",
            ));
        }
    }

    if let Some(name) = req.name {
        record.name = name;
    }
    if let Some(description) = req.description {
        record.description = description;
    }
    if let Some(status) = req.status {
        record.status = parse_enum(&status);
    }
    if let Some(area) = req.therapeutic_area {
        record.therapeutic_area = Some(area);
    }
    if let Some(drugs) = req.drug_names {
        record.drug_names = drugs;
    }
    if let Some(indications) = req.indications {
        record.indications = indications;
    }
    if let Some(sources) = req.data_sources {
        record.data_sources = sources;
    }
    if let Some(target) = req.target_completion {
        record.target_completion = Some(target);
    }
    if let Some(lead) = req.lead_user_id {
        record.lead_user_id = lead;
    }
    record.updated_at = DateTime::now();

    state
        .persistence
        .save_project(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(record_to_project(record)))
}

/// Advance project to next stage (Reviewer+ only — review gate)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{cid}/projects/{pid}/advance",
    request_body = AdvanceStageRequest,
    responses(
        (status = 200, description = "Stage advanced", body = Project),
    ),
    tag = "projects"
)]
pub async fn advance_stage(
    State(state): State<ApiState>,
    Path((circle_id, project_id)): Path<(String, String)>,
    Json(req): Json<AdvanceStageRequest>,
) -> Result<Json<Project>, ApiError> {
    // Require Reviewer+ for stage advancement (review gate)
    if !check_role(&state, &circle_id, &req.advanced_by, CircleRole::Reviewer).await? {
        return Err(err(
            "FORBIDDEN",
            "Requires Reviewer role or higher to advance stage",
        ));
    }

    let mut record = state
        .persistence
        .get_project(&project_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Project not found"))?;

    if record.circle_id != circle_id {
        return Err(err("NOT_FOUND", "Project not found in this circle"));
    }

    let next_stage = record
        .stage
        .next()
        .ok_or_else(|| err("CONFLICT", "Project is already in final stage"))?;

    let prev_stage = enum_to_str(&record.stage);
    record.stage = next_stage.clone();
    record.updated_at = DateTime::now();

    // Mark completed if reaching Closed
    if next_stage == ProjectStage::Closed {
        record.status = ProjectStatus::Completed;
        record.completed_at = Some(DateTime::now());
    }

    state
        .persistence
        .save_project(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Feed entry
    save_feed_best_effort(
        &state,
        FeedEntryRecord {
            id: nexcore_id::NexId::v4().to_string(),
            circle_id,
            entry_type: if next_stage == ProjectStage::Closed {
                FeedEntryType::ProjectCompleted
            } else {
                FeedEntryType::ProjectStageAdvanced
            },
            actor_user_id: req.advanced_by,
            content: format!(
                "Project \"{}\" advanced from {} to {}",
                record.name,
                prev_stage,
                enum_to_str(&record.stage)
            ),
            reference_id: Some(record.id.clone()),
            reference_type: Some("project".to_string()),
            created_at: DateTime::now(),
        },
    )
    .await;

    Ok(Json(record_to_project(record)))
}

// ============================================================================
// Deliverable CRUD
// ============================================================================

/// Create a deliverable within a project (Researcher+ only)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{cid}/projects/{pid}/deliverables",
    request_body = CreateDeliverableRequest,
    responses(
        (status = 200, description = "Deliverable created", body = Deliverable),
    ),
    tag = "projects"
)]
pub async fn create_deliverable(
    State(state): State<ApiState>,
    Path((circle_id, project_id)): Path<(String, String)>,
    Json(req): Json<CreateDeliverableRequest>,
) -> Result<Json<Deliverable>, ApiError> {
    if req.name.trim().is_empty() {
        return Err(err("VALIDATION_ERROR", "Deliverable name is required"));
    }

    // Verify project exists and belongs to circle
    let project = state
        .persistence
        .get_project(&project_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Project not found"))?;

    if project.circle_id != circle_id {
        return Err(err("NOT_FOUND", "Project not found in this circle"));
    }

    // Researcher+ required to create deliverables
    if !check_role(&state, &circle_id, &req.created_by, CircleRole::Researcher).await? {
        return Err(err("FORBIDDEN", "Requires Researcher role or higher"));
    }

    let now = DateTime::now();
    let record = DeliverableRecord {
        id: nexcore_id::NexId::v4().to_string(),
        project_id: project_id.clone(),
        circle_id: circle_id.clone(),
        name: req.name,
        deliverable_type: req
            .deliverable_type
            .map(|t| parse_enum(&t))
            .unwrap_or(DeliverableType::Report),
        status: DeliverableStatus::Draft,
        version: 1,
        file_url: req.file_url,
        content_hash: None,
        reviewed_by: None,
        review_status: ReviewStatus::Pending,
        review_notes: None,
        created_by: req.created_by.clone(),
        created_at: now,
        updated_at: now,
    };

    state
        .persistence
        .save_deliverable(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Feed entry
    save_feed_best_effort(
        &state,
        FeedEntryRecord {
            id: nexcore_id::NexId::v4().to_string(),
            circle_id,
            entry_type: FeedEntryType::DeliverableSubmitted,
            actor_user_id: req.created_by,
            content: format!("Submitted deliverable: {}", record.name),
            reference_id: Some(record.id.clone()),
            reference_type: Some("deliverable".to_string()),
            created_at: now,
        },
    )
    .await;

    Ok(Json(record_to_deliverable(record)))
}

/// List deliverables for a project
#[utoipa::path(
    get,
    path = "/api/v1/circles/{cid}/projects/{pid}/deliverables",
    responses(
        (status = 200, description = "Deliverable list", body = Vec<Deliverable>),
    ),
    tag = "projects"
)]
pub async fn list_deliverables(
    State(state): State<ApiState>,
    Path((circle_id, project_id)): Path<(String, String)>,
) -> Result<Json<Vec<Deliverable>>, ApiError> {
    // Verify project belongs to this circle
    let project = state
        .persistence
        .get_project(&project_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Project not found"))?;
    if project.circle_id != circle_id {
        return Err(err("NOT_FOUND", "Project not found in this circle"));
    }

    let records = state
        .persistence
        .list_deliverables(&project_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    let deliverables: Vec<Deliverable> = records.into_iter().map(record_to_deliverable).collect();
    Ok(Json(deliverables))
}

/// Update a deliverable
#[utoipa::path(
    patch,
    path = "/api/v1/circles/{cid}/projects/{pid}/deliverables/{did}",
    request_body = UpdateDeliverableRequest,
    responses(
        (status = 200, description = "Deliverable updated", body = Deliverable),
    ),
    tag = "projects"
)]
pub async fn update_deliverable(
    State(state): State<ApiState>,
    Path((circle_id, _project_id, deliverable_id)): Path<(String, String, String)>,
    Json(req): Json<UpdateDeliverableRequest>,
) -> Result<Json<Deliverable>, ApiError> {
    let mut record = state
        .persistence
        .get_deliverable(&deliverable_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Deliverable not found"))?;

    // Verify deliverable belongs to this circle
    if record.circle_id != circle_id {
        return Err(err("NOT_FOUND", "Deliverable not found in this circle"));
    }

    // Researcher+ or deliverable creator required
    if let Some(ref actor) = req.updated_by {
        let is_creator = record.created_by == *actor;
        if !is_creator
            && !check_role(&state, &record.circle_id, actor, CircleRole::Researcher).await?
        {
            return Err(err(
                "FORBIDDEN",
                "Requires Researcher role or deliverable creator",
            ));
        }
    }

    if let Some(name) = req.name {
        record.name = name;
    }
    if let Some(status) = req.status {
        record.status = parse_enum(&status);
    }
    if let Some(url) = req.file_url {
        record.file_url = Some(url);
    }
    if let Some(hash) = req.content_hash {
        record.content_hash = Some(hash);
    }
    record.updated_at = DateTime::now();

    state
        .persistence
        .save_deliverable(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(record_to_deliverable(record)))
}

/// Review a deliverable (Reviewer+ only)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{cid}/projects/{pid}/deliverables/{did}/review",
    request_body = ReviewDeliverableRequest,
    responses(
        (status = 200, description = "Review submitted", body = Deliverable),
    ),
    tag = "projects"
)]
pub async fn review_deliverable(
    State(state): State<ApiState>,
    Path((circle_id, _project_id, deliverable_id)): Path<(String, String, String)>,
    Json(req): Json<ReviewDeliverableRequest>,
) -> Result<Json<Deliverable>, ApiError> {
    // Require Reviewer+ for reviews
    if !check_role(&state, &circle_id, &req.reviewed_by, CircleRole::Reviewer).await? {
        return Err(err("FORBIDDEN", "Requires Reviewer role or higher"));
    }

    let mut record = state
        .persistence
        .get_deliverable(&deliverable_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Deliverable not found"))?;

    let review_status: ReviewStatus = parse_enum(&req.review_status);

    record.reviewed_by = Some(req.reviewed_by.clone());
    record.review_status = review_status.clone();
    record.review_notes = req.review_notes;
    record.updated_at = DateTime::now();

    // Update deliverable status based on review outcome
    match review_status {
        ReviewStatus::Approved => {
            record.status = DeliverableStatus::Approved;
        }
        ReviewStatus::RevisionRequested => {
            record.status = DeliverableStatus::Draft;
            record.version += 1;
        }
        _ => {}
    }

    state
        .persistence
        .save_deliverable(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Feed entry for approval
    if review_status == ReviewStatus::Approved {
        save_feed_best_effort(
            &state,
            FeedEntryRecord {
                id: nexcore_id::NexId::v4().to_string(),
                circle_id,
                entry_type: FeedEntryType::DeliverableApproved,
                actor_user_id: req.reviewed_by,
                content: format!("Approved deliverable: {}", record.name),
                reference_id: Some(record.id.clone()),
                reference_type: Some("deliverable".to_string()),
                created_at: DateTime::now(),
            },
        )
        .await;
    }

    Ok(Json(record_to_deliverable(record)))
}

// ============================================================================
// Router
// ============================================================================

/// Project + deliverable routes, nested under `/circles/{id}/projects`
pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/{pid}", get(get_project).patch(update_project))
        .route("/{pid}/advance", post(advance_stage))
        .route(
            "/{pid}/deliverables",
            get(list_deliverables).post(create_deliverable),
        )
        .route("/{pid}/deliverables/{did}", patch(update_deliverable))
        .route("/{pid}/deliverables/{did}/review", post(review_deliverable))
}
