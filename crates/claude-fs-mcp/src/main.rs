use claude_fs_mcp::{ClaudeFsMcpServer, create_backup};
use nexcore_error::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<()> {
    // Backup on session start (best-effort).
    let _ = create_backup();

    let server = ClaudeFsMcpServer::new();
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
