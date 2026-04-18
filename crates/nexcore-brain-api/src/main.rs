//! NexCore Brain API — HTTP surface over the unified brain store.
//!
//! Backing store: Cloud SQL Postgres (`nexvigilant-state` instance, `brain` DB).
//! Local and nexdev Claude Code hooks write here via HTTP. Single source of truth.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    pool: Arc<PgPool>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    db: &'static str,
}

#[derive(Serialize, sqlx::FromRow)]
struct Session {
    id: String,
    description: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    machine_id: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct CreateSessionReq {
    id: String,
    description: Option<String>,
    machine_id: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct Autopsy {
    session_id: String,
    outcome_verdict: Option<String>,
    lessons_count: Option<i32>,
    patterns_count: Option<i32>,
    proposition: Option<String>,
    rho_status: Option<String>,
    chain_level: Option<i32>,
    files_modified: Option<i32>,
    measured: Option<bool>,
}

#[derive(Deserialize)]
struct UpsertAutopsyReq {
    session_id: String,
    outcome_verdict: Option<String>,
    lessons_count: Option<i32>,
    patterns_count: Option<i32>,
    proposition: Option<String>,
    rho_status: Option<String>,
    chain_level: Option<i32>,
    files_modified: Option<i32>,
    measured: Option<bool>,
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let db = match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&*state.pool)
        .await
    {
        Ok(_) => "ok",
        Err(_) => "degraded",
    };
    Json(HealthResponse {
        status: "ok",
        service: "nexcore-brain-api",
        db,
    })
}

async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Session>>, (StatusCode, String)> {
    sqlx::query_as::<_, Session>(
        "SELECT id, description, created_at, machine_id, status FROM sessions ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(&*state.pool)
    .await
    .map(Json)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionReq>,
) -> Result<Json<Session>, (StatusCode, String)> {
    sqlx::query_as::<_, Session>(
        "INSERT INTO sessions (id, description, machine_id) VALUES ($1, $2, $3)
         ON CONFLICT (id) DO UPDATE SET description = EXCLUDED.description, updated_at = NOW()
         RETURNING id, description, created_at, machine_id, status",
    )
    .bind(&req.id)
    .bind(&req.description)
    .bind(&req.machine_id)
    .fetch_one(&*state.pool)
    .await
    .map(Json)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn get_autopsy(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Autopsy>, (StatusCode, String)> {
    sqlx::query_as::<_, Autopsy>(
        "SELECT session_id, outcome_verdict, lessons_count, patterns_count, proposition,
                rho_status, chain_level, files_modified, measured
         FROM autopsy_records WHERE session_id = $1",
    )
    .bind(&session_id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "no autopsy".into()))
    .map(Json)
}

async fn upsert_autopsy(
    State(state): State<AppState>,
    Json(req): Json<UpsertAutopsyReq>,
) -> Result<Json<Autopsy>, (StatusCode, String)> {
    sqlx::query_as::<_, Autopsy>(
        r#"INSERT INTO autopsy_records (session_id, outcome_verdict, lessons_count, patterns_count,
               proposition, rho_status, chain_level, files_modified, measured)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
           ON CONFLICT (session_id) DO UPDATE SET
               outcome_verdict=EXCLUDED.outcome_verdict,
               lessons_count=EXCLUDED.lessons_count,
               patterns_count=EXCLUDED.patterns_count,
               proposition=EXCLUDED.proposition,
               rho_status=EXCLUDED.rho_status,
               chain_level=EXCLUDED.chain_level,
               files_modified=EXCLUDED.files_modified,
               measured=EXCLUDED.measured
           RETURNING session_id, outcome_verdict, lessons_count, patterns_count, proposition,
                     rho_status, chain_level, files_modified, measured"#,
    )
    .bind(&req.session_id)
    .bind(&req.outcome_verdict)
    .bind(req.lessons_count)
    .bind(req.patterns_count)
    .bind(&req.proposition)
    .bind(&req.rho_status)
    .bind(req.chain_level)
    .bind(req.files_modified)
    .bind(req.measured)
    .fetch_one(&*state.pool)
    .await
    .map(Json)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost/brain".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Apply schema migrations lazily (idempotent).
    if let Ok(schema) = std::fs::read_to_string("/app/schema.sql") {
        let _ = sqlx::raw_sql(&schema).execute(&pool).await;
    }

    let state = AppState {
        pool: Arc::new(pool),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/autopsy", post(upsert_autopsy))
        .route("/autopsy/:session_id", get(get_autopsy))
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("nexcore-brain-api listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
