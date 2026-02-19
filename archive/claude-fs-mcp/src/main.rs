use anyhow::Result;
use claude_fs_mcp::{ClaudeFsMcpServer, create_backup};
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<()> {
    // Start serving immediately; run startup backup in background.
    // Set CLAUDE_FS_MCP_BACKUP_ON_START=0 to skip startup backup.
    let backup_on_start = std::env::var("CLAUDE_FS_MCP_BACKUP_ON_START").map_or(true, |v| v != "0");
    if backup_on_start {
        tokio::task::spawn_blocking(|| {
            let _ = create_backup();
        });
    }

    let server = ClaudeFsMcpServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
