//! NexCore Vault API — HTTP surface over the Obsidian vault corpus.
//!
//! Backing store:
//!   - Markdown content: mounted volume (Cloud Run + GCS FUSE) at /vault
//!   - Metadata + search index: Cloud SQL Postgres `vault_notes` table (TSVECTOR + GIN)
//!
//! Endpoints:
//!   GET  /health
//!   GET  /notes/*path              — fetch raw markdown
//!   GET  /search?q=<query>         — full-text search (ts_rank)
//!   GET  /tags/:tag                — notes matching tag
//!   GET  /recent?limit=N           — recently modified

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    pool: Arc<PgPool>,
    vault_root: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    vault_root: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct NoteMeta {
    path: String,
    title: Option<String>,
    folder: Option<String>,
    tags: Option<Vec<String>>,
    word_count: Option<i32>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct NoteContent {
    path: String,
    content: String,
    size_bytes: usize,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: i64,
}

#[derive(Deserialize)]
struct RecentQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    20
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "nexcore-vault-api",
        vault_root: state.vault_root.clone(),
    })
}

async fn get_note(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Json<NoteContent>, (StatusCode, String)> {
    // Guard: ensure path stays inside vault_root
    let full = std::path::Path::new(&state.vault_root).join(&path);
    let canonical = full
        .canonicalize()
        .map_err(|_| (StatusCode::NOT_FOUND, "note not found".into()))?;
    if !canonical.starts_with(&state.vault_root) {
        return Err((StatusCode::FORBIDDEN, "path escape".into()));
    }
    let content = tokio::fs::read_to_string(&canonical)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let size = content.len();
    Ok(Json(NoteContent {
        path,
        content,
        size_bytes: size,
    }))
}

async fn search(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<NoteMeta>>, (StatusCode, String)> {
    sqlx::query_as::<_, NoteMeta>(
        r#"SELECT path, title, folder, tags, word_count, updated_at
           FROM vault_notes
           WHERE tsv @@ plainto_tsquery('english', $1)
           ORDER BY ts_rank(tsv, plainto_tsquery('english', $1)) DESC
           LIMIT $2"#,
    )
    .bind(&q.q)
    .bind(q.limit)
    .fetch_all(&*state.pool)
    .await
    .map(Json)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn by_tag(
    State(state): State<AppState>,
    Path(tag): Path<String>,
) -> Result<Json<Vec<NoteMeta>>, (StatusCode, String)> {
    sqlx::query_as::<_, NoteMeta>(
        "SELECT path, title, folder, tags, word_count, updated_at
         FROM vault_notes WHERE $1 = ANY(tags) ORDER BY updated_at DESC LIMIT 100",
    )
    .bind(&tag)
    .fetch_all(&*state.pool)
    .await
    .map(Json)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn recent(
    State(state): State<AppState>,
    Query(q): Query<RecentQuery>,
) -> Result<Json<Vec<NoteMeta>>, (StatusCode, String)> {
    sqlx::query_as::<_, NoteMeta>(
        "SELECT path, title, folder, tags, word_count, updated_at
         FROM vault_notes ORDER BY updated_at DESC LIMIT $1",
    )
    .bind(q.limit)
    .fetch_all(&*state.pool)
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
    let vault_root = std::env::var("VAULT_ROOT").unwrap_or_else(|_| "/vault".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    let state = AppState {
        pool: Arc::new(pool),
        vault_root,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/notes/*path", get(get_note))
        .route("/search", get(search))
        .route("/tags/:tag", get(by_tag))
        .route("/recent", get(recent))
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("nexcore-vault-api listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
