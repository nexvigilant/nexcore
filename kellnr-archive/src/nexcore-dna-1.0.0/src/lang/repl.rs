//! Interactive REPL for the nexcore-dna language.
//!
//! Maintains persistent state across evaluations by accumulating
//! source lines and re-executing the full program each iteration.
//! Output tracking skips previously-shown values.
//!
//! ## Features
//!
//! - Variable/function persistence across lines
//! - Multi-line block input (auto-detects open `do`/`end` depth)
//! - Meta-commands: `:quit`, `:help`, `:reset`, `:asm`, `:lines`
//! - Error recovery: invalid lines are not committed
//!
//! Tier: T3 (ρ Recursion + ς State + σ Sequence + ∂ Boundary)

use crate::lang::compiler;

// ---------------------------------------------------------------------------
// REPL result type
// ---------------------------------------------------------------------------

/// Result of processing a single REPL input line.
///
/// Tier: T2-C (ς State + ∂ Boundary)
#[derive(Debug)]
pub enum ReplAction {
    /// New output values to display.
    Output(Vec<i64>),
    /// A meta-command was handled; message to display.
    Meta(String),
    /// An error occurred; message to display.
    Error(String),
    /// Empty or whitespace-only input (no-op).
    Empty,
    /// Multi-line block in progress; prompt should indicate continuation.
    Continue,
    /// Exit requested.
    Exit,
}

// ---------------------------------------------------------------------------
// REPL state
// ---------------------------------------------------------------------------

