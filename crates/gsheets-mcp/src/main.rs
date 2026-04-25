use gsheets_mcp::GSheetsMcpServer;
use nexcore_error::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing to stderr (MCP uses stdout for protocol).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(
                "gsheets_mcp=info"
                    .parse()
                    .map_err(|e| nexcore_error::NexError::msg(format!("filter parse: {e}")))?,
            ),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Google Sheets MCP server");

    let server = GSheetsMcpServer::new().await?;
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
