//! CLI entry point for the build orchestrator.
//!
//! This binary runs without the `ssr` feature — no Leptos/Axum needed.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clap::Parser;
use nexcore_build_orchestrator::cli::{Cli, Commands};
use nexcore_build_orchestrator::history::store::HistoryStore;
use nexcore_build_orchestrator::metrics::summary::BuildSummary;
use nexcore_build_orchestrator::pipeline::definition::PipelineDefinition;
use nexcore_build_orchestrator::pipeline::state::RunStatus;
use nexcore_build_orchestrator::workspace::scanner::scan_workspace;
use std::path::PathBuf;

fn main() {
    let cli = Cli::parse();

    // Resolve workspace root
    let workspace = resolve_workspace(&cli);

    match cli.command {
        Commands::Run { pipeline, force } => cmd_run(&workspace, &pipeline, force),
        Commands::Status => cmd_status(&workspace),
        Commands::History { limit, status } => cmd_history(&workspace, limit, status),
        Commands::Workspace => cmd_workspace(&workspace),
        Commands::Plan { pipeline } => cmd_plan(&pipeline),
        Commands::Prune { keep } => cmd_prune(&workspace, keep),
        Commands::Serve { port } => cmd_serve(port),
    }
}

fn resolve_workspace(cli: &Cli) -> PathBuf {
    if let Some(ref ws) = cli.workspace {
        PathBuf::from(ws)
    } else {
        nexcore_build_gate::find_workspace_root(&std::env::current_dir().unwrap_or_default())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                PathBuf::from(home).join("nexcore")
            })
    }
}

fn cmd_run(workspace: &PathBuf, pipeline_name: &str, _force: bool) {
    let Some(definition) = PipelineDefinition::builtin(pipeline_name) else {
        eprintln!("Unknown pipeline: {pipeline_name}");
        eprintln!(
            "Available: {}",
            PipelineDefinition::builtin_names().join(", ")
        );
        std::process::exit(1);
    };

    println!(
        "Running pipeline: {} ({})",
        definition.name, definition.description
    );
    println!("Workspace: {}", workspace.display());
    println!();

    match nexcore_build_orchestrator::execute_pipeline(&definition, workspace) {
        Ok(state) => {
            // Save to history
            if let Ok(store) = HistoryStore::new(workspace) {
                let _ = store.save(&state);
            }

            // Print results
            println!();
            println!("Pipeline: {} — {}", state.definition_name, state.status);
            for stage in &state.stages {
                let duration = stage
                    .duration
                    .map(|d| d.display())
                    .unwrap_or_else(|| "-".into());
                println!("  {} {} ({})", stage.status, stage.stage_id, duration);
            }
            if let Some(total) = state.total_duration {
                println!("\nTotal: {}", total.display());
            }

            if state.status != RunStatus::Completed {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Pipeline error: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_status(workspace: &PathBuf) {
    let Ok(store) = HistoryStore::new(workspace) else {
        eprintln!("No history available");
        return;
    };

    let ids = store.list_ids().unwrap_or_default();
    if ids.is_empty() {
        println!("No pipeline runs recorded.");
        return;
    }

    // Show most recent run
    if let Some(id) = ids.first() {
        if let Ok(state) = store.load(id) {
            let started = state
                .started_at
                .format("%Y-%m-%d %H:%M:%S UTC")
                .unwrap_or_else(|_| "invalid-timestamp".to_string());
            println!("Last run: {} — {}", state.id, state.status);
            println!("Pipeline: {}", state.definition_name);
            println!("Started: {}", started);
            if let Some(d) = state.total_duration {
                println!("Duration: {}", d.display());
            }
            println!(
                "Stages: {}/{} passed",
                state.success_count(),
                state.stages.len()
            );
        }
    }
}

fn cmd_history(workspace: &PathBuf, limit: usize, status_filter: Option<String>) {
    let Ok(store) = HistoryStore::new(workspace) else {
        eprintln!("No history available");
        return;
    };

    let runs = store.load_all().unwrap_or_default();
    if runs.is_empty() {
        println!("No pipeline runs recorded.");
        return;
    }

    let filtered: Vec<_> = runs
        .iter()
        .filter(|r| {
            if let Some(ref s) = status_filter {
                format!("{:?}", r.status).to_lowercase() == s.to_lowercase()
            } else {
                true
            }
        })
        .take(limit)
        .collect();

    let summary = BuildSummary::from_runs(&runs);
    println!(
        "Build Health: {:.0}% success ({}/{} runs)",
        summary.success_rate, summary.successful_runs, summary.total_runs
    );
    if let Some(avg) = summary.avg_duration {
        println!("Average duration: {}", avg.display());
    }
    println!();
    println!(
        "{:<30} {:<12} {:<12} {}",
        "ID", "Status", "Duration", "Date"
    );
    println!("{}", "-".repeat(80));

    for run in filtered {
        let duration = run
            .total_duration
            .map(|d| d.display())
            .unwrap_or_else(|| "-".into());
        let date = run
            .started_at
            .format("%Y-%m-%d %H:%M")
            .unwrap_or_else(|_| "invalid-timestamp".to_string());
        println!(
            "{:<30} {:<12} {:<12} {}",
            run.id,
            format!("{:?}", run.status),
            duration,
            date
        );
    }
}

fn cmd_workspace(workspace: &PathBuf) {
    match scan_workspace(workspace) {
        Ok(scan) => {
            println!("Workspace: {}", scan.workspace_root);
            println!("Crates: {} total", scan.crate_count);
            println!(
                "Status: {} clean, {} dirty",
                scan.clean_count(),
                scan.dirty_count()
            );
            if let Some(ref hash) = scan.workspace_hash {
                println!("Hash: {}...", &hash[..16.min(hash.len())]);
            }
            println!();

            for target in &scan.targets {
                let status = if target.needs_build { "dirty" } else { "clean" };
                println!("  [{status:>5}] {}", target.name);
            }
        }
        Err(e) => eprintln!("Workspace scan error: {e}"),
    }
}

fn cmd_plan(pipeline_name: &str) {
    let Some(definition) = PipelineDefinition::builtin(pipeline_name) else {
        eprintln!("Unknown pipeline: {pipeline_name}");
        std::process::exit(1);
    };

    let waves = nexcore_build_orchestrator::dry_run(&definition);
    println!("Pipeline: {} — {}", definition.name, definition.description);
    println!("Execution plan ({} waves):", waves.len());
    for (i, wave) in waves.iter().enumerate() {
        let ids: Vec<String> = wave.iter().map(|id| id.0.clone()).collect();
        println!("  Wave {}: {}", i + 1, ids.join(" | "));
    }
}

fn cmd_prune(workspace: &PathBuf, keep: usize) {
    let Ok(store) = HistoryStore::new(workspace) else {
        eprintln!("No history available");
        return;
    };

    match store.prune(keep) {
        Ok(pruned) => {
            if pruned > 0 {
                println!("Pruned {pruned} old runs (kept {keep})");
            } else {
                println!("Nothing to prune");
            }
        }
        Err(e) => eprintln!("Prune error: {e}"),
    }
}

fn cmd_serve(port: u16) {
    eprintln!("Web dashboard requires the 'ssr' feature.");
    eprintln!("Run: cargo run --bin build-orchestrator --features ssr -- --port {port}");
    std::process::exit(1);
}
