//! Convenience binary — delegates to `academy-forge compile`.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Compile pathway JSON into Studio TypeScript files.
#[derive(Parser)]
#[command(name = "forge-compile", version, about)]
struct Cli {
    /// Path to the pathway JSON file.
    pathway: PathBuf,
    /// Directory to write TypeScript files into.
    output_dir: PathBuf,
    /// Overwrite existing files.
    #[arg(long)]
    overwrite: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let params =
        academy_forge::compile::CompileParams::new(cli.pathway, cli.output_dir, cli.overwrite);
    match academy_forge::compile::compile(&params) {
        Ok(result) => {
            let count = result.files_written.len();
            eprintln!("Compiled {count} files");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
