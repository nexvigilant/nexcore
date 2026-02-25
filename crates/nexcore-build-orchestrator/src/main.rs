//! SSR server entry point for the build orchestrator web dashboard.
//!
//! Requires the `ssr` feature to compile.

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use nexcore_build_orchestrator::server;

    tracing_subscriber::fmt()
        .with_env_filter("nexcore_build_orchestrator=info,tower_http=debug")
        .init();

    tracing::info!("Starting build orchestrator dashboard");

    let port: u16 = std::env::args()
        .position(|a| a == "--port" || a == "-p")
        .and_then(|i| std::env::args().nth(i + 1))
        .and_then(|p| p.parse().ok())
        .unwrap_or(3100);

    if let Err(e) = server::serve(port).await {
        tracing::error!("Server error: {e}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {
    eprintln!("This binary requires the 'ssr' feature.");
    eprintln!("Run: cargo run --bin build-orchestrator --features ssr");
    std::process::exit(1);
}
