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
//! - `record` — drive a `BmsSource` and dump frames as NDJSON

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use stark_suit_station::bms::{BmsSource, MockBmsSource, ReplayBmsSource, SerialBmsSource};
use stark_suit_station::mcp::StarkSuitMcpServer;
use stark_suit_station::state::StationState;
use stark_suit_station::loops;
use clap::{Parser, Subcommand, ValueEnum};
use nexcore_error::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

/// Backend selector for the BMS source.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum BmsBackend {
    /// Baked-in 5-frame trace (default, smoke-stable).
    Mock,
    /// Replay an NDJSON trace file recorded earlier.
    Replay,
    /// Live JSON-over-serial. Requires `--port` (real `/dev/ttyUSB*` or pty path).
    Serial,
    /// CAN 11-bit identifier frames. Deferred to v0.5+ (no codec yet).
    Can,
}

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
    Run {
        /// BMS backend. `mock` is default; `replay` requires `--trace`;
        /// `serial` requires `--port`; `can` is deferred to v0.5+.
        #[arg(long, value_enum, default_value_t = BmsBackend::Mock)]
        bms_source: BmsBackend,
        /// NDJSON file to replay (required when `--bms-source replay`).
        #[arg(long)]
        trace: Option<PathBuf>,
        /// Replay speedup factor (1.0 = recorded cadence, 10.0 = 10x faster).
        #[arg(long, default_value_t = 1.0)]
        speedup: f64,
        /// Serial device path (required when `--bms-source serial`).
        #[arg(long)]
        port: Option<String>,
        /// Serial baud rate.
        #[arg(long, default_value_t = 115200)]
        baud: u32,
    },
    /// Spawn loops, wait 1 second, print snapshot, exit.
    Status,
    /// Single-tick each loop, print snapshot, exit.
    Tick,
    /// Drive a `BmsSource` for `--seconds` (default 5) and dump frames as NDJSON.
    Record {
        /// Output NDJSON file (one frame per line).
        #[arg(long)]
        out: PathBuf,
        /// Recording duration, seconds. Ignored when Ctrl+C arrives first.
        #[arg(long, default_value_t = 5)]
        seconds: u64,
        /// Source to record. Default `mock`. `replay` requires `--trace`.
        #[arg(long, value_enum, default_value_t = BmsBackend::Mock)]
        bms_source: BmsBackend,
        /// Source trace path when `--bms-source replay`.
        #[arg(long)]
        trace: Option<PathBuf>,
    },
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
    match cli.cmd.unwrap_or(Cmd::Run {
        bms_source: BmsBackend::Mock,
        trace: None,
        speedup: 1.0,
        port: None,
        baud: 115200,
    }) {
        Cmd::Run { bms_source, trace, speedup, port, baud } => {
            run_daemon(bms_source, trace, speedup, port, baud).await
        }
        Cmd::Status => run_status().await,
        Cmd::Tick => run_tick().await,
        Cmd::Record { out, seconds, bms_source, trace } => {
            run_record(out, seconds, bms_source, trace).await
        }
    }
}

/// Construct a `BmsSource` from CLI flags. Returns a clear error when required
/// args are missing or the backend is deferred.
fn build_source(
    backend: BmsBackend,
    trace: Option<PathBuf>,
    speedup: f64,
    port: Option<String>,
    baud: u32,
) -> Result<Arc<dyn BmsSource>> {
    match backend {
        BmsBackend::Mock => Ok(Arc::new(MockBmsSource::new())),
        BmsBackend::Replay => {
            let path = trace.ok_or_else(|| {
                nexcore_error::NexError::msg("--bms-source replay requires --trace <file>")
            })?;
            let src = ReplayBmsSource::from_file(&path, speedup).map_err(|e| {
                nexcore_error::NexError::msg(format!("replay load failed: {e}"))
            })?;
            Ok(Arc::new(src))
        }
        BmsBackend::Serial => {
            let p = port.ok_or_else(|| {
                nexcore_error::NexError::msg("--bms-source serial requires --port <device>")
            })?;
            let src = SerialBmsSource::open(&p, baud).map_err(|e| {
                nexcore_error::NexError::msg(format!("serial open failed: {e}"))
            })?;
            Ok(Arc::new(src))
        }
        BmsBackend::Can => Err(nexcore_error::NexError::msg(
            "--bms-source can deferred to v0.5+ (no CAN codec yet)",
        )),
    }
}

async fn run_daemon(
    backend: BmsBackend,
    trace: Option<PathBuf>,
    speedup: f64,
    port: Option<String>,
    baud: u32,
) -> Result<()> {
    let bms = build_source(backend, trace, speedup, port, baud)?;
    let state = StationState::new();
    spawn_loops(state.clone(), bms);
    info!("stark-suit-station: 4 loops spawned, MCP server starting on stdio");

    let server = StarkSuitMcpServer::new(state);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

async fn run_status() -> Result<()> {
    let state = StationState::new();
    let bms: Arc<dyn BmsSource> = Arc::new(MockBmsSource::new());
    spawn_loops(state.clone(), bms);
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

async fn run_record(
    out: PathBuf,
    seconds: u64,
    backend: BmsBackend,
    trace: Option<PathBuf>,
) -> Result<()> {
    let source = build_source(backend, trace, 1.0, None, 115200)?;
    let file = tokio::fs::File::create(&out).await?;
    let mut writer = tokio::io::BufWriter::new(file);
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    let deadline = tokio::time::sleep(Duration::from_secs(seconds));
    tokio::pin!(deadline);
    let mut count: u64 = 0;
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match source.poll().await {
                    Ok(frame) => {
                        let mut line = serde_json::to_string(&frame)?;
                        line.push('\n');
                        writer.write_all(line.as_bytes()).await?;
                        count += 1;
                    }
                    Err(e) => {
                        warn!("record: source returned {e}, stopping");
                        break;
                    }
                }
            }
            _ = &mut deadline => { break; }
            _ = tokio::signal::ctrl_c() => {
                info!("record: Ctrl+C received");
                break;
            }
        }
    }
    writer.flush().await?;
    info!(frames = count, path = %out.display(), "record: done");
    eprintln!("recorded {count} frames -> {}", out.display());
    Ok(())
}

fn spawn_loops(state: std::sync::Arc<StationState>, bms: Arc<dyn BmsSource>) {
    let s = state.clone();
    tokio::spawn(async move { loops::run_perception(s).await });
    let s = state.clone();
    let b = bms.clone();
    tokio::spawn(async move { loops::run_power(s, b).await });
    let s = state.clone();
    tokio::spawn(async move { loops::run_control(s).await });
    tokio::spawn(async move { loops::run_human_interface(state).await });
}
