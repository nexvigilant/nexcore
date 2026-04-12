#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Wallace Protocol CLI — fearless Rust code quality scanner.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nexcore_wallace::{Classification, ViolationKind, WorkspaceReport};

#[derive(Parser)]
#[command(
    name = "wallace",
    version,
    about = "Wallace Protocol — fearless Rust code quality scanner"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scan workspace crates directory for all violations
    Scan {
        /// Path to crates/ directory
        #[arg(default_value = "crates")]
        crates_dir: PathBuf,

        /// Output as JSON instead of table
        #[arg(long)]
        json: bool,

        /// Show only actionable violations
        #[arg(long)]
        actionable: bool,
    },

    /// Scan a single crate
    Crate {
        /// Path to the crate directory
        path: PathBuf,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Full pipeline: scan + classify + report
    Pipeline {
        /// Path to crates/ directory
        #[arg(default_value = "crates")]
        crates_dir: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan {
            crates_dir,
            json,
            actionable,
        } => cmd_scan(&crates_dir, json, actionable),
        Command::Crate { path, json } => cmd_crate(&path, json),
        Command::Pipeline { crates_dir } => cmd_pipeline(&crates_dir),
    }
}

fn cmd_scan(crates_dir: &PathBuf, json: bool, actionable_only: bool) -> ExitCode {
    eprintln!("Scanning {}...", crates_dir.display());

    let report = match nexcore_wallace::scan_workspace(crates_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::from(1);
        }
    };

    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(j) => println!("{j}"),
            Err(e) => {
                eprintln!("JSON error: {e}");
                return ExitCode::from(1);
            }
        }
    } else {
        print_table_report(&report, actionable_only);
    }

    eprintln!(
        "\nTotal: {} violations ({} actionable) across {} crates",
        report.total_violations(),
        report.total_actionable(),
        report.crates.len()
    );

    ExitCode::SUCCESS
}

fn cmd_crate(path: &PathBuf, json: bool) -> ExitCode {
    eprintln!("Scanning crate at {}...", path.display());

    let report = match nexcore_wallace::scan_crate(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::from(1);
        }
    };

    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(j) => println!("{j}"),
            Err(e) => {
                eprintln!("JSON error: {e}");
                return ExitCode::from(1);
            }
        }
    } else {
        eprintln!("Crate: {}", report.name);
        eprintln!(
            "  Unwrap: {}  Expect: {}  Clone: {}",
            report.by_kind(ViolationKind::Unwrap),
            report.by_kind(ViolationKind::Expect),
            report.by_kind(ViolationKind::Clone),
        );
        eprintln!(
            "  Total: {}  Actionable: {}",
            report.total(),
            report.actionable()
        );

        if report.actionable() > 0 {
            eprintln!("\nActionable violations:");
            for v in &report.violations {
                if v.classification.is_actionable() {
                    eprintln!(
                        "  {}:{} [{:?}] {:?} — {}",
                        v.file.display(),
                        v.line,
                        v.kind,
                        v.classification,
                        v.context
                    );
                }
            }
        }
    }

    ExitCode::SUCCESS
}

fn cmd_pipeline(crates_dir: &PathBuf) -> ExitCode {
    eprintln!("=== WALLACE PROTOCOL — FULL BATTLEFIELD REPORT ===\n");

    let report = match nexcore_wallace::scan_workspace(crates_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::from(1);
        }
    };

    // Phase 1: Classification summary
    let mut test_code = 0usize;
    let mut doc_comment = 0usize;
    let mut example = 0usize;
    let mut detection = 0usize;
    let mut allowed = 0usize;
    let mut infallible = 0usize;
    let mut main_fn = 0usize;
    let mut mechanical = 0usize;
    let mut sig_lift = 0usize;
    let mut necessary_clone = 0usize;

    for cr in &report.crates {
        for v in &cr.violations {
            match v.classification {
                Classification::TestCode => test_code += 1,
                Classification::DocComment => doc_comment += 1,
                Classification::Example => example += 1,
                Classification::DetectionCode => detection += 1,
                Classification::Allowed => allowed += 1,
                Classification::InfallibleStatic => infallible += 1,
                Classification::MainFunction => main_fn += 1,
                Classification::TypestateInvariant => {}
                Classification::Mechanical => mechanical += 1,
                Classification::SignatureLift => sig_lift += 1,
                Classification::Movable | Classification::Unnecessary => {}
                Classification::Necessary => necessary_clone += 1,
            }
        }
    }

    let total = report.total_violations();
    let actionable = report.total_actionable();

    eprintln!("CLASSIFICATION BREAKDOWN");
    eprintln!("  Test code:        {test_code:>6}");
    eprintln!("  Doc comments:     {doc_comment:>6}");
    eprintln!("  Examples:         {example:>6}");
    eprintln!("  Detection code:   {detection:>6}");
    eprintln!("  Allowed:          {allowed:>6}");
    eprintln!("  Infallible static:{infallible:>6}");
    eprintln!("  Main function:    {main_fn:>6}");
    eprintln!("  Necessary clone:  {necessary_clone:>6}");
    eprintln!("  ─────────────────────────");
    eprintln!("  Mechanical (fixable): {mechanical:>4}");
    eprintln!("  Signature lift:       {sig_lift:>4}");
    eprintln!("  ─────────────────────────");
    eprintln!("  Total:            {total:>6}");
    eprintln!("  Actionable:       {actionable:>6}");

    // Phase 2: Top offenders
    eprintln!("\nTOP 15 CRATES BY ACTIONABLE VIOLATIONS");
    let mut by_actionable: Vec<_> = report
        .crates
        .iter()
        .filter(|c| c.actionable() > 0)
        .map(|c| (c.name.as_str(), c.actionable(), c.total()))
        .collect();
    by_actionable.sort_by(|a, b| b.1.cmp(&a.1));

    for (name, act, tot) in by_actionable.iter().take(15) {
        eprintln!("  {act:>4} actionable / {tot:>5} total — {name}");
    }

    if actionable == 0 {
        eprintln!("\nFREEDOM! Zero actionable violations.");
    } else {
        eprintln!("\n{actionable} violations to fight. Rally the clans.");
    }

    ExitCode::SUCCESS
}

fn print_table_report(report: &WorkspaceReport, actionable_only: bool) {
    eprintln!(
        "\n{:<35} {:>6} {:>6} {:>6} {:>8}",
        "CRATE", "UNWRAP", "EXPECT", "CLONE", "ACTION"
    );
    eprintln!("{}", "-".repeat(70));

    let mut sorted: Vec<_> = report.crates.iter().collect();
    sorted.sort_by(|a, b| b.actionable().cmp(&a.actionable()));

    for cr in &sorted {
        if actionable_only && cr.actionable() == 0 {
            continue;
        }
        if cr.total() == 0 {
            continue;
        }
        eprintln!(
            "{:<35} {:>6} {:>6} {:>6} {:>8}",
            cr.name,
            cr.by_kind(ViolationKind::Unwrap),
            cr.by_kind(ViolationKind::Expect),
            cr.by_kind(ViolationKind::Clone),
            cr.actionable(),
        );
    }
}
