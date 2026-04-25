//! Skills MCP Server Entry Point
//!
//! Modes:
//! - stdio (default): Standard MCP stdio transport
//! - `--socket <path>`: Unix socket daemon mode for hot-reload

use nexcore_error::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use skills_mcp::SkillsMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(
                "skills_mcp=info"
                    .parse()
                    .map_err(|e| nexcore_error::NexError::msg(format!("filter parse: {e}")))?,
            ),
        )
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--socket" {
        let socket_path = args
            .get(2)
            .map(|s| s.as_str())
            .unwrap_or("/run/user/1000/skills-mcp.sock");
        return run_socket_mode(socket_path).await;
    }

    tracing::info!("Starting Skills MCP server (stdio)");

    let server = SkillsMcpServer::new()?;
    let service = server
        .serve(stdio())
        .await
        .map_err(|e| nexcore_error::NexError::msg(format!("serve: {e}")))?;
    service
        .waiting()
        .await
        .map_err(|e| nexcore_error::NexError::msg(format!("waiting: {e}")))?;

    Ok(())
}

/// Run as a Unix socket daemon, accepting one client at a time.
/// When the client disconnects, accept the next one (supports relay reconnection).
async fn run_socket_mode(socket_path: &str) -> Result<()> {
    use tokio::net::UnixListener;

    // Remove stale socket file
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;
    eprintln!("[skills-mcp] daemon listening on {socket_path}");

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                eprintln!("[skills-mcp] client connected");

                let server = SkillsMcpServer::new()?;
                let (reader, writer) = stream.into_split();
                let service = server.serve((reader, writer)).await;

                match service {
                    Ok(svc) => {
                        let _ = svc.waiting().await;
                        eprintln!("[skills-mcp] client disconnected, awaiting next");
                    }
                    Err(e) => {
                        eprintln!("[skills-mcp] serve error: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("[skills-mcp] accept error: {e}");
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }
}
