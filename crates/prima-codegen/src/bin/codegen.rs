// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Code Generator CLI
//!
//! Generates target language code from Prima source files.
//!
//! ## Usage
//!
//! ```bash
//! prima-codegen --target rust input.true > output.rs
//! prima-codegen --target typescript input.true -o output.ts
//! prima-codegen --target go input.true
//! ```

use clap::{Parser, ValueEnum};
use prima_codegen::{
    Backend, EmitContext, TargetLanguage,
    backends::{CBackend, GoBackend, PythonBackend, RustBackend, TypeScriptBackend},
};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prima-codegen")]
#[command(about = "Prima Universal Code Generator — T1 primitives to target languages")]
#[command(author = "Matthew Alexander Campion, PharmD")]
#[command(version)]
struct Cli {
    /// Target language for code generation
    #[arg(short, long, value_enum, default_value = "rust")]
    target: Target,

    /// Input Prima source file (.true or .prima)
    file: PathBuf,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show primitive usage summary
    #[arg(long)]
    show_primitives: bool,

    /// Quiet mode (no status messages)
    #[arg(short, long)]
    quiet: bool,
}

/// Target language for code generation
#[derive(Clone, Copy, ValueEnum)]
enum Target {
    Rust,
    Python,
    Typescript,
    Go,
    C,
}

impl Target {
    fn to_language(self) -> TargetLanguage {
        match self {
            Target::Rust => TargetLanguage::Rust,
            Target::Python => TargetLanguage::Python,
            Target::Typescript => TargetLanguage::TypeScript,
            Target::Go => TargetLanguage::Go,
            Target::C => TargetLanguage::C,
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("∂[error]: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Read source file
    let source = fs::read_to_string(&cli.file)?;

    // Parse Prima source
    let program = prima::parse(&source).map_err(|e| format!("Parse error: {}", e))?;

    // Generate code
    let mut ctx = EmitContext::new();
    let code = match cli.target {
        Target::Rust => RustBackend::new().emit_program(&program, &mut ctx),
        Target::Python => PythonBackend::new().emit_program(&program, &mut ctx),
        Target::Typescript => TypeScriptBackend::new().emit_program(&program, &mut ctx),
        Target::Go => GoBackend::new().emit_program(&program, &mut ctx),
        Target::C => CBackend::new().emit_program(&program, &mut ctx),
    }?;

    // Output result
    match &cli.output {
        Some(out_path) => {
            fs::write(out_path, &code)?;
            if !cli.quiet {
                let lang = cli.target.to_language();
                eprintln!(
                    "✓ Generated {} ({} bytes, transfer: {:.2})",
                    out_path.display(),
                    code.len(),
                    lang.transfer_confidence()
                );
            }
        }
        None => {
            io::stdout().write_all(code.as_bytes())?;
            println!();
        }
    }

    // Report primitive usage
    if cli.show_primitives && !ctx.primitives_used.is_empty() {
        let lang = cli.target.to_language();
        eprintln!(
            "⚙ Primitives used: {}",
            ctx.primitives_used
                .iter()
                .map(|p| p.symbol())
                .collect::<Vec<_>>()
                .join(" ")
        );
        eprintln!(
            "  Transfer confidence: {:.2}",
            ctx.transfer_confidence(lang)
        );
    }

    Ok(())
}
