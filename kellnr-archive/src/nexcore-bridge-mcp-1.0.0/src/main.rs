//! NexCore Bridge MCP Server Entry Point

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use anyhow::Result;
use clap::Parser;
use nexcore_bridge_mcp::NexCoreBridgeMcpServer;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[derive(Parser, Debug)]
#[command(author, version, about = "NexCore Bridge MCP Server")]
struct Args {
    /// NexCore API base URL
    #[arg(long, env = "NEXCORE_API_URL", default_value = "http://localhost:3030")]
    api_url: String,

    /// NexCore API Key
    #[arg(long, env = "NEXCORE_API_KEY")]
    api_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    let server = NexCoreBridgeMcpServer::new(args.api_url, args.api_key)?;

    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
