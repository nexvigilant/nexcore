//! NVREPL — NexVigilant REPL
//! Deterministic terminal interface for PV signal detection,
//! patient safety triage, and system monitoring.

mod commands;
mod completer;
mod energy;
mod format;
mod monitor;
mod repl;
mod safety;
mod signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .with_target(false)
        .init();

    repl::run().await
}
