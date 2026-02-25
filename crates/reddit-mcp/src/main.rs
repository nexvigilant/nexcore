#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Reddit MCP Server
//!
//! Exposes Reddit API and value signal detection as MCP tools.
//!
//! ## Environment Variables
//!
//! - `REDDIT_CLIENT_ID` - OAuth2 client ID
//! - `REDDIT_CLIENT_SECRET` - OAuth2 client secret
//! - `REDDIT_USERNAME` - Reddit username
//! - `REDDIT_PASSWORD` - Reddit password
//!
//! ## Tools
//!
//! - `reddit_status` - Check API status and rate limits
//! - `reddit_authenticate` - Authenticate with Reddit
//! - `reddit_hot_posts` - Get hot posts from a subreddit
//! - `reddit_new_posts` - Get new posts from a subreddit
//! - `reddit_subreddit_info` - Get subreddit metadata
//! - `reddit_detect_signals` - Detect value signals in posts
//! - `reddit_search_entity` - Search for posts mentioning an entity

use nexcore_error::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use tracing::info;

mod params;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is for MCP protocol)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("reddit_mcp=info".parse()?)
                .add_directive("rmcp=warn".parse()?),
        )
        .init();

    // Load .env file if present
    let _ = dotenvy::dotenv();

    info!("Starting Reddit MCP Server");

    // Create server handler
    let handler = server::RedditServer::new()?;

    // Run MCP server over stdio
    let service = handler.serve(stdio()).await?;
    service.waiting().await?;

    info!("Reddit MCP Server shutdown");
    Ok(())
}
