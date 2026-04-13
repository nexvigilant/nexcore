//! Convenience binary — delegates to `academy-forge validate`.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Validate pathway JSON files against the 27-rule engine.
#[derive(Parser)]
#[command(name = "forge-validate", version, about)]
struct Cli {
    /// One or more pathway JSON files to validate.
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut all_passed = true;
    for path in &cli.files {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {e}", path.display());
                all_passed = false;
                continue;
            }
        };
        let value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Invalid JSON in {}: {e}", path.display());
                all_passed = false;
                continue;
            }
        };
        let report = academy_forge::validate::validate(&value, None);
        if report.passed {
            eprintln!(
                "{}: PASS ({} checked)",
                path.display(),
                report.total_findings
            );
        } else {
            eprintln!("{}: FAIL ({} errors)", path.display(), report.error_count);
            for finding in &report.findings {
                eprintln!(
                    "  - [{:?}] {}: {}",
                    finding.severity, finding.rule, finding.message
                );
            }
            all_passed = false;
        }
    }
    if all_passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
