//! adventure-hud Server Entry Point
//!
//! This binary runs the SSR server. Build with:
//! cargo build --features ssr

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use adventure_hud::{shell, App};

    // Load configuration from Cargo.toml
    let conf = get_configuration(Some("Cargo.toml"))
        .map_err(|e| format!("Failed to load Leptos configuration: {e}"))?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Generate route list for Leptos
    let routes = generate_route_list(App);

    // Build Axum router
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // Start server
    println!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // Client-side entry point for WASM
    // This would be used with trunk or wasm-pack
    println!("Run with --features ssr for server mode");
}
