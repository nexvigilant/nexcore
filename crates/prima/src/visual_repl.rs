// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Visual REPL
//!
//! A REPL that visualizes the 15 primitives as they engage during computation.
//!
//! ## Tier: T2-C (ρ + σ + → + π + ∂)

use crate::error::PrimaResult;
use crate::interpret::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Token;
use crate::value::{Value, ValueData};
use lex_primitiva::prelude::LexPrimitiva;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::collections::HashSet;

// ═══════════════════════════════════════════════════════════════════════════
// VISUAL CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════

const PROMPT: &str = "⊳ ";

const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";

// The 15 Lex Primitiva in display order
const PRIMITIVE_DISPLAY: &[(LexPrimitiva, &str, &str)] = &[
    (LexPrimitiva::Sequence, "σ", "seq"),
    (LexPrimitiva::Mapping, "μ", "map"),
    (LexPrimitiva::State, "ς", "sta"),
    (LexPrimitiva::Recursion, "ρ", "rec"),
    (LexPrimitiva::Void, "∅", "voi"),
    (LexPrimitiva::Boundary, "∂", "bnd"),
    (LexPrimitiva::Frequency, "ν", "frq"),
    (LexPrimitiva::Existence, "∃", "exi"),
    (LexPrimitiva::Persistence, "π", "per"),
    (LexPrimitiva::Causality, "→", "cau"),
    (LexPrimitiva::Comparison, "κ", "cmp"),
    (LexPrimitiva::Quantity, "N", "qty"),
    (LexPrimitiva::Location, "λ", "loc"),
    (LexPrimitiva::Irreversibility, "∝", "irr"),
    (LexPrimitiva::Sum, "Σ", "sum"),
];

// ═══════════════════════════════════════════════════════════════════════════
// VISUAL REPL
// ═══════════════════════════════════════════════════════════════════════════

/// Run the visual REPL.
pub fn run() -> PrimaResult<()> {
    print_banner();
    let mut rl = DefaultEditor::new().map_err(|e| std::io::Error::other(e.to_string()))?;
    let mut interpreter = Interpreter::new();
    let mut show_tokens = false;
    let mut show_engine = true;

    loop {
        match rl.readline(PROMPT) {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(&line);

                if let Some(cmd) = line.strip_prefix(':') {
                    match handle_command(cmd, &mut show_tokens, &mut show_engine) {
                        CommandResult::Continue => {}
                        CommandResult::Quit => break,
                    }
                    continue;
                }

                eval_and_display(&line, &mut interpreter, show_tokens, show_engine);
            }
            Err(ReadlineError::Interrupted) => println!("^C"),
            Err(ReadlineError::Eof) => {
                println!("\n{}∎ Grounded to {{0, 1}}{}", DIM, RESET);
                break;
            }
            Err(e) => {
                eprintln!("{}∂[io]: {}{}", RED, e, RESET);
                break;
            }
        }
    }
    Ok(())
}

fn print_banner() {
    println!();
    println!(
        "{}╔═══════════════════════════════════════════════════════════╗{}",
        CYAN, RESET
    );
    println!(
        "{}║{}      {}PRIMA VISUAL ENGINE{} — See the Primitives Work       {}║{}",
        CYAN, RESET, BOLD, RESET, CYAN, RESET
    );
    println!(
        "{}╚═══════════════════════════════════════════════════════════╝{}",
        CYAN, RESET
    );
    println!();
    print_primitive_bar(&HashSet::new());
    println!();
    println!(
        "{}Type expressions to see primitives engage. :help for commands.{}",
        DIM, RESET
    );
    println!();
}

fn print_primitive_bar(active: &HashSet<LexPrimitiva>) {
    print!("  {}┃{} ", DIM, RESET);
    for (prim, symbol, _) in PRIMITIVE_DISPLAY {
        if active.contains(prim) {
            print!("{}[{}]{} ", YELLOW, symbol, RESET);
        } else {
            print!("{} {} {} ", DIM, symbol, RESET);
        }
    }
    println!("{}┃{}", DIM, RESET);
}

