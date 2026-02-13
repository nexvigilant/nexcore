//! Primitive Scanner CLI.

use clap::{Parser, Subcommand};

/// Primitive Scanner - Automated T1/T2/T3 extraction.
#[derive(Parser)]
#[command(name = "primitive-scanner")]
#[command(about = "Extract and classify domain primitives")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan sources for primitives.
    Scan {
        /// Domain name.
        #[arg(short, long)]
        domain: String,
        /// Source file patterns.
        #[arg(short, long)]
        sources: Vec<String>,
    },
    /// Batch test terms.
    BatchTest {
        /// Input file with terms.
        #[arg(short, long)]
        terms: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { domain, sources } => {
            println!("Scanning domain: {domain}");
            println!("Sources: {sources:?}");
        }
        Commands::BatchTest { terms } => {
            println!("Batch testing from: {terms}");
        }
    }
}
