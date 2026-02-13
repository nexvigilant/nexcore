// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima CLI

use clap::{Parser, Subcommand};
use nexcore_prima::prelude::*;
use nexcore_prima::{eval, parse, repl, tokenize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prima")]
#[command(about = "Prima (πρίμα) — Primitive-First Language")]
#[command(author = "Matthew Alexander Campion, PharmD")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the interactive REPL
    Repl,
    /// Run a Prima source file
    Run { file: PathBuf },
    /// Parse and show AST
    Parse { file: PathBuf },
    /// Show tokens
    Tokenize { file: PathBuf },
    /// Show grounding trace
    Trace { file: PathBuf },
    /// Evaluate an expression
    Eval { expr: String },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("∂[error]: {}", e);
        std::process::exit(1);
    }
}

fn run() -> PrimaResult<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Repl => repl::run_repl(),
        Commands::Run { file } => cmd_run(&file),
        Commands::Parse { file } => cmd_parse(&file),
        Commands::Tokenize { file } => cmd_tokenize(&file),
        Commands::Trace { file } => cmd_trace(&file),
        Commands::Eval { expr } => cmd_eval(&expr),
    }
}

fn cmd_run(path: &PathBuf) -> PrimaResult<()> {
    let source = fs::read_to_string(path)?;
    println!("{}", eval(&source)?);
    Ok(())
}

fn cmd_parse(path: &PathBuf) -> PrimaResult<()> {
    let source = fs::read_to_string(path)?;
    let program = parse(&source)?;
    println!("Statements: {}", program.statements.len());
    for (i, s) in program.statements.iter().enumerate() {
        println!("  [{}] {:?}", i, s);
    }
    Ok(())
}

fn cmd_tokenize(path: &PathBuf) -> PrimaResult<()> {
    let source = fs::read_to_string(path)?;
    for token in tokenize(&source)? {
        println!("  {} ({})", token, token.dominant_primitive().symbol());
    }
    Ok(())
}

fn cmd_trace(path: &PathBuf) -> PrimaResult<()> {
    let source = fs::read_to_string(path)?;
    let result = eval(&source)?;
    println!("Result: {}", result);
    println!("Tier: {}", result.tier().full_name());
    println!("Composition: {}", result.composition);
    println!("Transfer: {:.2}", result.transfer_confidence());
    println!("Grounds to: {{0, 1}} ∎");
    Ok(())
}

fn cmd_eval(expr: &str) -> PrimaResult<()> {
    let result = eval(expr)?;
    println!(
        "{} : {} ({})",
        result,
        result.tier().code(),
        result.composition
    );
    Ok(())
}
