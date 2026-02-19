use anyhow::Result;
use claude_fs_mcp::{ClaudeFsMcpServer, create_backup};
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<()> {
    // Backup on session start (best-effort).
    let _ = create_backup();

    let server = ClaudeFsMcpServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