/// Interactive REPL with persistent state.
///
/// Tier: T3 (ρ + ς + σ + ∂)
pub struct Repl {
    /// All committed source lines (successfully evaluated).
    history: Vec<String>,
    /// Buffer for multi-line block input.
    pending: String,
    /// Current block nesting depth (tracks `do`/`end` balance).
    block_depth: i32,
    /// Number of output values already shown.
    output_offset: usize,
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

impl Repl {
    /// Create a new REPL instance.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            pending: String::new(),
            block_depth: 0,
            output_offset: 0,
        }
    }

    /// Process a single line of input.
    ///
    /// Returns an action indicating what the caller should do.
    pub fn eval_line(&mut self, line: &str) -> ReplAction {
        let trimmed = line.trim();

        // Meta-commands (only when not in a block)
        if self.block_depth == 0 && trimmed.starts_with(':') {
            return self.handle_meta(trimmed);
        }

        // Empty lines
        if trimmed.is_empty() {
            if self.block_depth > 0 {
                // In a block, empty line is fine — just continue
                self.pending.push('\n');
                return ReplAction::Continue;
            }
            return ReplAction::Empty;
        }

        // Track block depth from this line
        let depth_delta = count_depth_delta(trimmed);
        self.block_depth += depth_delta;

        // Accumulate into pending buffer
        if !self.pending.is_empty() {
            self.pending.push('\n');
        }
        self.pending.push_str(trimmed);

        // If block is still open, continue accumulating
        if self.block_depth > 0 {
            return ReplAction::Continue;
        }

        // Block complete (or single-line) — try to evaluate
        let candidate = self.pending.clone();
        self.pending.clear();
        self.block_depth = 0;

        self.try_eval(&candidate)
    }

    /// Try to compile and execute with the candidate line(s) appended.
    fn try_eval(&mut self, candidate: &str) -> ReplAction {
        // Build full source: history + candidate
        let mut source = String::new();
        for line in &self.history {
            source.push_str(line);
            source.push('\n');
        }
        source.push_str(candidate);

        // Compile and run
        match compiler::eval(&source) {
            Ok(result) => {
                // Commit the candidate
                self.history.push(candidate.to_string());

                // Extract only new outputs
                let new_outputs: Vec<i64> =
                    result.output.into_iter().skip(self.output_offset).collect();

                self.output_offset += new_outputs.len();

                if new_outputs.is_empty() {
                    ReplAction::Output(vec![])
                } else {
                    ReplAction::Output(new_outputs)
                }
            }
            Err(e) => {
                // Don't commit — show error
                ReplAction::Error(format!("{e}"))
            }
        }
    }

    /// Handle a meta-command.
    fn handle_meta(&mut self, cmd: &str) -> ReplAction {
        match cmd {
            ":quit" | ":q" | ":exit" => ReplAction::Exit,
            ":help" | ":h" => ReplAction::Meta(HELP_TEXT.to_string()),
            ":reset" | ":r" => {
                self.history.clear();
                self.pending.clear();
                self.block_depth = 0;
                self.output_offset = 0;
                ReplAction::Meta("State reset.".to_string())
            }
            ":asm" | ":a" => {
                if self.history.is_empty() {
                    return ReplAction::Meta("No code to show.".to_string());
                }
                let source: String = self.history.join("\n");
                match compiler::compile_to_asm(&source) {
                    Ok(asm) => ReplAction::Meta(asm),
                    Err(e) => ReplAction::Error(format!("{e}")),
                }
            }
            ":lines" | ":l" => {
                if self.history.is_empty() {
                    return ReplAction::Meta("No lines.".to_string());
                }
                let listing: String = self
                    .history
                    .iter()
                    .enumerate()
                    .map(|(i, l)| format!("{:3}: {l}", i + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                ReplAction::Meta(listing)
            }
            _ => ReplAction::Error(format!("Unknown command: {cmd}")),
        }
    }

    /// Whether we're currently in multi-line input mode.
    pub fn is_continuation(&self) -> bool {
        self.block_depth > 0
    }

    /// Get the number of committed lines.
    pub fn line_count(&self) -> usize {
        self.history.len()
    }
}

// ---------------------------------------------------------------------------
// Block depth counting
// ---------------------------------------------------------------------------

/// Count the net block depth change from a line of source.
///
/// Counts `do` as +1, `end` as -1.
/// Skips content after `;` (comments).
fn count_depth_delta(line: &str) -> i32 {
    // Strip comment
    let code = match line.find(';') {
        Some(pos) => &line[..pos],
        None => line,
    };

    let mut delta = 0i32;
    for word in code.split_whitespace() {
        match word {
            "do" => delta += 1,
            "end" => delta -= 1,
            _ => {}
        }
    }
    delta
}

// ---------------------------------------------------------------------------
// Help text
// ---------------------------------------------------------------------------

const HELP_TEXT: &str = "\
nexcore-dna REPL — Interactive DNA Language

Commands:
  :help, :h      Show this help
  :quit, :q      Exit the REPL
  :reset, :r     Clear all state
  :asm, :a       Show generated assembly
  :lines, :l     Show committed source lines

Language:
  42             Integer literal (output)
  true, false    Boolean literals (1/0)
  let x = expr   Variable binding
  x = expr       Assignment
  x += expr      Compound assignment (+=, -=, *=, /=, %=)
  a & b | c ^ d  Bitwise AND, OR, XOR
  ~x             Bitwise NOT
  a << n, a >> n Shift left/right
  if cond do     Conditional (with elif/else)
    ...
  elif cond do
    ...
  end
  while cond do  Loop
    ...
  end
  for i in 0..n do  Range loop
    ...
  end
  fn name(a, b) do  Function definition
    return expr
  end
  print(x, y)   Output values
  assert(expr)  Assert value is truthy
  abs min max pow sqrt sign clamp log2  Built-ins";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repl_simple_eval() {
        let mut repl = Repl::new();
        match repl.eval_line("42") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![42]),
            other => panic!("expected Output, got {other:?}"),
        }
    }

    #[test]
    fn repl_persistent_vars() {
        let mut repl = Repl::new();

        // Define a variable
        match repl.eval_line("let x = 10") {
            ReplAction::Output(vals) => assert!(vals.is_empty()),
            other => panic!("expected empty Output, got {other:?}"),
        }

        // Use the variable
        match repl.eval_line("x + 5") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![15]),
            other => panic!("expected Output(15), got {other:?}"),
        }
    }

    #[test]
    fn repl_persistent_functions() {
        let mut repl = Repl::new();

        // Define function (multi-line)
        assert!(matches!(
            repl.eval_line("fn double(x) do"),
            ReplAction::Continue
        ));
        assert!(matches!(
            repl.eval_line("  return x * 2"),
            ReplAction::Continue
        ));
        match repl.eval_line("end") {
            ReplAction::Output(_) => {}
            other => panic!("expected Output after fn end, got {other:?}"),
        }

        // Call the function
        match repl.eval_line("double(21)") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![42]),
            other => panic!("expected Output(42), got {other:?}"),
        }
    }

    #[test]
    fn repl_error_recovery() {
        let mut repl = Repl::new();

        // Valid line
        match repl.eval_line("let x = 5") {
            ReplAction::Output(_) => {}
            other => panic!("expected Output, got {other:?}"),
        }

        // Invalid line — should not corrupt state
        match repl.eval_line("2 +") {
            ReplAction::Error(_) => {}
            other => panic!("expected Error, got {other:?}"),
        }

        // State should be intact
        match repl.eval_line("x") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![5]),
            other => panic!("expected Output(5), got {other:?}"),
        }
    }

    #[test]
    fn repl_empty_input() {
        let mut repl = Repl::new();
        assert!(matches!(repl.eval_line(""), ReplAction::Empty));
        assert!(matches!(repl.eval_line("   "), ReplAction::Empty));
    }

    #[test]
    fn repl_meta_quit() {
        let mut repl = Repl::new();
        assert!(matches!(repl.eval_line(":quit"), ReplAction::Exit));
        assert!(matches!(repl.eval_line(":q"), ReplAction::Exit));
    }

    #[test]
    fn repl_meta_help() {
        let mut repl = Repl::new();
        match repl.eval_line(":help") {
            ReplAction::Meta(text) => assert!(text.contains("REPL")),
            other => panic!("expected Meta, got {other:?}"),
        }
    }

    #[test]
    fn repl_meta_reset() {
        let mut repl = Repl::new();

        // Add state
        match repl.eval_line("let x = 42") {
            ReplAction::Output(_) => {}
            other => panic!("expected Output, got {other:?}"),
        }
        assert_eq!(repl.line_count(), 1);

        // Reset
        match repl.eval_line(":reset") {
            ReplAction::Meta(_) => {}
            other => panic!("expected Meta, got {other:?}"),
        }
        assert_eq!(repl.line_count(), 0);
    }

    #[test]
    fn repl_meta_lines() {
        let mut repl = Repl::new();
        match repl.eval_line("let a = 1") {
            ReplAction::Output(_) => {}
            other => panic!("expected Output, got {other:?}"),
        }
        match repl.eval_line("let b = 2") {
            ReplAction::Output(_) => {}
            other => panic!("expected Output, got {other:?}"),
        }
        match repl.eval_line(":lines") {
            ReplAction::Meta(text) => {
                assert!(text.contains("let a = 1"));
                assert!(text.contains("let b = 2"));
            }
            other => panic!("expected Meta, got {other:?}"),
        }
    }

    #[test]
    fn repl_meta_asm() {
        let mut repl = Repl::new();
        match repl.eval_line("2 + 3") {
            ReplAction::Output(_) => {}
            other => panic!("expected Output, got {other:?}"),
        }
        match repl.eval_line(":asm") {
            ReplAction::Meta(text) => {
                assert!(text.contains("entry"));
                assert!(text.contains("halt"));
            }
            other => panic!("expected Meta, got {other:?}"),
        }
    }

    #[test]
    fn repl_multiline_if() {
        let mut repl = Repl::new();
        assert!(matches!(repl.eval_line("if true do"), ReplAction::Continue));
        assert!(matches!(repl.eval_line("  42"), ReplAction::Continue));
        match repl.eval_line("end") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![42]),
            other => panic!("expected Output(42), got {other:?}"),
        }
    }

    #[test]
    fn repl_multiline_while() {
        let mut repl = Repl::new();
        match repl.eval_line("let sum = 0") {
            ReplAction::Output(_) => {}
            other => panic!("expected Output, got {other:?}"),
        }
        match repl.eval_line("let n = 3") {
            ReplAction::Output(_) => {}
            other => panic!("expected Output, got {other:?}"),
        }
        assert!(matches!(
            repl.eval_line("while n > 0 do"),
            ReplAction::Continue
        ));
        assert!(matches!(repl.eval_line("  sum += n"), ReplAction::Continue));
        assert!(matches!(repl.eval_line("  n -= 1"), ReplAction::Continue));
        match repl.eval_line("end") {
            ReplAction::Output(_) => {}
            other => panic!("expected Output, got {other:?}"),
        }
        match repl.eval_line("sum") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![6]),
            other => panic!("expected Output(6), got {other:?}"),
        }
    }

    #[test]
    fn repl_output_offset_tracking() {
        let mut repl = Repl::new();

        // First eval outputs 42
        match repl.eval_line("42") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![42]),
            other => panic!("expected Output, got {other:?}"),
        }

        // Second eval should NOT re-show 42
        match repl.eval_line("99") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![99]),
            other => panic!("expected Output(99), got {other:?}"),
        }
    }

    #[test]
    fn repl_bool_integration() {
        let mut repl = Repl::new();
        match repl.eval_line("true and true") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![1]),
            other => panic!("expected Output(1), got {other:?}"),
        }
    }

    #[test]
    fn repl_print_integration() {
        let mut repl = Repl::new();
        match repl.eval_line("print(10, 20)") {
            ReplAction::Output(vals) => assert_eq!(vals, vec![10, 20]),
            other => panic!("expected Output, got {other:?}"),
        }
    }

    #[test]
    fn repl_unknown_meta() {
        let mut repl = Repl::new();
        match repl.eval_line(":foobar") {
            ReplAction::Error(msg) => assert!(msg.contains("Unknown")),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn depth_delta_do_end() {
        assert_eq!(count_depth_delta("if true do"), 1);
        assert_eq!(count_depth_delta("end"), -1);
        assert_eq!(count_depth_delta("for i in 1..10 do"), 1);
        assert_eq!(count_depth_delta("fn foo() do"), 1);
        assert_eq!(count_depth_delta("let x = 42"), 0);
        assert_eq!(count_depth_delta("if x do end"), 0); // balanced on one line
    }

    #[test]
    fn depth_delta_ignores_comments() {
        assert_eq!(count_depth_delta("42 ; do end"), 0);
        assert_eq!(count_depth_delta("if x do ; end"), 1); // end is in comment
    }

    #[test]
    fn repl_is_continuation() {
        let mut repl = Repl::new();
        assert!(!repl.is_continuation());

        let _ = repl.eval_line("if true do");
        assert!(repl.is_continuation());

        let _ = repl.eval_line("  42");
        assert!(repl.is_continuation());

        let _ = repl.eval_line("end");
        assert!(!repl.is_continuation());
    }
}