fn print_primitive_bar_labeled(active: &HashSet<LexPrimitiva>) {
    // Top: symbols
    print!("  {}┃{} ", DIM, RESET);
    for (prim, symbol, _) in PRIMITIVE_DISPLAY {
        if active.contains(prim) {
            print!("{}[{}]{} ", YELLOW, symbol, RESET);
        } else {
            print!("{} {} {} ", DIM, symbol, RESET);
        }
    }
    println!("{}┃{}", DIM, RESET);

    // Bottom: 3-letter names
    print!("  {}┃{} ", DIM, RESET);
    for (prim, _, name) in PRIMITIVE_DISPLAY {
        if active.contains(prim) {
            print!("{}{}{} ", GREEN, name, RESET);
        } else {
            print!("{}{}{} ", DIM, name, RESET);
        }
    }
    println!("{}┃{}", DIM, RESET);
}

// ═══════════════════════════════════════════════════════════════════════════
// EVALUATION WITH VISUALIZATION
// ═══════════════════════════════════════════════════════════════════════════

fn eval_and_display(
    line: &str,
    interpreter: &mut Interpreter,
    show_tokens: bool,
    show_engine: bool,
) {
    // Tokenize
    let tokens = match Lexer::new(line).tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("  {}∂[lex]: {}{}", RED, e, RESET);
            return;
        }
    };

    if show_tokens {
        print_token_primitives(&tokens);
    }

    // Parse
    let program = match Parser::new(tokens).parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  {}∂[parse]: {}{}", RED, e, RESET);
            return;
        }
    };

    // Evaluate
    let result = match interpreter.eval_program(&program) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  {}∂[eval]: {}{}", RED, e, RESET);
            return;
        }
    };

    // Display result with primitive visualization
    if show_engine {
        print_result_visual(&result);
    } else {
        print_result_simple(&result);
    }
}

fn print_token_primitives(tokens: &[Token]) {
    println!();
    println!("  {}── Tokens ──{}", DIM, RESET);
    for token in tokens {
        let prim = token.dominant_primitive();
        println!(
            "    {}{:?}{} → {}{}{}",
            DIM,
            token.kind,
            RESET,
            MAGENTA,
            prim.symbol(),
            RESET
        );
    }
}

fn print_result_visual(result: &Value) {
    println!();

    // Collect active primitives
    let active: HashSet<LexPrimitiva> = result.composition.primitives.iter().copied().collect();

    // Print the primitive bar with active ones highlighted
    print_primitive_bar_labeled(&active);

    // Result value
    println!();
    print!("  {}→{} ", CYAN, RESET);
    print_value_colored(result);
    println!();

    // Tier and composition
    let tier = result.tier();
    let tier_color = match tier.code() {
        "T1" => GREEN,
        "T2-P" => CYAN,
        "T2-C" => YELLOW,
        _ => RED,
    };

    println!();
    println!(
        "  {}├─ Tier:{} {}{}{}",
        DIM,
        RESET,
        tier_color,
        tier.full_name(),
        RESET
    );
    println!("  {}├─ Composition:{} {}", DIM, RESET, result.composition);
    println!(
        "  {}├─ Transfer:{} {:.0}%",
        DIM,
        RESET,
        result.transfer_confidence() * 100.0
    );
    println!("  {}└─ Grounds to:{} {{0, 1}}", DIM, RESET);
    println!();
}

fn print_result_simple(result: &Value) {
    println!(
        "  = {} : {} ({})",
        result,
        result.tier().code(),
        result.composition
    );
}

