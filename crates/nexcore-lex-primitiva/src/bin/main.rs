//! Lex Primitiva CLI binary.

#![allow(
    clippy::print_stderr,
    reason = "Binary entry point reports errors to stderr by design"
)]

use clap::Parser;
use nexcore_lex_primitiva::cli::{Cli, run};

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
