use anyhow::Result;
use gvids_mcp::GVidsMcpServer;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing to stderr (MCP uses stdout for protocol).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gvids_mcp=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Google Vids MCP server");

    let server = GVidsMcpServer::new().await?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
