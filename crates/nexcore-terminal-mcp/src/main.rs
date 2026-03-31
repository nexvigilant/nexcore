// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! NexVigilant Terminal MCP Server
//!
//! Exposes terminal control as MCP tools via stdio transport.
//! Claude Code can spawn PTY sessions, send input, read output,
//! resize, manage layout, and control the terminal programmatically.

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use rmcp::{ServerHandler, ServiceExt};

mod tools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("nexcore_terminal_mcp=info")
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting NexVigilant Terminal MCP server");

    let server = tools::TerminalMcp::new();
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
