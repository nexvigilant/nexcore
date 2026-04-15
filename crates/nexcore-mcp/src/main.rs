//! NexVigilant Core MCP Server Entry Point
//!
//! Exposes NexVigilant Core's high-performance Rust APIs to Claude Code via MCP.
//!
//! ## Modes
//! - **stdio** (default): Claude Code spawns this process, communicates via stdin/stdout.
//! - **daemon** (`--daemon`): Listens on HTTP, always-on. Multiple consumers connect concurrently.

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use nexcore_error::Result;
use nexcore_mcp::NexCoreMcpServer;
use rmcp::ServiceExt;

const DEFAULT_DAEMON_PORT: u16 = 3031;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize telemetry writer when feature is enabled
    #[cfg(feature = "telemetry")]
    nexcore_mcp::telemetry::init_telemetry_writer();

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--daemon") {
        run_daemon(&args).await
    } else {
        run_stdio().await
    }
}

/// stdio mode — Claude Code spawns this, communicates via stdin/stdout.
async fn run_stdio() -> Result<()> {
    let server = NexCoreMcpServer::new();
    let service = server
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| nexcore_error::NexError::new(e.to_string()))?;
    service
        .waiting()
        .await
        .map_err(|e| nexcore_error::NexError::new(e.to_string()))?;
    Ok(())
}

/// Daemon mode — HTTP server on localhost, always-on, multiple consumers.
async fn run_daemon(args: &[String]) -> Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };
    use tokio_util::sync::CancellationToken;

    let port = parse_port(args);
    let ct = CancellationToken::new();

    let config = StreamableHttpServerConfig {
        stateful_mode: true,
        cancellation_token: ct.child_token(),
        ..Default::default()
    };

    let service: StreamableHttpService<NexCoreMcpServer, LocalSessionManager> =
        StreamableHttpService::new(|| Ok(NexCoreMcpServer::new()), Default::default(), config);

    let app = axum::Router::new()
        .nest_service("/mcp", service)
        .route("/health", axum::routing::get(|| async { "ok" }));

    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    eprintln!("nexcore-mcp daemon listening on http://{addr}/mcp");
    eprintln!("health check: http://{addr}/health");

    // Graceful shutdown on SIGINT/SIGTERM
    let ct_signal = ct.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        eprintln!("shutting down nexcore-mcp daemon");
        ct_signal.cancel();
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(async move { ct.cancelled().await })
        .await?;

    Ok(())
}

fn parse_port(args: &[String]) -> u16 {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--port" {
            if let Some(val) = args.get(i + 1) {
                if let Ok(p) = val.parse::<u16>() {
                    return p;
                }
            }
        }
    }
    DEFAULT_DAEMON_PORT
}
