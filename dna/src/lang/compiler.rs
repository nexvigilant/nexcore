//! Compiler: source text → Program.
//!
//! The public API that ties lexer → parser → codegen into a single call.
//! This is the entry point AI/LLMs use to compile high-level programs.
//!
//! Phase 4: Extended with variables, control flow, and functions.
//!
//! Tier: T3 (→ Causality + μ Mapping + σ Sequence + ∂ Boundary + ρ Recursion + ς State)

use crate::error::Result;
use crate::gene::Genome;
use crate::lang::codegen;
use crate::lang::optimizer;
use crate::lang::parser;
use crate::program::Program;
use crate::vm::VmResult;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compile source text into a DNA Program.
///
/// The source is statement-oriented:
/// - Expression lines are evaluated and output
/// - `let x = expr` binds a variable
/// - `x = expr` mutates a variable
/// - `if cond do ... end` / `if cond do ... else ... end`
/// - `while cond do ... end`
/// - `fn name(params) do ... end`
/// - `return expr`
///
/// Token ratio: ≤1.0 tokens per semantic operation.
pub fn compile(source: &str) -> Result<Program> {
    let stmts = parser::parse(source)?;
    let optimized = optimizer::optimize(&stmts);
    codegen::compile_stmts(&optimized)
}

/// Compile and immediately execute source text.
///
/// Returns the VM result including output, stack state, and cycle count.
/// This is the "eval" path for the REPL and MCP tools.
pub fn eval(source: &str) -> Result<VmResult> {
    let program = compile(source)?;
    program.run()
}

/// Compile source text to assembly text (for debugging/inspection).
pub fn compile_to_asm(source: &str) -> Result<String> {
    let stmts = parser::parse(source)?;
    let optimized = optimizer::optimize(&stmts);
    codegen::compile_to_asm(&optimized)
}

