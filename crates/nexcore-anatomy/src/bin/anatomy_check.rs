//! # anatomy-check — CI-compatible workspace boundary enforcement
//!
//! Exits 0 if workspace is healthy, 1 if boundary violations or cycles exist.
//! Designed for `just validate` and CI pipeline integration.
//!
//! ## Usage
//! ```text
//! cargo run -p nexcore-anatomy --bin anatomy-check [-- --strict]
//! ```
//!
//! Flags:
//! - `--strict`: Exit 1 on ANY violation (default: only on severity >= 2)
//! - `--json`:   Output full report as JSON
//! - `--quiet`:  Suppress output (exit code only)

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::arithmetic_side_effects,
    reason = "CLI tool requires stdout/stderr printing and metric calculations"
)]

use std::process::ExitCode;

use nexcore_anatomy::{
    AnatomyReport, BlastRadiusReport, ChomskyReport, DependencyGraph, HealthStatus,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let strict = args.iter().any(|a| a == "--strict");
    let json_output = args.iter().any(|a| a == "--json");
    let quiet = args.iter().any(|a| a == "--quiet");

    // Find workspace root Cargo.toml
    let manifest_path = find_workspace_manifest();

    let metadata = match cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("ERROR: Failed to read cargo metadata: {e}");
            return ExitCode::from(2);
        }
    };

    let graph = DependencyGraph::from_metadata(&metadata);
    let report = AnatomyReport::from_graph(graph.clone());
    let blast = BlastRadiusReport::from_graph(&graph);
    let chomsky = ChomskyReport::from_graph(&graph);

    if json_output {
        match report.to_json() {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("JSON serialization error: {e}"),
        }
        return exit_code_for_health(&report, strict);
    }

    if !quiet {
        println!("=== NexCore Anatomy Check ===");
        println!();

        // Health summary
        let health_icon = match report.summary.health {
            HealthStatus::Healthy => "\u{2714}",  // checkmark
            HealthStatus::Warning => "\u{26a0}",  // warning
            HealthStatus::Critical => "\u{2718}", // cross
            _ => "?",
        };
        println!("Health: {} {}", health_icon, report.summary.health.label());
        println!("Crates: {}", report.summary.total_crates);
        println!("Cycles: {}", report.summary.cycle_count);
        println!("Max depth: {}", report.summary.max_depth);
        println!(
            "Density: {:.4} ({})",
            report.summary.graph_density,
            if report.summary.graph_density < 0.1 {
                "sparse"
            } else if report.summary.graph_density < 0.3 {
                "moderate"
            } else {
                "dense"
            }
        );
        println!();

        // Bottleneck
        println!("--- Bottleneck ---");
        println!(
            "  {} (fan-in: {}, blast radius: {:.0}%)",
            blast.worst_case_crate,
            report.summary.max_fan_in,
            blast.worst_case_ratio * 100.0
        );
        println!();

        // Criticality
        println!("--- Criticality ---");
        println!(
            "  Critical: {}  Supporting: {}  Standard: {}  Experimental: {}",
            report.summary.critical_count,
            report.summary.supporting_count,
            report.summary.total_crates
                - report.summary.critical_count
                - report.summary.supporting_count
                - report.summary.experimental_count,
            report.summary.experimental_count
        );
        println!();

        // Chomsky
        println!("--- Chomsky Distribution ---");
        for (label, count) in &chomsky.level_distribution {
            println!("  {label}: {count}");
        }
        println!("  Avg generators: {:.2}", chomsky.avg_generators);
        println!();

        // Boundary violations
        if report.layers.violations.is_empty() {
            println!("--- Boundaries ---");
            println!("  No violations detected.");
        } else {
            println!(
                "--- Boundary Violations ({}) ---",
                report.layers.violations.len()
            );
            for v in &report.layers.violations {
                let severity_marker = if v.severity >= 2 { "!!" } else { " !" };
                println!(
                    "  {severity_marker} {} ({:?}) -> {} ({:?}) [severity={}]",
                    v.from_crate, v.from_layer, v.to_crate, v.to_layer, v.severity
                );
            }
        }
        println!();
    }

    exit_code_for_health(&report, strict)
}

fn exit_code_for_health(report: &AnatomyReport, strict: bool) -> ExitCode {
    match report.summary.health {
        HealthStatus::Healthy => ExitCode::SUCCESS,
        HealthStatus::Warning => {
            if strict {
                // In strict mode, any violation fails
                if report.layers.has_violations() {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            } else {
                // Default: only fail on severity >= 2
                let severe = report.layers.violations.iter().any(|v| v.severity >= 2);
                if severe {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
        }
        HealthStatus::Critical => ExitCode::FAILURE,
        _ => ExitCode::FAILURE,
    }
}

fn find_workspace_manifest() -> std::path::PathBuf {
    // Try to find the workspace root by walking up from current dir or using env
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::Path::new(&manifest_dir);
        // Walk up to find workspace root
        if let Some(parent) = p.parent().and_then(|p| p.parent()) {
            let workspace_toml = parent.join("Cargo.toml");
            if workspace_toml.exists() {
                return workspace_toml;
            }
        }
    }

    // Fallback: look for ~/nexcore/Cargo.toml
    if let Some(home) = std::env::var_os("HOME") {
        let nexcore_toml = std::path::PathBuf::from(home)
            .join("nexcore")
            .join("Cargo.toml");
        if nexcore_toml.exists() {
            return nexcore_toml;
        }
    }

    // Last resort: current directory
    std::path::PathBuf::from("Cargo.toml")
}
