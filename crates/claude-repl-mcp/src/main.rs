use claude_repl_mcp::ClaudeReplMcpServer;
use nexcore_error::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<()> {
    let server = ClaudeReplMcpServer::new();
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
