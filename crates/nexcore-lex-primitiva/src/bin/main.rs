//! Lex Primitiva CLI binary.

use clap::Parser;
use nexcore_lex_primitiva::cli::{Cli, run};

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
