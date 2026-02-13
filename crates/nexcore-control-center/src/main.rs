//! nexcore-control-center Server Entry Point

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use nexcore_control_center::{shell, App};
    use std::net::SocketAddr;
    use tower_http::services::ServeDir;

    // Load configuration - override with PORT env for Cloud Run
    let mut conf = get_configuration(Some("Cargo.toml"))?;
    if let Ok(port) = std::env::var("PORT") {
        if let Ok(port) = port.parse::<u16>() {
            // Cloud Run sets K_SERVICE and requires 0.0.0.0; local dev binds loopback
            let host: [u8; 4] = if std::env::var("K_SERVICE").is_ok() {
                [0, 0, 0, 0]
            } else {
                [127, 0, 0, 1]
            };
            conf.leptos_options.site_addr = SocketAddr::from((host, port));
        }
    }
    let addr: SocketAddr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Generate route list for Leptos
    let routes = generate_route_list(App);

    // Build Axum router with Leptos routes
    let app: Router<()> = Router::new()
        // Serve static CSS from /pkg/style.css
        .nest_service("/pkg", ServeDir::new("style"))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // Start server
    println!("NexCore Control Center running on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {
    println!("Run with --features ssr for server mode");
}
