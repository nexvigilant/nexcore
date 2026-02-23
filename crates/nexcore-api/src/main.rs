//! NexVigilant Core REST API - Binary Entry Point

use nexcore_api::persistence::firestore::{FirestorePersistence, MockPersistence};
use nexcore_api::{ApiState, build_app, persistence::Persistence};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> nexcore_error::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nexcore_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize persistence
    let persistence = if let Ok(project_id) = std::env::var("FIREBASE_PROJECT_ID") {
        let api_key = std::env::var("FIREBASE_API_KEY").ok();
        tracing::info!(
            "Initializing Firestore persistence for project: {}",
            project_id
        );
        Persistence::Firestore(FirestorePersistence::new(
            project_id,
            "reports".to_string(),
            api_key,
        ))
    } else {
        tracing::info!("Initializing Mock persistence (in-memory)");
        Persistence::Mock(MockPersistence::new())
    };

    let persistence = Arc::new(persistence);
    let skill_state = nexcore_api::routes::skills::SkillAppState::default();
    let state = ApiState {
        persistence,
        skill_state,
    };

    eprintln!("[DEBUG] State initialized, building app...");
    // Build application
    let app = build_app(state);
    eprintln!("[DEBUG] App built, determining port...");

    // Determine port
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    // Bind address
    let bind_addr: [u8; 4] = match std::env::var("BIND_ADDR").as_deref() {
        Ok("0.0.0.0") => [0, 0, 0, 0],
        _ => [127, 0, 0, 1],
    };
    let addr = SocketAddr::from((bind_addr, port));
    tracing::info!("NexVigilant Core API starting on http://{}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
