//! NCOS Server Entry Point
//!
//! Axum SSR server serving the Leptos application + static files.
//! Build with: cargo build --features ssr
//!
//! Tier: T2-C (Sequence + Boundary + State)

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use ncos::{shell, App};

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ncos=info".parse().expect("valid filter")),
        )
        .init();

    // Load Leptos configuration
    let conf = get_configuration(Some("Cargo.toml"))
        .map_err(|e| format!("Failed to load Leptos configuration: {e}"))?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Generate route list
    let routes = generate_route_list(App);

    // Build router
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    tracing::info!("NCOS listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // WASM entry point — hydration handled by cargo-leptos
}
