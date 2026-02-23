use nexcore_error::Result;
use gsheets_mcp::GSheetsMcpServer;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing to stderr (MCP uses stdout for protocol).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gsheets_mcp=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Google Sheets MCP server");

    let server = GSheetsMcpServer::new().await?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
