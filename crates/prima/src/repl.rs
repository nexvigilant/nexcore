// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima REPL
//!
//! Interactive Read-Eval-Print Loop.
//!
//! ## Tier: T2-C (ρ + σ + → + π)

use crate::error::PrimaResult;
use crate::interpret::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::FILE_EXTENSION;
use crate::value::Value;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

const PROMPT: &str = "σ> ";

/// Run the Prima REPL.
pub fn run_repl() -> PrimaResult<()> {
    print_banner();
    let mut rl = DefaultEditor::new().map_err(|e| std::io::Error::other(e.to_string()))?;
    let mut interpreter = Interpreter::new();
    repl_loop(&mut rl, &mut interpreter)
}

fn print_banner() {
    println!("Prima (πρίμα) — Primitive-First Language");
    println!("File extension: .{}", FILE_EXTENSION);
    println!("Root constants: 0, 1 | Primitives: σ μ ς ρ ∅ ∂ ν ∃ π → κ N λ ∝ Σ");
    println!("Type :help for commands, Ctrl-D to exit.\n");
}

fn repl_loop(rl: &mut DefaultEditor, interpreter: &mut Interpreter) -> PrimaResult<()> {
    loop {
        match rl.readline(PROMPT) {
            Ok(line) => {
                if !process_line(&line, rl, interpreter) {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => println!("^C"),
            Err(ReadlineError::Eof) => {
                println!("∎");
                break;
            }
            Err(e) => {
                eprintln!("∂[io]: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn process_line(line: &str, rl: &mut DefaultEditor, interpreter: &mut Interpreter) -> bool {
    if line.trim().is_empty() {
        return true;
    }
    let _ = rl.add_history_entry(line);

    if let Some(cmd) = line.strip_prefix(':') {
        return !handle_command(cmd);
    }

    match eval_line(line, interpreter) {
        Ok(result) => print_result(&result),
        Err(e) => eprintln!("  ∂ {}", e),
    }
    true
}

fn print_result(result: &Value) {
    println!("  = {}", result);
    println!("  : {} ({})", result.tier().code(), result.composition);
}

fn handle_command(cmd: &str) -> bool {
    match cmd.split_whitespace().next() {
        Some("quit" | "q" | "exit") => {
            println!("∎");
            return true;
        }
        Some("help" | "h" | "?") => print_help(),
        Some("primitives" | "p") => print_primitives(),
        Some("tiers" | "t") => print_tiers(),
        Some("clear" | "c") => print!("\x1B[2J\x1B[1;1H"),
        _ => println!("Unknown command. Type :help"),
    }
    false
}

fn eval_line(line: &str, interpreter: &mut Interpreter) -> PrimaResult<Value> {
    let tokens = Lexer::new(line).tokenize()?;
    let program = Parser::new(tokens).parse()?;
    interpreter.eval_program(&program)
}

fn print_help() {
    println!(":help     — Help  |  :primitives — List primitives");
    println!(":tiers    — Tiers |  :quit       — Exit");
    println!("\nBuiltins: print, tier, composition, transfer, constants");
}

fn print_primitives() {
    println!("σ Sequence | μ Mapping | ς State | ρ Recursion | ∅ Void");
    println!("∂ Boundary | ν Frequency | ∃ Existence | π Persistence");
    println!("→ Causality | κ Comparison | N Quantity | λ Location");
    println!("∝ Irreversibility | Σ Sum");
    println!("\nRoots: 0 (absence), 1 (existence)");
}

fn print_tiers() {
    println!("T1 (1 prim): 1.0 | T2-P (2-3): 0.9 | T2-C (4-5): 0.7 | T3 (6+): 0.4");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_line() {
        let mut interpreter = Interpreter::new();
        let result = eval_line("1 + 2", &mut interpreter).unwrap();
        assert_eq!(result, Value::int(3));
    }
}
