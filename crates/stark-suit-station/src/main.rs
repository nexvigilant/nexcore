//! # Stark Suit Station
//!
//! Daemon that turns the `stark-suit` umbrella from a library into a
//! runnable product. Spawns 4 control loops (one per compound), serves
//! their state via MCP over stdio.
//!
//! ## Subcommands
//!
//! - `run`    — spawn loops + MCP server (default)
//! - `status` — print one snapshot to stdout and exit (loops run for ~1 s)
//! - `tick`   — single-tick each loop, print snapshot, exit

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod bms;
mod loops;
mod mcp;
mod state;

use crate::bms::{BmsSource, MockBmsSource};
use crate::mcp::StarkSuitMcpServer;
use crate::state::StationState;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use nexcore_error::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use std::time::Duration;
use tracing::info;

/// CLI entry.
#[derive(Parser)]
#[command(name = "stark-suit-station", about = "Iron Vigil Stark Suit station daemon")]
struct Cli {
    /// Subcommand. Defaults to `run`.
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Spawn 4 control loops + serve MCP over stdio (default).
    Run,
    /// Spawn loops, wait 1 second, print snapshot, exit.
    Status,
    /// Single-tick each loop, print snapshot, exit.
    Tick,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("stark_suit_station=info".parse()?),
        )
        .init();

    let cli = Cli::parse();
    match cli.cmd.unwrap_or(Cmd::Run) {
        Cmd::Run => run_daemon().await,
        Cmd::Status => run_status().await,
        Cmd::Tick => run_tick().await,
    }
}

async fn run_daemon() -> Result<()> {
    let state = StationState::new();
    spawn_loops(state.clone());
    info!("stark-suit-station: 4 loops spawned, MCP server starting on stdio");

    let server = StarkSuitMcpServer::new(state);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

async fn run_status() -> Result<()> {
    let state = StationState::new();
    spawn_loops(state.clone());
    tokio::time::sleep(Duration::from_secs(1)).await;
    let snap = state.snapshot().await;
    let s = serde_json::to_string_pretty(&snap)?;
    println!("{s}");
    Ok(())
}

async fn run_tick() -> Result<()> {
    let state = StationState::new();
    let bms: Arc<dyn BmsSource> = Arc::new(MockBmsSource::new());
    let s = state.clone();
    tokio::spawn(async move { loops::run_perception(s).await });
    let s = state.clone();
    let b = bms.clone();
    tokio::spawn(async move { loops::run_power(s, b).await });
    let s = state.clone();
    tokio::spawn(async move { loops::run_control(s).await });
    let s = state.clone();
    tokio::spawn(async move { loops::run_human_interface(s).await });
    tokio::time::sleep(Duration::from_millis(300)).await;
    let snap = state.snapshot().await;
    let s = serde_json::to_string_pretty(&snap)?;
    println!("{s}");
    Ok(())
}

fn spawn_loops(state: std::sync::Arc<StationState>) {
    let bms: Arc<dyn BmsSource> = Arc::new(MockBmsSource::new());
    let s = state.clone();
    tokio::spawn(async move { loops::run_perception(s).await });
    let s = state.clone();
    let b = bms.clone();
    tokio::spawn(async move { loops::run_power(s, b).await });
    let s = state.clone();
    tokio::spawn(async move { loops::run_control(s).await });
    tokio::spawn(async move { loops::run_human_interface(state).await });
}