/// Compile source text into a Genome with annotated gene boundaries.
///
/// A Genome contains the full compiled DNA strand plus a catalog of
/// all functions as Gene objects, enabling:
/// - Individual gene expression (`genome.express("fn_name", &args)`)
/// - Gene mutation (point, insertion, deletion)
/// - Crossover between genes
/// - Gene extraction as Plasmids
pub fn compile_genome(source: &str) -> Result<Genome> {
    let stmts = parser::parse(source)?;
    let optimized = optimizer::optimize(&stmts);
    let (program, labels, annotations) = codegen::compile_genome_stmts(&optimized)?;
    Genome::from_program(&program, annotations, &labels)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::HaltReason;

    fn eval_output(source: &str) -> Vec<i64> {
        match eval(source) {
            Ok(r) => r.output,
            Err(_) => vec![],
        }
    }

    #[test]
    fn eval_simple() {
        assert_eq!(eval_output("42"), vec![42]);
    }

    #[test]
    fn eval_arithmetic() {
        assert_eq!(eval_output("2 + 3 * 4"), vec![14]);
    }

    #[test]
    fn eval_multiline() {
        assert_eq!(eval_output("5 + 1\n3 * 3"), vec![6, 9]);
    }

    #[test]
    fn eval_with_comments() {
        assert_eq!(eval_output("42 ; the answer"), vec![42]);
    }

    #[test]
    fn eval_empty() {
        let result = eval("");
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(r.output.is_empty());
            assert!(matches!(r.halt_reason, HaltReason::Normal));
        }
    }

    #[test]
    fn eval_cycle_count() {
        let result = eval("1 + 1");
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(r.cycles > 0);
            assert!(r.cycles < 100);
        }
    }

    #[test]
    fn compile_produces_strand() {
        let result = compile("2 + 3");
        assert!(result.is_ok());
        if let Ok(p) = result {
            let codon_count = p.codon_count();
            assert!(codon_count.is_ok());
            if let Ok(count) = codon_count {
                assert!(count > 0);
            }
        }
    }

    #[test]
    fn eval_error_on_bad_syntax() {
        let result = eval("2 +");
        assert!(result.is_err());
    }

    // --- Boolean literal eval tests ---

    #[test]
    fn eval_true() {
        assert_eq!(eval_output("true"), vec![1]);
    }

    #[test]
    fn eval_false() {
        assert_eq!(eval_output("false"), vec![0]);
    }

    #[test]
    fn eval_bool_logic() {
        assert_eq!(eval_output("true and true"), vec![1]);
        assert_eq!(eval_output("true and false"), vec![0]);
        assert_eq!(eval_output("false or true"), vec![1]);
        assert_eq!(eval_output("not false"), vec![1]);
    }

    #[test]
    fn eval_bool_in_while() {
        // `while false do` → zero iterations
        let source = "
let ran = 0
while false do
  ran = 1
end
ran
";
        assert_eq!(eval_output(source), vec![0]);
    }

    // --- Print eval tests ---

    #[test]
    fn eval_print() {
        assert_eq!(eval_output("print(42)"), vec![42]);
    }

    #[test]
    fn eval_print_multi() {
        assert_eq!(eval_output("print(10, 20, 30)"), vec![10, 20, 30]);
    }

    // --- Variable eval tests ---

    #[test]
    fn eval_variables() {
        assert_eq!(eval_output("let x = 10\nlet y = 20\nx + y"), vec![30]);
    }

    #[test]
    fn eval_assignment() {
        assert_eq!(eval_output("let x = 5\nx = 10\nx"), vec![10]);
    }

    // --- Control flow eval tests ---

    #[test]
    fn eval_if_true() {
        assert_eq!(eval_output("if 1 > 0 do\n  42\nend"), vec![42]);
    }

    #[test]
    fn eval_if_else() {
        assert_eq!(eval_output("if 0 > 1 do\n  1\nelse\n  2\nend"), vec![2]);
    }

    #[test]
    fn eval_while() {
        let source = "
let n = 3
let total = 0
while n > 0 do
  total = total + n
  n = n - 1
end
total
";
        assert_eq!(eval_output(source), vec![6]); // 3+2+1
    }

    // --- Function eval tests ---

    #[test]
    fn eval_function() {
        let source = "
fn triple(x) do
  return x * 3
end
triple(14)
";
        assert_eq!(eval_output(source), vec![42]);
    }

    // --- Debug: compile to asm ---

    #[test]
    fn compile_to_asm_works() {
        let result = compile_to_asm("2 + 3");
        assert!(result.is_ok());
        if let Ok(asm) = result {
            assert!(asm.contains("entry"));
            assert!(asm.contains("halt"));
            // Optimizer folds 2+3 → lit 5, so no "add" instruction
            assert!(asm.contains("lit 5"));
        }
    }

    // --- Genome compilation tests ---

    #[test]
    fn compile_genome_basic() {
        let source = "fn double(x) do\n  return x * 2\nend\ndouble(5)";
        let result = compile_genome(source);
        assert!(result.is_ok());
        if let Ok(g) = result {
            assert_eq!(g.gene_count(), 1);
            assert!(g.codon_count() > 0);
        }
    }

    #[test]
    fn compile_genome_catalog() {
        let source = "
fn add(a, b) do
  return a + b
end
fn sub(a, b) do
  return a - b
end
add(1, 2)
";
        let result = compile_genome(source);
        assert!(result.is_ok());
        if let Ok(g) = result {
            let cat = g.catalog();
            assert_eq!(cat.len(), 2);
            // Both functions should have arity 2
            assert_eq!(cat[0].1, 2);
            assert_eq!(cat[1].1, 2);
        }
    }

    #[test]
    fn compile_genome_express() {
        let source = "fn triple(x) do\n  return x * 3\nend\ntriple(1)";
        let result = compile_genome(source);
        assert!(result.is_ok());
        if let Ok(g) = result {
            let expr_result = g.express("triple", &[14]);
            assert!(expr_result.is_ok());
            if let Ok(r) = expr_result {
                assert!(!r.stack.is_empty());
                assert_eq!(r.stack[r.stack.len() - 1], 42);
            }
        }
    }

    #[test]
    fn compile_genome_no_functions() {
        let source = "42";
        let result = compile_genome(source);
        assert!(result.is_ok());
        if let Ok(g) = result {
            assert_eq!(g.gene_count(), 0);
            // Should still run main code
            let run_result = g.run();
            assert!(run_result.is_ok());
            if let Ok(r) = run_result {
                assert_eq!(r.output, vec![42]);
            }
        }
    }

    // --- Bitwise eval integration tests ---

    #[test]
    fn eval_bitwise_and() {
        assert_eq!(eval_output("12 & 10"), vec![8]);
    }

    #[test]
    fn eval_bitwise_or() {
        assert_eq!(eval_output("12 | 10"), vec![14]);
    }

    #[test]
    fn eval_bitwise_xor() {
        assert_eq!(eval_output("12 ^ 10"), vec![6]);
    }

    #[test]
    fn eval_shifts() {
        assert_eq!(eval_output("1 << 8"), vec![256]);
        assert_eq!(eval_output("256 >> 8"), vec![1]);
    }

    #[test]
    fn eval_bitnot() {
        assert_eq!(eval_output("~0"), vec![-1]);
    }

    // --- Elif eval integration tests ---

    #[test]
    fn eval_elif_basic() {
        let source = "
let x = 2
if x == 1 do
  10
elif x == 2 do
  20
else
  30
end
";
        assert_eq!(eval_output(source), vec![20]);
    }

    #[test]
    fn eval_elif_to_else() {
        let source = "
let x = 99
if x == 1 do
  1
elif x == 2 do
  2
else
  99
end
";
        assert_eq!(eval_output(source), vec![99]);
    }
}
