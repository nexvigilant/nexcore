// Copyright 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

use nexcore_error::Result;
use perplexity_mcp::PerplexityMcpServer;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<()> {
    let server = PerplexityMcpServer::new();
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
