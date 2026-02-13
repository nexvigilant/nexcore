use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use nexcore_api::{ApiState, persistence::Persistence, persistence::firestore::MockPersistence};
use serde_json::json;
use std::sync::Arc;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_reporting_flow() -> anyhow::Result<()> {
    // Environment setup
    unsafe {
        std::env::set_var("RATE_LIMIT_RPS", "1000");
    }

    let persistence = Arc::new(Persistence::Mock(MockPersistence::new()));
    let skill_state = nexcore_api::routes::skills::SkillAppState::default();
    let state = ApiState {
        persistence,
        skill_state,
    };

    let app = nexcore_api::build_app(state);

    // 1. Generate a report
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/reporting/generate")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "report_type": "signal_summary"
            })
            .to_string(),
        ))?;

    let response = app.clone().oneshot(req).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 2. List reports
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/reporting/list")
        .body(Body::empty())?;

    let response = app.oneshot(req).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let reports: serde_json::Value = serde_json::from_slice(&body)?;

    assert!(reports.is_array());
    assert_eq!(reports.as_array().unwrap().len(), 1);
    assert_eq!(reports[0]["report_type"], "signal_summary");

    Ok(())
}
