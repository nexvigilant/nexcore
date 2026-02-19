//! NexVigilant Core MCP Server Entry Point
//!
//! Exposes NexVigilant Core's high-performance Rust APIs to Claude Code via MCP.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use anyhow::Result;
use nexcore_mcp::NexCoreMcpServer;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize telemetry writer when feature is enabled
    #[cfg(feature = "telemetry")]
    nexcore_mcp::telemetry::init_telemetry_writer();

    let server = NexCoreMcpServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
