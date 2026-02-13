// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima CLI

use clap::{Parser, Subcommand};
use prima::{PrimaResult, builtins, dev, nottrue, parse, repl, reverse, tokenize, visual_repl};
use std::fs;
use std::path::{Path, PathBuf};

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
    /// Start the VISUAL REPL (shows primitives engaging)
    Visual,
    /// Run a Prima source file
    Run {
        file: PathBuf,
        /// Arguments to pass to the script (accessible via args())
        #[arg(trailing_var_arg = true)]
        script_args: Vec<String>,
    },
    /// Parse and show AST
    Parse { file: PathBuf },
    /// Show tokens
    Tokenize { file: PathBuf },
    /// Show grounding trace
    Trace { file: PathBuf },
    /// Evaluate an expression
    Eval { expr: String },
    /// Check a .not.true antipattern file
    Check {
        file: PathBuf,
        /// Strict mode — exit with error on any violation
        #[arg(long)]
        strict: bool,
    },
    /// Watch a file and re-run on changes
    Dev {
        /// Prima source file to watch
        file: PathBuf,
        /// Show AST after parsing
        #[arg(long)]
        ast: bool,
        /// Show tokens after lexing
        #[arg(long)]
        tokens: bool,
        /// Show grounding trace
        #[arg(long)]
        trace: bool,
        /// Run .not.true sibling check
        #[arg(long)]
        check: bool,
    },
    /// Watch a file and re-run on changes (alias for dev)
    Develop {
        /// Prima source file to watch
        file: PathBuf,
        /// Show AST after parsing
        #[arg(long)]
        ast: bool,
        /// Show tokens after lexing
        #[arg(long)]
        tokens: bool,
        /// Show grounding trace
        #[arg(long)]
        trace: bool,
        /// Run .not.true sibling check
        #[arg(long)]
        check: bool,
    },
    /// Reverse-transcribe data into Prima source
    Reverse {
        /// JSON input file
        file: PathBuf,
        /// Also generate .not.true antipattern file
        #[arg(long)]
        violations: bool,
        /// Verify round-trip fidelity (data → source → eval)
        #[arg(long)]
        verify: bool,
        /// Emit as expression instead of let-bindings
        #[arg(long)]
        expr: bool,
    },
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
        Commands::Visual => visual_repl::run(),
        Commands::Run { file, script_args } => cmd_run(&file, script_args),
        Commands::Parse { file } => cmd_parse(&file),
        Commands::Tokenize { file } => cmd_tokenize(&file),
        Commands::Trace { file } => cmd_trace(&file),
        Commands::Eval { expr } => cmd_eval(&expr),
        Commands::Check { file, strict } => cmd_check(&file, strict),
        Commands::Dev {
            file,
            ast,
            tokens,
            trace,
            check,
        }
        | Commands::Develop {
            file,
            ast,
            tokens,
            trace,
            check,
        } => cmd_dev(&file, ast, tokens, trace, check),
        Commands::Reverse {
            file,
            violations,
            verify,
            expr,
        } => cmd_reverse(&file, violations, verify, expr),
    }
}

fn cmd_run(path: &PathBuf, script_args: Vec<String>) -> PrimaResult<()> {
    // Initialize CLI args for the script
    builtins::set_cli_args(script_args);
    builtins::clear_exit_code();

    let source = fs::read_to_string(path)?;
    let result = prima::compile_and_run(&source)?;

    // Only print non-void results
    if !matches!(result.data, prima::value::ValueData::Void) {
        println!("{}", result);
    }

    // Check for exit code set by script
    if let Some(code) = builtins::get_exit_code() {
        std::process::exit(code);
    }

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
    let result = prima::compile_and_run(&source)?;
    println!("Result: {}", result);
    println!("Tier: {}", result.tier().full_name());
    println!("Composition: {}", result.composition);
    println!("Transfer: {:.2}", result.transfer_confidence());
    println!("Grounds to: {{0, 1}} ∎");
    Ok(())
}

fn cmd_eval(expr: &str) -> PrimaResult<()> {
    let result = prima::compile_and_run(expr)?;
    println!(
        "{} : {} ({})",
        result,
        result.tier().code(),
        result.composition
    );
    Ok(())
}

fn cmd_reverse(path: &PathBuf, violations: bool, verify: bool, expr_mode: bool) -> PrimaResult<()> {
    let json_str = fs::read_to_string(path)?;
    let name = path.display().to_string();

    let config = reverse::EngineConfig {
        emit_mode: if expr_mode {
            reverse::EmitMode::Expression
        } else {
            reverse::EmitMode::Bindings
        },
        synthesize_violations: violations,
        verify_roundtrip: verify,
        source_name: name.clone(),
    };

    let mut engine = reverse::TranscriptionEngine::with_config(config);

    // Check if input is a JSON array (batch mode)
    let json: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| prima::PrimaError::runtime(format!("JSON parse error: {e}")))?;

    if json.is_array() {
        // Batch mode
        let batch = engine.batch(&json_str)?;

        println!("// ═══ Batch Reverse Transcription ═══");
        println!("// Source: {}", name);
        println!("// Records: {}", batch.items.len());
        println!("// ═══════════════════════════════════");

        for (i, item) in batch.items.iter().enumerate() {
            println!();
            println!("// ─── Record {} ───", i + 1);
            println!("{}", item.source);
        }

        if violations {
            println!();
            println!("// ─── Merged Violations ───");
            println!("{}", batch.merged_not_true);
        }
    } else {
        // Single record mode
        let result = engine.reverse(&json_str)?;

        println!("// ═══ Reverse Transcription ═══");
        println!("// Source: {}", name);
        println!("// Statements: {}", result.program.statements.len());
        println!("// ═════════════════════════════");
        println!();
        println!("{}", result.source);

        if violations {
            println!();
            println!("{}", result.not_true);
        }
    }

    // Round-trip verification
    if verify {
        println!();
        println!("// ═══ Round-Trip Verification ═══");
        let rt = engine.roundtrip(&json_str)?;
        println!("// Fidelity: {}", rt.fidelity);
        println!("// Source: {}", rt.source);
        if let reverse::RoundtripFidelity::Exact = rt.fidelity {
            println!("// ∎ f⁻¹(f(data)) = data — VERIFIED");
        }
    }

    println!();
    print!("{}", engine.stats());

    Ok(())
}

fn cmd_dev(path: &Path, ast: bool, tokens: bool, trace: bool, check: bool) -> PrimaResult<()> {
    let mut config = dev::DevConfig::new(path.to_path_buf());
    config.show_ast = ast;
    config.show_tokens = tokens;
    config.show_trace = trace;
    config.run_check = check;
    dev::watch(&config)
}

fn cmd_check(path: &PathBuf, strict: bool) -> PrimaResult<()> {
    let source = fs::read_to_string(path)?;
    let name = path.display().to_string();

    if strict {
        let report = nottrue::check_strict(&source, &name)?;
        print!("{report}");
    } else {
        let report = nottrue::check(&source, &name);
        print!("{report}");
        if !report.all_passed() {
            std::process::exit(1);
        }
    }
    Ok(())
}