fn print_value_colored(value: &Value) {
    match &value.data {
        ValueData::Void => print!("{}∅{}", DIM, RESET),
        ValueData::Int(n) => print!("{}{}{}", GREEN, n, RESET),
        ValueData::Float(f) => print!("{}{}{}", GREEN, f, RESET),
        ValueData::Bool(b) => print!("{}{}{}", CYAN, if *b { "true" } else { "false" }, RESET),
        ValueData::String(s) => print!("{}\"{}\"{}", YELLOW, s, RESET),
        ValueData::Symbol(s) => print!("{}'{}{}", MAGENTA, s, RESET),
        ValueData::Sequence(v) => {
            print!("{}σ{}{}", CYAN, DIM, RESET);
            print!("[");
            for (i, item) in v.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print_value_colored(item);
            }
            print!("]");
        }
        ValueData::Mapping(m) => {
            print!("{}μ{}", CYAN, RESET);
            print!("(");
            for (i, (k, v)) in m.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{} → ", k);
                print_value_colored(v);
            }
            print!(")");
        }
        ValueData::Function { .. } => print!("{}μ.fn{}", MAGENTA, RESET),
        ValueData::Builtin(name) => print!("{}μ.{}{}", MAGENTA, name, RESET),
        ValueData::Quoted(inner) => {
            print!("{}'{:?}{}", DIM, inner, RESET);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMMANDS
// ═══════════════════════════════════════════════════════════════════════════

enum CommandResult {
    Continue,
    Quit,
}

fn handle_command(cmd: &str, show_tokens: &mut bool, show_engine: &mut bool) -> CommandResult {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    match parts.first().copied() {
        Some("quit" | "q" | "exit") => {
            return CommandResult::Quit;
        }
        Some("help" | "h" | "?") => print_help(),
        Some("primitives" | "p") => print_primitives_detailed(),
        Some("tokens" | "tok") => {
            *show_tokens = !*show_tokens;
            println!(
                "  Token display: {}",
                if *show_tokens { "ON" } else { "OFF" }
            );
        }
        Some("engine" | "e") => {
            *show_engine = !*show_engine;
            println!(
                "  Engine display: {}",
                if *show_engine { "ON" } else { "OFF" }
            );
        }
        Some("clear" | "c") => {
            print!("\x1B[2J\x1B[1;1H");
            print_banner();
        }
        Some("flow") => print_flow_example(),
        Some("tiers" | "t") => print_tiers(),
        _ => println!("  Unknown command. Type {}:help{}", YELLOW, RESET),
    }
    CommandResult::Continue
}

fn print_help() {
    println!();
    println!(
        "  {}╭─ Commands ─────────────────────────────────────╮{}",
        DIM, RESET
    );
    println!(
        "  {}│{} :primitives  Show the 15 Lex Primitiva        {}│{}",
        DIM, RESET, DIM, RESET
    );
    println!(
        "  {}│{} :tiers       Show tier classification         {}│{}",
        DIM, RESET, DIM, RESET
    );
    println!(
        "  {}│{} :tokens      Toggle token primitive display   {}│{}",
        DIM, RESET, DIM, RESET
    );
    println!(
        "  {}│{} :engine      Toggle engine visualization      {}│{}",
        DIM, RESET, DIM, RESET
    );
    println!(
        "  {}│{} :flow        Show primitive flow example      {}│{}",
        DIM, RESET, DIM, RESET
    );
    println!(
        "  {}│{} :clear       Clear screen                     {}│{}",
        DIM, RESET, DIM, RESET
    );
    println!(
        "  {}│{} :quit        Exit                             {}│{}",
        DIM, RESET, DIM, RESET
    );
    println!(
        "  {}╰────────────────────────────────────────────────╯{}",
        DIM, RESET
    );
    println!();
}

fn print_primitives_detailed() {
    println!();
    println!(
        "  {}╭─ The 15 Lex Primitiva ─────────────────────────╮{}",
        CYAN, RESET
    );
    println!(
        "  {}│{}                                                {}│{}",
        CYAN, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}σ{} Sequence      Ordered collection          {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}μ{} Mapping       Key→Value association       {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}ς{} State         Mutable container           {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}ρ{} Recursion     Self-reference              {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}∅{} Void          Absence/null                {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}∂{} Boundary      Condition/edge              {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}ν{} Frequency     Count/occurrence            {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}∃{} Existence     Present/defined             {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}π{} Persistence   Storage/I/O                 {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}→{} Causality     Transformation              {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}κ{} Comparison    Equality/ordering           {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}N{} Quantity      Numeric value               {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}λ{} Location      Position/address            {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}∝{} Irreversibility  One-way/entropy          {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  {}Σ{} Sum           Aggregation                 {}│{}",
        CYAN, RESET, YELLOW, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}                                                {}│{}",
        CYAN, RESET, CYAN, RESET
    );
    println!(
        "  {}│{}  All computation grounds to: {}{{0, 1}}{}          {}│{}",
        CYAN, RESET, GREEN, RESET, CYAN, RESET
    );
    println!(
        "  {}╰────────────────────────────────────────────────╯{}",
        CYAN, RESET
    );
    println!();
}

