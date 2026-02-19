//! Axum server — REST API + SSE endpoints for the dashboard.

pub mod api;
pub mod sse;

use crate::error::BuildOrcResult;

/// Start the dashboard server on the given port.
pub async fn serve(port: u16) -> BuildOrcResult<()> {
    use axum::Router;

    let app = Router::new()
        .merge(api::api_routes())
        .merge(sse::sse_routes());

    // Bind to localhost only (internal dashboard). Use BIND_ADDR=0.0.0.0 for external access.
    let bind_addr: [u8; 4] = match std::env::var("BIND_ADDR").as_deref() {
        Ok("0.0.0.0") => [0, 0, 0, 0],
        _ => [127, 0, 0, 1],
    };
    let addr = std::net::SocketAddr::from((bind_addr, port));
    tracing::info!("Dashboard listening on http://{addr}");

    let listener =
        tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| crate::error::BuildOrcError::Io {
                path: std::path::PathBuf::from(format!("0.0.0.0:{port}")),
                source: e,
            })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::BuildOrcError::Io {
            path: std::path::PathBuf::from(format!("0.0.0.0:{port}")),
            source: e,
        })?;

    Ok(())
}
