//! CLI entry point for the skill compiler.

use clap::Parser;
use nexcore_skill_compiler::error::CompilerError;

/// Compile 2+ skills into a single compound skill binary.
#[derive(Parser)]
#[command(name = "skill-compile", version, about)]
struct Cli {
    /// Comma-separated list of skill names.
    #[arg(long, value_delimiter = ',')]
    skills: Vec<String>,

    /// Composition strategy: sequential, parallel, feedback_loop.
    #[arg(long, default_value = "sequential")]
    strategy: String,

    /// Name for the compound skill.
    #[arg(long, default_value = "compound")]
    name: String,

    /// Build the binary (otherwise just generates source).
    #[arg(long, default_value_t = false)]
    build: bool,

    /// Only run compatibility check (no codegen).
    #[arg(long, default_value_t = false)]
    check: bool,

    /// Path to a compound.toml spec file.
    #[arg(long)]
    spec: Option<String>,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let result = if cli.check {
        run_check(&cli)
    } else if let Some(ref spec_path) = cli.spec {
        run_from_spec(spec_path, cli.build)
    } else {
        run_from_args(&cli)
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run_check(cli: &Cli) -> Result<(), CompilerError> {
    let report = nexcore_skill_compiler::check(&cli.skills, &cli.strategy)?;
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        println!("{json}");
    }
    Ok(())
}

fn run_from_spec(spec_path: &str, do_build: bool) -> Result<(), CompilerError> {
    let toml_text = std::fs::read_to_string(spec_path)?;
    let result = nexcore_skill_compiler::compile(&toml_text, do_build)?;
    if let Ok(json) = serde_json::to_string_pretty(&result) {
        println!("{json}");
    }
    Ok(())
}

fn run_from_args(cli: &Cli) -> Result<(), CompilerError> {
    let toml_text =
        nexcore_skill_compiler::spec_from_params(&cli.skills, &cli.strategy, &cli.name)?;
    let result = nexcore_skill_compiler::compile(&toml_text, cli.build)?;
    if let Ok(json) = serde_json::to_string_pretty(&result) {
        println!("{json}");
    }
    Ok(())
}