fn print_tiers() {
    println!();
    println!(
        "  {}╭─ Tier Classification ──────────────────────────╮{}",
        DIM, RESET
    );
    println!(
        "  {}│{}                                                {}│{}",
        DIM, RESET, DIM, RESET
    );
    println!(
        "  {}│{}  {}T1{}     1 primitive     Transfer: {}100%{}       {}│{}",
        DIM, RESET, GREEN, RESET, GREEN, RESET, DIM, RESET
    );
    println!(
        "  {}│{}  {}T2-P{}   2-3 primitives  Transfer: {}90%{}        {}│{}",
        DIM, RESET, CYAN, RESET, CYAN, RESET, DIM, RESET
    );
    println!(
        "  {}│{}  {}T2-C{}   4-5 primitives  Transfer: {}70%{}        {}│{}",
        DIM, RESET, YELLOW, RESET, YELLOW, RESET, DIM, RESET
    );
    println!(
        "  {}│{}  {}T3{}     6+ primitives   Transfer: {}40%{}        {}│{}",
        DIM, RESET, RED, RESET, RED, RESET, DIM, RESET
    );
    println!(
        "  {}│{}                                                {}│{}",
        DIM, RESET, DIM, RESET
    );
    println!(
        "  {}╰────────────────────────────────────────────────╯{}",
        DIM, RESET
    );
    println!();
}

fn print_flow_example() {
    println!();
    println!(
        "  {}╭─ Primitive Flow: Hook Example ─────────────────╮{}",
        MAGENTA, RESET
    );
    println!(
        "  {}│{}                                                {}│{}",
        MAGENTA, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}  {}stdin(){} ─────────────────────→ String        {}│{}",
        MAGENTA, RESET, CYAN, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}    │  {}π{} (Persistence)                         {}│{}",
        MAGENTA, RESET, YELLOW, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}    ▼                                           {}│{}",
        MAGENTA, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}  {}json_parse(){} ──────────────→ Mapping        {}│{}",
        MAGENTA, RESET, CYAN, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}    │  {}μ{} (Mapping) + {}→{} (Causality)            {}│{}",
        MAGENTA, RESET, YELLOW, RESET, YELLOW, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}    ▼                                           {}│{}",
        MAGENTA, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}  {}if condition{} ───────────────→ Branch        {}│{}",
        MAGENTA, RESET, CYAN, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}    │  {}∂{} (Boundary) + {}κ{} (Comparison)          {}│{}",
        MAGENTA, RESET, YELLOW, RESET, YELLOW, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}    ▼                                           {}│{}",
        MAGENTA, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}  {}exit(N){} ────────────────────→ Void          {}│{}",
        MAGENTA, RESET, CYAN, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}       {}→{} (Causality) + {}∅{} (Void)               {}│{}",
        MAGENTA, RESET, YELLOW, RESET, YELLOW, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}                                                {}│{}",
        MAGENTA, RESET, MAGENTA, RESET
    );
    println!(
        "  {}│{}  {}Hook = π → μ → ∂ → →{}                         {}│{}",
        MAGENTA, RESET, GREEN, RESET, MAGENTA, RESET
    );
    println!(
        "  {}╰────────────────────────────────────────────────╯{}",
        MAGENTA, RESET
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_display_count() {
        assert_eq!(PRIMITIVE_DISPLAY.len(), 15);
    }
}
